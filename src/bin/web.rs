// Copyright (c) 2023 Isis <root@5ht2.me>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

#![forbid(unsafe_code)]
#![deny(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::unwrap_in_result,
    clippy::unwrap_used
)]
#![cfg_attr(let_underscore_untyped_pedantic, allow(clippy::let_underscore_untyped))]

use axum::{middleware, Server};
use color_eyre::eyre::Result;
use std::net::SocketAddr;
use tower_http::{
    services::ServeDir,
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
};
use tracing::{info, span, warn, Instrument, Level};

use heartbeat::{handle_errors, routes::router, AppState, Config};

#[tokio::main]
async fn main() -> Result<()> {
    // early exit for --version
    if matches!(std::env::args().nth(1), Some(arg) if arg == "--version") {
        println!("Heartbeat v{}", heartbeat::VERSION);
        return Ok(());
    }
    tracing_subscriber::fmt::init();
    color_eyre::install()?;
    let config = Config::try_new()?;
    let bind = config.bind.parse::<SocketAddr>()?;
    let router = router(&config);
    let app_state = AppState::from_config(config).await?;
    let trace_service = TraceLayer::new_for_http()
        .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
        .on_response(DefaultOnResponse::new().level(Level::INFO));
    let router = router
        .with_state(app_state.clone())
        .fallback_service(ServeDir::new("static/"))
        .layer(middleware::from_fn_with_state(app_state, handle_errors))
        .layer(trace_service);
    let graceful_shutdown = async {
        let _ = tokio::signal::ctrl_c().await;
        warn!("Initiating graceful shutdown");
    };
    let server = Server::bind(&bind).serve(router.into_make_service_with_connect_info::<SocketAddr>());
    info!("Listening on {}", server.local_addr());
    Ok(server
        .with_graceful_shutdown(graceful_shutdown)
        .instrument(span!(Level::INFO, "server"))
        .await?)
}
