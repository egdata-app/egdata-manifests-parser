use byteorder::{LittleEndian, ReadBytesExt};
use hex;
use log::debug;
use serde::{Deserialize, Serialize};
use std::io::{Read, Seek};
use uuid::Uuid;
use napi_derive::napi;

use crate::error::ManifestError;

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

impl ChunkDataList {
    pub fn read<R: Read + Seek>(mut rdr: R) -> Result<Self, ManifestError> {
        debug!(
            "Reading chunk list at position: {} (0x{:x})",
            rdr.stream_position()?,
            rdr.stream_position()?
        );

        let data_size = rdr.read_u32::<LittleEndian>()?;
        debug!("  Data size: {} (0x{:x})", data_size, data_size);

        if data_size == 0 || data_size > 1024 * 1024 * 1024 {
            // 1GB max
            return Err(ManifestError::Invalid(format!(
                "Invalid data size: {} (0x{:x}). Must be between 1 and 1GB",
                data_size, data_size
            )));
        }

        let data_version = rdr.read_u8()?;
        debug!("  Data version: {} (0x{:x})", data_version, data_version);

        let count = rdr.read_u32::<LittleEndian>()?;
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
            let mut guid_bytes = [0u8; 16];
            rdr.read_exact(&mut guid_bytes)?;
            let guid = Uuid::from_bytes(guid_bytes);
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
            let hash = rdr.read_u64::<LittleEndian>()?;
            chunk.hash = format!("{:016x}", hash);
        }

        debug!("\nReading SHA hashes...");
        for chunk in &mut elements {
            let mut sha_hash = [0u8; 20];
            rdr.read_exact(&mut sha_hash)?;
            chunk.sha_hash = hex::encode(sha_hash);
        }

        debug!("\nReading groups...");
        for chunk in &mut elements {
            chunk.group = rdr.read_u8()?;
        }

        debug!("\nReading window sizes...");
        for chunk in &mut elements {
            chunk.window_size = rdr.read_u32::<LittleEndian>()?;
        }

        debug!("\nReading file sizes...");
        for chunk in &mut elements {
            let file_size = rdr.read_u64::<LittleEndian>()?;
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
        let data_size = rdr.read_u32::<LittleEndian>()?;

        // Read GUID
        let mut guid_bytes = [0u8; 16];
        rdr.read_exact(&mut guid_bytes)?;
        let parent_guid = Uuid::from_bytes(guid_bytes).to_string();

        // Validate parent GUID exists in chunk lookup
        if !chunk_lookup.contains_key(&parent_guid) {
            return Err(ManifestError::Invalid(format!(
                "Parent GUID {} not found in chunk lookup",
                parent_guid
            )));
        }

        let offset = rdr.read_u32::<LittleEndian>()?;
        let size = rdr.read_u32::<LittleEndian>()?;

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
