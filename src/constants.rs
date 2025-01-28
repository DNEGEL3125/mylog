use std::{path::PathBuf, sync::LazyLock};

pub static PKG_NAME: &str = std::env!("CARGO_PKG_NAME");

pub const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

pub static CONFIG_DIR_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    dirs::home_dir()
        .expect("Home directory not found!")
        .join(".config")
        .join(PKG_NAME)
});

pub static CONFIG_FILE_PATH: LazyLock<PathBuf> =
    LazyLock::new(|| CONFIG_DIR_PATH.join("conf.toml"));
