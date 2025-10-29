use std::sync::{LazyLock, RwLock};

pub static CONFIG: LazyLock<RwLock<Config>> = LazyLock::new(|| RwLock::new(load_config()));

#[derive(Debug)]
pub struct Config {
    pub single_tx_max_size: usize,
    pub tx_max_size: usize,
}

pub fn load_config() -> Config {
    Config {
        single_tx_max_size: 1000,
        tx_max_size: 4000,
    }
}

#[inline]
pub fn get_config() -> std::sync::RwLockReadGuard<'static, Config> {
    CONFIG.read().expect("CONFIG read lock poisoned")
}
