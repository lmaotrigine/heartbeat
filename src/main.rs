// Copyright (c) 2023 VJ <root@5ht2.me>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

#![forbid(unsafe_code)] // I am not a C programmer.
#![deny(clippy::all, clippy::cargo, clippy::pedantic, clippy::nursery)]
#![allow(
    clippy::cast_precision_loss, // quote from the docs: "this isn't bad at all"
    clippy::multiple_crate_versions, // dependency hell. idk.
)]

use axum::{extract::FromRef, middleware, Server};
use error::handle_errors;
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::{
    net::SocketAddr,
    sync::{Arc, Mutex, OnceLock},
};
use tower_http::services::ServeDir;
use tracing::warn;

mod config;
mod error;
mod guards;
mod models;
mod routes;
mod templates;
mod util;

use routes::query::fetch_stats;
pub use routes::query::SERVER_START_TIME;

#[derive(Debug, Clone, FromRef)]
pub struct AppState {
    stats: Arc<Mutex<models::Stats>>,
    pool: PgPool,
}
pub static GIT_HASH: OnceLock<&'static str> = OnceLock::new();
pub static CONFIG: OnceLock<config::Config> = OnceLock::new();

#[cfg(feature = "webhook")]
pub static WEBHOOK: OnceLock<util::Webhook> = OnceLock::new();

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    GIT_HASH
        .set(option_env!("HB_GIT_COMMIT").unwrap_or_default())
        .expect("GIT_HASH to not be set");
    let config = CONFIG
        .get_or_init(|| config::Config::try_new().expect("failed to load config file"))
        .clone();
    SERVER_START_TIME
        .get_or_init(|| routes::query::get_server_start_time(&config.database.dsn))
        .await;
    #[cfg(feature = "webhook")]
    WEBHOOK.get_or_init(|| util::Webhook::new(config.webhook.clone()));
    let pool = PgPoolOptions::default()
        .max_connections(10)
        .connect(&config.database.dsn)
        .await
        .unwrap();
    let stats = {
        let conn = pool.acquire().await.unwrap();
        Arc::new(Mutex::new(fetch_stats(conn).await))
    };
    let router = routes::get_all()
        .with_state(AppState { stats, pool })
        .fallback_service(ServeDir::new("static/"))
        .layer(middleware::from_fn(handle_errors));
    let graceful_shutdown = async {
        _ = tokio::signal::ctrl_c().await;
        warn!("Initiating graceful shutdown");
    };
    Server::bind(&config.bind.parse().unwrap())
        .serve(router.into_make_service_with_connect_info::<SocketAddr>())
        .with_graceful_shutdown(graceful_shutdown)
        .await
        .unwrap();
}
