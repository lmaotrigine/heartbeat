// Copyright (c) 2023 Isis <root@5ht2.me>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

#[cfg(feature = "webhook")]
use crate::util::WebhookColour;
use crate::{
    auth::{Device as DeviceAuth, Master as MasterAuth},
    config::WebhookLevel,
    devices::{Device, PostDevice},
    error::Error,
    util::{generate_token, Snowflake, SnowflakeGenerator},
    AppState,
};
use axum::{
    extract::{
        ws::{Message, WebSocket},
        Path, State, WebSocketUpgrade,
    },
    http::StatusCode,
    response::Response,
    Json,
};
use chrono::Utc;
use serde::Serialize;
use sqlx::postgres::types::PgInterval;
use std::time::UNIX_EPOCH;
use tracing::{error, info};

#[allow(unused_variables)]
async fn fire_webhook(state: AppState, title: &str, message: &str, level: WebhookLevel) {
    #[cfg(not(feature = "webhook"))]
    return;
    #[cfg(feature = "webhook")]
    {
        let colour = match level {
            WebhookLevel::All => WebhookColour::Blue,
            WebhookLevel::NewDevices => WebhookColour::Green,
            WebhookLevel::LongAbsences => WebhookColour::Orange,
            WebhookLevel::None => return,
        };
        match state.webhook.execute(title, message, level, colour, state.config).await {
            Ok(()) => (),
            Err(e) => error!("{e}"),
        }
    }
}

#[axum::debug_handler]
pub async fn handle_beat_req(State(state): State<AppState>, info: DeviceAuth) -> (StatusCode, String) {
    let now = Utc::now();
    let prev_beat = sqlx::query!(
        r"
    WITH discard AS (
        INSERT INTO heartbeat.beats (time_stamp, device) VALUES ($1, $2)
    ),
    discard_this_too AS (
        UPDATE heartbeat.devices SET num_beats = num_beats + 1 WHERE id = $2
    ),
    longest_absence AS (
        SELECT longest_absence FROM heartbeat.stats
    )
    SELECT beats.time_stamp, longest_absence.longest_absence
    FROM heartbeat.beats beats, longest_absence
    ORDER BY time_stamp DESC
    LIMIT 1;
    ",
        now,
        info.id
    )
    .fetch_optional(&state.pool)
    .await
    .unwrap_or_else(|e| {
        error!("Failed to update database on successful beat: {e:?}");
        None
    });
    if let Some(record) = prev_beat {
        let diff = now - record.time_stamp;
        let update_longest_absence = {
            let mut ret = false;
            let mut w = state.stats.lock();
            if diff > w.longest_absence {
                w.longest_absence = diff;
                ret = true;
            }
            w.last_seen = Some(now);
            if let Some(x) = w.devices.iter_mut().find(|x| x.id == info.id) {
                x.last_beat = Some(now);
                x.num_beats += 1;
            }
            w.total_beats += 1;
            ret
        };
        if update_longest_absence {
            let pg_diff = PgInterval::try_from(chrono::Duration::microseconds(
                diff.num_microseconds().unwrap_or(i64::MAX - 1),
            ))
            .expect("We have travelled way too far into the future.");
            let _ = sqlx::query!(
                "UPDATE heartbeat.stats SET longest_absence = $1 WHERE _id = 0;",
                pg_diff
            )
            .execute(&state.pool)
            .await;
        }
        if diff.num_hours() >= 1 {
            fire_webhook(
                state.clone(),
                "Absence longer than 1 hour",
                &format!("From <t:{}> to <t:{}>", record.time_stamp.timestamp(), now.timestamp()),
                WebhookLevel::LongAbsences,
            )
            .await;
        }
    }
    let name = info.name.unwrap_or_else(|| format!("<unknown> ({})", info.id));
    info!(id = %info.id, "Successful beat from device {name}");
    fire_webhook(
        state,
        "Successful beat",
        &format!("From `{name}` on <t:{0}:D> at <t:{0}:T>", now.timestamp()),
        WebhookLevel::All,
    )
    .await;
    (StatusCode::OK, format!("{}", now.timestamp()))
}

