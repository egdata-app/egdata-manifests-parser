pub use thiserror::Error;

#[derive(Debug, Error)]
pub enum ManifestError {
    #[error("I/O: {0}")]
    Io(#[from] std::io::Error),

    #[error("zlib-ng: {0}")]
    Inflate(String),

    #[error("invalid data: {0}")]
    Invalid(String),

    #[error("encrypted manifests are not supported")]
    EncryptedManifest,

    #[error("SHA-1 mismatch (corrupted file?)")]
    Sha1Mismatch,

    #[error("JSON: {0}")]
    Json(#[from] serde_json::Error),

    #[error("hex: {0}")]
    Hex(#[from] hex::FromHexError),
}

impl AsRef<str> for ManifestError {
    fn as_ref(&self) -> &str {
        match self {
            ManifestError::Io(_) => "I/O error",
            ManifestError::Inflate(_) => "zlib-ng error",
            ManifestError::Invalid(_) => "invalid data",
            ManifestError::EncryptedManifest => "encrypted manifests are not supported",
            ManifestError::Sha1Mismatch => "SHA-1 mismatch (corrupted file?)",
            ManifestError::Json(_) => "JSON error",
            ManifestError::Hex(_) => "hex error",
        }
    }
}
