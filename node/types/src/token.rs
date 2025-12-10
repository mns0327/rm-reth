use std::sync::{Arc, Weak};

use linker::{InnerLinkerUtils, Linker, LinkerHolder};
use parity_scale_codec::{Decode, Encode};

#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

use crate::{Address, error::TypeError, int::Uint256};

pub static TOKEN_HOLDER: LinkerHolder<Address, Balance> = LinkerHolder::new();

#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
#[derive(Debug, Encode, Decode, PartialEq, Clone, Default)]
pub struct Balance {
    pub addr: Address,
    pub amount: Uint256,
}

impl Balance {
    #[inline]
    pub const fn new(addr: Address) -> Self {
        Self {
            addr,
            amount: Uint256::zero(),
        }
    }

    #[inline]
    pub fn checked_sub_amount(self, rhs: Uint256) -> Option<Self> {
        self.amount.checked_sub(rhs).map(|new_amt| Self {
            addr: self.addr,
            amount: new_amt,
        })
    }

    #[inline]
    pub fn checked_add_amount(self, rhs: Uint256) -> Option<Self> {
        self.amount.checked_add(rhs).map(|new_amt| Self {
            addr: self.addr,
            amount: new_amt,
        })
    }
}

pub type TokenLinker = Linker<Address, Balance>;

impl From<Address> for TokenLinker {
    #[inline]
    fn from(value: Address) -> Self {
        TokenLinker::new(value)
    }
}

impl InnerLinkerUtils<Address, Balance> for TokenLinker {
    type Error = TypeError;

    #[inline]
    fn load(&mut self) -> Result<(), Self::Error> {
        let block = TOKEN_HOLDER
            .entry(self.id)
            .or_insert(Arc::new(Balance::new(self.id)))
            .clone();
        self.pointer = Arc::downgrade(&block);

        Ok(())
    }

    #[inline]
    fn drop(&mut self) {
        self.pointer = Weak::new()
    }
}
