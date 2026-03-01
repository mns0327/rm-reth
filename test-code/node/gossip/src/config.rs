use std::time::Duration;

#[derive(Debug, Clone)]
pub struct GossipConfig {
    // Message Config
    pub cache_size: u64,
    pub max_age: Duration,

    // Gossip Strategy Config
    pub fanout_size: usize,
    pub max_peers: usize,

    // Peer Config
    pub heartbeat_interval: Duration,
}
