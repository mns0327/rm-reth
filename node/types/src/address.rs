use std::ops::{Deref, DerefMut};

use parity_scale_codec::{Decode, Encode};
use redb::TypeName;
use serde::{Deserialize, Serialize};

pub use crate::bytes::FixedBytes;
use crate::{Key, token::TokenLinker};

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

impl From<Address> for FixedBytes<20> {
    #[inline]
    fn from(value: Address) -> Self {
        value.0
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

impl AsRef<Address> for Address {
    #[inline]
    fn as_ref(&self) -> &Address {
        &self
    }
}

impl Address {
    #[inline]
    pub fn to_token_linker(&self) -> TokenLinker {
        TokenLinker::new(*self)
    }
}

impl redb::Value for Address {
    type SelfType<'a>
        = Address
    where
        Self: 'a;

    type AsBytes<'a>
        = &'a [u8]
    where
        Self: 'a;

    fn fixed_width() -> Option<usize> {
        Some(20)
    }

    fn from_bytes<'a>(data: Self::AsBytes<'a>) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        Address(data.try_into().unwrap())
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
    where
        Self: 'b,
    {
        value.as_slice()
    }

    fn type_name() -> TypeName {
        TypeName::new("Address")
    }
}

impl redb::Key for Address {
    fn compare(data1: &[u8], data2: &[u8]) -> std::cmp::Ordering {
        data1.cmp(data2)
    }
}

impl Key for Address {}
