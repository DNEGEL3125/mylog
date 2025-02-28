use std::{fmt::Display, path::PathBuf};

use crate::constants::{CONFIG_DIR_ENV_VAR, PKG_NAME};

#[derive(Debug)]
pub enum Error {
    LogDirNotFound(PathBuf),
    DateParse {
        input: String,
        source: chrono::ParseError,
    },
    Io(std::io::Error),
    InvalidKey(String),
    EmptyLogMessage,
    SerializeConfigFile(toml::ser::Error),
    DeserializeConfigFile(String),
    DetermineConfigDir,
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::DateParse { source, .. } => Some(source),
            Self::Io(err) => Some(err),
            Self::SerializeConfigFile(source) => Some(source),
            _ => None,
        }
    }
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
            Self::DateParse { input, source } => {
                write!(f, "invalid date '{}': {}", input, source)
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
            Self::SerializeConfigFile(source) => {
                write!(f, "fail to serialize the config file: {}", source)
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
