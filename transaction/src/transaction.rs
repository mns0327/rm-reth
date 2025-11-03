use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Encode, Decode, PartialEq, Clone)]
pub struct Transaction(pub Vec<u8>);

impl Transaction {
    #[inline]
    pub fn size(&self) -> usize {
        self.0.len()
    }

    pub fn dummy() -> Self {
        use rand::Rng;

        let mut rng = rand::rng();

        let value = rng.random::<u8>();

        Self(vec![value])
    }

    pub fn dummy_size(size: usize) -> Self {
        use rand::RngCore;

        let mut rng = rand::rng();

        let mut value = vec![0u8; size];

        rng.fill_bytes(&mut value);

        Self(value)
    }
}
