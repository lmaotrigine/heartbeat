// Copyright (c) 2023 Isis <root@5ht2.me>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::util::serde::ts;
use chrono::serde::ts_seconds;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct Beat {
    #[serde(with = "ts_seconds")]
    time_stamp: chrono::DateTime<chrono::Utc>,
    device: Device,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Device {
    pub id: i64,
    pub name: Option<String>,
    #[serde(serialize_with = "ts::serialize")]
    pub last_beat: Option<chrono::DateTime<chrono::Utc>>,
    pub num_beats: i64,
}

#[derive(Deserialize)]
pub struct PostDevice {
    pub name: String,
}
