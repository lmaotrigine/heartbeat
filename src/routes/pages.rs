// Copyright (c) 2023 VJ <root@5ht2.me>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use super::query::{fetch_stats, incr_visits};
use crate::{
    templates::{index, privacy, stats as stats_template},
    AppState,
};
use axum::{extract::State, response::IntoResponse};
use html::{Markup, PreEscaped};
use reqwest::StatusCode;
use tracing::error;

#[axum::debug_handler]
pub async fn index_page(State(AppState { stats, pool }): State<AppState>) -> impl IntoResponse {
    let mut conn = match pool.acquire().await.map_err(|e| {
        error!("Failed to acquire connection from pool. {e:?}");
        StatusCode::INTERNAL_SERVER_ERROR
    }) {
        Ok(x) => x,
        Err(e) => {
            return (
                e,
                PreEscaped("<samp>This service is temporarily unavailable</samp>".into()),
            )
        }
    };
    {
        stats.lock().unwrap().num_visits += 1;
    }
    incr_visits(&mut conn).await;
    let stats = fetch_stats(conn).await;
    (StatusCode::OK, index(&stats))
}

#[axum::debug_handler]
pub async fn stats_page(State(AppState { stats, pool }): State<AppState>) -> (StatusCode, Markup) {
    {
        stats.lock().unwrap().num_visits += 1;
    }
    let mut conn = match pool.acquire().await.map_err(|e| {
        error!("Failed to acquire connection from pool. {e:?}");
        StatusCode::INTERNAL_SERVER_ERROR
    }) {
        Ok(x) => x,
        Err(e) => {
            return (
                e,
                PreEscaped("<samp>This service is temporarily unavailable</samp>".into()),
            )
        }
    };
    incr_visits(&mut conn).await;
    let stats = fetch_stats(conn).await;
    (StatusCode::OK, stats_template(&stats).await)
}

#[axum::debug_handler]
pub async fn privacy_page() -> Markup {
    privacy()
}
