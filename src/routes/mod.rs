// Copyright (c) 2023 Isis <root@5ht2.me>
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

//! Router utilities

use crate::{config::Config, AppState};
use api::{get_stats, handle_beat_req, post_device, realtime_stats, regenerate_device_token};
use axum::{
    routing::{get, post},
    Router,
};
#[cfg(feature = "badges")]
use badge_routes::{last_seen, total_beats};
use pages::{index_page, privacy_page, stats_page};

mod api;
#[cfg(feature = "badges")]
#[path = "badges.rs"]
mod badge_routes;
mod pages;

pub(crate) async fn health_check() -> &'static str {
    "OK"
}

/// Creates and returns a [`Router`] with only the routes determined by
/// crate features and the provided [`Config`].
pub fn router(config: &Config) -> Router<AppState> {
    let mut router = Router::new()
        .route("/", get(index_page))
        .route("/.well-known/health", get(health_check))
        .route("/api/beat", post(handle_beat_req))
        .route("/api/stats", get(get_stats))
        .route("/api/stats/ws", get(realtime_stats))
        .route("/privacy", get(privacy_page))
        .route("/stats", get(stats_page));
    if !config.secret_key.is_empty() {
        router = router
            .route("/api/devices", post(post_device))
            .route("/api/devices/:device_id/token/generate", post(regenerate_device_token));
    }

    #[cfg(not(feature = "badges"))]
    return router;
    router
        .route("/badge/last-seen", get(last_seen))
        .route("/badge/total-beats", get(total_beats))
}
