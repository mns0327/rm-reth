use faster_hex::hex_encode;
use parity_scale_codec::{Decode, Encode};
use std::{
    fmt::{Debug, Display},
    ops::Deref,
};

#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

use crate::bytes::FixedBytes;

#[repr(transparent)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Encode, Decode, Default)]
pub struct Hash(FixedBytes<32>);

impl Deref for Hash {
    type Target = FixedBytes<32>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Hash {
    pub const fn empty() -> Self {
        Hash(FixedBytes([0u8; 32]))
    }

    #[inline]
    pub fn as_hex(&self) -> String {
        let mut buf = [0u8; 64];
        hex_encode(self.0.as_ref(), &mut buf).unwrap();
        unsafe { String::from_utf8_unchecked(buf.to_vec()) }
    }

    #[inline]
    pub fn hash(buf: &[u8]) -> Self {
        blake3::hash(buf).into()
    }
}

impl From<Hash> for [u8; 32] {
    #[inline]
    fn from(value: Hash) -> Self {
        value.into()
    }
}

impl Into<Hash> for [u8; 32] {
    #[inline]
    fn into(self) -> Hash {
        Hash(self.into())
    }
}

impl Into<Hash> for blake3::Hash {
    #[inline]
    fn into(self) -> Hash {
        Hash(FixedBytes(*self.as_bytes()))
    }
}

impl Display for Hash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut buf = [0u8; 64];
        faster_hex::hex_encode(self.0.as_ref(), &mut buf).unwrap();
        f.write_str("0x")?;
        f.write_str(unsafe { std::str::from_utf8_unchecked(&buf) })
    }
}

impl Debug for Hash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut buf = [0u8; 64];
        faster_hex::hex_encode(self.0.as_ref(), &mut buf).unwrap();
        f.write_str("0x")?;
        f.write_str(unsafe { std::str::from_utf8_unchecked(&buf) })
    }
}
