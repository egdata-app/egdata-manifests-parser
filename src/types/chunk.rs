use hex;
use log::debug;
use serde::{Deserialize, Serialize};
use std::io::{Read, Seek, SeekFrom};
use uuid::Uuid;
use napi_derive::napi;

use crate::error::ManifestError;
use crate::parser::reader::ReadExt;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[napi(object)]
pub struct Chunk {
    pub guid: String,
    pub hash: String, // Store as hex string for NAPI compatibility
    pub sha_hash: String,
    pub group: u8,
    pub window_size: u32,
    pub file_size: String, // Store as string for NAPI compatibility
}

impl Chunk {
    pub fn guid(&self) -> String {
        self.guid.to_string()
    }

    pub fn hash(&self) -> String {
        self.hash.to_string()
    }

    pub fn sha_hash(&self) -> String {
        self.sha_hash.to_string()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[napi(object)]
pub struct ChunkDataList {
    pub data_size: u32,
    pub data_version: u8,
    pub count: u32,
    pub elements: Vec<Chunk>,
    #[serde(skip)]
    pub chunk_lookup: std::collections::HashMap<String, u32>,
}

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
            limit: std::cmp::min(limit, data.len()),
        }
    }
}

impl<'a> Read for LimitedReader<'a> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let available = self.limit.saturating_sub(self.position);
        if available == 0 {
            return Ok(0);
        }
        
        let to_read = std::cmp::min(buf.len(), available);
        let end_pos = self.position + to_read;
        
        if end_pos <= self.data.len() {
            buf[..to_read].copy_from_slice(&self.data[self.position..end_pos]);
            self.position = end_pos;
            Ok(to_read)
        } else {
            Ok(0)
        }
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
        
        self.position = std::cmp::min(new_pos, self.limit);
        Ok(self.position as u64)
    }
}

