use num_enum::{IntoPrimitive, TryFromPrimitive};

pub const ERRORCODE: u8 = 255;

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
    pub fn from_byte(byte: u8) -> Self {
        Self::try_from(byte).unwrap_or(HostCommand::Error)
    }

    #[inline]
    pub fn as_byte(self) -> u8 {
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