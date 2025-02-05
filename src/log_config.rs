use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::{
    fs::{create_dir_all, File},
    io::{Read, Write},
    path::{Path, PathBuf},
};

use crate::constants::{CONFIG_DIR_PATH, CONFIG_FILE_PATH}; // You may need to add the `dirs` crate to your `Cargo.toml`

#[derive(Deserialize, Serialize, PartialEq, Debug)]
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
        let config_dir_path: &PathBuf = &CONFIG_DIR_PATH;
        let config_file_path: &PathBuf = &CONFIG_FILE_PATH;
        if config_file_path.exists() {
            return;
        }
        create_dir_all(config_dir_path).expect("Can't create config file");
        let file = File::create(config_file_path).expect("Can't create config file");
        LogConfig::default().write_to_file(&file);
    }

    pub fn from_config_file<P: AsRef<Path>>(file_path: P) -> LogConfig {
        let mut file = File::open(file_path).expect("Can't create the config file");
        let mut content = String::new();
        file.read_to_string(&mut content)
            .expect("Can't read the config file");
        toml::from_str(&content).expect("Invalid toml file")
    }

    fn write_to_file(&self, mut file: &File) {
        let content = toml::to_string(self).expect("Unknown error");
        file.write_all(content.as_bytes())
            .expect("Can't write to the config file");
    }
}

pub fn construct_log_file_path(log_dir_path: &Path, date: &NaiveDate) -> PathBuf {
    let date_string = date.format("%Y-%m-%d").to_string();
    let filename = format!("{}.log", date_string);
    log_dir_path.join(filename)
}

#[cfg(test)]
mod test {
    use super::LogConfig;

    #[test]
    fn test_loading_and_generating_config_file() {
        let (test_config_file, file_path) = crate::utils::fs::create_unique_temp_file();
        let mut log_config = LogConfig::default();
        log_config.log_dir_path = "/var/log/mylog".into();
        log_config.write_to_file(&test_config_file);
        std::mem::drop(test_config_file);
        assert_eq!(log_config, LogConfig::from_config_file(&file_path));
        std::fs::remove_file(&file_path).expect("Unable to delete the file");
    }
}
