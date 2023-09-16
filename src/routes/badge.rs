// Copyright (c) 2023 Isis <root@5ht2.me>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::{
    util::{formats::FormatNum, hf_time::HumanTime},
    AppState, ConnectionExt,
};
use axum::{
    extract::State,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use badges::{Badge, Colour, Render};
use tracing::error;

const B64_IMG: &str = concat!(
    "data:image/png;base64,",
    "iVBORw0KGgoAAAANSUhEUgAAAEAAAABACAYAAACqaXHeAAAACXBIWXMAAES4AABEuAH3N9d6AAADrklEQVR42u2ZT4hVVRzHP8d5joUVg9SYf5ocm",
    "cxNSLhxUW5yaCFBGWLUokWI4FJXgQiCu2oltQlCbCVIIAiRomVYQSWZb0jCyhx1/L9wAjXL+bZ4v9v7vdt9zcwbu2eme77wOL/zvb977vl97/nzu+",
    "dBQkJCQkJCQkJCQqmQtEDSSUl1SY/H7k8MATaria2x+jErogYLnD1QRQEec/ayKgrQ18aujADznb1E0uyqCbDY2TUgyk4QRQBJPcBDObo6AgALC7g",
    "oC2EsAR4t4PqrJEDRcK+UAIsKuMpPgT5JoSoC+C3wNyt7gIerIkC2Bgj4wvGlfxPEngI3gSHHl54LlC6ApbyPWPUicMpdLn0hjDECFgNdZg8Dv7pr",
    "S6oggM8CR4CfXX1pFQTwW+Al4Dxwy+r/jykg6WVJr7TZ1/1ByNkQwl0TAaBX0v1lClD7D4J/CdiXVYG9ORd/+HHBytPAEzTWhn7gh9IEkLQCeG2SY",
    "owCu0II13PBzwJ2OOrFAgH8QcglK39x3ECpAgAHaM3MJop+4PUctwF4ytWfk9RlwzyDf9Y5K884brekm22eKeAEsD6EcFvSO8B6WqeygG+BDSGEO5",
    "JWAR/wz/OHu8B7SBpSZxjxc1xSTdKpAr+VLRFIZ4y/I6lm3NpJPntQUs84Ps9Y23v+xWe0BqwB1tLcm8fDFuBJGsfaAzTmLzSm0XL3FjJxBoHj1pk",
    "umlPgagjhT7OPAfXc6GmHIeDrEMINSR8B6wp8TtgPGlNwHTC3YDTtmWDMLW/wLafgG8bNlvST49929qfu3nmO/6ag7fvG+xXc0z0Bn66CtronHbw1",
    "9oIL4kPjNjrupAly1eq/S3rQ/J52fvs66sA9Rid5wJfAmNnPSpoDbHPXt4cQ/gCyN98NrDbbL4AjsYPvSADb+upW7QN20tzbjwP7zf7E3TZoZdEWO",
    "LMEMBzN9AD8H5vbQwgy+xCNhQbgeSt9FniOaYBOBTjm7Gy1/wr4+G8yhGHgR6suV+MvcJ8FDscOfioCfE7z7Wbwbz/DQWevofVD6HLs4KeEXNLzWR",
    "sfn+DslfS9qz8QO4apCvCuC2Z1G5+5km6bzzVJo2Zfn+zzph0k9UraJWnTOH5HClLQ+kSfM+Mh6c0CAQ7F7leGMk6EDhZwZ2MHXqYA3wFXctyFThq",
    "akQKEEMaAwzl6WmSBpQhgyM/58x21MlMhaZGkMbcIrojdpxgi1C34MUm9sfsTQ4BX7Yzg/dh9SUhISEhISEhISIC/AB8nCh3wr1ifAAAAAElFTkSu",
    "QmCC",
);

const BLUE_MAGENTA: Colour = Colour::from_rgb(136, 126, 224);
const CORNFLOWER_BLUE: Colour = Colour::from_rgb(100, 149, 237);

pub struct BResponse {
    svg: String,
    status: StatusCode,
}

impl BResponse {
    pub const fn new(status: StatusCode, svg: String) -> Self {
        Self { svg, status }
    }
}

impl IntoResponse for BResponse {
    fn into_response(self) -> Response {
        (
            self.status,
            [
                (header::CONTENT_TYPE, "image/svg+xml"),
                (header::CACHE_CONTROL, "no-cache, max-age=0, must-revalidate"),
            ],
            self.svg,
        )
            .into_response()
    }
}

#[axum::debug_handler]
pub async fn last_seen(State(AppState { pool, .. }): State<AppState>) -> BResponse {
    let Ok(mut conn) = pool.acquire().await.map_err(|e| {
        error!("Failed to acquire connection from pool. {e:?}");
    }) else {
        return BResponse::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            Badge::builder()
                .label("Error")
                .message("An internal error occurred")
                .colour(Colour::RED)
                .logo(B64_IMG)
                .build()
                .render(),
        );
    };
    let last_seen = sqlx::query_scalar!(
        r#"
        SELECT
            MAX(time_stamp) AS last_seen
        FROM heartbeat.beats;
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
    let _ = conn.incr_visits().await;
    BResponse::new(
        StatusCode::OK,
        Badge::builder()
            .label("Last Online")
            .message(&message)
            .colour(BLUE_MAGENTA)
            .logo(B64_IMG)
            .build()
            .render(),
    )
}

#[axum::debug_handler]
pub async fn total_beats(State(AppState { pool, .. }): State<AppState>) -> BResponse {
    let Ok(mut conn) = pool.acquire().await.map_err(|e| {
        error!("Failed to acquire connection from pool. {e:?}");
    }) else {
        return BResponse::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            Badge::builder()
                .label("Error")
                .message("An internal error occurred")
                .colour(Colour::RED)
                .build()
                .render(),
        );
    };
    let total_beats = sqlx::query_scalar!("SELECT SUM(num_beats)::BIGINT AS total_beats FROM heartbeat.devices;")
        .fetch_one(&mut *conn)
        .await
        .unwrap_or_default()
        .unwrap_or_default()
        .format();
    let _ = conn.incr_visits().await;
    BResponse::new(
        StatusCode::OK,
        Badge::builder()
            .label("Total Beats")
            .message(&total_beats)
            .colour(CORNFLOWER_BLUE)
            .build()
            .render(),
    )
}

#[cfg(test)]
mod tests {
    use super::B64_IMG;
    use base64ct::{Base64Unpadded, Encoding};
    const BUFFER_SIZE: usize = 1360;
    const IMG_BYTES: &[u8] = include_bytes!("../../static/favicon-white.png");
    #[test]
    fn test_my_manual_typing_actually_matches_the_image() {
        assert_eq!(
            Base64Unpadded::encode(IMG_BYTES, &mut [0u8; BUFFER_SIZE]).unwrap(),
            &B64_IMG["data:image/png;base64,".len()..],
        );
    }
    #[test]
    fn get_b64_img() {
        println!(
            "{}",
            Base64Unpadded::encode(IMG_BYTES, &mut [0u8; BUFFER_SIZE]).unwrap()
        );
    }
}
