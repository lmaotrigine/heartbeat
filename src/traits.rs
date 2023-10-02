// Copyright (c) 2023 Isis <root@5ht2.me>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use chrono::{DateTime, Utc};
use sqlx::PgPool;

/// Extension trait for [`PgPool`].
///
/// This is just a convenience wrapper around some common queries.
#[axum::async_trait]
pub trait PoolExt {
    /// Increment the number of visits to the site.
    async fn incr_visits(&self) -> sqlx::Result<()>;
    /// Get the server epoch (time of first ever deployment)
    async fn server_start_time(&self) -> DateTime<Utc>;
}

#[axum::async_trait]
impl PoolExt for PgPool {
    async fn incr_visits(&self) -> sqlx::Result<()> {
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

    async fn server_start_time(&self) -> DateTime<Utc> {
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
