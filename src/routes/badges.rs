// Copyright (c) 2023 VJ <root@5ht2.me>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use super::query::incr_visits;
use crate::{
    util::{badge, hf_time::HumanTime},
    AppState,
};
use axum::{extract::State, http::Response, http::StatusCode, response::IntoResponse};
use base64::prelude::*;

lazy_static::lazy_static! {
    static ref B64_IMG: String = {
        let engine = BASE64_STANDARD_NO_PAD;
        let img = include_bytes!("../../static/favicon-white.png");
        format!("data:image/png;base64,{}", engine.encode(img))
    };
}

pub struct BadgeResponse(String);

impl IntoResponse for BadgeResponse {
    fn into_response(self) -> axum::response::Response {
        let res = self.0.into_response();
        Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "image/svg+xml")
            .body(res.into_body())
            .unwrap()
    }
}

#[axum::debug_handler]
pub async fn last_seen_badge(State(AppState { pool, .. }): State<AppState>) -> BadgeResponse {
    let mut conn = pool.acquire().await.unwrap();
    let last_seen = sqlx::query!(
        r#"
        SELECT
            MAX(time_stamp) AS last_seen
        FROM beats;
        "#
    )
    .fetch_one(&mut *conn)
    .await
    .unwrap()
    .last_seen;
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
        Some(B64_IMG.as_str()),
        None,
    ))
}

#[axum::debug_handler]
pub async fn total_beats_badge(State(AppState { pool, .. }): State<AppState>) -> BadgeResponse {
    let mut conn = pool.acquire().await.unwrap();
    let total_beats = sqlx::query!(
        r#"
        WITH indiv AS (
            SELECT num_beats FROM devices
        )
        SELECT SUM(indiv.num_beats)::BIGINT AS total_beats FROM indiv;
        "#
    )
    .fetch_one(&mut *conn)
    .await
    .unwrap()
    .total_beats
    .unwrap_or_default()
    .to_string()
    .as_bytes()
    .rchunks(3)
    .rev()
    .map(std::str::from_utf8)
    .collect::<Result<Vec<&str>, _>>()
    .unwrap()
    .join(",");
    incr_visits(&mut conn).await;
    BadgeResponse(badge::make(
        Some("Total Beats"),
        total_beats.as_str(),
        Some("#6495ed"),
        None,
        Some(B64_IMG.as_str()),
        None,
    ))
}
