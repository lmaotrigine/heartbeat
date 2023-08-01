/**
 * Copyright (c) 2023 VJ <root@5ht2.me>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
use crate::config::WebhookLevel;
use crate::guards::Authorized;
use crate::util::{generate_token, SnowflakeGenerator};
use crate::DbPool;
use crate::{models::*, WrappedStats};
#[cfg(feature = "webhook")]
use crate::{util::WebhookColour, WEBHOOK};
use chrono::Utc;
use rocket::serde::json::Json;
use rocket::serde::json::Value;
use rocket::{get, post, State};
use rocket_db_pools::Connection;
use sqlx::postgres::types::PgInterval;

#[cfg(feature = "webhook")]
async fn fire_webhook(title: String, message: String, level: WebhookLevel) {
    let colour = match level {
        WebhookLevel::All => WebhookColour::Blue,
        WebhookLevel::NewDevices => WebhookColour::Green,
        WebhookLevel::LongAbsences => WebhookColour::Orange,
        WebhookLevel::None => return,
    };
    match WEBHOOK.execute(title, message, level, colour).await {
        Ok(()) => (),
        Err(e) => eprintln!("{}", e),
    }
}

#[cfg(not(feature = "webhook"))]
async fn fire_webhook(_title: String, _message: String, _level: WebhookLevel) {}

#[post("/api/beat")]
pub async fn handle_beat_req(mut conn: Connection<DbPool>, info: AuthInfo, stats: &State<WrappedStats>) -> String {
    let now = Utc::now();
    let prev_beat = sqlx::query!(
        r#"
    WITH dummy1 AS (
        INSERT INTO beats (time_stamp, device) VALUES ($1, $2)
    ),
    dummy2 AS (
        UPDATE devices SET num_beats = num_beats + 1 WHERE id = $2
    ),
    dummy3 AS (
        SELECT longest_absence FROM stats
    )
    SELECT beats.time_stamp, dummy3.longest_absence
    FROM beats, dummy3
    ORDER BY time_stamp DESC
    LIMIT 1;
    "#,
        now,
        info.id
    )
    .fetch_optional(&mut *conn)
    .await
    .unwrap();
    let mut w = stats.write().await;
    w.last_seen = Some(now);
    if let Some(x) = w.devices.iter_mut().find(|x| x.id == info.id) {
        x.last_beat = Some(now);
        x.num_beats += 1;
    }
    w.total_beats += 1;
    if let Some(prev) = prev_beat {
        let diff = now - prev.time_stamp;
        if diff > w.longest_absence {
            w.longest_absence = diff;
            drop(w);
            let pg_diff = PgInterval::try_from(chrono::Duration::microseconds(
                diff.num_microseconds().unwrap_or(i64::MAX - 1),
            ))
            .unwrap();
            sqlx::query!(
                r#"
                UPDATE stats SET longest_absence = $1 WHERE _id = 0;
                "#,
                pg_diff
            )
            .execute(&mut *conn)
            .await
            .unwrap();
        } else {
            drop(w);
        }
        if diff.num_hours() >= 1 {
            fire_webhook(
                "Absence longer than 1 hour".into(),
                format!("From <t:{}> to <t:{}>", prev.time_stamp.timestamp(), now.timestamp()),
                WebhookLevel::LongAbsences,
            )
            .await
        }
    }
    fire_webhook(
        "Successful beat".into(),
        format!(
            "From `{}` on <t:{}:D> at <t:{}:T>",
            info.name.unwrap_or_else(|| "unknown device".into()),
            now.timestamp(),
            now.timestamp()
        ),
        WebhookLevel::All,
    )
    .await;
    format!("{}", now.timestamp())
}

#[get("/api/stats")]
pub async fn get_stats(stats: &State<WrappedStats>) -> Value {
    let r = stats.read().await;
    let last_seen_relative = (Utc::now() - r.last_seen.unwrap_or(Utc::now())).num_seconds();
    rocket::serde::json::json!({
        "last_seen": r.last_seen.map(|x| x.timestamp()),
        "last_seen_relative": last_seen_relative,
        "longest_absence": (if last_seen_relative > r.longest_absence.num_seconds() {
            last_seen_relative
        } else {
            r.longest_absence.num_seconds()
        }),
        "num_visits": r.num_visits,
        "total_beats": r.total_beats,
        "devices": r.devices,
        "uptime": (Utc::now() - *crate::SERVER_START_TIME.get().unwrap()).num_seconds(),
    })
}

#[post("/api/devices", data = "<device>")]
pub async fn post_device(
    mut conn: Connection<DbPool>,
    _auth: Authorized,
    device: Json<PostDevice>,
    stats: &State<WrappedStats>,
) -> Value {
    let id = SnowflakeGenerator::default().generate();
    let res = sqlx::query!(
        r#"INSERT INTO devices (id, name, token) VALUES ($1, $2, $3) RETURNING *;"#,
        id.clone().id() as i64,
        device.name,
        generate_token(id),
    )
    .fetch_one(&mut *conn)
    .await
    .unwrap();
    let mut w = stats.write().await;
    w.devices.push(Device {
        id: res.id,
        name: res.name.clone(),
        last_beat: None,
        num_beats: 0,
    });
    fire_webhook(
        "New Device added".into(),
        format!(
            "A new device called `{}` was added on <t:{}:D> at <t:{}:T>",
            device.name,
            id.created_at().timestamp(),
            id.created_at().timestamp()
        ),
        WebhookLevel::NewDevices,
    )
    .await;
    rocket::serde::json::json!({
        "id": &res.id,
        "name": &res.name,
        "token": &res.token,
    })
}
