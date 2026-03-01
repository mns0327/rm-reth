use rm_reth_types::peers::{Peer, PeerPool};

use crate::message::{GossipMessage, MessageCache};

pub struct GossipManager {
    peer_pool: PeerPool,
    message_cache: MessageCache,
    // peer_selector: PeerSelector,
    // heartbeat: PeerHeartBeat,
    // config: Arc<GossipConfig>,
}

impl GossipManager {
    pub async fn broadcast_peers(&mut self, content: Peer) -> (Vec<Peer>, GossipMessage<Peer>) {
        let msg = GossipMessage::new(content);

        let peers = self
            .peer_pool
            .iter()
            .map(|p| (p.key().clone(), p.value().clone()))
            .collect();

        (peers, msg)
    }

    pub async fn handle_received(&mut self, msg: GossipMessage<Peer>) {
        if !self.message_cache.should_process_message(&msg.id) {
            return;
        }

        if msg.should_forward() {
            return;
        }

        self.peer_pool.insert(msg.content.0, msg.content.1);
    }

    // pub fn update_heartbeat(&self, id: PeerId) {
    //     self.heartbeat.0.insert(id, ());
    // }
}
