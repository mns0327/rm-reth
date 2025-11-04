use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Encode, Decode, PartialEq, Clone)]
pub struct Transaction {
    // pub from: Address,
    // pub to: Address,
    // pub amount: Uint256,
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

        Self { data: vec![value] }
    }

    pub fn dummy_size(size: usize) -> Self {
        use rand::RngCore;

        let mut rng = rand::rng();

        let mut value = vec![0u8; size];

        rng.fill_bytes(&mut value);

        Self { data: value }
    }
}
