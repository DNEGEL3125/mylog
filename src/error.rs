use std::{fmt::Display, path::PathBuf};

use crate::constants::CONFIG_FILE_PATH;

pub enum Error {
    LogDirNotFound(PathBuf),
    DateParse(String),
    Io(std::io::Error),
    InvalidKey(String),
    EmptyLogMessage,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LogDirNotFound(log_dir_path) => {
                let config_file_path = &CONFIG_FILE_PATH;
                write!(
                    f,
                    "The log directory '{}' doesn't exist.\nYou can configure it in '{}'",
                    log_dir_path.display(),
                    config_file_path.display()
                )
            }
            Self::DateParse(date_string) => {
                write!(f, "error: invalid date `{}`", date_string)
            }
            Self::Io(io_error) => {
                write!(f, "error: {}", io_error)
            }
            Self::InvalidKey(key) => {
                write!(f, "error: invalid key: `{}`", key)
            }
            Self::EmptyLogMessage => {
                write!(f, "Aborting due to empty log message.")
            }
        }
    }
}
