pub mod config;
pub mod heartbeat;
pub mod manager;
pub mod message;
pub mod selector;

// mod test {
//     use rand::{rng, seq::index::sample};
//     use rm_reth_types::{
//         peers::{PeerId, PeerPool},
//         socket::SocketAddrCodec,
//     };

//     use crate::selector::SelectionStrategy;

//     pub struct TestStrategy;

//     impl SelectionStrategy for TestStrategy {
//         fn select_peers(
//             &self,
//             available: &PeerPool,
//             count: usize,
//         ) -> Vec<(PeerId, SocketAddrCodec)> {
//             let random: Vec<usize> = sample(&mut rng(), available.len(), count)
//                 .into_iter()
//                 .collect();

//             let mut i = 0usize;
//             available
//                 .iter()
//                 .filter_map(|peer| {
//                     let peer = if random.contains(&i) {
//                         let pair = peer.pair();
//                         Some((pair.0.clone(), pair.1.clone()))
//                     } else {
//                         None
//                     };
//                     i += 1;
//                     peer
//                 })
//                 .collect()
//         }
//     }
// }
