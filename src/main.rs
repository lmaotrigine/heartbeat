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

use axum::{middleware, Server};
use base64ct::{Base64Url, Encoding};
use clap::Parser;
use color_eyre::eyre::Result;
use heartbeat::{handle_errors, routes::router, AppState, Cli, Config, Subcmd, WebCli};
use std::net::SocketAddr;
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing::{info, span, warn, Instrument, Level};

#[tokio::main]
async fn main() -> Result<()> {
    heartbeat::init_logging();
    color_eyre::install()?;
    let cli = Cli::parse();
    match cli.subcommand.unwrap_or_default() {
        Subcmd::Run(cli) => web(cli).await,
        #[cfg(feature = "migrate")]
        Subcmd::Migrate(cli) => migrate(cli).await,
        Subcmd::GenKey => gen_key(),
    }
}

async fn web(cli: WebCli) -> Result<()> {
    let config = Config::try_new(cli)?;
    info!(config = ?config, "Loaded config");
    let bind = config.bind;
    let router = router(&config);
    let app_state = AppState::from_config(config).await?;
    let trace_service = TraceLayer::new_for_http()
        .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
        .on_response(DefaultOnResponse::new().level(Level::INFO));
    let router = router
        .with_state(app_state.clone())
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

#[cfg(feature = "migrate")]
async fn migrate(cli: heartbeat::MigrateCli) -> Result<()> {
    use std::io;

    use heartbeat_sys::heartbeat_home;
    use sqlx::PgPool;
    let from_toml = || {
        let default = || {
            let mut path = heartbeat_home().ok()?;
            path.push("config.toml");
            Some(path)
        };
        let config = toml::from_str::<toml::Table>(&std::fs::read_to_string(
            cli.config_file
                .as_ref()
                .cloned()
                .or_else(default)
                .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "could not determine heartbeat home dir"))?,
        )?)?;
        let maybe_dsn = config
            .get("database")
            .and_then(|v| v.get("dsn"))
            .and_then(toml::Value::as_str)
            .map(String::from);
        maybe_dsn.ok_or_else(|| color_eyre::eyre::eyre!("Database DSN not provided."))
    };
    let dsn = if let Some(dsn) = cli.database_dsn {
        dsn
    } else {
        from_toml()?
    };
    info!("Using DSN: {dsn}");
    let pool = PgPool::connect(&dsn).await?;
    info!("Running migrations...");
    Ok(sqlx::migrate!().run(&pool).await?)
}

fn gen_key() -> color_eyre::Result<()> {
    use rand::RngCore;
    const NUM_BYTES: usize = 48;
    const STR_LEN: usize = 64;
    let mut buf = [0u8; NUM_BYTES];
    rand::thread_rng().fill_bytes(&mut buf);
    let mut s = [0u8; STR_LEN];
    println!("{}", Base64Url::encode(&buf, &mut s).map_err(color_eyre::Report::msg)?);
    Ok(())
}
