// Copyright (c) 2023 Isis <root@5ht2.me>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::{
    sealed::Connection,
    stats::Stats,
    templates::{index, privacy, stats as stats_template},
    AppState, ConnectionExt,
};
use axum::{extract::State, http::StatusCode};
use html::Markup;

#[axum::debug_handler]
pub async fn index_page(
    State(AppState {
        stats,
        git_hash,
        config,
        ..
    }): State<AppState>,
    mut conn: Connection,
) -> (StatusCode, Markup) {
    {
        stats.lock().unwrap().num_visits += 1;
    }
    let _ = conn.incr_visits().await;
    (
        StatusCode::OK,
        index(&Stats::fetch(conn.into()).await, git_hash, &config),
    )
}

#[axum::debug_handler]
pub async fn stats_page(
    State(AppState {
        stats,
        server_start_time,
        config,
        ..
    }): State<AppState>,
    mut conn: Connection,
) -> (StatusCode, Markup) {
    {
        stats.lock().unwrap().num_visits += 1;
    }
    let _ = conn.incr_visits().await;
    let stats = Stats::fetch(conn.into()).await;
    (StatusCode::OK, stats_template(&stats, &config, server_start_time))
}

#[axum::debug_handler]
pub async fn privacy_page(State(AppState { config, .. }): State<AppState>) -> Markup {
    privacy(&config)
}
