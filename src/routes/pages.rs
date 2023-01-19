use rocket::{get, response::Responder, serde::json::Value};
use rocket_db_pools::Connection;
use rocket_dyn_templates::Template;

use super::query::{fetch_stats, incr_visits};
use crate::{DbPool, CONFIG, GIT_HASH};

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
pub async fn index_page(mut conn: Connection<DbPool>) -> Page {
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
            "last_seen_relative" => stats.last_seen_relative.num_seconds(),
            "now" => chrono::Utc::now().timestamp(),
            "repo" => *CONFIG.repo,
            "git_hash" => *GIT_HASH,
            "total_beats" => stats.total_beats,
            "longest_absence" => stats.longest_absence.num_seconds()
        },
    }
}

#[get("/stats")]
pub async fn stats_page(mut conn: Connection<DbPool>) -> Page {
    incr_visits(&mut conn).await;
    let stats = fetch_stats(conn).await;
    Page {
        kind: PageKind::Stats,
        context: context! {
            "server_name" => *CONFIG.server_name,
            "visits" => stats.num_visits,
            "devices" => stats.devices.len(),
            "beats" => stats.total_beats,
            "uptime" => stats.uptime.num_seconds()
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