fn get_stats(state: &AppState) -> impl Serialize {
    #[derive(Serialize)]
    struct StatsResp {
        last_seen: Option<i64>,
        last_seen_relative: i64,
        longest_absence: i64,
        num_visits: i64,
        total_beats: i64,
        devices: Vec<Device>,
        uptime: i64,
    }
    let r = state.stats.lock().clone();
    let last_seen_relative = (Utc::now() - r.last_seen.unwrap_or_else(|| UNIX_EPOCH.into())).num_seconds();
    StatsResp {
        last_seen: r.last_seen.map(|x| x.timestamp()),
        last_seen_relative,
        longest_absence: (if last_seen_relative > r.longest_absence.num_seconds() {
            last_seen_relative
        } else {
            r.longest_absence.num_seconds()
        }),
        num_visits: r.num_visits,
        total_beats: r.total_beats,
        devices: r.devices.as_slice().to_vec(),
        uptime: (Utc::now() - state.server_start_time).num_seconds(),
    }
}

#[axum::debug_handler]
pub async fn get_stats_(State(stats): State<AppState>) -> Json<impl Serialize> {
    Json(get_stats(&stats))
}

#[axum::debug_handler]
pub async fn realtime_stats(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(|ws| async { stream_stats(state, ws).await })
}

async fn stream_stats(app_state: AppState, mut ws: WebSocket) {
    loop {
        let stats = get_stats(&app_state);
        let _ = ws
            .send(Message::Text(serde_json::to_string(&stats).unwrap_or_default()))
            .await;
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}

#[derive(Serialize)]
pub struct DeviceAddResp {
    id: i64,
    name: Option<String>,
    token: String,
}

#[axum::debug_handler]
pub async fn post_device(
    _: MasterAuth,
    State(state): State<AppState>,
    Json(device): Json<PostDevice>,
) -> (StatusCode, Json<DeviceAddResp>) {
    let id = SnowflakeGenerator::default().generate();
    let res = match sqlx::query!(
        r"INSERT INTO heartbeat.devices (id, name, token) VALUES ($1, $2, $3) RETURNING *;",
        i64::try_from(id.id()).expect("snowflake out of i64 range. Is it 2089 already?"),
        device.name,
        generate_token(id),
    )
    .fetch_one(&state.pool)
    .await
    {
        Ok(record) => record,
        Err(e) => {
            error!("Failed to insert new device into database: {e:?}");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(DeviceAddResp {
                    id: -1,
                    name: None,
                    token: String::new(),
                }),
            );
        }
    };
    {
        let mut w = state.stats.lock();
        w.devices.push(Device {
            id: res.id,
            name: res.name.clone(),
            last_beat: None,
            num_beats: 0,
        });
    }
    fire_webhook(
        state,
        "New Device added",
        &format!(
            "A new device called `{}` was added on <t:{}:D> at <t:{}:T>",
            device.name,
            id.created_at().timestamp(),
            id.created_at().timestamp()
        ),
        WebhookLevel::NewDevices,
    )
    .await;

    (
        StatusCode::OK,
        Json(DeviceAddResp {
            id: res.id,
            name: res.name,
            token: res.token,
        }),
    )
}

#[axum::debug_handler]
pub async fn regenerate_device_token(
    _: MasterAuth,
    State(state): State<AppState>,
    Path(device_id): Path<i64>,
    method: axum::http::Method,
    uri: axum::http::Uri,
) -> Result<Json<impl Serialize>, Error> {
    #[derive(Serialize)]
    struct DeviceUpdateResp {
        id: i64,
        name: Option<String>,
        token: String,
    }

    let found = sqlx::query_scalar!(
        "SELECT EXISTS(SELECT 1 FROM heartbeat.devices WHERE id = $1);",
        device_id
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|e| {
        error!("Failed to check if device exists: {e:?}");
        Error::new(
            uri.path(),
            &method,
            StatusCode::INTERNAL_SERVER_ERROR,
            &state.config.server_name,
        )
    })?
    .unwrap_or(false);
    if !found {
        return Err(Error::new(
            uri.path(),
            &method,
            StatusCode::NOT_FOUND,
            &state.config.server_name,
        ));
    }
    let token = generate_token(Snowflake::from(device_id));
    let res = sqlx::query_as!(
        DeviceUpdateResp,
        "UPDATE heartbeat.devices SET token = $1 WHERE id = $2 RETURNING id, name, token;",
        token,
        device_id
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|e| {
        error!("Failed to update device token: {e:?}");
        Error::new(
            uri.path(),
            &method,
            StatusCode::INTERNAL_SERVER_ERROR,
            &state.config.server_name,
        )
    })?;
    Ok(Json(res))
}
