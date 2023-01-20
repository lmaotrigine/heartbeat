/**
 * Copyright (c) 2023 VJ <root@5ht2.me>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
use chrono::serde::ts_seconds;
use rocket::serde::Deserialize;

pub fn serialize_ts<S>(ts: &Option<chrono::DateTime<chrono::Utc>>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match ts {
        Some(ts) => ts_seconds::serialize(ts, serializer),
        None => serializer.serialize_none(),
    }
}

pub fn serialize_duration<S>(duration: &chrono::Duration, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_i64(duration.num_seconds())
}

pub fn deserialize_duration<'de, D>(deserializer: D) -> Result<chrono::Duration, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let seconds = i64::deserialize(deserializer)?;
    Ok(chrono::Duration::seconds(seconds))
}
