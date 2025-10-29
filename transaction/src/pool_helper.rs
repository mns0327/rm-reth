use bincode::{
    BorrowDecode, Decode, Encode,
    de::{BorrowDecoder, Decoder},
    enc::Encoder,
};
use config::get_config;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::{
    error::TransactionError,
    transaction::{Transaction, TxSize},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct TxPoolHelper {
    #[serde(default)]
    #[serde(skip_serializing, skip_deserializing)]
    tx_size: Mutex<TxSize>,
    #[serde(default)]
    #[serde(skip_serializing, skip_deserializing)]
    pub pool: Vec<Transaction>,
}

impl TxPoolHelper {
    pub fn new() -> Self {
        Self {
            tx_size: Mutex::new(TxSize::default()),
            pool: vec![],
        }
    }

    pub fn pool(self) -> Vec<Transaction> {
        self.pool
    }

    pub async fn add_tx(&mut self, tx: Transaction) -> Result<(), TransactionError> {
        let mut gruad = self.tx_size.lock().await;
        let config = get_config();

        let tx_size = tx.size();

        if tx_size > config.single_tx_max_size {
            return Err(TransactionError::TxSingleSizeError);
        }

        if gruad.size + tx_size > config.tx_max_size {
            return Err(TransactionError::TxSizeError);
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
