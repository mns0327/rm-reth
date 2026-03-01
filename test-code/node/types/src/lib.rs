pub mod address;
pub mod api;
pub mod block;
pub mod bytes;
pub mod dashmap;
pub mod error;
pub mod hash;
pub mod init;
pub mod int;
pub mod peers;
pub mod socket;
pub mod token;
pub mod tx;

pub use crate::address::Address;

pub fn current_time() -> u64 {
    chrono::Utc::now().timestamp() as u64
}
