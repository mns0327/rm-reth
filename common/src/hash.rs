use parity_scale_codec::{Encode, Decode};
use faster_hex::hex_encode;
use serde::{Deserialize, Serialize};
use std::{
    fmt::{Debug, Display},
    ops::Deref,
};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Encode, Decode, Default)]
pub struct Hash([u8; 32]);

impl Deref for Hash {
    type Target = [u8; 32];

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Hash {
    #[inline]
    pub fn empty() -> Self {
        [0u8; 32].into()
    }

    #[inline]
    pub fn as_hex(&self) -> String {
        let mut buf = [0u8; 64];
        hex_encode(&self.0, &mut buf).unwrap();
        unsafe { String::from_utf8_unchecked(buf.to_vec()) }
    }

    #[inline]
    pub fn hash(buf: &[u8]) -> Self {
        blake3::hash(buf).into()
    }
}

impl Into<Hash> for [u8; 32] {
    #[inline]
    fn into(self) -> Hash {
        Hash(self)
    }
}

impl Into<Hash> for blake3::Hash {
    #[inline]
    fn into(self) -> Hash {
        Hash(*self.as_bytes())
    }
}

impl Display for Hash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut buf = [0u8; 64];
        faster_hex::hex_encode(&self.0, &mut buf).unwrap();
        f.write_str("0x")?;
        f.write_str(unsafe { std::str::from_utf8_unchecked(&buf) })
    }
}

impl Debug for Hash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut buf = [0u8; 64];
        faster_hex::hex_encode(&self.0, &mut buf).unwrap();
        f.write_str("0x")?;
        f.write_str(unsafe { std::str::from_utf8_unchecked(&buf) })
    }
}
