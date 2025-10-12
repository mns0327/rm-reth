#[derive(thiserror::Error, Debug)]
pub enum TypeUtilError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("slice error: {0}")]
    SliceError(#[from] std::array::TryFromSliceError),
}
