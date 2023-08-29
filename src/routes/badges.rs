// Copyright (c) 2023 VJ <root@5ht2.me>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::sync::OnceLock;

use super::query::incr_visits;
use crate::{
    util::{badge, formats::FormatNum, hf_time::HumanTime},
    AppState,
};
use axum::{extract::State, http::Response, http::StatusCode, response::IntoResponse};
use base64::prelude::*;
use tracing::error;

static B64_IMG: OnceLock<String> = OnceLock::new();

pub struct BadgeResponse(String);

impl IntoResponse for BadgeResponse {
    fn into_response(self) -> axum::response::Response {
        let res = self.0.into_response();
        Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "image/svg+xml")
            .header("Cache-Control", "no-cache, max-age=0, must-revalidate")
            .body(res.into_body())
            .unwrap()
    }
}

fn init_b64() -> String {
    let engine = BASE64_STANDARD_NO_PAD;
    let img = include_bytes!("../../static/favicon-white.png");
    format!("data:image/png;base64,{}", engine.encode(img))
}
#[axum::debug_handler]
pub async fn last_seen_badge(State(AppState { pool, .. }): State<AppState>) -> BadgeResponse {
    let Ok(mut conn) = pool.acquire().await.map_err(|e| {
        error!("Failed to acquire connection from pool. {e:?}");
        StatusCode::INTERNAL_SERVER_ERROR
    }) else {
        return BadgeResponse(badge::make(
            Some("Error"),
            "An internal error occurred",
            Some("#FF0000"),
            None,
            None,
            None,
        ));
    };
    let last_seen = sqlx::query_scalar!(
        r#"
        SELECT
            MAX(time_stamp) AS last_seen
        FROM beats;
        "#
    )
    .fetch_one(&mut *conn)
    .await
    .unwrap_or_default();
    let message = last_seen.map_or_else(
        || "never".to_string(),
        |last_seen| {
            let diff = last_seen - chrono::Utc::now();
            format!("{:#}", HumanTime::from(diff))
        },
    );
    incr_visits(&mut conn).await;
    BadgeResponse(badge::make(
        Some("Last Online"),
        message.as_str(),
        Some("#887ee0"),
        None,
        Some(B64_IMG.get_or_init(init_b64)),
        None,
    ))
}

#[axum::debug_handler]
pub async fn total_beats_badge(State(AppState { pool, .. }): State<AppState>) -> BadgeResponse {
    let Ok(mut conn) = pool.acquire().await.map_err(|e| {
        error!("Failed to acquire connection from pool. {e:?}");
        StatusCode::INTERNAL_SERVER_ERROR
    }) else {
        return BadgeResponse(badge::make(
            Some("Error"),
            "An internal error occurred",
            Some("#FF0000"),
            None,
            None,
            None,
        ));
    };
    let total_beats = sqlx::query_scalar!("SELECT SUM(num_beats)::BIGINT AS total_beats FROM devices;")
        .fetch_one(&mut *conn)
        .await
        .unwrap_or_default()
        .unwrap_or_default()
        .format();
    incr_visits(&mut conn).await;
    BadgeResponse(badge::make(
        Some("Total Beats"),
        total_beats.as_str(),
        Some("#6495ed"),
        None,
        Some(B64_IMG.get_or_init(init_b64)),
        None,
    ))
}
