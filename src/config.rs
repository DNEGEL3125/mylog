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
    pub dir: String,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Default)]
pub struct Config {
    pub log: LogConfig,
}

impl Config {
    pub fn get_by_key(&self, key: &str) -> Option<&str> {
        match key {
            "log.dir" => Some(self.log.dir.as_ref()),
            _ => None,
        }
    }

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
        let mut file = File::open(file_path).map_err(|_| "fail to create the config file")?;
        let mut content = String::new();
        file.read_to_string(&mut content)
            .map_err(|_| "fail to read the config file")?;
        toml::from_str(&content)
            .map_err(|error| format!("fail to read the config file: {}", error.message()))
    }

    pub fn write_to_file(&self, mut file: &File) {
        let content = toml::to_string(self).expect("Unknown error");
        file.write_all(content.as_bytes())
            .expect("Can't write to the config file");
    }
}

/// This function updates a specific key in a TOML file with a new value.
/// It reads the entire file, parses it as TOML, updates the value of the given key,
/// and then writes the modified TOML back to the file.
pub fn set_by_key(config_file_path: &Path, key: &str, value: String) -> Result<(), String> {
    let file_content =
        std::fs::read_to_string(config_file_path).map_err(|error| error.to_string())?;
    let mut toml_doc = file_content
        .parse::<toml_edit::DocumentMut>()
        .map_err(|_| "invalid toml")?;
    let mut current_toml_node_opt: Option<&mut toml_edit::Item> = None;
    for key_part in key.split('.') {
        println!("key part: {}", key_part);
        let new_node: &mut toml_edit::Item;
        if let Some(current_toml_node) = current_toml_node_opt {
            println!("node: '{}'", current_toml_node);
            new_node = &mut current_toml_node[key_part];
        } else {
            println!("doc: '{}'", toml_doc);
            new_node = &mut toml_doc[key_part];
            //            new_node = toml_doc
            //                .get_mut(key_part)
            //                .unwrap_or(Err(format!("invalid key: {}", key))?);
        }
        if new_node.is_none() {
            Err(format!("invalid key: {}", key))?
        }
        current_toml_node_opt = Some(new_node);
    }
    if let Some(current_toml_node) = current_toml_node_opt {
        *current_toml_node = toml_edit::value(value);
        let mut config_file = File::create(config_file_path).map_err(|error| error.to_string())?;

        // Write the updated TOML content back to the config_file.
        config_file
            .write_all(toml_doc.to_string().as_bytes())
            .map_err(|error| error.to_string())?;

        // Ensure all buffered writes are written to the file.
        config_file.flush().map_err(|error| error.to_string())?;
    } else {
        return Err(format!("invalid key: {}", key));
    }

    Ok(())
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
    use crate::config::Config;

    #[test]
    fn test_loading_and_generating_config_file() {
        let (test_config_file, file_path) = crate::utils::fs::create_unique_temp_file();
        let mut log_config = Config::default();
        log_config.log.dir = "/var/log/mylog".into();
        log_config.write_to_file(&test_config_file);
        std::mem::drop(test_config_file);
        assert_eq!(Ok(log_config), Config::from_config_file(&file_path));
        std::fs::remove_file(&file_path).expect("Unable to delete the file");
    }
}
