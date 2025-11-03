use dashmap::DashMap;
use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
    sync::{Arc, LazyLock},
};

#[derive(Debug)]
pub struct LinkerHolder<K, T>
where
    K: Eq + std::hash::Hash + Debug + 'static,
    T: Debug + 'static,
{
    pub _inner: LazyLock<DashMap<K, Arc<T>>>,
}

impl<K, T> LinkerHolder<K, T>
where
    K: Eq + std::hash::Hash + Debug,
    T: Clone + Debug,
{
    #[inline]
    pub const fn new() -> Self {
        Self {
            _inner: LazyLock::new(|| DashMap::new()),
        }
    }
}

impl<K, T> Deref for LinkerHolder<K, T>
where
    K: Eq + std::hash::Hash + Debug,
    T: Clone + Debug,
{
    type Target = DashMap<K, Arc<T>>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self._inner
    }
}

impl<K, T> DerefMut for LinkerHolder<K, T>
where
    K: Eq + std::hash::Hash + Debug,
    T: Clone + Debug,
{
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self._inner
    }
}
