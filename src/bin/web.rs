// Copyright (c) 2023 Isis <root@5ht2.me>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

#![forbid(unsafe_code)]
#![deny(clippy::all, clippy::pedantic, clippy::nursery)]

use axum::{middleware, Extension, Server};
use axum_shutdown::Shutdown;
use std::net::SocketAddr;
use tower_http::services::ServeDir;
use tracing::{span, warn, Instrument, Level};

use heartbeat::{handle_errors, routes::router, AppState, Config};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let config = Config::try_new().expect("failed to load config file");
    let bind = config.bind.parse::<SocketAddr>().unwrap();
    let router = router(&config);
    let app_state = AppState::from_config(config).await;
    let (shutdown, signal) = Shutdown::new();
    let router = router
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
    Server::bind(&bind)
        .serve(router.into_make_service_with_connect_info::<SocketAddr>())
        .with_graceful_shutdown(graceful_shutdown)
        .instrument(span!(Level::INFO, "server"))
        .await
        .unwrap();
}
