// Copyright (c) 2023 Isis <root@5ht2.me>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

// Very rudimentary ID system
// This uses 42 bits for timestamp, 10 bits for node ID (soon:tm:, for now
// always 1), and 12 bits for sequence number
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::TimeZone;

const EPOCH: u64 = 1_577_836_800_000; // 2020-01-01 00:00:00 UTC

const fn bitmask(shift: u8) -> u64 {
    u64::MAX << shift
}

const fn max(shift: u8) -> u64 {
    !bitmask(shift)
}

#[derive(Debug, Default)]
pub struct Generator {
    last_timestamp: u64,
    seq: u64,
}

#[derive(Debug, Clone, Copy)]
pub struct Snowflake(u64);

impl Snowflake {
    #[allow(clippy::cast_sign_loss)] // we just..panic
    pub const fn from(id: i64) -> Self {
        assert!(id >= 0, "snowflake ID must be positive!");
        Self(id as _)
    }

    pub const fn id(self) -> u64 {
        self.0
    }

    pub fn created_at(self) -> chrono::DateTime<chrono::Utc> {
        let ts = (self.0 >> 22) & max(42);
        chrono::Utc
            .timestamp_opt(
                i64::try_from(ts + EPOCH).expect("it is now the year 292,278,994") / 1000,
                0,
            )
            .single()
            .unwrap_or_else(|| panic!("snowflake {self} had invalid timestamp {ts}"))
    }
}

impl std::fmt::Display for Snowflake {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.pad(&self.id().to_string())
    }
}

pub fn ts() -> u64 {
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("broken system clock");
    t.as_secs() * 1000 + (u64::from(t.subsec_nanos())) / 1_000_000
}

impl Generator {
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
        let mut gen = Generator {
            last_timestamp: 0,
            seq: 0,
        };
        let id = gen.generate();
        println!("id: {id}");
        println!("created_at: {}", id.created_at());
    }
}
