use std::fmt;

#[derive(Debug)]
pub enum DbUtilsError {
    Storage(storage::error::StorageError),
    Io(std::io::Error),
    Serialization(serde_json::Error),
    InvalidTable(String),
    DatabaseNotFound(std::path::PathBuf),
}

impl fmt::Display for DbUtilsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DbUtilsError::Storage(e) => write!(f, "Storage error: {}", e),
            DbUtilsError::Io(e) => write!(f, "IO error: {}", e),
            DbUtilsError::Serialization(e) => write!(f, "JSON serialization error: {}", e),
            DbUtilsError::InvalidTable(table) => write!(f, "Invalid table name: {}", table),
            DbUtilsError::DatabaseNotFound(path) => {
                write!(f, "Database file not found: {}", path.display())
            }
        }
    }
}

impl std::error::Error for DbUtilsError {}

impl From<storage::error::StorageError> for DbUtilsError {
    fn from(err: storage::error::StorageError) -> Self {
        DbUtilsError::Storage(err)
    }
}

impl From<std::io::Error> for DbUtilsError {
    fn from(err: std::io::Error) -> Self {
        DbUtilsError::Io(err)
    }
}

impl From<serde_json::Error> for DbUtilsError {
    fn from(err: serde_json::Error) -> Self {
        DbUtilsError::Serialization(err)
    }
}

pub type Result<T> = std::result::Result<T, DbUtilsError>;