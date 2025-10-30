use std::sync::{LazyLock, RwLock};

pub static CONFIG: LazyLock<RwLock<Config>> = LazyLock::new(|| RwLock::new(load_config()));

#[derive(Debug)]
pub struct Config {
    pub single_tx_max_size: usize,
    pub tx_max_size: usize,
    pub min_tx_threshold: usize,
}

pub fn load_config() -> Config {
    Config {
        single_tx_max_size: 100,
        tx_max_size: 1000,
        min_tx_threshold: 100,
    }
}

#[inline]
pub fn get_config() -> std::sync::RwLockReadGuard<'static, Config> {
    CONFIG.read().expect("CONFIG read lock poisoned")
}
