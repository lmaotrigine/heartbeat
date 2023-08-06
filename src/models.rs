// Copyright (c) 2023 VJ <root@5ht2.me>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::util::{deserialize_duration, serialize_duration, serialize_ts};

#[derive(Serialize)]
pub struct Beat {
    #[serde(serialize_with = "chrono::serde::ts_seconds::serialize")]
    time_stamp: chrono::DateTime<chrono::Utc>,
    device: Device,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Stats {
    #[serde(serialize_with = "serialize_ts")]
    pub last_seen: Option<chrono::DateTime<chrono::Utc>>,
    pub devices: Vec<Device>,
    #[serde(serialize_with = "serialize_duration")]
    #[serde(deserialize_with = "deserialize_duration")]
    pub longest_absence: chrono::Duration,
    pub num_visits: u64,
    pub total_beats: u64,
}

impl Stats {
    #[allow(clippy::cast_sign_loss)]
    pub async fn fetch(dsn: &str) -> Self {
        crate::SERVER_START_TIME
            .get_or_init(|| crate::routes::query::get_server_start_time(dsn))
            .await;
        let conn = sqlx::PgPool::connect(dsn).await.unwrap();
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
        .fetch_all(&conn)
        .await
        .unwrap();
        // can't find a way to not make this a second query
        let visits = sqlx::query!("SELECT total_visits FROM stats;")
            .fetch_optional(&conn)
            .await
            .unwrap()
            .map_or(0, |v| v.total_visits) as u64;
        let longest = if devs.is_empty() {
            chrono::Duration::zero()
        } else {
            devs[0]
                .longest_absence
                .map_or_else(chrono::Duration::zero, chrono::Duration::seconds)
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
        let last_beat = devices.iter().max_by_key(|d| d.last_beat).and_then(|d| d.last_beat);
        let total_beats = devices.iter().map(|d| d.num_beats).sum::<u64>();
        Self {
            last_seen: last_beat,
            devices,
            longest_absence: longest.max(Utc::now() - last_beat.unwrap_or_else(Utc::now)),
            num_visits: visits,
            total_beats,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Device {
    pub id: i64,
    pub name: Option<String>,
    #[serde(serialize_with = "serialize_ts")]
    pub last_beat: Option<chrono::DateTime<chrono::Utc>>,
    pub num_beats: u64,
}

#[derive(Deserialize)]
pub struct PostDevice {
    pub name: String,
}

pub struct AuthInfo {
    pub id: i64,
    pub name: Option<String>,
}
