use std::sync::Arc;

use rm_reth_types::{
    peers::{PeerId, PeerPool},
    socket::SocketAddrCodec,
};

use crate::config::GossipConfig;

pub struct PeerSelector {
    strategy: Box<dyn SelectionStrategy>,
    config: Arc<GossipConfig>,
}

impl PeerSelector {
    pub fn new(strategy: Box<dyn SelectionStrategy>, config: Arc<GossipConfig>) -> Self {
        Self { strategy, config }
    }

    pub fn select_for_broadcast(&self, available: &PeerPool) -> Vec<(PeerId, SocketAddrCodec)> {
        self.strategy
            .select_peers(available, self.config.fanout_size)
    }
}

pub trait SelectionStrategy {
    fn select_peers(&self, available: &PeerPool, count: usize) -> Vec<(PeerId, SocketAddrCodec)>;
}
