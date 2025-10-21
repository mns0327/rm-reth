#[derive(thiserror::Error, Debug)]
pub enum NodeError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("slice error: {0}")]
    SliceError(#[from] std::array::TryFromSliceError),
    #[error("connection error: {0}")]
    ConnectionError(String),
    #[error("timeout: {0}")]
    Timeout(String),
    #[error("types util error: {0}")]
    TypeUtilError(#[from] types::error::TypeUtilError),
}
