pub mod bytes;
pub mod int;

pub use crate::bytes::FixedBytes;

pub type Address = FixedBytes<20>;
