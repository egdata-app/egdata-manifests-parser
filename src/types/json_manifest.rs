use serde::{Deserialize, Serialize};
use crate::error::ManifestError;
use crate::types::manifest::Manifest;
use crate::types::header::ManifestHeader;
use crate::types::meta::ManifestMeta;
use crate::types::chunk::{ChunkDataList, Chunk};
use crate::types::file::{FileManifestList, FileManifest};
use crate::types::chunk::ChunkPart;
use uuid::Uuid;
use std::str::FromStr;
use hex;

/// JSON representation of a manifest file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonManifest {
    #[serde(rename = "ManifestFileVersion")]
    pub manifest_file_version: String,
    #[serde(rename = "bIsFileData")]
    pub is_file_data: bool,
    #[serde(rename = "AppID")]
    pub app_id: String,
    #[serde(rename = "AppNameString")]
    pub app_name_string: String,
    #[serde(rename = "BuildVersionString")]
    pub build_version_string: String,
    #[serde(rename = "LaunchExeString")]
    pub launch_exe_string: String,
    #[serde(rename = "LaunchCommand")]
    pub launch_command: String,
    #[serde(rename = "PrereqIds")]
    pub prereq_ids: Vec<String>,
    #[serde(rename = "PrereqName")]
    pub prereq_name: String,
    #[serde(rename = "PrereqPath")]
    pub prereq_path: String,
    #[serde(rename = "PrereqArgs")]
    pub prereq_args: String,
    #[serde(rename = "FileManifestList")]
    pub file_manifest_list: Vec<JsonFileManifest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonFileManifest {
    #[serde(rename = "Filename")]
    pub filename: String,
    #[serde(rename = "FileHash")]
    pub file_hash: String,
    #[serde(rename = "bIsUnixExecutable", default)]
    pub is_unix_executable: Option<bool>,
    #[serde(rename = "FileChunkParts")]
    pub file_chunk_parts: Vec<JsonFileChunkPart>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonFileChunkPart {
    #[serde(rename = "Guid")]
    pub guid: String,
    #[serde(rename = "Offset")]
    pub offset: String,
    #[serde(rename = "Size")]
    pub size: String,
}

impl JsonManifest {
    /// Parse JSON manifest from string
    pub fn from_str(json_str: &str) -> Result<Self, ManifestError> {
        serde_json::from_str(json_str)
            .map_err(|e| ManifestError::Invalid(format!("JSON parsing error: {}", e)))
    }

    /// Convert JSON manifest to standard Manifest structure
    pub fn to_manifest(self) -> Result<Manifest, ManifestError> {
        // Create a basic header (not used for JSON manifests)
        let header = ManifestHeader {
            header_size: 0,
            data_size_uncompressed: 0,
            data_size_compressed: 0,
            sha1_hash: String::new(),
            stored_as: 0,
            version: self.parse_version()? as i32,
            guid: String::new(),
            rolling_hash: 0,
            hash_type: 0,
        };

        // Create metadata
        let meta = ManifestMeta {
            data_size: 0, // Not applicable for JSON
            data_version: 0,
            feature_level: 0,
            is_file_data: true,
            app_id: self.parse_app_id()? as i32,
            app_name: self.app_name_string.clone(),
            build_version: self.build_version_string.clone(),
            launch_exe: self.launch_exe_string.clone(),
            launch_command: String::new(),
            prereq_ids: Vec::new(),
            prereq_name: String::new(),
            prereq_path: String::new(),
            prereq_args: String::new(),
            build_id: None,
        };

        // Extract unique chunks from file chunk parts
        let mut chunks = std::collections::HashMap::new();
        for file in &self.file_manifest_list {
            for chunk_part in &file.file_chunk_parts {
                let guid = Uuid::from_str(&chunk_part.guid)
                    .map_err(|e| ManifestError::Invalid(format!("Invalid GUID: {}", e)))?;
                
                if !chunks.contains_key(&guid.to_string()) {
                    chunks.insert(guid.to_string(), Chunk {
                        guid: guid.to_string(),
                        hash: String::new(), // Not available in JSON format
                        sha_hash: String::new(), // Not available in JSON format
                        group: 0,
                        window_size: 0,
                        file_size: self.parse_hex_string(&chunk_part.size)?.to_string(),
                    });
                }
            }
        }

        let chunk_lookup = chunks.iter().enumerate()
            .map(|(i, (guid, _))| (guid.clone(), i as u32))
            .collect();

        let chunk_list = ChunkDataList {
            data_size: 0, // Not applicable for JSON
            data_version: 0,
            count: chunks.len() as u32,
            elements: chunks.into_values().collect(),
            chunk_lookup,
        };

        // Convert file manifest list
        let mut files = Vec::new();
        for json_file in &self.file_manifest_list {
            let mut chunk_parts = Vec::new();
            for json_chunk_part in &json_file.file_chunk_parts {
                let guid = Uuid::from_str(&json_chunk_part.guid)
                    .map_err(|e| ManifestError::Invalid(format!("Invalid GUID: {}", e)))?;
                
                chunk_parts.push(ChunkPart {
                    data_size: 0, // Not applicable for JSON
                    parent_guid: guid.to_string(),
                    offset: self.parse_hex_string(&json_chunk_part.offset)? as u32,
                    size: self.parse_hex_string(&json_chunk_part.size)? as u32,
                    chunk: None, // Will be populated later if needed
                });
            }

            let file_size: i64 = chunk_parts.iter().map(|cp| cp.size as i64).sum();
            
            files.push(FileManifest {
                filename: json_file.filename.clone(),
                symlink_target: String::new(),
                sha_hash: hex::encode(self.parse_file_hash(&json_file.file_hash)?),
                file_meta_flags: if json_file.is_unix_executable.unwrap_or(false) { 4 } else { 0 }, // UnixExecutable = 1 << 2 = 4
                install_tags: Vec::new(),
                chunk_parts,
                file_size,
                mime_type: String::new(),
            });
        }

        let file_list = FileManifestList {
            data_size: 0, // Not applicable for JSON
            data_version: 0,
            count: files.len() as u32,
            file_manifest_list: files,
        };

        Ok(Manifest {
            header,
            meta: Some(meta),
            chunk_list: Some(chunk_list),
            file_list: Some(file_list),
        })
    }

    fn parse_version(&self) -> Result<u32, ManifestError> {
        // Handle large version numbers by taking only the last 8 digits or converting to a reasonable value
        if self.manifest_file_version.len() > 8 {
            // Take the last 8 digits to fit in u32
            let trimmed = &self.manifest_file_version[self.manifest_file_version.len() - 8..];
            trimmed.parse::<u32>()
                .map_err(|e| ManifestError::Invalid(format!("Invalid version format: {}", e)))
        } else {
            self.manifest_file_version.parse::<u32>()
                .map_err(|e| ManifestError::Invalid(format!("Invalid version format: {}", e)))
        }
    }

    fn parse_app_id(&self) -> Result<u32, ManifestError> {
        // Parse app ID string like "000000000000" to u32
        self.app_id.parse::<u32>()
            .map_err(|e| ManifestError::Invalid(format!("Invalid app ID format: {}", e)))
    }

    fn parse_hex_string(&self, hex_str: &str) -> Result<i64, ManifestError> {
        // Parse as u64 first, then convert to i64
        let value = u64::from_str_radix(hex_str, 16)
            .map_err(|e| ManifestError::Invalid(format!("Invalid hex string '{}': {}", hex_str, e)))?;
        
        Ok(value as i64)
    }

    fn parse_file_hash(&self, hash_str: &str) -> Result<[u8; 20], ManifestError> {
        // Parse file hash string to 20-byte array
        if hash_str.len() != 60 { // 20 bytes * 3 digits each
            return Err(ManifestError::Invalid(format!("Invalid file hash length: {}", hash_str.len())));
        }

        let mut hash = [0u8; 20];
        for i in 0..20 {
            let start = i * 3;
            let end = start + 3;
            let byte_str = &hash_str[start..end];
            hash[i] = byte_str.parse::<u8>()
                .map_err(|e| ManifestError::Invalid(format!("Invalid hash byte '{}': {}", byte_str, e)))?;
        }
        Ok(hash)
    }
}

/// Detect if the input data is a JSON manifest
pub fn is_json_manifest(data: &[u8]) -> bool {
    // Check if the data starts with '{' and contains expected JSON manifest fields
    if data.is_empty() || data[0] != b'{' {
        return false;
    }

    // Try to parse as JSON and check for required fields
    if let Ok(json_str) = std::str::from_utf8(data) {
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(json_str) {
            return value.get("ManifestFileVersion").is_some() 
                && value.get("FileManifestList").is_some();
        }
    }
    
    false
}