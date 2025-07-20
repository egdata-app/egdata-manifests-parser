use byteorder::{LittleEndian, ReadBytesExt};
use log::debug;
use serde::{Deserialize, Serialize};
use std::io::{Read, Seek};
use napi_derive::napi;

use crate::error::ManifestError;
use crate::parser::reader::ReadExt;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[napi(object)]
pub struct ManifestMeta {
    pub data_size: u32,
    pub data_version: u8,
    pub feature_level: i32,
    pub is_file_data: bool,
    pub app_id: i32,
    #[serde(serialize_with = "trim_null_chars")]
    pub app_name: String,
    #[serde(serialize_with = "trim_null_chars")]
    pub build_version: String,
    #[serde(serialize_with = "trim_null_chars")]
    pub launch_exe: String,
    #[serde(serialize_with = "trim_null_chars")]
    pub launch_command: String,
    pub prereq_ids: Vec<String>,
    #[serde(serialize_with = "trim_null_chars")]
    pub prereq_name: String,
    #[serde(serialize_with = "trim_null_chars")]
    pub prereq_path: String,
    #[serde(serialize_with = "trim_null_chars")]
    pub prereq_args: String,
    pub build_id: Option<String>,
}

fn trim_null_chars<S>(value: &String, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let trimmed = value.trim_end_matches('\0');
    serializer.serialize_str(trimmed)
}

impl ManifestMeta {
    pub fn read_meta<R: Read + Seek>(rdr: &mut R) -> Result<(Self, u64), ManifestError> {
        let start_pos = rdr.stream_position()?;

        debug!("Reading metadata:");
        let data_size = rdr.read_u32::<LittleEndian>()?;
        debug!("  Data size: {} (0x{:x})", data_size, data_size);

        // Validate data size
        if data_size == 0 || data_size > 1024 * 1024 * 1024 {
            // 1GB max
            return Err(ManifestError::Invalid(format!(
                "Invalid data size: {} (0x{:x}). Must be between 1 and 1GB",
                data_size, data_size
            )));
        }

        let data_version = rdr.read_u8()?;
        debug!("  Data version: {} (0x{:x})", data_version, data_version);

        let feature_level = rdr.read_i32::<LittleEndian>()?;
        debug!("  Feature level: {} (0x{:x})", feature_level, feature_level);

        let is_file_data = rdr.read_u8()? != 0;
        debug!("  Is file data: {}", is_file_data);

        let app_id = rdr.read_i32::<LittleEndian>()?;
        debug!("  App ID: {} (0x{:x})", app_id, app_id);

        let app_name = rdr.fstring()?;
        debug!("  App name: {}", app_name);

        let build_version = rdr.fstring()?;
        debug!("  Build version: {}", build_version);

        let launch_exe = rdr.fstring()?;
        debug!("  Launch exe: {}", launch_exe);

        let launch_command = rdr.fstring()?;
        debug!("  Launch command: {}", launch_command);

        let prereq_ids = rdr.fstring_array()?;
        debug!("  Prerequisite IDs: {:?}", prereq_ids);

        let prereq_name = rdr.fstring()?;
        debug!("  Prerequisite name: {}", prereq_name);

        let prereq_path = rdr.fstring()?;
        debug!("  Prerequisite path: {}", prereq_path);

        let prereq_args = rdr.fstring()?;
        debug!("  Prerequisite args: {}", prereq_args);

        let build_id = if data_version >= 1 {
            let build_id = rdr.fstring()?;
            debug!("  Build ID: {}", build_id);
            Some(build_id)
        } else {
            None
        };

        let end_pos = rdr.stream_position()?;
        let bytes_read = end_pos - start_pos;

        Ok((
            Self {
                data_size,
                data_version,
                feature_level,
                is_file_data,
                app_id,
                app_name,
                build_version,
                launch_exe,
                launch_command,
                prereq_ids,
                prereq_name,
                prereq_path,
                prereq_args,
                build_id,
            },
            bytes_read,
        ))
    }
}
