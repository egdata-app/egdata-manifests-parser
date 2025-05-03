pub mod types {
    pub mod chunk;
    pub mod file;
    pub mod flags;
    pub mod header;
    pub mod manifest;
    pub mod meta;
}

pub mod parser {
    pub mod reader;
}

pub mod error;

// Re-export commonly used types
pub use types::chunk::ChunkDataList;
pub use types::file::FileManifestList;
pub use types::header::ManifestHeader;
pub use types::manifest::Manifest;
pub use types::meta::ManifestMeta;

use std::{
    fs,
    io::{Cursor, Seek},
    path::Path,
};

use error::Error;

use hex;
use log::{debug, error, info, warn};
use miniz_oxide::inflate::decompress_to_vec_zlib;
use sha1::{Digest, Sha1};
use tokio::fs as tokio_fs;

/// Read → verify → parse
pub fn load(path: impl AsRef<Path>) -> Result<Manifest, Error> {
    let buf = fs::read(&path)?;
    process_manifest_data(buf)
}

/// Async version of load
pub async fn load_async(path: impl AsRef<Path>) -> Result<Manifest, Error> {
    let buf = tokio_fs::read(&path).await?;
    process_manifest_data(buf)
}

/// Process manifest data from a buffer
fn process_manifest_data(buf: Vec<u8>) -> Result<Manifest, Error> {
    let mut rdr = Cursor::new(&buf);
    let header = ManifestHeader::read(&mut rdr)?;

    // ---------------------------------------------------------------- body
    let payload_compressed = {
        let start = header.header_size as usize;
        let size = if header.is_compressed() {
            header.data_size_compressed
        } else {
            header.data_size_uncompressed
        };
        let end = start + size as usize;
        if start >= buf.len() || end > buf.len() {
            return Err(Error::Invalid("payload out of bounds".to_string()));
        }
        &buf[start..end]
    };

    if header.is_encrypted() {
        return Err(Error::EncryptedManifest);
    }

    let payload = if header.is_compressed() {
        info!("Decompressing data...");
        debug!("  Compressed size: {}", payload_compressed.len());
        debug!(
            "  Compressed data starts with: {:02x?}",
            &payload_compressed[..std::cmp::min(16, payload_compressed.len())]
        );

        // Try to find zlib header
        let mut offset = 0;
        while offset < payload_compressed.len() - 2 {
            if payload_compressed[offset] == 0x78
                && (payload_compressed[offset + 1] == 0x01
                    || payload_compressed[offset + 1] == 0x9C
                    || payload_compressed[offset + 1] == 0xDA)
            {
                if offset == 0 {
                    debug!("  Found zlib header at start");
                } else {
                    debug!("  Found zlib header at offset {}", offset);
                }
                break;
            }
            offset += 1;
        }

        if offset < payload_compressed.len() - 2 {
            debug!("  Decompressing from offset {}", offset);
            decompress_to_vec_zlib(&payload_compressed[offset..])
                .map_err(|e| Error::Inflate(format!("decompression failed: {}", e)))?
        } else {
            debug!("  No zlib header found in compressed data");
            payload_compressed.to_vec()
        }
    } else {
        // Try to find zlib header in uncompressed data
        if payload_compressed.len() > 9
            && payload_compressed[9] == 0x78
            && (payload_compressed[10] == 0x01
                || payload_compressed[10] == 0x9C
                || payload_compressed[10] == 0xDA)
        {
            debug!("  Found zlib header at offset 9 in uncompressed data");
            let compressed_data = &payload_compressed[9..];
            debug!("  Decompressing {} bytes of data", compressed_data.len());
            debug!(
                "  Compressed data starts with: {:02x?}",
                &compressed_data[..std::cmp::min(16, compressed_data.len())]
            );
            decompress_to_vec_zlib(compressed_data)
                .map_err(|e| Error::Inflate(format!("decompression failed: {}", e)))?
        } else {
            debug!("  No zlib header found, treating as uncompressed");
            payload_compressed.to_vec()
        }
    };

    debug!("Payload length: {}", payload.len());
    debug!(
        "Payload starts with: {:02x?}",
        &payload[..std::cmp::min(16, payload.len())]
    );

    // Calculate SHA-1 of the payload
    let mut hasher = Sha1::new();
    hasher.update(&payload);
    let payload_sha = hasher.finalize();
    debug!("Payload SHA-1: {}", hex::encode(payload_sha));
    debug!("Header SHA-1: {}", header.sha1_hash);

    if hex::encode(payload_sha) != header.sha1_hash {
        warn!("Warning: Payload SHA-1 does not match header SHA-1");
    }

    let mut cur = Cursor::new(payload.clone());

    // --- Metadata Reading ---
    let meta_start_pos = cur.position();
    info!(
        "\nReading metadata starting at position: {} (0x{:x})",
        meta_start_pos, meta_start_pos
    );

    // Read metadata and process the result
    let meta_result = ManifestMeta::read_meta(&mut cur);

    // Map the result directly to Option<ManifestMeta> and handle side-effects
    let meta: Option<ManifestMeta> = match meta_result {
        Ok((parsed_meta, _)) => {
            info!(
                "Successfully parsed metadata. Data size: {} (0x{:x})",
                parsed_meta.data_size, parsed_meta.data_size
            );
            Some(parsed_meta)
        }
        Err(e) => {
            error!("Failed to parse metadata: {}", e);
            None
        }
    };

    // Always seek to the end of the metadata section based on the reported data size
    if let Some(meta) = &meta {
        let expected_meta_end_pos = meta_start_pos + meta.data_size as u64;
        let current_pos = cur.position();
        info!(
            "Seeking to end of metadata section. Current: {} (0x{:x}), Expected: {} (0x{:x})",
            current_pos, current_pos, expected_meta_end_pos, expected_meta_end_pos
        );
        cur.seek(std::io::SeekFrom::Start(expected_meta_end_pos))?;
    }

    // --- Chunk List Reading ---
    let chunk_list_start_pos = cur.position();
    info!(
        "\nReading chunk list starting at position: {} (0x{:x})",
        chunk_list_start_pos, chunk_list_start_pos
    );

    let chunk_list = ChunkDataList::read(&mut cur)?;

    // --- File List Reading ---
    let file_list_start_pos = cur.position();
    info!(
        "\nReading file list starting at position: {} (0x{:x})",
        file_list_start_pos, file_list_start_pos
    );

    let file_list = FileManifestList::read(&mut cur, &chunk_list)?;

    Ok(Manifest {
        header,
        meta,
        chunk_list: Some(chunk_list),
        file_list: Some(file_list),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_parse_manifest() {
        let manifest_path = PathBuf::from("manifest.manifest");
        let manifest = load(&manifest_path).expect("Failed to load manifest");

        // Basic validation
        assert!(!manifest.header.sha1_hash.is_empty());
        assert!(manifest.meta.is_some());

        // Print some basic info
        println!("Manifest version: {}", manifest.header.version);
        if let Some(meta) = &manifest.meta {
            println!("App name: {}", meta.app_name);
            println!("Build version: {}", meta.build_version);
        }

        // Validate chunk and file lists
        assert!(manifest.chunk_list.is_some());
        assert!(manifest.file_list.is_some());

        if let Some(file_list) = &manifest.file_list {
            println!("Number of files: {}", file_list.count);
        }
    }

    #[tokio::test]
    async fn test_parse_manifest_async() {
        let manifest_path = PathBuf::from("manifest.manifest");
        let manifest = load_async(&manifest_path)
            .await
            .expect("Failed to load manifest");

        // Basic validation
        assert!(!manifest.header.sha1_hash.is_empty());
        assert!(manifest.meta.is_some());

        // Print some basic info
        println!("Manifest version: {}", manifest.header.version);
        if let Some(meta) = &manifest.meta {
            println!("App name: {}", meta.app_name);
            println!("Build version: {}", meta.build_version);
        }

        // Validate chunk and file lists
        assert!(manifest.chunk_list.is_some());
        assert!(manifest.file_list.is_some());

        if let Some(file_list) = &manifest.file_list {
            println!("Number of files: {}", file_list.count);
        }
    }

    #[tokio::test]
    async fn test_sync_vs_async_manifest_loading() {
        let manifest_path = PathBuf::from("manifest.manifest");

        // Load manifest using both methods
        let sync_manifest = load(&manifest_path).expect("Failed to load manifest synchronously");
        let async_manifest = load_async(&manifest_path)
            .await
            .expect("Failed to load manifest asynchronously");

        // Compare headers
        assert_eq!(sync_manifest.header.version, async_manifest.header.version);
        assert_eq!(
            sync_manifest.header.sha1_hash,
            async_manifest.header.sha1_hash
        );
        assert_eq!(
            sync_manifest.header.header_size,
            async_manifest.header.header_size
        );
        assert_eq!(
            sync_manifest.header.data_size_compressed,
            async_manifest.header.data_size_compressed
        );
        assert_eq!(
            sync_manifest.header.data_size_uncompressed,
            async_manifest.header.data_size_uncompressed
        );

        // Compare metadata
        assert_eq!(
            sync_manifest.meta.as_ref().map(|m| &m.app_name),
            async_manifest.meta.as_ref().map(|m| &m.app_name)
        );
        assert_eq!(
            sync_manifest.meta.as_ref().map(|m| &m.build_version),
            async_manifest.meta.as_ref().map(|m| &m.build_version)
        );

        // Compare chunk lists
        let sync_chunks = sync_manifest
            .chunk_list
            .as_ref()
            .expect("Sync manifest missing chunk list");
        let async_chunks = async_manifest
            .chunk_list
            .as_ref()
            .expect("Async manifest missing chunk list");
        assert_eq!(sync_chunks.count, async_chunks.count);
        assert_eq!(sync_chunks.elements.len(), async_chunks.elements.len());

        // Compare file lists
        let sync_files = sync_manifest
            .file_list
            .as_ref()
            .expect("Sync manifest missing file list");
        let async_files = async_manifest
            .file_list
            .as_ref()
            .expect("Async manifest missing file list");
        assert_eq!(sync_files.count, async_files.count);
        assert_eq!(
            sync_files.file_manifest_list.len(),
            async_files.file_manifest_list.len()
        );

        // Compare individual files
        for (sync_file, async_file) in sync_files
            .file_manifest_list
            .iter()
            .zip(async_files.file_manifest_list.iter())
        {
            assert_eq!(sync_file.filename, async_file.filename);
            assert_eq!(sync_file.symlink_target, async_file.symlink_target);
            assert_eq!(sync_file.sha_hash, async_file.sha_hash);
            assert_eq!(sync_file.chunk_parts.len(), async_file.chunk_parts.len());
        }

        println!("Sync and async manifest loading produced identical results!");
    }
}
