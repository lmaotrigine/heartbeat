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
use lazy_static::lazy_static;
use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
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
use sqlx::{postgres::PgPoolOptions, PgPool};

#[derive(Debug, Clone, FromRef)]
pub struct AppState {
    stats: Arc<Mutex<models::Stats>>,
    pool: PgPool,
}

lazy_static! {
    pub static ref GIT_HASH: &'static str = option_env!("HB_GIT_COMMIT").map_or("", |s| s);
    pub static ref CONFIG: config::Config = config::Config::try_new().expect("failed to load config file");
}

#[cfg(feature = "webhook")]
lazy_static! {
    pub static ref WEBHOOK: util::Webhook = util::Webhook::new(&CONFIG.webhook);
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    lazy_static::initialize(&GIT_HASH);
    lazy_static::initialize(&CONFIG);
    SERVER_START_TIME
        .get_or_init(|| routes::query::get_server_start_time(&CONFIG.database.dsn))
        .await;
    #[cfg(feature = "webhook")]
    lazy_static::initialize(&WEBHOOK);
    let pool = PgPoolOptions::default()
        .max_connections(10)
        .connect(&CONFIG.database.dsn)
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
    Server::bind(&CONFIG.bind.parse().unwrap())
        .serve(router.into_make_service_with_connect_info::<SocketAddr>())
        .with_graceful_shutdown(graceful_shutdown)
        .await
        .unwrap();
}
