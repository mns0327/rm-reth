use thiserror::Error;

#[derive(Debug, Error)]
pub enum TypeError {
    #[error("bytes length error(expected: {expected}, got: {got})")]
    LengthError { expected: usize, got: usize },
}
