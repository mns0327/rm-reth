use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::{error::TransactionError, pool_helper::TxPoolHelper, transaction::Transaction};

#[derive(Debug, Serialize, Deserialize, Encode, Decode)]
pub enum TxPool {
    Pending(TxPoolHelper),
    Ready(Vec<Transaction>),
    Invalid,
}

impl TxPool {
    pub fn new() -> Self {
        Self::Pending(TxPoolHelper::new())
    }

    pub async fn add_tx(&mut self, tx: Transaction) -> Result<(), TransactionError> {
        match self {
            Self::Pending(helper) => {
                if helper.add_tx(tx).await? {
                    self.finish();
                }
                Ok(())
            }
            _ => Err(TransactionError::TxPoolFinalized),
        }
    }

    pub fn finish(&mut self) {
        if let Self::Pending(helper) = std::mem::replace(self, Self::Invalid) {
            let mut pool = helper.pool();

            pool.shrink_to_fit();

            *self = Self::Ready(pool);
        }
    }
}
