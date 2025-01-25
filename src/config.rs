// Copyright (c) 2023 Isis <root@5ht2.me>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use clap::{Arg, Args, FromArgMatches, Parser, Subcommand};
use erased_debug::Erased;
use heartbeat_sys::heartbeat_home;
use serde::Deserialize;
use std::{
    fmt::Debug,
    fs::read_to_string,
    io::Error as IoError,
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    path::{Path, PathBuf},
};
use toml::de::Error as TomlDeError;
use tracing::{debug, info};

#[doc = env!("CARGO_PKG_DESCRIPTION")]
#[derive(Debug, Parser)]
#[clap(about, author, version = crate::VERSION)]
#[clap(help_template = r"{name} {version}
{author-with-newline}{about-with-newline}
{usage-heading} {usage}

{all-args}")]
pub struct Cli {
    /// The subcommand to run.
    #[clap(subcommand)]
    pub subcommand: Option<Subcmd>,
}

/// Commands
#[derive(Debug, Subcommand)]
pub enum Subcmd {
    /// Run the web server.
    Run(Box<WebCli>),
    /// Generate a new secret key.
    GenKey,
    /// Migrate the database.
    #[cfg(feature = "migrate")]
    Migrate(MigrateCli),
}

impl Default for Subcmd {
    fn default() -> Self {
        Self::Run(Box::new(WebCli::parse()))
    }
}

/// Apply database migrations.
#[cfg(feature = "migrate")]
#[derive(Debug, Parser)]
pub struct MigrateCli {
    /// The path to the configuration file.
    #[command(flatten)]
    pub config_file: __ConfigFile,
    /// The `PostgreSQL` connection string. [default:
    /// `postgres://heartbeat@db/heartbeat` if running in Docker,
    /// `postgres://postgres@localhost/postgres` otherwise]
    #[clap(long, short, env = "HEARTBEAT_DATABASE_DSN")]
    pub database_dsn: Option<String>,
}

/// Run the web server.
#[derive(Debug, Parser)]
pub struct WebCli {
    /// A `PostgreSQL` connection string. [default:
    /// `postgres://heartbeat@db/heartbeat` if running in Docker,
    /// `postgres://postgres@localhost/postgres` otherwise]
    #[clap(long, short, env = "HEARTBEAT_DATABASE_DSN")]
    pub database_dsn: Option<String>,
    #[cfg(feature = "webhook")]
    #[clap(long, env = "HEARTBEAT_WEBHOOK_URL")]
    /// The URL of the Discord webhook. [default: none]
    pub webhook_url: Option<String>,
    #[cfg(feature = "webhook")]
    #[clap(long, env = "HEARTBEAT_WEBHOOK_LEVEL")]
    /// The minimum level of events that triggers a webhook. [default: none]
    pub webhook_level: Option<WebhookLevel>,
    /// A random URL-safe string used as a master Authorization header
    /// for adding new devices.
    #[clap(long, short = 's', env = "HEARTBEAT_SECRET_KEY")]
    pub secret_key: Option<String>,
    /// The GitHub repository URL of the project. [default: <https://github.com/lmaotrigine/heartbeat>]
    #[clap(long, short = 'r', env = "HEARTBEAT_REPO")]
    pub repo: Option<String>,
    /// A human-readable name for the server used in <title> tags
    /// and other metadata. [default: Some person's heartbeat]
    #[clap(long, env = "HEARTBEAT_SERVER_NAME")]
    pub server_name: Option<String>,
    /// The publicly accessible URL of the server. [default: `http://127.0.0.1:6060`]
    #[clap(long, short = 'u', env = "HEARTBEAT_LIVE_URL")]
    pub live_url: Option<String>,
    /// The bind address for the server. [default: `127.0.0.1:6060`]
    #[clap(long, short, env = "HEARTBEAT_BIND")]
    pub bind: Option<SocketAddr>,
    /// The path to the configuration file.
    #[command(flatten)]
    pub config_file: __ConfigFile,
}

#[derive(Debug)]
pub struct __ConfigFile {
    path: Option<PathBuf>,
}

impl std::ops::Deref for __ConfigFile {
    type Target = Option<PathBuf>;

    fn deref(&self) -> &Self::Target {
        &self.path
    }
}

impl FromArgMatches for __ConfigFile {
    fn from_arg_matches(matches: &clap::ArgMatches) -> Result<Self, clap::Error> {
        Ok(Self {
            path: matches.get_one::<PathBuf>("config-file").cloned(),
        })
    }

