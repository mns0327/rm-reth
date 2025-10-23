use serde::{Deserialize, Serialize};

use crate::error::TypeUtilError;
use std::{collections::HashSet, net::SocketAddr};

pub mod api;
pub mod error;
pub mod server;

#[derive(Debug, Serialize, Deserialize)]
pub struct P2pPoints {
    pub peers: HashSet<SocketAddr>,
}

impl P2pPoints {
    pub fn new() -> Self {
        P2pPoints {
            peers: HashSet::new(),
        }
    }

    pub fn insert(&mut self, pfx: impl Into<SocketAddr>) {
        self.peers.insert(pfx.into());
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, TypeUtilError> {
        let config = bincode::config::standard();

        let buf = bincode::encode_to_vec(&self.peers, config)?;

        Ok(buf)
    }

    pub fn from_bytes(buf: &[u8]) -> Result<Self, TypeUtilError> {
        let config = bincode::config::standard();

        let (peers, _len) = bincode::decode_from_slice(buf, config)?;

        Ok(P2pPoints { peers })
    }
}

#[macro_export]
macro_rules! null_terminated {
    ($bytes:expr) => {{
        const BYTES: &[u8] = $bytes;

        const fn is_null_terminated(b: &[u8]) -> bool {
            match b {
                [] => false,
                _ => b[b.len() - 1] == 0,
            }
        }

        if is_null_terminated(BYTES) {
            BYTES
        } else {
            const LEN: usize = {
                let mut i = 0;
                while i < BYTES.len() {
                    i += 1;
                }
                i
            };

            const OUT: [u8; LEN + 1] = {
                let mut arr = [0u8; LEN + 1];
                let mut i = 0;
                while i < LEN {
                    arr[i] = BYTES[i];
                    i += 1;
                }
                arr
            };

            &OUT as &[u8]
        }
    }};
}

#[inline]
pub fn strip_null_terminator(bytes: &[u8]) -> &[u8] {
    match bytes.last() {
        Some(&0) => &bytes[..bytes.len() - 1],
        _ => bytes,
    }
}

// pub enum IpBytes {
//     V4([u8; 4]),
//     V6([u8; 16])
// }

// impl AsRef<[u8]> for IpBytes {
//     fn as_ref(&self) -> &[u8] {
//         match self {
//             Self::V4(v4) => v4,
//             Self::V6(v6) => v6
//         }
//     }
// }

// pub type Ipv4Bytes = u32;
// pub type Ipv6Bytes = u128;

// pub trait IpTypes {
//     type Warpper;
//     fn addr(&self) -> Self::Warpper;
// }

// impl IpTypes for Ipv4Bytes {
//     type Warpper = Ipv4Addr;

//     #[inline]
//     fn addr(&self) -> Self::Warpper {
//         Ipv4Addr::from(*self)
//     }
// }

// impl IpTypes for Ipv6Bytes {
//     type Warpper = Ipv6Addr;

//     #[inline]
//     fn addr(&self) -> Self::Warpper {
//         Ipv6Addr::from(*self)
//     }
// }

// #[derive(Debug)]
// pub struct P2pPointNode<T: IpTypes> {
//     pub prefix: T,
//     pub p_len: u8,      // 0x00: has value
//     pub child: [NodeLink<T>; 2]
// }

// impl <T>P2pPointNode<T>
// where
//     T: IpTypes + num_traits::PrimInt + Display + LowerHex + Copy
// {
//     const BIT_WIDTH: u8 = (std::mem::size_of::<T>() * 8) as u8;

//     #[inline]
//     pub fn prefix_gen(a: T, b: T, mut p_len: u8) -> u8 {
//         let mut end = Self::BIT_WIDTH;

//         while end - p_len > 1 {
//             let np_len = (end + p_len) / 2;
//             let shift = Self::BIT_WIDTH - np_len;
//             let mask = (!T::zero()).unsigned_shl(shift.into());

//             if (a & mask) == (b & mask) {
//                 p_len = np_len;
//             } else {
//                 end = np_len;
//             }
//         }

//         p_len
//     }
// }

// #[derive(Debug)]
// pub struct NodeLink<T: IpTypes>(pub Option<Box<P2pPointNode<T>>>);

// impl <T>NodeLink<T>
// where
//     T: IpTypes + num_traits::PrimInt + Display + LowerHex + Copy
// {
//     pub fn add_node(&mut self, pfx: impl Into<T>) -> Result<(), anyhow::Error> {
//         let pfx = pfx.into();

//         if let Some(node) = &mut self.0 {
//             let pfx = pfx.into();

//             let np_len = P2pPointNode::prefix_gen(node.prefix, pfx, node.p_len);
//             // drop node

//             let shift = P2pPointNode::<T>::BIT_WIDTH - np_len;

//             if node.p_len == 0 {
//                 let new_node = Box::new(P2pPointNode {
//                     prefix: pfx,
//                     p_len: 0,
//                     child: [NodeLink(None), NodeLink(None)]
//                 });

//                 let old_node = mem::replace(&mut self.0, None);

//                 let new_parant = Box::new(P2pPointNode {
//                     prefix: pfx & (!T::zero()).unsigned_shl(shift.into()),
//                     p_len: np_len,
//                     child: [NodeLink(Some(new_node)), NodeLink(old_node)]
//                 });

//                 self.0 = Some(new_parant);
//             } else {
//                 if node.p_len < np_len {
//                     if let Some(c_node) = &node.child[0].0 {
//                         if (c_node.prefix.unsigned_shr(shift.into())) ^ (pfx.unsigned_shr(shift.into())) == T::zero() {
//                             node.child[0].add_node(pfx)?;
//                         } else {
//                             node.child[1].add_node(pfx)?;
//                         }
//                     }
//                 } else if node.p_len > np_len {
//                     let new_node = Box::new(P2pPointNode {
//                         prefix: pfx,
//                         p_len: 0,
//                         child: [NodeLink(None), NodeLink(None)]
//                     });

//                     let old_node = mem::replace(&mut self.0, None);

//                     let new_parant = Box::new(P2pPointNode {
//                         prefix: pfx & (!T::zero()).unsigned_shl(shift.into()),
//                         p_len: np_len,
//                         child: [NodeLink(Some(new_node)), NodeLink(old_node)]
//                     });

//                     self.0 = Some(new_parant);
//                 }
//             }
//         } else {
//             let node = Box::new(
//                 P2pPointNode {
//                     prefix: pfx.into(),
//                     p_len: 0u8,
//                     child: [NodeLink(None), NodeLink(None)]
//                 }
//             );

//             self.0 = Some(node);
//         }

//         Ok(())
//     }

//     pub fn remove(&mut self) -> Result<(), anyhow::Error> {

//         Ok(())
//     }
// }

// #[derive(Debug)]
// pub struct P2pPoints {
//     pub v4_root: NodeLink<Ipv4Bytes>,
//     pub v6_root: NodeLink<Ipv6Bytes>
// }

// impl P2pPoints {
//     pub fn new() -> Self {
//         P2pPoints {
//             v4_root: NodeLink(None),
//             v6_root: NodeLink(None)
//         }
//     }

//     pub fn insert_prefix(&mut self, pfx: impl Into<IpAddr>) {
//         match pfx.into() {
//             IpAddr::V4(pfx) => {
//                 let _ = self.v4_root.add_node(pfx.to_bits()).unwrap();
//                 1
//             },
//             IpAddr::V6(_) => 1
//         };
//     }
// }
