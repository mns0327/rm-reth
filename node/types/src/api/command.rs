use std::sync::Arc;

use async_trait::async_trait;
use blake3::Hash;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use tokio::sync::Mutex;

use crate::{
    Address, api::stream::Stream, block::block::Block, int::Uint256, tx::transaction::Transaction,
};

pub const ERRORCODE: u8 = 255;

#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum NodeApiComand {
    Hello = 1,
    Bye = 2,
    CustemApi = 3,
    Done = 4,
    Error = ERRORCODE,
}

#[async_trait]
pub trait ApiHelper: Sized {
    type Value;

    fn from_byte(byte: u8) -> Option<Self>;

    fn as_byte(self) -> u8;

    async fn handler(&self, stream: Arc<Mutex<Stream>>, value: Self::Value) -> anyhow::Result<()>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum HostCommand {
    Hello = 1,
    Bye = 2,
    Done = 3,
    AddPeer = 4,
    Peer = 5,
    Error = ERRORCODE,
}

impl HostCommand {
    #[inline]
    fn from_byte(byte: u8) -> Self {
        Self::try_from(byte).unwrap_or(HostCommand::Error)
    }

    #[inline]
    fn as_byte(self) -> u8 {
        self.into()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum NodeCommand {
    Hello = 1,
    Bye = 2,
    Done = 3,
    AddPeer = 4,
    Peer = 5,
    CheckPeer = 6,
    Error = ERRORCODE,
}

impl NodeCommand {
    #[inline]
    pub fn from_byte(byte: u8) -> Self {
        Self::try_from(byte).unwrap_or(NodeCommand::Error)
    }

    #[inline]
    pub fn as_byte(self) -> u8 {
        self.into()
    }
}

#[repr(C)]
pub struct ApiErrorFrame<'a> {
    pub len: u8,
    pub cmd: u8,
    pub data: &'a [u8],
}

impl<'a> ApiErrorFrame<'a> {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(3 + self.data.len());
        buf.extend_from_slice(&[ERRORCODE, self.len, self.cmd]);
        buf.extend_from_slice(self.data);
        buf
    }
}
