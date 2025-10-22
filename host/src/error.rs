#[derive(thiserror::Error, Debug)]
pub enum HostApiError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("connection error: {0}")]
    ConnectionError(String),
    #[error("types util error: {0}")]
    TypeUtilError(#[from] types::error::TypeUtilError),
}
