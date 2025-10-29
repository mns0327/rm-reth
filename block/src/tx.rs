use bincode::{
    BorrowDecode, Decode, Encode,
    de::{BorrowDecoder, Decoder},
    enc::Encoder,
};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::error::BlockError;
use config::get_config;

#[derive(Debug, Serialize, Deserialize, Encode, Decode)]
pub enum TxPool {
    Pending(TxPoolHelper),
    Ready(Vec<Transaction>),
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct TxSize {
    count: usize,
    size: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TxPoolHelper {
    #[serde(default)]
    #[serde(skip_serializing, skip_deserializing)]
    tx_size: Mutex<TxSize>,
    #[serde(default)]
    #[serde(skip_serializing, skip_deserializing)]
    pool: Vec<Transaction>,
}

impl TxPoolHelper {
    pub fn new() -> Self {
        Self {
            tx_size: Mutex::new(TxSize::default()),
            pool: vec![],
        }
    }

    pub async fn add_tx(&mut self, tx: Transaction) -> Result<(), BlockError> {
        let mut gruad = self.tx_size.lock().await;
        let config = get_config();

        let tx_size = tx.size();

        if tx_size > config.single_tx_max_size {
            return Err(BlockError::TxSingleSizeError);
        }

        if gruad.size + tx_size > config.tx_max_size {
            return Err(BlockError::TxSizeError);
        }

        self.pool.push(tx);

        gruad.count += 1;
        gruad.size += tx_size;

        drop(gruad);
        Ok(())
    }
}

impl Encode for TxPoolHelper {
    fn encode<E: Encoder>(&self, _encoder: &mut E) -> Result<(), bincode::error::EncodeError> {
        Err(bincode::error::EncodeError::Other("TxPool is not ready"))
    }
}

impl<Cx> Decode<Cx> for TxPoolHelper {
    fn decode<D: Decoder<Context = Cx>>(
        _decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        Err(bincode::error::DecodeError::Other("TxPool is not ready"))
    }
}

impl<'de, Cx> BorrowDecode<'de, Cx> for TxPoolHelper {
    fn borrow_decode<D: BorrowDecoder<'de, Context = Cx>>(
        _decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        Err(bincode::error::DecodeError::Other("TxPool is not ready"))
    }
}

#[derive(Debug, Serialize, Deserialize, Encode, Decode)]
pub struct Transaction(String);

impl Transaction {
    #[inline]
    pub fn size(&self) -> usize {
        self.0.len()
    }
}
