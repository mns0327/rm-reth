use bincode::{
    BorrowDecode, Decode, Encode,
    de::{BorrowDecoder, Decoder},
    enc::Encoder,
};
use config::get_config;
use serde::{Deserialize, Serialize};

use crate::{error::TransactionError, transaction::Transaction};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct TxPoolHelper {
    pub tx_count: usize,
    pub pool_size: usize,
    #[serde(default)]
    #[serde(skip_serializing, skip_deserializing)]
    pub pool: Vec<Transaction>,
}

impl TxPoolHelper {
    pub fn new() -> Self {
        Self {
            tx_count: 0,
            pool_size: 0,
            pool: vec![],
        }
    }

    pub fn pool(self) -> Vec<Transaction> {
        self.pool
    }

    pub async fn add_tx(&mut self, tx: Transaction) -> Result<bool, TransactionError> {
        let config = get_config();

        let tx_size = tx.size();

        if tx_size > config.single_tx_max_size {
            return Err(TransactionError::TxSingleSizeError);
        }

        if self.pool_size + tx_size > config.tx_max_size {
            return Err(TransactionError::TxSizeError);
        }

        self.pool.push(tx);

        self.tx_count += 1;
        self.pool_size += tx_size;

        let result = (config.tx_max_size - self.pool_size) < config.min_tx_threshold;

        Ok(result)
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
