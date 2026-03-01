use std::{
    hash::{DefaultHasher, Hash, Hasher},
    time::Duration,
};

use moka::sync::Cache;
use parity_scale_codec::{Decode, Encode};
use rm_reth_types::current_time;

static MAX_MESSAGE_AGE: u64 = 60;

type MessageId = u64;

#[repr(transparent)]
#[derive(Debug, Clone)]
pub struct MessageCache(Cache<MessageId, ()>);

impl MessageCache {
    pub fn new(max_capacity: u64, ttl: Duration) -> Self {
        let cache = Cache::builder()
            .max_capacity(max_capacity)
            .initial_capacity(max_capacity as usize)
            .time_to_live(ttl)
            .build();

        Self(cache)
    }

    pub fn should_process_message(&mut self, msg_id: &MessageId) -> bool {
        if self.0.contains_key(msg_id) {
            return false;
        }

        self.0.insert(msg_id.clone(), ());
        true
    }
}

pub struct GossipMessage<T: Encode + Decode> {
    pub id: MessageId,
    pub content: T,
    timestamp: u64,
}

impl<T: Encode + Decode> GossipMessage<T> {
    pub fn new(content: T) -> Self {
        let mut hasher = DefaultHasher::new();

        let encoded = content.encode();

        encoded.as_slice().hash(&mut hasher);

        let id = hasher.finish();

        Self {
            id,
            content,
            timestamp: current_time(),
        }
    }

    pub fn should_forward(&self) -> bool {
        (current_time() - self.timestamp) < MAX_MESSAGE_AGE
    }
}
