use serde::{Deserialize, Serialize};

use crate::error::ApiError;
use std::{collections::HashSet, net::SocketAddr};

pub mod command;
pub mod error;
pub mod handler;
pub mod stream;

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

    pub fn to_bytes(&self) -> Result<Vec<u8>, ApiError> {
        let config = bincode::config::standard();

        let buf = bincode::encode_to_vec(&self.peers, config)?;

        Ok(buf)
    }

    pub fn from_bytes(buf: &[u8]) -> Result<Self, ApiError> {
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
