// Copyright (c) 2023 VJ <root@5ht2.me>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use serde::{Deserialize, Serialize};
use std::{fs::read_to_string, io::Error as IoError, path::Path};
use toml::{self, de::Error as TomlDeError};

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub database: Database,
    pub webhook: Webhook,
    pub secret_key: Option<String>,
    pub repo: String,
    pub server_name: String,
    pub live_url: String,
    pub github: GitHub,
    pub bind: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct GitHub {
    pub webhook_secret: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Database {
    pub dsn: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Webhook {
    pub url: String,
    pub level: WebhookLevel,
}

#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
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
            "all" => Ok(Self::All),
            "new_devices" => Ok(Self::NewDevices),
            "long_absences" => Ok(Self::LongAbsences),
            "none" | "" => Ok(Self::None),
            _ => Err(format!("Invalid webhook level: {s}")),
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
            Self::Io(error) => write!(f, "IO error: {error}"),
            Self::Invalid(error) => write!(f, "TOML error: {error}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<IoError> for Error {
    fn from(error: IoError) -> Self {
        Self::Io(error)
    }
}

impl From<TomlDeError> for Error {
    fn from(error: TomlDeError) -> Self {
        Self::Invalid(error)
    }
}

impl Config {
    pub fn try_new() -> Result<Self, Error> {
        let config_str = read_to_string(Path::new("config.toml")).map_err(Into::<Error>::into)?;
        let config: Self = toml::from_str(&config_str)?;
        Ok(config)
    }
}
