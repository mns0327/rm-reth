#[derive(thiserror::Error, Debug)]
pub enum ApiError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("slice error: {0}")]
    SliceError(#[from] std::array::TryFromSliceError),

    #[error("SCALE encoding failed: {0}")]
    ScaleError(#[from] parity_scale_codec::Error),
}
