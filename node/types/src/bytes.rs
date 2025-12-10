use core::fmt;
use std::{borrow::Borrow, fmt::LowerHex};

use parity_scale_codec::{Decode, Encode};

#[cfg(feature = "json")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::error::TypeError;

#[repr(transparent)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode)]
pub struct FixedBytes<const N: usize>(pub [u8; N]);

impl<const N: usize> FixedBytes<N> {
    #[inline]
    pub const fn new(inner: [u8; N]) -> Self {
        Self(inner)
    }

    #[inline]
    pub const fn as_array(&self) -> &[u8; N] {
        &self.0
    }

    #[inline]
    pub const fn into_array(self) -> [u8; N] {
        self.0
    }

    #[inline]
    pub const fn as_slice(&self) -> &[u8] {
        &self.0
    }

    #[inline]
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }

    #[inline]
    pub const fn len(&self) -> usize {
        N
    }
}

impl<const N: usize> Default for FixedBytes<N> {
    #[inline]
    fn default() -> Self {
        Self([u8::default(); N])
    }
}

impl<const N: usize> AsRef<[u8]> for FixedBytes<N> {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl<const N: usize> Borrow<[u8]> for FixedBytes<N> {
    #[inline]
    fn borrow(&self) -> &[u8] {
        &self.0
    }
}

impl<const N: usize> From<[u8; N]> for FixedBytes<N> {
    #[inline]
    fn from(v: [u8; N]) -> Self {
        Self(v)
    }
}

impl<const N: usize> TryFrom<&[u8]> for FixedBytes<N> {
    type Error = TypeError;

    #[inline]
    fn try_from(s: &[u8]) -> Result<Self, Self::Error> {
        if s.len() == N {
            let mut a = [0u8; N];
            a.copy_from_slice(s);
            Ok(Self(a))
        } else {
            Err(TypeError::LengthError {
                expected: N,
                got: s.len(),
            })
        }
    }
}

impl<const N: usize> LowerHex for FixedBytes<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for b in &self.0 {
            write!(f, "{:02x}", b)?;
        }
        Ok(())
    }
}

impl<const N: usize> fmt::UpperHex for FixedBytes<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for b in &self.0 {
            write!(f, "{:02X}", b)?;
        }
        Ok(())
    }
}
impl<const N: usize> fmt::Display for FixedBytes<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::LowerHex::fmt(self, f)
    }
}


#[cfg(feature = "json")]
impl<const N: usize> Serialize for FixedBytes<N> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(&self.0)
    }
}

#[cfg(feature = "json")]
impl<'de, const N: usize> Deserialize<'de> for FixedBytes<N> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bytes = <&[u8]>::deserialize(deserializer)?;
        if bytes.len() != N {
            return Err(serde::de::Error::invalid_length(
                bytes.len(),
                &format!("expected {N} bytes").as_str(),
            ));
        }
        let mut arr = [0u8; N];
        arr.copy_from_slice(bytes);
        Ok(FixedBytes(arr))
    }
}
