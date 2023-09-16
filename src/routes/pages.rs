// Copyright (c) 2023 Isis <root@5ht2.me>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::{
    stats::Stats,
    templates::{index, privacy, stats as stats_template},
    AppState, ConnectionExt,
};
use axum::{extract::State, http::StatusCode};
use html::{Markup, PreEscaped};
use tracing::error;

#[axum::debug_handler]
pub async fn index_page(
    State(AppState {
        stats,
        pool,
        git_hash,
        config,
        ..
    }): State<AppState>,
) -> (StatusCode, Markup) {
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
    let _ = conn.incr_visits().await;
    (StatusCode::OK, index(&Stats::fetch(conn).await, git_hash, &config))
}

#[axum::debug_handler]
pub async fn stats_page(
    State(AppState {
        stats,
        pool,
        server_start_time,
        config,
        ..
    }): State<AppState>,
) -> (StatusCode, Markup) {
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
    let _ = conn.incr_visits().await;
    let stats = Stats::fetch(conn).await;
    (StatusCode::OK, stats_template(&stats, &config, server_start_time))
}

#[axum::debug_handler]
pub async fn privacy_page(State(AppState { config, .. }): State<AppState>) -> Markup {
    privacy(&config)
}
