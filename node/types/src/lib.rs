pub mod bytes;
pub mod error;
pub mod int;
pub mod token;
pub mod block;
pub mod tx;
pub mod init;
pub mod hash;

pub use crate::bytes::FixedBytes;
use crate::token::TokenLinker;

pub type Address = FixedBytes<20>;

impl Address {
    #[inline]
    pub fn to_token_linker(&self) -> TokenLinker {
        TokenLinker::new(*self)
    }
}
