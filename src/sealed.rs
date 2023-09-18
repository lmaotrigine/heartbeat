// Copyright (c) 2023 Isis <root@5ht2.me>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::ops::{Deref, DerefMut};

use axum::{extract::FromRequestParts, http::StatusCode};
use chrono::{DateTime, Utc};
use sqlx::{pool::PoolConnection, PgConnection, PgPool, Postgres};
use tracing::error;

use crate::{error::Error, AppState};

mod private {
    pub trait Sealed {}
}

impl private::Sealed for PgConnection {}
impl private::Sealed for PgPool {}

/// Extension trait for [`PgConnection`].
///
/// This is just a convenience wrapper around some common queries.
#[axum::async_trait]
pub trait ConnectionExt: private::Sealed {
    /// Increment the number of visits to the site.
    async fn incr_visits(&mut self) -> sqlx::Result<()>;
    /// Get the server epoch (time of first ever deployment)
    async fn server_start_time(&mut self) -> DateTime<Utc>;
}

/// Extension trait for [`PgPool`].
///
/// This is similar to [`ConnectionExt`].
#[axum::async_trait]
pub trait PoolExt: private::Sealed {
    /// Increment the number of visits to the site.
    async fn incr_visits(&mut self) -> sqlx::Result<()>;
}

pub struct Connection(PoolConnection<Postgres>);

impl Deref for Connection {
    type Target = PoolConnection<Postgres>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Connection {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Connection> for PoolConnection<Postgres> {
    fn from(conn: Connection) -> Self {
        conn.0
    }
}

#[axum::async_trait]
impl FromRequestParts<AppState> for Connection {
    type Rejection = Error;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        state
            .pool
            .acquire()
            .await
            .map_err(|e| {
                error!("Failed to acquire connection from pool: {e:?}");
                Error::new(
                    parts.uri.path(),
                    &parts.method,
                    StatusCode::INTERNAL_SERVER_ERROR,
                    &state.config.server_name,
                )
            })
            .map(Connection)
    }
}

#[axum::async_trait]
impl ConnectionExt for PgConnection {
    async fn incr_visits(&mut self) -> sqlx::Result<()> {
        sqlx::query!(
            r#"
            UPDATE heartbeat.stats
            SET total_visits = total_visits + 1
            RETURNING total_visits;
            "#
        )
        .fetch_one(self)
        .await?;
        Ok(())
    }

    async fn server_start_time(&mut self) -> DateTime<Utc> {
        let now = Utc::now();
        sqlx::query_scalar!(
            r#"
            WITH dummy AS (
                INSERT INTO heartbeat.stats (_id)
                VALUES (0)
                ON CONFLICT (_id) DO NOTHING
            )
            SELECT server_start_time
            FROM heartbeat.stats
            WHERE _id = 0;
            "#
        )
        .fetch_optional(self)
        .await
        .unwrap_or_default()
        .unwrap_or(now)
    }
}

#[axum::async_trait]
impl PoolExt for PgPool {
    async fn incr_visits(&mut self) -> sqlx::Result<()> {
        let mut conn = self.acquire().await?;
        conn.incr_visits().await
    }
}
