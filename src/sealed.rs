// Copyright (c) 2023 Isis <root@5ht2.me>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use chrono::{DateTime, Utc};
use sqlx::PgConnection;

mod private {
    pub trait Sealed {}
}

impl private::Sealed for PgConnection {}

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
        let rec = sqlx::query!(
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
        .unwrap_or_default();
        rec.map_or(now, |rec| rec.server_start_time)
    }
}
