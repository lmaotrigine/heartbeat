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

use axum::{extract::FromRef, middleware, Extension, Server};
use chrono::{DateTime, Utc};
use config::Config;
use error::handle_errors;
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tower_http::services::ServeDir;
use tracing::{span, warn, Instrument, Level};

mod config;
mod error;
mod guards;
mod models;
mod routes;
mod templates;
mod util;

use routes::{query::fetch_stats, shutdown::Shutdown};

#[derive(Debug, Clone, FromRef)]
pub struct AppState {
    stats: Arc<Mutex<models::Stats>>,
    pool: PgPool,
    config: Config,
    git_hash: &'static str,
    #[cfg(feature = "webhook")]
    webhook: util::Webhook,
    server_start_time: DateTime<Utc>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let config = Config::try_new().expect("failed to load config file");
    let server_start_time = routes::query::get_server_start_time(&config.database.dsn).await;
    let pool = PgPoolOptions::default()
        .max_connections(10)
        .connect(&config.database.dsn)
        .await
        .expect("create database pool to not fail");
    let stats = {
        let conn = pool.acquire().await.expect("wtf literally the first connection");
        Arc::new(Mutex::new(fetch_stats(conn).await))
    };
    let app_state = AppState {
        stats,
        pool,
        config: config.clone(),
        git_hash: option_env!("HB_GIT_COMMIT").unwrap_or_default(),
        #[cfg(feature = "webhook")]
        webhook: util::Webhook::new(config.webhook.clone()),
        server_start_time,
    };
    let (shutdown, signal) = Shutdown::new();
    let router = routes::get_all(&config)
        .with_state(app_state.clone())
        .fallback_service(ServeDir::new("static/"))
        .layer(middleware::from_fn_with_state(app_state, handle_errors))
        .layer(Extension(shutdown));
    #[allow(clippy::redundant_pub_crate)]
    let graceful_shutdown = async {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => (),
            _ = signal => (),
        }
        warn!("Initiating graceful shutdown");
    };
    Server::bind(&config.bind.parse().unwrap())
        .serve(router.into_make_service_with_connect_info::<SocketAddr>())
        .with_graceful_shutdown(graceful_shutdown)
        .instrument(span!(Level::INFO, "server"))
        .await
        .unwrap();
}
