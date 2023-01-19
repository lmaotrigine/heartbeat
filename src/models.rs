use rocket::serde::Serialize;
use serde::Deserialize;

use crate::util::{deserialize_duration, serialize_duration, serialize_ts};

#[derive(Serialize)]
pub struct Beat {
    #[serde(serialize_with = "chrono::serde::ts_seconds::serialize")]
    time_stamp: chrono::DateTime<chrono::Utc>,
    device: Device,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Stats {
    #[serde(serialize_with = "serialize_ts")]
    pub last_seen: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(serialize_with = "serialize_duration")]
    #[serde(deserialize_with = "deserialize_duration")]
    pub last_seen_relative: chrono::Duration,
    pub devices: Vec<Device>,
    #[serde(serialize_with = "serialize_duration")]
    #[serde(deserialize_with = "deserialize_duration")]
    pub longest_absence: chrono::Duration,
    pub num_visits: u64,
    #[serde(serialize_with = "serialize_duration")]
    #[serde(deserialize_with = "deserialize_duration")]
    pub uptime: chrono::Duration,
    pub total_beats: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Device {
    pub id: i64,
    pub name: Option<String>,
    #[serde(serialize_with = "serialize_ts")]
    pub last_beat: Option<chrono::DateTime<chrono::Utc>>,
    pub num_beats: u64,
}

#[derive(Deserialize)]
pub struct PostDevice {
    pub name: String,
}

pub struct AuthInfo {
    pub id: i64,
    pub name: Option<String>,
}
