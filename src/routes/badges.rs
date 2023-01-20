/**
 * Copyright (c) 2023 VJ <root@5ht2.me>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
use super::query::incr_visits;
use crate::{
    util::{badge::make_badge, hf_time::*},
    DbPool,
};
use base64::prelude::*;
use rocket::{get, response::Responder};
use rocket_db_pools::Connection;

lazy_static::lazy_static! {
    static ref B64_IMG: String = {
        let engine = BASE64_STANDARD_NO_PAD;
        let img = include_bytes!("../../static/favicon-white.png");
        format!("data:image/png;base64,{}", engine.encode(img))
    };
}

#[derive(Responder)]
#[response(status = 200, content_type = "image/svg+xml;charset=utf-8")]
pub struct BadgeResponse(String);

#[get("/badge/last-seen")]
pub async fn last_seen_badge(mut conn: Connection<DbPool>) -> BadgeResponse {
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
    let message = match last_seen {
        Some(last_seen) => {
            let diff = last_seen - chrono::Utc::now();
            format!("{:#}", HumanTime::from(diff))
        }
        None => "never".to_string(),
    };
    incr_visits(&mut *conn).await;
    BadgeResponse(make_badge(
        Some("Last Online"),
        message.as_str(),
        Some("#887ee0"),
        None,
        Some(B64_IMG.as_str()),
        None,
    ))
}

#[get("/badge/total-beats")]
pub async fn total_beats_badge(mut conn: Connection<DbPool>) -> BadgeResponse {
    let total_beats = sqlx::query!(
        r#"
        SELECT
            COUNT(*) AS total_beats
        FROM beats;
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
    incr_visits(&mut *conn).await;
    BadgeResponse(make_badge(
        Some("Total Beats"),
        total_beats.as_str(),
        Some("#6495ed"),
        None,
        Some(B64_IMG.as_str()),
        None,
    ))
}