impl ChunkDataList {
    pub fn read<R: Read + Seek>(mut rdr: R) -> Result<Self, ManifestError> {
        let start_pos = rdr.stream_position()?;
        debug!(
            "Reading chunk list at position: {} (0x{:x})",
            start_pos, start_pos
        );

        let data_size = rdr.u32()?;
        debug!("  Data size: {} (0x{:x})", data_size, data_size);

        if data_size == 0 || data_size > 1024 * 1024 * 1024 {
            // 1GB max
            return Err(ManifestError::Invalid(format!(
                "Invalid data size: {} (0x{:x}). Must be between 1 and 1GB",
                data_size, data_size
            )));
        }

        // Read remaining data into buffer and use LimitedReader
        let adjusted_data_size = data_size.saturating_sub(4); // Subtract the 4 bytes we already read for data_size
        // Use tolerant reading to handle cases where less data is available than expected
        let remaining_data = rdr.read_bytes_tolerant(adjusted_data_size as usize)?;
        let actual_size = remaining_data.len();
        
        if actual_size < adjusted_data_size as usize {
            debug!(
                "Warning: Expected {} bytes but only {} bytes available for chunk data. Using available data.",
                adjusted_data_size, actual_size
            );
        }
        
        let mut limited_reader = LimitedReader::new(&remaining_data, actual_size);
        let rdr = &mut limited_reader;
        
        debug!(
            "ChunkDataList: using limited reader with {} bytes",
            adjusted_data_size
        );

        let data_version = rdr.u8()?;
        debug!("  Data version: {} (0x{:x})", data_version, data_version);

        let count = rdr.u32()?;
        debug!("  Count: {} (0x{:x})", count, count);

        if count > 1_000_000 {
            // Reasonable max chunk count
            return Err(ManifestError::Invalid(format!(
                "Invalid count: {} (0x{:x}). Must be less than 1,000,000",
                count, count
            )));
        }

        let mut elements = Vec::with_capacity(count as usize);
        let mut chunk_lookup = std::collections::HashMap::with_capacity(count as usize);

        debug!("\nReading GUIDs...");
        for i in 0..count {
            let guid_bytes = rdr.read_bytes_tolerant(16)?;
            if guid_bytes.len() != 16 {
                debug!("Warning: Expected 16 bytes for GUID but got {} bytes for chunk {}", guid_bytes.len(), i);
                return Err(ManifestError::Invalid(format!(
                    "Expected 16 bytes for GUID but got {} bytes for chunk {}", 
                    guid_bytes.len(), i
                )));
            }
            let mut guid_array = [0u8; 16];
            guid_array.copy_from_slice(&guid_bytes);
            let guid = Uuid::from_bytes(guid_array);
            let guid_str = guid.to_string();
            chunk_lookup.insert(guid_str.clone(), i);
            elements.push(Chunk {
                guid: guid_str,
                hash: String::new(),
                sha_hash: String::new(),
                group: 0,
                window_size: 0,
                file_size: String::new(),
            });
        }

        debug!("\nReading hashes...");
        for chunk in &mut elements {
            let hash = rdr.u64()?;
            chunk.hash = format!("{:016x}", hash);
        }

        debug!("\nReading SHA hashes...");
        for (i, chunk) in elements.iter_mut().enumerate() {
            let hash_bytes = rdr.read_bytes_tolerant(20)?;
            if hash_bytes.len() == 20 {
                chunk.sha_hash = hex::encode(hash_bytes);
            } else {
                debug!("Warning: Expected 20 bytes for SHA hash but got {} bytes for chunk {}", hash_bytes.len(), i);
                let mut padded_hash = hash_bytes;
                padded_hash.resize(20, 0);
                chunk.sha_hash = hex::encode(padded_hash);
            }
        }

        debug!("\nReading groups...");
        for chunk in &mut elements {
            chunk.group = rdr.u8()?;
        }

        debug!("\nReading window sizes...");
        for chunk in &mut elements {
            chunk.window_size = rdr.u32()?;
        }

        debug!("\nReading file sizes...");
        for chunk in &mut elements {
            let file_size = rdr.u64()?;
            chunk.file_size = file_size.to_string();
        }

        Ok(Self {
            data_size,
            data_version,
            count,
            elements,
            chunk_lookup,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[napi(object)]
pub struct ChunkPart {
    pub data_size: u32,
    pub parent_guid: String,
    pub offset: u32,
    pub size: u32,
    #[serde(skip)]
    pub chunk: Option<Chunk>, // Reference to parent chunk
}

impl ChunkPart {
    pub fn read<R: Read + Seek>(
        rdr: &mut R,
        chunk_lookup: &std::collections::HashMap<String, u32>,
        chunks: &[Chunk],
    ) -> Result<Self, ManifestError> {
        // Check if we have enough bytes to read a complete chunk part (28 bytes total)
        let current_pos = rdr.stream_position()?;
        
        let data_size = rdr.u32().map_err(|e| {
            debug!("Failed to read data_size at position {}: {}", current_pos, e);
            ManifestError::Io(e)
        })?;

        // Read GUID
        let guid_bytes = rdr.read_bytes_tolerant(16).map_err(|e| {
            debug!("Failed to read GUID at position {}: {}", rdr.stream_position().unwrap_or(0), e);
            ManifestError::Io(e)
        })?;
        
        if guid_bytes.len() != 16 {
            return Err(ManifestError::Invalid(format!(
                "Expected 16 bytes for GUID but got {} bytes", 
                guid_bytes.len()
            )));
        }
        
        let mut guid_array = [0u8; 16];
        guid_array.copy_from_slice(&guid_bytes);
        let parent_guid = Uuid::from_bytes(guid_array).to_string();

        // Validate parent GUID exists in chunk lookup
        if !chunk_lookup.contains_key(&parent_guid) {
            return Err(ManifestError::Invalid(format!(
                "Parent GUID {} not found in chunk lookup",
                parent_guid
            )));
        }

        let offset = rdr.u32().map_err(|e| {
            debug!("Failed to read offset at position {}: {}", rdr.stream_position().unwrap_or(0), e);
            ManifestError::Io(e)
        })?;
        
        let size = rdr.u32().map_err(|e| {
            debug!("Failed to read size at position {}: {}", rdr.stream_position().unwrap_or(0), e);
            ManifestError::Io(e)
        })?;

        // Get reference to parent chunk
        let chunk_idx = chunk_lookup[&parent_guid];
        let chunk = chunks.get(chunk_idx as usize).cloned();

        Ok(Self {
            data_size,
            parent_guid,
            offset,
            size,
            chunk,
        })
    }
}
