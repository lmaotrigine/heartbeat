#![forbid(unsafe_code)]
#![deny(clippy::all, clippy::pedantic, clippy::nursery)]

use serde::Deserialize;
use sqlx::PgPool;
use std::{env, fs::read_to_string};
use tracing::{error, info};

#[derive(Deserialize)]
struct Config {
    database: Dsn,
}

#[derive(Deserialize)]
struct Dsn {
    dsn: String,
}

#[derive(Debug)]
enum Error {
    Sqlx(sqlx::Error),
    Migration(sqlx::migrate::MigrateError),
    Io(std::io::Error),
    Toml(toml::de::Error),
}

impl From<sqlx::Error> for Error {
    fn from(error: sqlx::Error) -> Self {
        Self::Sqlx(error)
    }
}

impl From<sqlx::migrate::MigrateError> for Error {
    fn from(error: sqlx::migrate::MigrateError) -> Self {
        Self::Migration(error)
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<toml::de::Error> for Error {
    fn from(error: toml::de::Error) -> Self {
        Self::Toml(error)
    }
}

#[tokio::main]
async fn main() {
    migrate().await.unwrap_or_else(|e| error!("{e:?}"));
}

async fn migrate() -> Result<(), Error> {
    tracing_subscriber::fmt::init();
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
