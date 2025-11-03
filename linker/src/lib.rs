use bincode::{
    BorrowDecode, Decode, Encode,
    de::{BorrowDecoder, Decoder},
    enc::Encoder,
};
use serde::{Deserialize, Serialize};
use std::{
    fmt::Debug,
    sync::{Arc, Weak},
};

pub mod holder;

pub use holder::LinkerHolder;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields, bound(deserialize = "H: Deserialize<'de>"))]
pub struct Linker<H, T> {
    pub id: H,
    #[serde(default)]
    #[serde(skip_serializing, skip_deserializing)]
    pub pointer: Weak<T>,
}

impl<K, T> Linker<K, T> {
    #[inline]
    pub fn new(id: K) -> Self {
        Self {
            id,
            pointer: Weak::new(),
        }
    }

    pub fn value(&self) -> Option<Arc<T>> {
        self.pointer.upgrade()
    }
}

// pub trait LinkerUtils<K, T> {
// }

// impl<K, T, U> LinkerUtils<K, T> for U
// where
//     U: InnerLinkerUtils<K, T> + Deref<Target = Linker<K, T>>,
// {
// }

pub trait InnerLinkerUtils<K, T> {
    type Error;

    fn load(&mut self) -> Result<(), Self::Error>;
    fn drop(&mut self);
}

impl<K, T> Encode for Linker<K, T>
where
    K: Encode,
{
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), bincode::error::EncodeError> {
        self.id.encode(encoder)?;
        Ok(())
    }
}

impl<K, T, Cx> Decode<Cx> for Linker<K, T>
where
    K: Decode<Cx>,
{
    fn decode<D: Decoder<Context = Cx>>(
        decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        let id = K::decode(decoder)?;
        Ok(Self {
            id,
            pointer: Weak::new(),
        })
    }
}

impl<'de, K, T, Cx> BorrowDecode<'de, Cx> for Linker<K, T>
where
    K: BorrowDecode<'de, Cx>,
{
    fn borrow_decode<D: BorrowDecoder<'de, Context = Cx>>(
        decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        let id = K::borrow_decode(decoder)?;
        Ok(Self {
            id,
            pointer: Weak::new(),
        })
    }
}

impl<H, T> Linker<H, T>
where
    H: Default + Encode,
{
    pub fn empty() -> Self {
        Self {
            id: H::default(),
            pointer: Weak::new(),
        }
    }
}
