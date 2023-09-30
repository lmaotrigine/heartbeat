// Copyright (c) 2023 Isis <root@5ht2.me>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use chrono::{DateTime, Utc};
use hmac::Mac;
use sqlx::PgPool;

/// Extension trait for [`PgPool`].
///
/// This is just a convenience wrapper around some common queries.
#[axum::async_trait]
pub trait PoolExt {
    /// Increment the number of visits to the site.
    async fn incr_visits(&self) -> sqlx::Result<()>;
    /// Get the server epoch (time of first ever deployment)
    async fn server_start_time(&self) -> DateTime<Utc>;
}

#[axum::async_trait]
impl PoolExt for PgPool {
    async fn incr_visits(&self) -> sqlx::Result<()> {
        sqlx::query!(
            r#"
            UPDATE heartbeat.stats
            SET total_visits = total_visits + 1
            RETURNING total_visits;
            "#
        )
        .fetch_one(self)
        .await?;
        Ok(())
    }

    async fn server_start_time(&self) -> DateTime<Utc> {
        let now = Utc::now();
        sqlx::query_scalar!(
            r#"
            WITH dummy AS (
                INSERT INTO heartbeat.stats (_id)
                VALUES (0)
                ON CONFLICT (_id) DO NOTHING
            )
            SELECT server_start_time
            FROM heartbeat.stats
            WHERE _id = 0;
            "#
        )
        .fetch_optional(self)
        .await
        .unwrap_or_default()
        .unwrap_or(now)
    }
}

const HEX_CHARS: &[u8; 16] = b"0123456789abcdef";

struct BytesToHexChars<'a> {
    inner: std::slice::Iter<'a, u8>,
    next: Option<char>,
}

impl<'a> BytesToHexChars<'a> {
    fn new(inner: &'a [u8]) -> Self {
        Self {
            inner: inner.iter(),
            next: None,
        }
    }
}

impl<'a> Iterator for BytesToHexChars<'a> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next.take() {
            Some(current) => Some(current),
            None => self.inner.next().map(|byte| {
                let current = HEX_CHARS[(byte >> 4) as usize] as char;
                self.next = Some(HEX_CHARS[(byte & 0x0f) as usize] as char);
                current
            }),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let length = self.len();
        (length, Some(length))
    }
}

impl<'a> ExactSizeIterator for BytesToHexChars<'a> {
    fn len(&self) -> usize {
        let mut length = self.inner.len() * 2;
        if self.next.is_some() {
            length += 1;
        }
        length
    }
}

/// Encode values as hexadecimal.
///
/// This trait is implemented for all `T` that implement `AsRef<[u8]>`, which includes
/// `String`, `&str`, `Vec<u8>`, and `[u8]`.
pub trait ToHex {
    /// Encode the hex value represented by `self` into the result.
    fn encode_hex<T: std::iter::FromIterator<char>>(&self) -> T;
}

impl<T: AsRef<[u8]>> ToHex for T {
    #[inline]
    fn encode_hex<U: std::iter::FromIterator<char>>(&self) -> U {
        BytesToHexChars::new(self.as_ref()).collect()
    }
}

pub trait MacExt {
    fn with_data(self, data: &[u8]) -> Self;
}

impl<M: Mac> MacExt for M {
    fn with_data(mut self, data: &[u8]) -> Self {
        self.update(data);
        self
    }
}
