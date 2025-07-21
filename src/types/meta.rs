use log::debug;
use serde::{Deserialize, Serialize};
use std::io::{Read, Seek, SeekFrom};
use napi_derive::napi;

use crate::error::ManifestError;
use crate::parser::reader::ReadExt;

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

        // Read remaining data into buffer and use LimitedReader
        let adjusted_data_size = data_size.saturating_sub(4); // Subtract the 4 bytes we already read for data_size
        // Use tolerant reading to handle cases where less data is available than expected
        let remaining_data = rdr.read_bytes_tolerant(adjusted_data_size as usize)?;
        let actual_size = remaining_data.len();
        
        if actual_size < adjusted_data_size as usize {
            debug!(
                "Warning: Expected {} bytes but only {} bytes available for metadata. Using available data.",
                adjusted_data_size, actual_size
            );
        }
        
        let mut limited_reader = LimitedReader::new(&remaining_data, actual_size);
        let rdr = &mut limited_reader;
        
        debug!(
            "ManifestMeta: using limited reader with {} bytes",
            adjusted_data_size
        );

        let data_version = rdr.u8()?;
        debug!("  Data version: {} (0x{:x})", data_version, data_version);

        let feature_level = rdr.i32()?;
        debug!("  Feature level: {} (0x{:x})", feature_level, feature_level);

        let is_file_data = rdr.u8()? != 0;
        debug!("  Is file data: {}", is_file_data);

        let app_id = rdr.i32()?;
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
