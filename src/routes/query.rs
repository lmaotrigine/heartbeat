/**
 * Copyright (c) 2023 VJ <root@5ht2.me>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
use crate::{
    models::{Device, Stats},
    DbPool,
};
use chrono::Utc;
use rocket::tokio::sync::OnceCell;
use rocket_db_pools::Connection;

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
    .fetch_optional(&mut sqlx::PgPool::connect(dsn).await.unwrap().acquire().await.unwrap())
    .await
    .unwrap();
    match rec {
        Some(rec) => rec.server_start_time,
        None => now,
    }
}

pub async fn incr_visits(conn: &mut sqlx::PgConnection) {
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

pub async fn fetch_stats(mut conn: Connection<DbPool>) -> Stats {
    let devs = sqlx::query!(
        r#"
    WITH a AS (
        SELECT
            b.time_stamp - LAG(b.time_stamp) OVER (ORDER BY b.time_stamp) AS diff
        FROM beats b
    )
    SELECT
        COUNT(DISTINCT (b.device, b.time_stamp)) AS num_beats,
        EXTRACT('epoch' FROM MAX(a.diff))::BIGINT AS longest_absence,
        MAX(b.time_stamp) AS last_beat,
        d.id,
        d.name
    FROM a, beats b JOIN devices d ON b.device = d.id
    GROUP BY d.id
    "#
    )
    .fetch_all(&mut *conn)
    .await
    .unwrap();
    // can't find a way to not make this a second query
    let visits = match sqlx::query!("SELECT total_visits FROM stats;")
        .fetch_optional(&mut *conn)
        .await
        .unwrap()
    {
        Some(v) => v.total_visits,
        None => 0,
    } as u64;
    let longest = if devs.is_empty() {
        chrono::Duration::zero()
    } else {
        match devs[0].longest_absence {
            Some(d) => chrono::Duration::seconds(d),
            None => chrono::Duration::zero(),
        }
    };
    let mut devices = vec![];
    for dev in devs {
        devices.push(Device {
            id: dev.id,
            name: dev.name,
            last_beat: dev.last_beat,
            num_beats: dev.num_beats.unwrap_or_default() as u64,
        });
    }
    let last_beat = match devices.iter().max_by_key(|d| d.last_beat) {
        Some(d) => d.last_beat,
        None => None,
    };
    let total_beats = devices.iter().map(|d| d.num_beats).sum::<u64>();
    Stats {
        last_seen: last_beat,
        devices,
        longest_absence: longest.max(Utc::now() - last_beat.unwrap_or_else(Utc::now)),
        num_visits: visits,
        total_beats,
    }
}
