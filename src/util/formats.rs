// Copyright (c) 2023 VJ <root@5ht2.me>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use super::hf_time::{Accuracy, HumanTime, Tense};

pub fn format_relative(dur: chrono::Duration) -> String {
    HumanTime::from(dur).to_text(Accuracy::Precise, Tense::Present)
}

pub fn format_num(input: i64) -> String {
    input
        .to_string()
        .as_bytes()
        .rchunks(3)
        .rev()
        .map(std::str::from_utf8)
        .collect::<std::result::Result<Vec<&str>, _>>()
        .unwrap()
        .join(",")
}
