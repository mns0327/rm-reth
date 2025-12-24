use thiserror::Error;

#[derive(Debug, Error)]
pub enum TypeError {
    #[error("bytes length error(expected: {expected}, got: {got})")]
    LengthError { expected: usize, got: usize },

    #[error("parity scale codec error: ({0})")]
    CodecError(#[from] parity_scale_codec::Error),
}
