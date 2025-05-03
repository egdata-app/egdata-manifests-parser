use crate::types::{
    chunk::ChunkDataList, file::FileManifestList, header::ManifestHeader, meta::ManifestMeta,
};
use serde::{Deserialize, Serialize};

/// Whole manifest, JSON-serialisable.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Manifest {
    pub header: ManifestHeader,
    pub meta: Option<ManifestMeta>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub chunk_list: Option<ChunkDataList>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_list: Option<FileManifestList>,
}
