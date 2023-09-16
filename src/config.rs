// Copyright (c) 2023 Isis <root@5ht2.me>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use serde::{Deserialize, Serialize};
use std::{fs::read_to_string, io::Error as IoError, path::Path};
use toml::{self, de::Error as TomlDeError};

/// The configuration for the server.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    /// Database configuration.
    pub database: Database,
    /// Webhook configuration.
    pub webhook: Webhook,
    /// A random URL-safe string used as a master Authorization header
    /// for adding new devices.
    pub secret_key: Option<String>,
    /// The GitHub repository URL of the project.
    pub repo: String,
    /// A human-readable name for the server used in <title> tags
    /// and other metadata.
    pub server_name: String,
    /// The publicly accessible URL of the server.
    pub live_url: String,
    /// Configuration related to automatic deployment using GitHub
    /// webhooks.
    pub github: GitHub,
    /// The bind address for the server. Must be parsable by [`std::net::ToSocketAddrs`].
    pub bind: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GitHub {
    /// The secret used to verify the authenticity of GitHub webhooks.
    pub webhook_secret: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Database {
    /// A PostgreSQL connection string.
    pub dsn: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Webhook {
    /// The URL of the Discord webhook.
    pub url: String,
    /// The minimum level of events that triggers a webhook.
    pub level: WebhookLevel,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
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

/// Represents errors that can occur while loading the configuration.
#[derive(Debug)]
pub enum Error {
    /// An [I/O error][`std::io::Error`].
    Io(IoError),
    /// A [TOML deserialization error][`toml::de::Error`]
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
    /// Tries to parse a [`Config`] from a `config.toml` file in the current directory.
    ///
    /// # Errors
    ///
    /// This function will return an error if the file does not exist, is not valid TOML, or
    /// does not contain the required fields.
    pub fn try_new() -> Result<Self, Error> {
        let config_str = read_to_string(Path::new("config.toml")).map_err(Into::<Error>::into)?;
        let config: Self = toml::from_str(&config_str)?;
        Ok(config)
    }
}
