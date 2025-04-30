#![forbid(unsafe_code)]
#![deny(
    missing_docs,
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::unwrap_used,
    clippy::unwrap_in_result
)]

//! A server to keep a live heartbeat (ping) of your devices.

// if we're using reqwest, ensure a tls backend is configured
#[cfg(not(all(
    feature = "reqwest",
    any(feature = "tls-rustls", feature = "tls-native", feature = "tls-native-vendored")
)))]
compile_error!(
    "reqwest requires a TLS backend to be configured in order to fire webhooks, please enable one of tls-rustls, \
     tls-native, or tls-native-vendored. The former is recommended unless `rustls` does not run on your architecture \
     (e.g. mips)"
);

use axum::extract::FromRef;
use chrono::{DateTime, Utc};
use parking_lot::Mutex;
use sqlx::{postgres::PgPoolOptions, PgPool};
use stats::Stats;
use std::sync::Arc;
use tracing::Level;
use tracing_subscriber::util::SubscriberInitExt;
use traits::PoolExt;

mod auth;
mod config;
mod devices;
mod error;
mod server;
mod stats;
mod templates;
mod traits;
mod util;

pub mod routes;

#[cfg(feature = "migrate")]
pub use config::MigrateCli;
pub use config::{Cli, Config, Subcmd, WebCli};
pub use error::handle_errors;
pub use server::serve;

/// Crate version and git commit hash.
pub const VERSION: &str = env!("HB_VERSION");

/// Like [`tracing_subscriber::fmt::init`], but defaults to DEBUG on
/// debug builds.
pub fn init_logging() {
    let level = std::env::var("RUST_LOG")
        .map_err(|_| ())
        .and_then(|s| s.parse().map_err(|_| ()))
        .unwrap_or({
            if cfg!(debug_assertions) {
                Level::DEBUG
            } else {
                Level::INFO
            }
        });
    tracing_subscriber::fmt().with_max_level(level).finish().init();
}

/// Global application state.
#[derive(Debug, Clone, FromRef)]
pub struct AppState {
    stats: Arc<Mutex<stats::Stats>>,
    pool: PgPool,
    config: &'static Config,
    git_revision: &'static str,
    #[cfg(feature = "webhook")]
    webhook: Arc<util::Webhook>,
    server_start_time: DateTime<Utc>,
}

impl AppState {
    /// Returns a new [`AppState`] from a [`Config`].
    ///
    /// This consumes the [`Config`] and should thus only be called after
    /// constructing all other components that take a reference to the
    /// [`Config`].
    ///
    /// # Errors
    ///
    /// This function will return an error if the database pool could not be
    /// created, or (for some reason), fetching the stats using the pool
    /// fails.
    pub async fn from_config(config: &'static Config) -> sqlx::Result<Self> {
        #[cfg(feature = "webhook")]
        let webhook = util::Webhook::new(&config.webhook);
        let pool = PgPoolOptions::default()
            .max_connections(10)
            .connect_lazy(&config.database.dsn)?;
        let (server_start_time, stats) = {
            (
                pool.server_start_time().await,
                Arc::new(Mutex::new(Stats::fetch(&pool).await)),
            )
        };
        Ok(Self {
            stats,
            pool,
            config,
            git_revision: env!("HB_GIT_REVISION"),
            #[cfg(feature = "webhook")]
            webhook: Arc::new(webhook),
            server_start_time,
        })
    }
}