    fn update_from_arg_matches(&mut self, matches: &clap::ArgMatches) -> Result<(), clap::Error> {
        self.path = matches.get_one::<PathBuf>("config-file").cloned();
        Ok(())
    }
}

impl __ConfigFile {
    fn default() -> PathBuf {
        heartbeat_home()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join("config.toml")
    }
}

impl Args for __ConfigFile {
    fn augment_args(cmd: clap::Command) -> clap::Command {
        cmd.arg(
            Arg::new("config-file")
                .short('c')
                .long("config-file")
                .required(false)
                .help(format!(
                    "The path to the configuration file [default: {}]",
                    Self::default().display()
                ))
                .env("HEARTBEAT_CONFIG_FILE"),
        )
    }

    fn augment_args_for_update(cmd: clap::Command) -> clap::Command {
        cmd.arg(
            Arg::new("config-file")
                .short('c')
                .long("config-file")
                .required(false)
                .help(format!(
                    "The path to the configuration file [default: {}]",
                    Self::default().display()
                ))
                .env("HEARTBEAT_CONFIG_FILE"),
        )
    }
}

/// The configuration for the server.
#[derive(Debug)]
pub struct Config {
    /// Database configuration.
    pub database: Database,
    /// Webhook configuration.
    #[cfg(feature = "webhook")]
    pub webhook: Webhook,
    /// A random URL-safe string used as a master Authorization header
    /// for adding new devices.
    pub secret_key: Erased<String>,
    /// The GitHub repository URL of the project.
    pub repo: String,
    /// A human-readable name for the server used in <title> tags
    /// and other metadata.
    pub server_name: String,
    /// The publicly accessible URL of the server.
    pub live_url: String,
    /// The bind address for the server.
    pub bind: SocketAddr,
}

#[derive(Debug, Deserialize)]
pub struct Database {
    /// A `PostgreSQL` connection string.
    pub dsn: String,
}

#[cfg(feature = "webhook")]
#[derive(Debug, Deserialize)]
pub struct Webhook {
    /// The URL of the Discord webhook.
    pub url: String,
    /// The minimum level of events that triggers a webhook.
    pub level: WebhookLevel,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
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
    /// A field is missing from the configuration.
    MissingField(&'static str),
    /// The path to the configuration file is invalid.
    InvalidConfigPath(PathBuf),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(f, "IO error: {error}"),
            Self::Invalid(error) => write!(f, "TOML error: {error}"),
            Self::MissingField(field) => write!(f, "Missing field: {field}"),
            Self::InvalidConfigPath(path) => write!(f, "{} is not a file", path.display()),
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

// this handles all the lookup bits in the right order
// so it goes CLI -> env vars -> profile-specific overrides -> bare values in
// TOML -> hardcoded fallback
macro_rules! config_field {
    ($first:ident.$second:ident, $field:ident, $type:ty$(, $default:expr)?) => {
        fn $field(&self) -> Result<$type, Error> {
            let value: Result<_, Error> = if let Some(ref $field) = self.cli.$field {
                debug!($field = ?$field, "Read field from CLI (or env var)");
                Ok($field.to_owned())
            } else {
                let field = concat!(stringify!($first), ".", stringify!($second));
                self.toml_value_nested(stringify!($first), stringify!($second)).ok_or_else(|| Error::MissingField(field))
            };
            value$(.or_else(|_| { debug!($field = ?$default, "Using default value {:?} for field", $default);Ok($default)}))?
        }
    };
    ($field:ident, $type:ty$(, $default:expr)?) => {
        fn $field(&self) -> Result<$type, Error> {
            let value = if let Some(ref $field) = self.cli.$field {
                Ok::<_, Error>($field.to_owned())
            } else {
                self.toml_value(stringify!($field))
            };
            value$(.or_else(|_| {debug!($field = ?$default, "Using default value {:?} for field", $default);Ok($default)}))?
        }
    };
}

struct Merge<'a> {
    cli: WebCli,
    toml: &'a toml::Value,
}

#[inline]
fn is_docker() -> bool {
    let path = Path::new("/proc/self/cgroup");
    let dockerenv = Path::new("/.dockerenv");
    dockerenv.exists() || (read_to_string(path).map_or(false, |s| s.lines().any(|l| l.contains("docker"))))
}

impl<'a> Merge<'a> {
    #[cfg(debug_assertions)]
    const PROFILE: &'static str = "debug";
    #[cfg(not(debug_assertions))]
    const PROFILE: &'static str = "release";

    config_field!(database.dsn, database_dsn, String, {
        if is_docker() {
            String::from("postgres://heartbeat@db/heartbeat")
        } else {
            String::from("postgres://postgres@localhost/postgres")
        }
    });

