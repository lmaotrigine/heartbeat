#![forbid(unsafe_code)]
#![deny(clippy::all, clippy::pedantic, clippy::nursery)]

use color_eyre::eyre::Result;
use serde::Deserialize;
use sqlx::PgPool;
use std::{env, fs::read_to_string};
use tracing::info;

#[derive(Deserialize)]
struct Config {
    database: Dsn,
}

#[derive(Deserialize)]
struct Dsn {
    dsn: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    color_eyre::install()?;
    let dsn = if let Ok(dsn) = env::var("DATABASE_URL") {
        dsn
    } else {
        let conf_file = read_to_string("config.toml")?;
        let config: Config = toml::from_str(&conf_file)?;
        config.database.dsn
    };
    info!("Using DSN: {dsn}");
    let pool = PgPool::connect(&dsn).await?;
    info!("Running migrations...");
    Ok(sqlx::migrate!().run(&pool).await?)
}
