use std::collections::BTreeSet;

use parity_scale_codec::{Decode, Encode};

use crate::api::socket::SocketAddrCodec;

#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
#[derive(Debug, Encode, Decode, Clone)]
pub struct P2pPoints {
    pub peers: BTreeSet<SocketAddrCodec>,
}

impl P2pPoints {
    pub fn new() -> Self {
        P2pPoints {
            peers: BTreeSet::new(),
        }
    }

    pub fn insert(&mut self, pfx: impl Into<SocketAddrCodec>) {
        self.peers.insert(pfx.into());
    }

    #[inline]
    pub fn to_bytes(&self) -> Vec<u8> {
        self.encode()
    }

    #[inline]
    pub fn from_bytes(buf: Vec<u8>) -> Option<Self> {
        Self::decode(&mut &buf[..]).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use parity_scale_codec::{Decode, Encode};
    use rand::{Rng, seq::SliceRandom};
    use std::{
        collections::BTreeSet,
        net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6},
    };

    fn gen_v4<R: Rng + ?Sized>(rng: &mut R) -> SocketAddr {
        let ip = Ipv4Addr::new(rng.random(), rng.random(), rng.random(), rng.random());
        let port = rng.random::<u16>();
        SocketAddr::V4(SocketAddrV4::new(ip, port))
    }

    fn gen_v6<R: Rng + ?Sized>(rng: &mut R) -> SocketAddr {
        let mut bytes = [0u8; 16];
        rng.fill_bytes(&mut bytes);
        let ip = Ipv6Addr::from(bytes);
        let port = rng.random::<u16>();
        let flowinfo = rng.random::<u32>();
        let scope_id = rng.random::<u32>();
        SocketAddr::V6(SocketAddrV6::new(ip, port, flowinfo, scope_id))
    }

    fn gen_addr<R: Rng + ?Sized>(rng: &mut R) -> SocketAddrCodec {
        if rng.random::<bool>() {
            SocketAddrCodec(gen_v4(rng))
        } else {
            SocketAddrCodec(gen_v6(rng))
        }
    }

    fn gen_set<R: Rng + ?Sized>(rng: &mut R, max_len: usize) -> BTreeSet<SocketAddrCodec> {
        let len = if max_len == 0 {
            0
        } else {
            rng.random::<u64>() as usize % (max_len + 1)
        };
        let mut set = BTreeSet::new();
        for _ in 0..len {
            set.insert(gen_addr(rng));
        }
        set
    }

    fn encode_bytes<T: Encode>(v: &T) -> Vec<u8> {
        v.encode()
    }

    #[test]
    fn p2p_points_btreeset_roundtrip_random() {
        let mut rng = rand::rng();
        const ITERS: usize = 500;
        const MAX_LEN: usize = 256;

        for _ in 0..ITERS {
            let peers = gen_set(&mut rng, MAX_LEN);
            let original = P2pPoints {
                peers: peers.clone(),
            };

            let bytes = encode_bytes(&original);
            let decoded: P2pPoints = P2pPoints::decode(&mut &bytes[..]).expect("valid decode");

            assert_eq!(decoded.peers, original.peers, "decoded must equal original");

            let bytes2 = encode_bytes(&decoded);
            assert_eq!(bytes2, bytes, "encoding must be deterministic");
        }
    }

    #[test]
    fn p2p_points_deterministic_order_independent_of_insertion() {
        let mut rng = rand::rng();

        let mut items: Vec<SocketAddrCodec> = (0..128).map(|_| gen_addr(&mut rng)).collect();

        let mut set_a = BTreeSet::new();
        for v in &items {
            set_a.insert(*v);
        }

        items.shuffle(&mut rng);
        let mut set_b = BTreeSet::new();
        for v in &items {
            set_b.insert(*v);
        }

        let a = P2pPoints { peers: set_a };
        let b = P2pPoints { peers: set_b };

        let enc_a = a.encode();
        let enc_b = b.encode();

        assert_eq!(enc_a, enc_b, "encoding must not depend on insertion order");
    }

    #[test]
    fn p2p_points_uniqueness_and_length_consistency() {
        let mut rng = rand::rng();

        let mut v = Vec::new();
        for _ in 0..200 {
            v.push(gen_addr(&mut rng));
            if rng.random::<u8>() % 3 == 0 {
                v.push(*v.last().unwrap());
            }
        }

        let unique: BTreeSet<_> = v.iter().cloned().collect();
        let p = P2pPoints {
            peers: unique.clone(),
        };

        let bytes = p.encode();
        let decoded: P2pPoints = P2pPoints::decode(&mut &bytes[..]).expect("decode ok");
        assert_eq!(
            decoded.peers.len(),
            unique.len(),
            "unique count must be preserved"
        );
        assert_eq!(decoded.peers, unique, "content must match");
    }

    #[test]
    fn p2p_points_ipv4_ipv6_mix_roundtrip() {
        let mut rng = rand::rng();

        let mut make_case = |kind: u8| {
            let mut set = BTreeSet::new();
            for _ in 0..200 {
                let addr = match kind {
                    0 => SocketAddrCodec(gen_v4(&mut rng)),
                    1 => SocketAddrCodec(gen_v6(&mut rng)),
                    _ => gen_addr(&mut rng),
                };
                set.insert(addr);
            }
            P2pPoints { peers: set }
        };

        for kind in 0..=2 {
            let case = make_case(kind);
            let enc = case.encode();
            let dec: P2pPoints = P2pPoints::decode(&mut &enc[..]).expect("decode ok");
            assert_eq!(
                dec.peers, case.peers,
                "roundtrip must hold for kind={}",
                kind
            );
            assert_eq!(enc, dec.encode(), "deterministic bytes for kind={}", kind);
        }
    }

    #[test]
    fn p2p_points_empty_set_roundtrip() {
        let p = P2pPoints {
            peers: BTreeSet::new(),
        };
        let enc = p.encode();
        let dec: P2pPoints = P2pPoints::decode(&mut &enc[..]).expect("decode ok");
        assert!(dec.peers.is_empty());
        assert_eq!(
            enc,
            dec.encode(),
            "encoding must be deterministic for empty set"
        );
    }
}
