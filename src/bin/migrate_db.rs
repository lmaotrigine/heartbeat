#![forbid(unsafe_code)]
#![deny(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::unwrap_in_result,
    clippy::unwrap_used
)]

use clap::Parser;
use color_eyre::eyre::Result;
use serde::Deserialize;
use sqlx::PgPool;
use std::{fs::read_to_string, path::PathBuf};
use tracing::info;

#[derive(Parser)]
struct Config {
    #[clap(short, long, env = "HEARTBEAT_CONFIG_FILE")]
    config_file: Option<PathBuf>,
    #[clap(short, long, env = "HEARTBEAT_DATABASE_DSN")]
    database_dsn: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    heartbeat::init_logging();
    color_eyre::install()?;
    let conf = Config::parse();
    let dsn = if let Some(dsn) = conf.database_dsn {
        dsn
    } else {
        read_toml(conf)?
    };
    info!("Using DSN: {dsn}");
    let pool = PgPool::connect(&dsn).await?;
    info!("Running migrations...");
    Ok(sqlx::migrate!().run(&pool).await?)
}

fn read_toml(conf: Config) -> Result<String> {
    #[derive(Deserialize)]
    struct Toml {
        database: Database,
    }
    #[derive(Deserialize)]
    struct Database {
        dsn: String,
    }
    let c: Toml = toml::from_str(&read_to_string(
        conf.config_file.unwrap_or_else(|| "config.toml".into()),
    )?)?;
    Ok(c.database.dsn)
}
