#![forbid(unsafe_code)]
#![deny(missing_docs, clippy::all, clippy::pedantic, clippy::nursery, clippy::unwrap_used)]
#![allow(clippy::cast_precision_loss)] // quote from the docs: "this isn't bad at all"

//! A server to keep a live heartbeat (ping) of your devices.

use axum::extract::FromRef;
use chrono::{DateTime, Utc};
use parking_lot::Mutex;
use sqlx::{postgres::PgPoolOptions, PgPool};
use stats::Stats;
use std::sync::Arc;

mod config;
mod error;
mod guards;
mod models;
mod sealed;
mod stats;
mod templates;
mod util;

pub mod routes;

pub use config::Config;
pub use error::handle_errors;
pub use sealed::{ConnectionExt, PoolExt};

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
            let mut conn = pool.acquire().await?;
            (
                conn.server_start_time().await,
                Arc::new(Mutex::new(Stats::fetch(conn).await)),
            )
        };
        Ok(Self {
            stats,
            pool,
            config,
            git_hash: option_env!("HB_GIT_COMMIT").unwrap_or_default(),
            #[cfg(feature = "webhook")]
            webhook,
            server_start_time,
        })
    }
}
