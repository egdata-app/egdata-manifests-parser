use hex;
use log::debug;
use serde::{Deserialize, Serialize};
use std::io::{Read, Seek, SeekFrom};
use napi_derive::napi;

use crate::error::ManifestError;
use crate::parser::reader::ReadExt;
use crate::types::chunk::{ChunkDataList, ChunkPart};

/// A wrapper that limits reading to a specific range of data
struct LimitedReader<'a> {
    data: &'a [u8],
    position: usize,
    limit: usize,
}

impl<'a> LimitedReader<'a> {
    fn new(data: &'a [u8], limit: usize) -> Self {
        Self {
            data,
            position: 0,
            limit: limit.min(data.len()),
        }
    }
}

impl<'a> Read for LimitedReader<'a> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.position >= self.limit {
            return Ok(0); // EOF
        }
        
        let available = self.limit - self.position;
        let to_read = buf.len().min(available);
        
        if to_read == 0 {
            return Ok(0);
        }
        
        buf[..to_read].copy_from_slice(&self.data[self.position..self.position + to_read]);
        self.position += to_read;
        Ok(to_read)
    }
}

impl<'a> Seek for LimitedReader<'a> {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        let new_pos = match pos {
            SeekFrom::Start(offset) => offset as usize,
            SeekFrom::End(offset) => {
                if offset >= 0 {
                    self.limit + offset as usize
                } else {
                    self.limit.saturating_sub((-offset) as usize)
                }
            }
            SeekFrom::Current(offset) => {
                if offset >= 0 {
                    self.position + offset as usize
                } else {
                    self.position.saturating_sub((-offset) as usize)
                }
            }
        };
        
        self.position = new_pos.min(self.limit);
        Ok(self.position as u64)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[napi(object)]
pub struct FileManifest {
    #[serde(serialize_with = "trim_null_chars")]
    pub filename: String,
    pub symlink_target: String,
    pub sha_hash: String,
    pub file_meta_flags: u8,
    #[serde(serialize_with = "vector_trim_null_chars")]
    pub install_tags: Vec<String>,
    pub chunk_parts: Vec<ChunkPart>,
    pub file_size: i64,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub mime_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[napi(object)]
pub struct FileManifestList {
    pub data_size: u32,
    pub data_version: u8,
    pub count: u32,
    pub file_manifest_list: Vec<FileManifest>,
}

fn trim_null_chars<S>(value: &String, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let trimmed = value.trim_end_matches('\0');
    serializer.serialize_str(trimmed)
}

fn vector_trim_null_chars<S>(value: &Vec<String>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let trimmed: Vec<String> = value
        .iter()
        .map(|s| s.trim_end_matches('\0').to_string())
        .collect();
    trimmed.serialize(serializer)
}

// File meta flags from .NET implementation
#[repr(u8)]
pub enum EFileMetaFlags {
    None = 0,
    ReadOnly = 1 << 0,
    Compressed = 1 << 1,
    UnixExecutable = 1 << 2,
}

impl FileManifest {
    pub fn is_readonly(&self) -> bool {
        self.file_meta_flags & EFileMetaFlags::ReadOnly as u8 != 0
    }

    pub fn is_compressed(&self) -> bool {
        self.file_meta_flags & EFileMetaFlags::Compressed as u8 != 0
    }

