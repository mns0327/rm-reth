use std::{sync::Arc, time::Duration};

use moka::sync::Cache;

use rm_reth_types::peers::{PeerId, PeerPool};

pub struct PeerHeartBeat(pub Cache<PeerId, ()>);

impl PeerHeartBeat {
    pub fn new(pool: Arc<PeerPool>, max_capacity: u64, heartbeat_interval: Duration) -> Self {
        let cache: Cache<u32, _> = Cache::builder()
            .max_capacity(max_capacity)
            .time_to_live(heartbeat_interval)
            .eviction_listener(move |peer_id, _, _| {
                pool.remove(&peer_id);
            })
            .build();

        cache.run_pending_tasks();

        Self(cache)
    }

    pub fn sync_data_with_pool(&self, pool: &PeerPool) {
        let mut remove_ids: Vec<PeerId> = vec![];

        for peer in pool.iter() {
            if self.0.get(peer.key()).is_none() {
                remove_ids.push(peer.key().clone());
            }
        }

        for id in remove_ids {
            pool.remove(&id);
        }
    }
}
