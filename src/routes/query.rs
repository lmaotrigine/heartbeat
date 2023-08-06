// Copyright (c) 2023 VJ <root@5ht2.me>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::models::{Device, Stats};
use chrono::Utc;
use sqlx::{pool::PoolConnection, PgConnection, Postgres};
use tokio::sync::OnceCell;

pub static SERVER_START_TIME: OnceCell<chrono::DateTime<Utc>> = OnceCell::const_new();

pub async fn get_server_start_time(dsn: &str) -> chrono::DateTime<Utc> {
    let now = Utc::now();
    let rec = sqlx::query!(
        r#"
            WITH dummy AS (
                INSERT INTO stats (_id)
                VALUES (0)
                ON CONFLICT (_id) DO NOTHING
            )
            SELECT server_start_time
            FROM stats
            WHERE _id = 0;
            "#
    )
    .fetch_optional(&mut *sqlx::PgPool::connect(dsn).await.unwrap().acquire().await.unwrap())
    .await
    .unwrap();
    rec.map_or(now, |rec| rec.server_start_time)
}

pub async fn incr_visits(conn: &mut PgConnection) {
    // ignore errors because they shouldn't happen
    sqlx::query!(
        r#"
        INSERT INTO STATS (_id, total_visits)
        VALUES (0, 1)
        ON CONFLICT (_id) DO UPDATE
        SET total_visits = stats.total_visits + 1;
        "#
    )
    .execute(conn)
    .await
    .unwrap_or_default();
}

#[allow(clippy::cast_sign_loss)]
pub async fn fetch_stats(mut conn: PoolConnection<Postgres>) -> Stats {
    let devs = sqlx::query!(
        r#"
    WITH b AS (
        SELECT device, time_stamp FROM beats ORDER BY time_stamp DESC LIMIT 1
    )
    SELECT
        d.num_beats,
        MAX(b.time_stamp) AS last_beat,
        d.id,
        d.name
    FROM b, beats JOIN devices d ON beats.device = d.id
    GROUP BY d.id
    "#
    )
    .fetch_all(&mut *conn)
    .await
    .unwrap();
    // can't find a way to not make this a second query
    let (visits, longest_absence) =
        sqlx::query!("SELECT EXTRACT(epoch FROM longest_absence)::BIGINT as longest_absence, total_visits FROM stats;")
            .fetch_optional(&mut *conn)
            .await
            .unwrap()
            .map_or((0, Some(0)), |v| (v.total_visits, v.longest_absence));
    let longest = chrono::Duration::seconds(longest_absence.unwrap_or_default());
    let mut devices = vec![];
    for dev in devs {
        devices.push(Device {
            id: dev.id,
            name: dev.name,
            last_beat: dev.last_beat,
            num_beats: dev.num_beats as u64,
        });
    }
    let last_beat = devices.iter().max_by_key(|d| d.last_beat).and_then(|d| d.last_beat);
    let total_beats = devices.iter().map(|d| d.num_beats).sum::<u64>();
    Stats {
        last_seen: last_beat,
        devices,
        longest_absence: longest.max(Utc::now() - last_beat.unwrap_or_else(Utc::now)),
        num_visits: visits as _,
        total_beats,
    }
}
