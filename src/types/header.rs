use hex;
use log::debug;
use serde::{Deserialize, Serialize};
use std::io::{Read, Seek};
use napi_derive::napi;

use crate::parser::reader::ReadExt;
use crate::{error::ManifestError, types::flags::*};

const MANIFEST_MAGIC: u32 = 0x44BEC00C;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[napi(object)]
pub struct ManifestHeader {
    pub header_size: i32,
    pub data_size_uncompressed: i32,
    pub data_size_compressed: i32,
    pub sha1_hash: String,
    pub stored_as: u8,
    pub version: i32,
    pub guid: String,
    pub rolling_hash: i64,
    pub hash_type: u32,
}

impl ManifestHeader {
    pub fn read<R: Read + Seek>(mut rdr: R) -> Result<Self, ManifestError> {
        // Read and verify magic number
        let magic = rdr.u32()?;
        if magic != MANIFEST_MAGIC {
            return Err(ManifestError::Invalid("invalid manifest magic number".to_string()));
        }

        // Read header size
        let header_size = rdr.i32()?;
        debug!("  Header size from file: {}", header_size);

        // Read data sizes
        let data_size_uncompressed = rdr.i32()?;
        let data_size_compressed = rdr.i32()?;

        // Read SHA1 hash
        let mut hash = [0u8; 20];
        rdr.read_exact(&mut hash)?;
        debug!("Raw SHA-1 bytes from file: {:02x?}", hash);

        // Read stored_as flag
        let stored_as = rdr.u8()?;

        // Read version if header size > 37 bytes
        let version = if header_size > 37 {
            rdr.i32()?
        } else {
            0 // Default to 0 for older versions
        };

        // Skip to the end of the header
        let current_pos = rdr.stream_position()?;
        if current_pos < header_size as u64 {
            rdr.seek(std::io::SeekFrom::Start(header_size as u64))?;
        }

        Ok(Self {
            header_size,
            data_size_uncompressed,
            data_size_compressed,
            sha1_hash: hex::encode(hash),
            stored_as,
            version,
            guid: String::new(), // Not used in newer versions
            rolling_hash: 0,     // Not used in newer versions
            hash_type: 0,        // Not used in newer versions
        })
    }

    /// helpers
    pub fn is_compressed(&self) -> bool {
        self.stored_as & STORED_COMPRESSED != 0
    }
    pub fn is_encrypted(&self) -> bool {
        self.stored_as & STORED_ENCRYPTED != 0
    }
}
