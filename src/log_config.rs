use chrono::NaiveDate;
use dirs::home_dir;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::{
    fs::{create_dir_all, File},
    io::{Read, Write},
    path::PathBuf,
};

use crate::constants::PKG_NAME; // You may need to add the `dirs` crate to your `Cargo.toml`

lazy_static! {
    pub static ref CONFIG_DIR_PATH: PathBuf = home_dir()
        .expect("Home directory not found!")
        .join(".config")
        .join(PKG_NAME);
    pub static ref CONFIG_FILE_PATH: PathBuf = CONFIG_DIR_PATH.join("conf.toml");
}

#[derive(Deserialize, Serialize)]
pub struct LogConfig {
    pub log_dir_path: PathBuf,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            log_dir_path: PathBuf::new(),
        }
    }
}

impl LogConfig {
    pub fn create_config_file_if_not_exists() {
        if CONFIG_FILE_PATH.exists() {
            return;
        }
        create_dir_all(CONFIG_DIR_PATH.clone()).expect("Can't create config file");
        let file = File::create(CONFIG_FILE_PATH.clone()).expect("Can't create config file");
        LogConfig::default().write_to_file(&file);
    }

    pub fn from_config_file() -> LogConfig {
        let mut file = File::open(CONFIG_FILE_PATH.clone()).expect("Can't create the config file");
        let mut content = String::new();
        file.read_to_string(&mut content)
            .expect("Can't read the config file");
        let config: LogConfig = toml::from_str(&content).expect("Invalid toml file");
        return config;
    }

    fn write_to_file(&self, mut file: &File) {
        let content = toml::to_string(self).expect("Unknown error");
        file.write(content.as_bytes())
            .expect("Can't write to the config file");
    }
}

pub fn construct_log_file_path(log_dir_path: &PathBuf, date: NaiveDate) -> PathBuf {
    let date_string = date.format("%Y-%m-%d").to_string();
    let filename = format!("{}.log", date_string);
    log_dir_path.join(filename)
}
