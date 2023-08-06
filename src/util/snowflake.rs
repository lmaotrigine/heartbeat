// Copyright (c) 2023 VJ <root@5ht2.me>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

// Very rudimentary ID system
// This uses 42 bits for timestamp, 10 bits for node ID (soon:tm:, for now always 1), and 12 bits for sequence number
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::TimeZone;

static EPOCH: u64 = 1_577_836_800_000; // 2020-01-01 00:00:00 UTC

const fn bitmask(shift: u8) -> u64 {
    u64::MAX << shift
}

const fn max(shift: u8) -> u64 {
    !bitmask(shift)
}

#[derive(Debug, Default)]
pub struct SnowflakeGenerator {
    last_timestamp: u64,
    seq: u64,
}

#[derive(Debug, Clone, Copy)]
pub struct Snowflake(u64);

impl Snowflake {
    pub const fn id(self) -> u64 {
        self.0
    }

    #[allow(clippy::cast_possible_wrap)]
    #[allow(dead_code)] // unused as of now
    pub fn created_at(self) -> chrono::DateTime<chrono::Utc> {
        let ts = self.0 >> 22 & max(42);
        chrono::Utc.timestamp_opt((ts + EPOCH) as i64 / 1000, 0).unwrap()
    }
}

impl From<u64> for Snowflake {
    fn from(id: u64) -> Self {
        Self(id)
    }
}

impl std::fmt::Display for Snowflake {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.pad(&self.id().to_string())
    }
}

#[allow(clippy::cast_lossless)]
pub fn ts() -> u64 {
    let t = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    t.as_secs() * 1000 + (t.subsec_nanos() as u64) / 1_000_000
}

impl SnowflakeGenerator {
    pub fn generate(&mut self) -> Snowflake {
        let now = ts();
        assert!(
            now >= self.last_timestamp,
            "clock moved backwards, check your NTP settings"
        );
        let elapsed = now - EPOCH;
        let seq = if now == self.last_timestamp { self.seq + 1 } else { 0 };
        assert!(seq <= max(12), "sequence number exceeds sequence bits limit!");
        let ts_mask = bitmask(22);
        let node_mask = bitmask(12) ^ ts_mask;
        self.last_timestamp = now;
        self.seq = seq;
        Snowflake(((elapsed << (22)) & ts_mask) | ((1 << 12) & node_mask) | seq & max(12))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stuff() {
        let mut gen = SnowflakeGenerator {
            last_timestamp: 0,
            seq: 0,
        };
        let id = gen.generate();
        println!("id: {id}");
        println!("created_at: {}", id.created_at());
    }
}
