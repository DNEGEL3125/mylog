use std::{fmt::Display, path::PathBuf};

use crate::constants::{CONFIG_DIR_ENV_VAR, PKG_NAME};

#[derive(Debug, PartialEq)]
pub enum Error {
    LogDirNotFound(PathBuf),
    DateParse(String),
    Io(std::io::ErrorKind),
    InvalidKey(String),
    EmptyLogMessage,
    DeserializeConfigFile(String),
    DetermineConfigDir,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LogDirNotFound(log_dir_path) => {
                let pkg_name = PKG_NAME;
                write!(
                    f,
                    "The log directory '{}' doesn't exist.\nYou can configure it by running `{} config log.dir <your-log-dir>`.",
                    log_dir_path.display(),
                    pkg_name
                )
            }
            Self::DateParse(date_string) => {
                write!(f, "error: invalid date: `{}`", date_string)
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
            Self::DeserializeConfigFile(error_message) => {
                write!(
                    f,
                    "error: fail to deserialize the config file: {}",
                    error_message
                )
            }
            Self::DetermineConfigDir => {
                write!(f, "error: could not determine the config directory\nTry setting the environment variable `{}` to customize your configuration directory.", CONFIG_DIR_ENV_VAR)
            }
        }
    }
}
