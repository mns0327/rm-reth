use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
};

use crossbeam::queue::ArrayQueue;

use crate::tx::transaction::Transaction;

#[repr(transparent)]
#[derive(Debug)]
pub struct TransactionQueue(Arc<ArrayQueue<Transaction>>);

impl TransactionQueue {
    pub fn new() -> Self {
        Self(Arc::new(ArrayQueue::new(100)))
    }
}

impl Deref for TransactionQueue {
    type Target = Arc<ArrayQueue<Transaction>>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for TransactionQueue {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl AsRef<Arc<ArrayQueue<Transaction>>> for TransactionQueue {
    #[inline]
    fn as_ref(&self) -> &Arc<ArrayQueue<Transaction>> {
        &self.0
    }
}
