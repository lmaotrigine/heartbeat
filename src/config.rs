use rocket::serde::{Deserialize, Serialize};
use std::{fs::read_to_string, io::Error as IoError, path::Path};
use toml::{self, de::Error as TomlDeError};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub database: DatabaseConfig,
    pub webhook: WebhookConfig,
    pub secret_key: Option<String>,
    pub repo: String,
    pub server_name: String,
    pub live_url: String,
}

#[derive(Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub dsn: String,
}

#[derive(Serialize, Deserialize)]
pub struct WebhookConfig {
    pub url: String,
    pub level: WebhookLevel,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum WebhookLevel {
    All,
    NewDevices,
    LongAbsences,
    None,
}

impl std::str::FromStr for WebhookLevel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "all" => Ok(WebhookLevel::All),
            "new_devices" => Ok(WebhookLevel::NewDevices),
            "long_absences" => Ok(WebhookLevel::LongAbsences),
            "none" => Ok(WebhookLevel::None),
            "" => Ok(WebhookLevel::None),
            _ => Err(format!("Invalid webhook level: {}", s)),
        }
    }
}

#[derive(Debug)]
pub enum Error {
    Io(IoError),
    Invalid(TomlDeError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Io(error) => write!(f, "IO error: {}", error),
            Error::Invalid(error) => write!(f, "TOML error: {}", error),
        }
    }
}

impl std::error::Error for Error {}

impl From<IoError> for Error {
    fn from(error: IoError) -> Self {
        Error::Io(error)
    }
}

impl From<TomlDeError> for Error {
    fn from(error: TomlDeError) -> Self {
        Error::Invalid(error)
    }
}

impl Config {
    pub fn try_new() -> Result<Self, Error> {
        let page_config = read_to_string(Path::new("config.toml")).map_err(Into::<Error>::into)?;
        let config: Config = toml::from_str(&page_config)?;
        Ok(config)
    }
}
