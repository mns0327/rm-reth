use parity_scale_codec::{Decode, Encode};
use std::ops::Deref;

use crate::{dashmap::DashMapCodec, socket::SocketAddrCodec};

// TODO: to be gossip algorithm
#[repr(transparent)]
#[derive(Debug, Clone, Encode, Decode)]
pub struct Points {
    points: DashMapCodec<u32, SocketAddrCodec>,
}

impl Points {
    pub fn new() -> Self {
        Self {
            points: DashMapCodec::new(),
        }
    }

    #[inline]
    pub fn encode(&self) -> Vec<u8> {
        self.points.encode()
    }

    #[inline]
    pub fn decode(value: &[u8]) -> Option<Self> {
        let mut bytes = value;

        DashMapCodec::<u32, SocketAddrCodec>::decode(&mut bytes)
            .ok()
            .map(|points| Points { points })
    }
}

impl Deref for Points {
    type Target = DashMapCodec<u32, SocketAddrCodec>;

    fn deref(&self) -> &Self::Target {
        &self.points
    }
}

// // test sharing code
// pub struct SharingManager {
//     pub current_point_id: u32,
//     pub points: Arc<Points>,
//     pub cache_points: Arc<DashMap<u64, u32>>,
// }

// impl SharingManager {
//     pub fn get_share_points(&self) -> Vec<(u32, SocketAddrCodec)> {
//         self.points
//             .iter()
//             .filter(|point| *point.key() != self.current_point_id)
//             .map(|point| {
//                 let pair = point.pair();
//                 (pair.0.clone(), pair.1.clone())
//             })
//             .collect()
//     }
// }

// #[derive(Debug, Clone, Encode, Decode)]
// pub struct SharingSinglePoints {
//     id: u64,
//     propagation_score: u32,
//     new_point: (u32, SocketAddrCodec),
// }

// impl SharingSinglePoints {
//     #[inline]
//     pub fn id(&self) -> u64 {
//         self.id
//     }

//     #[inline]
//     pub fn points(&self) -> &(u32, SocketAddrCodec) {
//         &self.new_point
//     }

//     #[inline]
//     pub fn propagation_score(&self) -> u32 {
//         self.propagation_score
//     }

//     #[inline]
//     pub fn set_propagation_score(&mut self, score: u32) {
//         self.propagation_score = score;
//     }

//     pub fn from_point(new_point: (u32, SocketAddrCodec)) -> SharingSinglePoints {
//         let mut hasher = DefaultHasher::new();

//         new_point.encode().as_slice().hash(&mut hasher);

//         let id = hasher.finish();

//         Self {
//             id,
//             propagation_score: 0,
//             new_point,
//         }
//     }

//     pub fn next(&mut self) -> bool {
//         // hard coding
//         if self.propagation_score >= 10 {
//             return false;
//         }

//         self.propagation_score += 1;

//         return true;
//     }
// }

// #[cfg(test)]
// mod tests {
//     use std::{
//         collections::HashMap,
//         net::{IpAddr, Ipv4Addr, SocketAddr},
//         sync::Arc,
//         time::Duration,
//     };

//     use dashmap::DashMap;
//     use tokio::{sync::broadcast, time::sleep};

//     use crate::points::{Points, SharingManager, SharingSinglePoints};

//     fn new_socket(id: u8) -> SocketAddr {
//         SocketAddr::new(IpAddr::V4(Ipv4Addr::new(id, id, id, id)), u16::from(id))
//     }

//     fn create_manager(id: u32, points: &Points) -> Arc<SharingManager> {
//         Arc::new(SharingManager {
//             current_point_id: id,
//             points: Arc::new(points.clone()),
//             cache_points: Arc::new(DashMap::new()),
//         })
//     }

//     async fn handle_point_sharing(
//         mut receiver: broadcast::Receiver<SharingSinglePoints>,
//         sender_map: HashMap<u32, broadcast::Sender<SharingSinglePoints>>,
//         manager: Arc<SharingManager>,
//     ) {
//         while let Ok(mut new_point) = receiver.recv().await {
//             if let Some(score) = manager.cache_points.get(&new_point.id()) {
//                 if new_point.propagation_score() < *score {
//                     new_point.set_propagation_score(*score);
//                 }
//             }

//             manager
//                 .cache_points
//                 .insert(new_point.id(), new_point.propagation_score());

//             sleep(Duration::from_millis(10)).await;

//             for point in &manager.get_share_points() {
//                 if let Some(sender) = sender_map.get(&point.0) {
//                     if sender.send(new_point.clone()).is_err() {
//                         break;
//                     }
//                 }
//             }

//             if !new_point.next() {
//                 break;
//             }

//             manager
//                 .points
//                 .insert(new_point.points().0, new_point.points().1);
//         }
//     }

//     #[tokio::test]
//     async fn points_sharing() {
//         const CHANNEL_CAPACITY: usize = 20;
//         const NUM_MANAGERS: u32 = 3;

//         let points = Points::new();
//         for id in 0..NUM_MANAGERS {
//             points.insert(id, new_socket(id as u8).into());
//         }

//         let managers = (0..NUM_MANAGERS)
//             .map(|id| create_manager(id, &points))
//             .collect::<Vec<_>>();

//         let mut channels = HashMap::new();
//         let mut receivers = Vec::new();

//         for id in 0..NUM_MANAGERS {
//             let (sender, receiver) = broadcast::channel::<SharingSinglePoints>(CHANNEL_CAPACITY);
//             channels.insert(id, sender);
//             receivers.push(receiver);
//         }

//         let handlers = managers
//             .iter()
//             .cloned()
//             .zip(receivers)
//             .map(|(manager, receiver)| {
//                 let sender_map = channels.clone();
//                 tokio::spawn(handle_point_sharing(receiver, sender_map, manager))
//             })
//             .collect::<Vec<_>>();

//         sleep(Duration::from_millis(100)).await;

//         let new_point = SharingSinglePoints::from_point((3, new_socket(3).into()));
//         channels.get(&2).unwrap().send(new_point).unwrap();

//         drop(channels);

//         for handler in handlers {
//             handler.await.unwrap();
//         }

//         let encoded_points = managers.iter()
//             .map(|manager| manager.points.encode())
//             .collect::<Vec<_>>();

//         for i in 1..encoded_points.len() {
//             assert_eq!(encoded_points[0], encoded_points[i]);
//         }
//     }
// }
