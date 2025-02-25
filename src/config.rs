use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::{
    fs::{create_dir_all, File},
    io::{Read, Write},
    path::{Path, PathBuf},
};

use crate::constants::{CONFIG_DIR_PATH, CONFIG_FILE_PATH}; // You may need to add the `dirs` crate to your `Cargo.toml`

#[derive(Deserialize, Serialize, PartialEq, Debug, Default)]
pub struct LogConfig {
    pub directory: PathBuf,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Default)]
pub struct Config {
    pub log: LogConfig,
}

impl Config {
    pub fn create_config_file_if_not_exists() {
        let config_dir_path: &PathBuf = &CONFIG_DIR_PATH;
        let config_file_path: &PathBuf = &CONFIG_FILE_PATH;
        if config_file_path.exists() {
            return;
        }
        create_dir_all(config_dir_path).expect("Can't create config file");
        let file = File::create(config_file_path).expect("Can't create config file");
        Config::default().write_to_file(&file);
        println!(
            "Created the config file in `{}`",
            config_file_path.display()
        );
    }

    pub fn from_config_file<P: AsRef<Path>>(file_path: P) -> Result<Config, String> {
        let mut file = File::open(file_path).map_err(|_| "Can't create the config file")?;
        let mut content = String::new();
        file.read_to_string(&mut content)
            .map_err(|_| "Can't read the config file")?;
        toml::from_str(&content).map_err(|error| error.message().to_string())
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

pub fn get_date_from_log_file_name(file_name: &str) -> Option<NaiveDate> {
    if !file_name.ends_with(".log") {
        None
    } else {
        NaiveDate::parse_from_str(&file_name.replace(".log", ""), "%Y-%m-%d")
            .map(Some)
            .unwrap_or(None)
    }
}

#[cfg(test)]
mod test {
    use super::Config;

    #[test]
    fn test_loading_and_generating_config_file() {
        let (test_config_file, file_path) = crate::utils::fs::create_unique_temp_file();
        let mut log_config = Config::default();
        log_config.log.directory = "/var/log/mylog".into();
        log_config.write_to_file(&test_config_file);
        std::mem::drop(test_config_file);
        assert_eq!(log_config, Config::from_config_file(&file_path));
        std::fs::remove_file(&file_path).expect("Unable to delete the file");
    }
}
