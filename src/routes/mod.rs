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

use crate::{config::Config, AppState};
use api::{get_stats, handle_beat_req, post_device, realtime_stats};
use axum::{
    routing::{get, post},
    Router,
};
#[cfg(feature = "badges")]
use badges::{last_seen_badge, total_beats_badge};
use pages::{index_page, privacy_page, stats_page};

use self::shutdown::deploy;

mod api;
#[cfg(feature = "badges")]
mod badges;
mod pages;
pub mod query;
pub mod shutdown;

pub async fn health_check() -> &'static str {
    "OK"
}

pub fn get_all(config: &Config) -> Router<AppState> {
    let r = Router::new()
        .route("/", get(index_page))
        .route("/.well-known/deploy", post(deploy))
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
    match config.secret_key.as_ref() {
        Some(s) if !s.is_empty() => r,
        _ => r.route("/api/devices", post(post_device)),
    }
}
