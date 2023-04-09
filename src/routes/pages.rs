/**
 * Copyright (c) 2023 VJ <root@5ht2.me>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
use rocket::{get, response::Responder, serde::json::Value, State};
use rocket_db_pools::Connection;
use rocket_dyn_templates::Template;

use super::query::{fetch_stats, incr_visits};
use crate::{DbPool, WrappedStats, CONFIG, GIT_HASH};

#[macro_export]
macro_rules! context {
    ($(key:expr => $value:expr,)+) => {
        context! {$($key => $value),*}
    };
    ($($key:expr => $value:expr),*) => {{
        let mut map: ::rocket::serde::json::serde_json::Map<::std::string::String, ::rocket::serde::json::Value> = ::rocket::serde::json::serde_json::Map::new();
        $(map.insert($key.into(), $value.into());)*
        let as_value: ::rocket::serde::json::Value = map.into();
        as_value
    }};
}

pub enum PageKind {
    Index,
    Stats,
    Privacy,
    Error,
}

impl PageKind {
    fn template_name(&self) -> &'static str {
        match self {
            PageKind::Index => "index",
            PageKind::Stats => "stats",
            PageKind::Privacy => "privacy",
            PageKind::Error => "error",
        }
    }
}

pub struct Page {
    kind: PageKind,
    context: Value,
}

impl Page {
    pub fn new(kind: PageKind, context: Value) -> Self {
        Self { kind, context }
    }
}

impl<'r, 'o: 'r> Responder<'r, 'o> for Page {
    fn respond_to(self, request: &'r rocket::Request<'_>) -> rocket::response::Result<'o> {
        //println!("{:#?}", self.context);
        Template::render(self.kind.template_name(), self.context).respond_to(request)
    }
}

#[get("/")]
pub async fn index_page(mut conn: Connection<DbPool>, stats: &State<WrappedStats>) -> Page {
    stats.write().await.num_visits += 1;
    incr_visits(&mut conn).await;
    let stats = fetch_stats(conn).await;
    Page {
        kind: PageKind::Index,
        context: context! {
            "server_name" => *CONFIG.server_name,
            "last_seen" => match stats.last_seen {
                Some(time) => time.timestamp(),
                None => 0,
            },
            "last_seen_relative" => match stats.last_seen {
                Some(t) => (chrono::Utc::now() - t).num_seconds(),
                None => i64::MAX,
            },
            "now" => chrono::Utc::now().timestamp(),
            "repo" => *CONFIG.repo,
            "git_hash" => *GIT_HASH,
            "total_beats" => stats.total_beats,
            "longest_absence" => stats.longest_absence.num_seconds()
        },
    }
}

#[get("/stats")]
pub async fn stats_page(mut conn: Connection<DbPool>, stats: &State<WrappedStats>) -> Page {
    stats.write().await.num_visits += 1;
    incr_visits(&mut conn).await;
    let stats = fetch_stats(conn).await;
    Page {
        kind: PageKind::Stats,
        context: context! {
            "server_name" => *CONFIG.server_name,
            "visits" => stats.num_visits,
            "devices" => stats.devices.len(),
            "beats" => stats.total_beats,
            "uptime" => (chrono::Utc::now() - *crate::SERVER_START_TIME.get().unwrap()).num_seconds()
        },
    }
}

#[get("/privacy")]
pub async fn privacy_page() -> Page {
    Page {
        kind: PageKind::Privacy,
        context: context! {
            "server_name" => *CONFIG.server_name
        },
    }
}
