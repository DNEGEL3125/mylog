use std::{path::PathBuf, sync::LazyLock};

pub const PKG_NAME: &str = std::env!("CARGO_PKG_NAME");

pub const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

pub static CONFIG_DIR_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    dirs::config_dir()
        .expect("Config directory not found!")
        .join(PKG_NAME)
});

pub static CONFIG_FILE_PATH: LazyLock<PathBuf> =
    LazyLock::new(|| CONFIG_DIR_PATH.join("conf.toml"));
