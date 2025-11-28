use parity_scale_codec::{Decode, Encode};
use serde::{Deserialize, Serialize};
use crate::{Address, int::Uint256};

#[derive(Debug, Serialize, Deserialize, Encode, Decode, PartialEq, Clone)]
pub struct Transaction {
    pub from: Address,
    pub to: Address,
    pub amount: Uint256,
    pub data: Vec<u8>,
}

impl Transaction {
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