    #[cfg(feature = "webhook")]
    config_field!(webhook.url, webhook_url, String, String::new());

    #[cfg(feature = "webhook")]
    config_field!(webhook.level, webhook_level, WebhookLevel, WebhookLevel::None);

    config_field!(secret_key, String, String::new());

    config_field!(repo, String, String::from("https://github.com/lmaotrigine/heartbeat"));

    config_field!(server_name, String, String::from("Some person's heartbeat"));

    config_field!(live_url, String, String::from("http://127.0.0.1:6060"));

    config_field!(
        bind,
        SocketAddr,
        SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 6060))
    );

    fn profile_value<T: Debug + Deserialize<'a>>(&self, field: &'a str) -> Option<T> {
        let value = self
            .toml
            .get(Self::PROFILE)
            .and_then(|v| v.get(field))
            .and_then(|v| T::deserialize(v.clone()).ok());
        if let Some(value) = &value {
            debug!(field = field, value = ?value, "Read field from `{}` TOML table", Self::PROFILE);
        }
        value
    }

    fn toml_value<T: Debug + Deserialize<'a>>(&self, field: &'static str) -> Result<T, Error> {
        self.profile_value(field)
            .or_else(|| {
                let value = self.toml.get(field).and_then(|v| T::deserialize(v.clone()).ok());
                if let Some(value) = &value {
                    debug!(field = field, value = ?value, "Read field from top-level TOML table");
                }
                value
            })
            .ok_or(Error::MissingField(field))
    }

    fn profile_value_nested<T: Debug + Deserialize<'a>>(&self, outer: &'a str, inner: &'a str) -> Option<T> {
        let value = self
            .toml
            .get(Self::PROFILE)
            .and_then(|v| v.get(outer))
            .and_then(|v| v.get(inner))
            .and_then(|v| T::deserialize(v.clone()).ok());
        if let Some(value) = &value {
            debug!(outer = outer, inner = inner, value = ?value, "Read field from `{}` TOML table", Self::PROFILE);
        }
        value
    }

    fn toml_value_nested<T: Debug + Deserialize<'a>>(&self, outer: &'static str, inner: &'a str) -> Option<T> {
        self.profile_value_nested(outer, inner).or_else(|| {
            let value = self
                .toml
                .get(outer)
                .and_then(|v| v.get(inner))
                .and_then(|v| T::deserialize(v.clone()).ok());
            if let Some(value) = &value {
                debug!(outer = outer, inner = inner, value = ?value, "Read field from top-level TOML table");
            }
            value
        })
    }

    pub fn try_into(self) -> Result<Config, Error> {
        Ok(Config {
            database: Database {
                dsn: self.database_dsn()?,
            },
            #[cfg(feature = "webhook")]
            webhook: Webhook {
                url: self.webhook_url()?,
                level: self.webhook_level()?,
            },
            secret_key: self.secret_key()?.into(),
            repo: self.repo()?,
            server_name: self.server_name()?,
            live_url: self.live_url()?,
            bind: self.bind()?,
        })
    }
}

impl Config {
    /// Tries to parse a [`Config`] from the command line arguments, environment
    /// variables, and a TOML configuration file.
    ///
    /// The TOML configuration file is located at `$HEARTBEAT_HOME/config.toml`
    /// by default, but can be changed with the `--config-file` command line
    /// argument. `$HEARTBEAT_HOME` is ~/.heartbeat by default.
    ///
    /// # Errors
    ///
    /// This function will return an error if a path was explicitly specified
    /// and doesn't point to a regular file, the file could not be read from
    /// (if it exists), is not valid TOML, or the required fields are not
    /// provided by any of the sources.
    pub fn try_new(cli: WebCli) -> Result<Self, Error> {
        let mut fail_on_not_exists = true;
        let config_path = cli.config_file.as_ref().map_or_else(
            || {
                fail_on_not_exists = false;
                __ConfigFile::default()
            },
            Clone::clone,
        );
        let toml_config = if config_path.is_file() {
            info!("Reading configuration from {}", config_path.display());
            let config_str = read_to_string(config_path).map_err(Into::<Error>::into)?;
            toml::from_str(&config_str)?
        } else if fail_on_not_exists {
            return Err(Error::InvalidConfigPath(config_path));
        } else {
            // just an empty table
            toml::Value::Table(toml::map::Map::new())
        };
        let config = Merge {
            cli,
            toml: &toml_config,
        };
        config.try_into()
    }
}
