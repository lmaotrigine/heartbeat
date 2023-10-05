// Copyright (c) 2023 Isis <root@5ht2.me>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::devices::Device;
use chrono::Utc;
use sqlx::PgPool;

#[derive(Debug, Clone)]
pub struct Stats {
    pub last_seen: Option<chrono::DateTime<chrono::Utc>>,
    pub devices: Vec<Device>,
    pub longest_absence: chrono::Duration,
    pub num_visits: i64,
    pub total_beats: i64,
}

impl Stats {
    pub async fn fetch(pool: &PgPool) -> Self {
        let devs = sqlx::query!(
            r"
    WITH b AS (
        SELECT device, time_stamp FROM heartbeat.beats ORDER BY time_stamp DESC LIMIT 1
    )
    SELECT
        d.num_beats,
        MAX(b.time_stamp) AS last_beat,
        d.id,
        d.name
    FROM b, heartbeat.beats beats JOIN heartbeat.devices d ON beats.device = d.id
    GROUP BY d.id
    "
        )
        .fetch_all(pool)
        .await
        .unwrap_or_default();
        // can't find a way to not make this a second query
        let (visits, longest_absence) = sqlx::query!(
            "SELECT EXTRACT(epoch FROM longest_absence)::BIGINT as longest_absence, total_visits FROM heartbeat.stats;"
        )
        .fetch_optional(pool)
        .await
        .unwrap_or_default()
        .map_or((0, Some(0)), |v| (v.total_visits, v.longest_absence));
        let longest = chrono::Duration::seconds(longest_absence.unwrap_or_default());
        let mut devices = vec![];
        for dev in devs {
            devices.push(Device {
                id: dev.id,
                name: dev.name,
                last_beat: dev.last_beat,
                num_beats: dev.num_beats,
            });
        }
        let last_beat = devices.iter().max_by_key(|d| d.last_beat).and_then(|d| d.last_beat);
        let total_beats = devices.iter().map(|d| d.num_beats).sum();
        Self {
            last_seen: last_beat,
            devices,
            longest_absence: longest.max(Utc::now() - last_beat.unwrap_or_else(Utc::now)),
            num_visits: visits as _,
            total_beats,
        }
    }
}
