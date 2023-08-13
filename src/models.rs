// Copyright (c) 2023 VJ <root@5ht2.me>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::util::{deserialize_duration, serialize_duration, serialize_ts};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct Beat {
    #[serde(serialize_with = "chrono::serde::ts_seconds::serialize")]
    time_stamp: chrono::DateTime<chrono::Utc>,
    device: Device,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Stats {
    #[serde(serialize_with = "serialize_ts")]
    pub last_seen: Option<chrono::DateTime<chrono::Utc>>,
    pub devices: Vec<Device>,
    #[serde(serialize_with = "serialize_duration")]
    #[serde(deserialize_with = "deserialize_duration")]
    pub longest_absence: chrono::Duration,
    pub num_visits: i64,
    pub total_beats: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Device {
    pub id: i64,
    pub name: Option<String>,
    #[serde(serialize_with = "serialize_ts")]
    pub last_beat: Option<chrono::DateTime<chrono::Utc>>,
    pub num_beats: i64,
}

#[derive(Deserialize)]
pub struct PostDevice {
    pub name: String,
}

pub struct AuthInfo {
    pub id: i64,
    pub name: Option<String>,
}
