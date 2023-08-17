// Copyright (c) 2023 VJ <root@5ht2.me>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

// these are all because of [`axum::debug_handler`]
#![allow(
    clippy::diverging_sub_expression,
    clippy::unused_async,
    clippy::items_after_statements
)]

use crate::{AppState, CONFIG};
use api::{get_stats, handle_beat_req, post_device, realtime_stats};
use axum::{
    routing::{get, post},
    Router,
};
#[cfg(feature = "badges")]
use badges::{last_seen_badge, total_beats_badge};
use pages::{index_page, privacy_page, stats_page};

mod api;
#[cfg(feature = "badges")]
mod badges;
mod pages;
pub mod query;

pub async fn health_check() -> &'static str {
    "OK"
}

pub fn get_all() -> Router<AppState> {
    let r = Router::new()
        .route("/", get(index_page))
        .route("/.well-known/health", get(health_check))
        .route("/stats", get(stats_page))
        .route("/privacy", get(privacy_page))
        .route("/api/beat", post(handle_beat_req))
        .route("/api/stats", get(get_stats))
        .route("/api/stats/ws", get(realtime_stats));
    #[cfg(feature = "badges")]
    let r = r
        .route("/badge/last-seen", get(last_seen_badge))
        .route("/badge/total-beats", get(total_beats_badge));
    if CONFIG
        .get()
        .expect("config to be initialized")
        .clone()
        .secret_key
        .unwrap_or_default()
        .is_empty()
    {
        r
    } else {
        r.route("/api/devices", post(post_device))
    }
}
