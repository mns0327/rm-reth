use bincode::{
    BorrowDecode, Decode, Encode,
    de::{BorrowDecoder, Decoder},
    enc::Encoder,
};
use serde::{Deserialize, Serialize};
use std::rc::Weak;

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields, bound(deserialize = "H: Deserialize<'de>"))]
pub struct Linker<T, H> {
    pub id: H,
    #[serde(default)]
    #[serde(skip_serializing, skip_deserializing)]
    pub pointer: Weak<T>,
}

impl<T, H> Encode for Linker<T, H>
where
    H: Encode,
{
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), bincode::error::EncodeError> {
        self.id.encode(encoder)?;
        Ok(())
    }
}

impl<T, H, Cx> Decode<Cx> for Linker<T, H>
where
    H: Decode<Cx>,
{
    fn decode<D: Decoder<Context = Cx>>(
        decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        let id = H::decode(decoder)?;
        Ok(Self {
            id,
            pointer: Weak::new(),
        })
    }
}

impl<'de, T, H, Cx> BorrowDecode<'de, Cx> for Linker<T, H>
where
    H: BorrowDecode<'de, Cx>,
{
    fn borrow_decode<D: BorrowDecoder<'de, Context = Cx>>(
        decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        let id = H::borrow_decode(decoder)?;
        Ok(Self {
            id,
            pointer: Weak::new(),
        })
    }
}

impl<T, H> Linker<T, H>
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
