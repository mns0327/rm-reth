use config::get_config;
use parity_scale_codec::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::tx::{error::TransactionError, transaction::Transaction};

#[derive(Debug, Serialize, Deserialize, Encode, Decode, Clone, PartialEq)]
pub struct TxPoolHelper {
    pub tx_count: u64,
    pub pool_size: u64,
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

        let tx_size = tx.size() as u64;

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
