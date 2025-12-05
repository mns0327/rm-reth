pub mod address;
pub mod api;
pub mod block;
pub mod bytes;
pub mod error;
pub mod hash;
pub mod init;
pub mod int;
pub mod token;
pub mod tx;

pub use crate::address::Address;

pub trait Key: redb::Key
where
    Self: for<'a> AsRef<Self::SelfType<'a>>,
{
}

pub trait Value: redb::Value + Default + Sized
where
    for<'a> Self: AsRef<Self::SelfType<'a>>,
    for<'a> Self: From<<Self as redb::Value>::SelfType<'a>>,
{
}
