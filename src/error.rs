use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
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
