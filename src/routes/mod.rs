use rocket::routes;
mod api;
#[cfg(feature = "badges")]
mod badges;
mod pages;
mod query;

use crate::{context, CONFIG};
use api::*;
#[cfg(feature = "badges")]
use badges::*;
use pages::*;

pub fn get_routes() -> Vec<rocket::Route> {
    let mut v = Vec::new();
    v.extend(routes![
        index_page,
        stats_page,
        privacy_page,
        handle_beat_req,
        get_stats
    ]);
    if !(CONFIG.secret_key.as_ref().unwrap_or(&"".into()).is_empty()) {
        v.extend(routes![post_device]);
    }
    #[cfg(feature = "badges")]
    v.extend(routes![last_seen_badge, total_beats_badge]);
    v
}

#[rocket::catch(default)]
pub fn default_catcher(status: rocket::http::Status, req: &rocket::Request) -> Page {
    eprintln!(
        "returned {} to {} - tried to connect to {} with Authorization {}",
        status.code,
        req.client_ip().unwrap_or_else(|| "0.0.0.0".parse().unwrap()),
        req.uri().path(),
        req.headers().get_one("Authorization").unwrap_or_default()
    );
    Page::new(
        PageKind::Error,
        context! {
            "server_name" => *crate::CONFIG.server_name,
            "message" => format!("{}", status.reason_lossy()),
            "status" => status.code,
            "path" => req.uri().path().to_string(),
            "method" => req.method().to_string()
        },
    )
}
