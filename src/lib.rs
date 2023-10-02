#![forbid(unsafe_code)]
#![deny(missing_docs, clippy::all, clippy::pedantic, clippy::nursery, clippy::unwrap_used)]

//! A server to keep a live heartbeat (ping) of your devices.

use axum::extract::FromRef;
use chrono::{DateTime, Utc};
use parking_lot::Mutex;
use sqlx::{postgres::PgPoolOptions, PgPool};
use stats::Stats;
use std::sync::Arc;
use traits::PoolExt;

mod auth;
mod config;
mod devices;
mod error;
mod stats;
mod templates;
mod traits;
mod util;

pub mod routes;

pub use config::Config;
pub use error::handle_errors;

/// Crate version and git commit hash.
pub const VERSION: &str = concat!(env!("CARGO_PKG_VERSION"), "-", env!("HB_GIT_COMMIT"));

/// Global application state.
#[derive(Debug, Clone, FromRef)]
pub struct AppState {
    stats: Arc<Mutex<stats::Stats>>,
    pool: PgPool,
    config: Config,
    git_hash: &'static str,
    #[cfg(feature = "webhook")]
    webhook: util::Webhook,
    server_start_time: DateTime<Utc>,
}

impl AppState {
    /// Returns a new [`AppState`] from a [`Config`].
    ///
    /// This consumes the [`Config`] and should thus only be called after constructing
    /// all other components that take a reference to the [`Config`].
    ///
    /// # Errors
    ///
    /// This function will return an error if the database pool could not be created, or
    /// (for some reason), fetching the stats using the pool fails.
    pub async fn from_config(config: Config) -> sqlx::Result<Self> {
        #[cfg(feature = "webhook")]
        let webhook = util::Webhook::new(config.webhook.clone());
        let pool = PgPoolOptions::default()
            .max_connections(10)
            .connect(&config.database.dsn)
            .await?;
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
            git_hash: env!("HB_GIT_COMMIT"),
            #[cfg(feature = "webhook")]
            webhook,
            server_start_time,
        })
    }
}
