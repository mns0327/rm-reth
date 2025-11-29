pub mod api;
pub mod block;
pub mod bytes;
pub mod error;
pub mod hash;
pub mod init;
pub mod int;
pub mod token;
pub mod tx;

use std::ops::{Deref, DerefMut};

use parity_scale_codec::{Decode, Encode};
use serde::{Deserialize, Serialize};

pub use crate::bytes::FixedBytes;
use crate::token::TokenLinker;

// pub type Address = FixedBytes<20>;

#[repr(transparent)]
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
    Encode,
    Decode,
    Default,
)]

pub struct Address(FixedBytes<20>);

impl From<[u8; 20]> for Address {
    #[inline]
    fn from(value: [u8; 20]) -> Self {
        Self(value.into())
    }
}

impl Deref for Address {
    type Target = FixedBytes<20>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Address {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Address {
    #[inline]
    pub fn to_token_linker(&self) -> TokenLinker {
        TokenLinker::new(*self)
    }
}
