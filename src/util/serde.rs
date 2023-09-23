// Copyright (c) 2023 Isis <root@5ht2.me>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

pub mod ts {
    use chrono::serde::ts_seconds;

    pub fn serialize<S>(ts: &Option<chrono::DateTime<chrono::Utc>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match ts {
            Some(ts) => ts_seconds::serialize(ts, serializer),
            None => serializer.serialize_none(),
        }
    }
}
