use parity_scale_codec::{Decode, Encode, Output};
use serde::{Deserialize, Serialize};
use std::net::{Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};

#[repr(transparent)]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord, Clone, Copy)]
pub struct SocketAddrCodec(pub SocketAddr);

impl From<SocketAddr> for SocketAddrCodec {
    #[inline]
    fn from(a: SocketAddr) -> Self {
        Self(a)
    }
}

impl From<SocketAddrCodec> for SocketAddr {
    #[inline]
    fn from(w: SocketAddrCodec) -> Self {
        w.0
    }
}

impl Encode for SocketAddrCodec {
    #[inline]
    fn size_hint(&self) -> usize {
        match self.0 {
            SocketAddr::V4(_) => 1 + 4 + 2,
            SocketAddr::V6(_) => 1 + 16 + 2 + 4 + 4,
        }
    }

    #[inline]
    fn encode_to<T: Output + ?Sized>(&self, dest: &mut T) {
        match self.0 {
            SocketAddr::V4(v4) => {
                dest.push_byte(0);
                dest.write(&v4.ip().octets());
                dest.write(&v4.port().to_le_bytes());
            }
            SocketAddr::V6(v6) => {
                dest.push_byte(1);
                dest.write(&v6.ip().octets());
                dest.write(&v6.port().to_le_bytes());
                dest.write(&v6.flowinfo().to_le_bytes());
                dest.write(&v6.scope_id().to_le_bytes());
            }
        }
    }
}

impl Decode for SocketAddrCodec {
    #[inline]
    fn decode<I: parity_scale_codec::Input>(
        input: &mut I,
    ) -> Result<Self, parity_scale_codec::Error> {
        let tag = input.read_byte()?;
        match tag {
            0 => {
                let mut ip = [0u8; 4];
                input.read(&mut ip)?;
                let mut port = [0u8; 2];
                input.read(&mut port)?;
                let addr = SocketAddrV4::new(ip.into(), u16::from_le_bytes(port));
                Ok(Self(SocketAddr::V4(addr)))
            }
            1 => {
                let mut ip = [0u8; 16];
                input.read(&mut ip)?;
                let mut port = [0u8; 2];
                input.read(&mut port)?;
                let mut flow = [0u8; 4];
                input.read(&mut flow)?;
                let mut scope = [0u8; 4];
                input.read(&mut scope)?;
                let addr = SocketAddrV6::new(
                    Ipv6Addr::from(ip),
                    u16::from_le_bytes(port),
                    u32::from_le_bytes(flow),
                    u32::from_le_bytes(scope),
                );
                Ok(Self(SocketAddr::V6(addr)))
            }
            _ => Err("invalid socket addr tag".into()),
        }
    }
}
