// Copyright (c) 2023 Isis <root@5ht2.me>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::{
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
        mut pool,
        ..
    }): State<AppState>,
) -> (StatusCode, Markup) {
    let stats = {
        let mut guard = stats.lock().unwrap();
        guard.num_visits += 1;
        guard.clone()
    };
    tokio::spawn(async move {
        let _ = pool.incr_visits().await;
    });
    (StatusCode::OK, index(&stats, git_hash, &config))
}

#[axum::debug_handler]
pub async fn stats_page(
    State(AppState {
        stats,
        server_start_time,
        config,
        mut pool,
        ..
    }): State<AppState>,
) -> (StatusCode, Markup) {
    let stats = {
        let mut guard = stats.lock().unwrap();
        guard.num_visits += 1;
        guard.clone()
    };
    tokio::spawn(async move {
        let _ = pool.incr_visits().await;
    });
    (StatusCode::OK, stats_template(&stats, &config, server_start_time))
}

#[axum::debug_handler]
pub async fn privacy_page(State(AppState { config, .. }): State<AppState>) -> Markup {
    privacy(&config)
}