    pub fn is_unix_executable(&self) -> bool {
        self.file_meta_flags & EFileMetaFlags::UnixExecutable as u8 != 0
    }
}

impl FileManifestList {
    pub fn read<R: Read + Seek>(rdr: &mut R, chunk_list: &ChunkDataList) -> Result<Self, ManifestError> {
        let start_pos = rdr.stream_position()?;
        debug!(
            "\nReading file list at position: {} (0x{:x})",
            start_pos, start_pos
        );

        // Read data size (uint32 in Go)
        let data_size = rdr.u32()?;
        debug!("  Data size: {} (0x{:x})", data_size, data_size);

        // Validate data size
        if data_size == 0 || data_size > 1024 * 1024 * 1024 {
            // 1GB max
            return Err(ManifestError::Invalid(format!(
                "Invalid data size: {} (0x{:x}). Must be between 1 and 1GB",
                data_size, data_size
            )));
        }

        // Read data version (uint8 in Go)
        let data_version = rdr.u8()?;
        debug!("  Data version: {} (0x{:x})", data_version, data_version);

        // Validate data version
        if data_version > 2 {
            return Err(ManifestError::Invalid(format!(
                "Invalid data version: {} (0x{:x}). Must be 0, 1, or 2",
                data_version, data_version
            )));
        }

        // Read count (uint32 in Go)
        let count = rdr.u32()?;
        debug!("  Count: {} (0x{:x})", count, count);

        // Read the remaining data into a buffer and use LimitedReader
        // Use tolerant reading to handle cases where less data is available than expected
        let remaining_data = rdr.read_bytes_tolerant(data_size as usize)?;
        let actual_size = remaining_data.len();
        
        if actual_size < data_size as usize {
            debug!(
                "Warning: Expected {} bytes but only {} bytes available. Using available data.",
                data_size, actual_size
            );
        }
        
        let mut limited_reader = LimitedReader::new(&remaining_data, actual_size);
        let rdr = &mut limited_reader;
        
        debug!(
            "FileManifestList: using limited reader with {} bytes",
            data_size
        );

        // Validate count
        if count > 1_000_000 {
            return Err(ManifestError::Invalid(
                "File count exceeds reasonable limit".to_string(),
            ));
        }

        // Initialize file list with capacity
        let mut files = Vec::with_capacity(count as usize);

        // Read filenames in batch
        debug!("\nReading filenames...");
        for _ in 0..count {
            let mut file = FileManifest::default();
            file.filename = rdr.fstring()?;
            files.push(file);
        }

        // Read symlink targets in batch
        debug!("\nReading symlink targets...");
        for i in 0..count {
            files[i as usize].symlink_target = rdr.fstring()?;
        }

        // Read SHA hashes in batch
        debug!("\nReading file hashes...");
        for i in 0..count {
            let hash_bytes = rdr.read_bytes_tolerant(20)?;
            if hash_bytes.len() == 20 {
                files[i as usize].sha_hash = hex::encode(hash_bytes);
            } else {
                debug!("Warning: Expected 20 bytes for SHA hash but got {} bytes for file {}", hash_bytes.len(), i);
                // Pad with zeros if needed or use empty hash
                let mut padded_hash = hash_bytes;
                padded_hash.resize(20, 0);
                files[i as usize].sha_hash = hex::encode(padded_hash);
            }
        }

        // Read file meta flags in batch
        debug!("\nReading file meta flags...");
        for i in 0..count {
            files[i as usize].file_meta_flags = rdr.u8()?;
        }

        // Read install tags in batch
        debug!("\nReading install tags...");
        for i in 0..count {
            files[i as usize].install_tags = rdr.fstring_array()?;
        }

        // Read chunk parts in batch
        debug!("\nReading chunk parts...");
        let mut total_chunk_parts = 0;
        let mut total_chunk_size = 0i64;
        for i in 0..count {
            let chunk_count = rdr.u32()?;
            let pos = rdr.stream_position()?;
            debug!(
                "File {}: Reading {} chunk parts at position {}",
                i, chunk_count, pos
            );

            // Validate chunk count - use a reasonable limit
            if chunk_count > 10_000 {
                debug!(
                    "   Warning: Invalid chunk count ({}) for file {} at position {}, skipping.",
                    chunk_count, i, pos
                );
                files[i as usize].chunk_parts = Vec::new();
                continue;
            }

            // Read chunks
            let mut chunks = Vec::with_capacity(chunk_count as usize);
            let mut file_chunk_size = 0i64;
            let mut valid_chunks = 0;

            for j in 0..chunk_count {
                let chunk_pos = rdr.stream_position()?;
                match ChunkPart::read(rdr, &chunk_list.chunk_lookup, &chunk_list.elements) {
                    Ok(chunk) => {
                        file_chunk_size += chunk.size as i64;
                        chunks.push(chunk);
                        valid_chunks += 1;
                        if j == 0 || j == chunk_count - 1 {
                            debug!(
                                "  Chunk part {}: size={}, offset={}, parent={} (at pos {})",
                                j,
                                chunks[j as usize].size,
                                chunks[j as usize].offset,
                                chunks[j as usize].parent_guid,
                                chunk_pos
                            );
                        }
                    }
                    Err(e) => {
                        debug!(
              "   Warning: Failed to read chunk part {} for file {}: {}. Skipping remaining chunks.",
              j, i, e
            );
                        break;
                    }
                }
            }

            if valid_chunks > 0 {
                total_chunk_parts += valid_chunks;
                total_chunk_size += file_chunk_size;
                files[i as usize].chunk_parts = chunks;
                files[i as usize].file_size = file_chunk_size; // Calculate file size from chunks
            } else {
                debug!(
                    "   Warning: No valid chunks found for file {}, skipping.",
                    i
                );
                files[i as usize].chunk_parts = Vec::new();
            }
        }

        // Handle version 2+ specific data with EOF tolerance
        if data_version >= 2 {
            debug!("\nReading version 2+ specific data...");
            
            // Try to read version 2+ data, but handle EOF gracefully
            let mut version2_success = true;
            
            // Skip unknown arrays with EOF handling
            for i in 0..count {
                match rdr.u32() {
                    Ok(array_size) => {
                        if let Err(e) = rdr.seek(SeekFrom::Current(array_size as i64 * 16)) {
                            debug!("Warning: Failed to seek past unknown array for file {}: {}. Stopping version 2+ parsing.", i, e);
                            version2_success = false;
                            break;
                        }
                    }
                    Err(e) => {
                        debug!("Warning: Failed to read array size for file {}: {}. Stopping version 2+ parsing.", i, e);
                        version2_success = false;
                        break;
                    }
                }
            }

            // Read MIME types with EOF handling
            if version2_success {
                for i in 0..count {
                    match rdr.fstring() {
                        Ok(mime_type) => {
                            files[i as usize].mime_type = mime_type;
                        }
                        Err(e) => {
                            debug!("Warning: Failed to read MIME type for file {}: {}. Stopping MIME type parsing.", i, e);
                            break;
                        }
                    }
                }
            }

            // Skip unknown data with EOF handling
            if version2_success {
                for i in 0..count {
                    if let Err(e) = rdr.seek(SeekFrom::Current(32)) {
                        debug!("Warning: Failed to seek past unknown data for file {}: {}. Stopping unknown data parsing.", i, e);
                        break;
                    }
                }
            }
            
            if !version2_success {
                debug!("Note: Version 2+ specific data parsing was incomplete due to EOF, but this is acceptable for corrupted/truncated manifests.");
            }
        }

        debug!(
            "Total chunk parts: {}, Total chunk size: {} bytes",
            total_chunk_parts, total_chunk_size
        );

        debug!("FileManifestList parsing completed successfully");

        Ok(Self {
            data_size,
            data_version,
            count,
            file_manifest_list: files,
        })
    }
}
