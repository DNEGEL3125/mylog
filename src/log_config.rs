use chrono::NaiveDate;
use dirs::home_dir;
use std::path::PathBuf; // You may need to add the `dirs` crate to your `Cargo.toml`

pub struct LogConfig {
    pub app_config_dir: PathBuf,
    pub config_json_path: PathBuf,
    pub log_dir_path: PathBuf,
}

impl LogConfig {
    pub fn new() -> Self {
        let app_config_dir: PathBuf = match home_dir() {
            Some(home) => home.join(".config/myapp"),
            None => panic!("Home directory not found!"),
        };
        let config_json_path = app_config_dir.join("log-config.json");

        LogConfig {
            config_json_path,
            app_config_dir,
            log_dir_path: PathBuf::from("/Users/dnegel3125/Documents/.private/MyLogs"),
        }
    }
}

pub fn construct_log_file_path(log_dir_path: &PathBuf, date: NaiveDate) -> PathBuf {
    let date_string = date.format("%Y-%m-%d").to_string();
    let filename = format!("{}.log", date_string);
    log_dir_path.join(filename)
}
