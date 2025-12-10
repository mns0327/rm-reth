use crate::{Address, int::Uint256};
use parity_scale_codec::{Decode, Encode};

#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
#[derive(Debug, Encode, Decode, PartialEq, Clone)]
pub struct Transaction {
    pub from: Address,
    pub to: Address,
    pub amount: Uint256,
    pub data: Vec<u8>,
}

impl Transaction {
    pub fn new(from: Address, to: Address, amount: Uint256, data: Vec<u8>) -> Self {
        Self {
            from,
            to,
            amount,
            data,
        }
    }

    #[inline]
    pub fn size(&self) -> usize {
        self.data.len()
    }

    pub fn dummy() -> Self {
        use rand::Rng;

        let mut rng = rand::rng();

        let value = rng.random::<u8>();

        Self {
            from: Address::default(),
            amount: Uint256::default(),
            to: Address::default(),
            data: vec![value],
        }
    }

    pub fn dummy_size(size: usize) -> Self {
        use rand::RngCore;

        let mut rng = rand::rng();

        let mut data = vec![0u8; size];

        rng.fill_bytes(&mut data);

        Self {
            from: Address::default(),
            amount: Uint256::default(),
            to: Address::default(),
            data,
        }
    }
}
