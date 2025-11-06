use parity_scale_codec::{Decode, Encode};
use serde::{Deserialize, Serialize};
use std::{
    fmt::Debug,
    sync::{Arc, Weak},
};

pub mod holder;

pub use holder::LinkerHolder;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields, bound(deserialize = "K: Deserialize<'de>"))]
pub struct Linker<K, T> {
    pub id: K,
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

pub trait InnerLinkerUtils<K, T> {
    type Error;

    fn load(&mut self) -> Result<(), Self::Error>;
    fn drop(&mut self);
}

impl<K, T> Encode for Linker<K, T>
where
    K: Encode,
{
    fn encode_to<U: parity_scale_codec::Output + ?Sized>(&self, dest: &mut U) {
        self.id.encode_to(dest);
    }
}

impl<K, T> Decode for Linker<K, T>
where
    K: Decode,
{
    fn decode<I: parity_scale_codec::Input>(
        input: &mut I,
    ) -> Result<Self, parity_scale_codec::Error> {
        let id = K::decode(input)?;
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
