// Copyright (c) 2023 Isis <root@5ht2.me>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use super::Snowflake;
use base64ct::{Base64Unpadded, Encoding};
use rand::Rng;

const ENCODE_TABLE: [char; 32] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'J', 'K', 'M', 'N', 'P',
    'Q', 'R', 'S', 'T', 'V', 'W', 'X', 'Y', 'Z',
];

fn encode(high: u64, low: u64, buf: &mut String) {
    buf.push(ENCODE_TABLE[(high >> 61) as usize]);
    buf.push(ENCODE_TABLE[((high >> 56) & 0x1F) as usize]);
    buf.push(ENCODE_TABLE[((high >> 51) & 0x1F) as usize]);
    buf.push(ENCODE_TABLE[((high >> 46) & 0x1F) as usize]);
    buf.push(ENCODE_TABLE[((high >> 41) & 0x1F) as usize]);
    buf.push(ENCODE_TABLE[((high >> 36) & 0x1F) as usize]);
    buf.push(ENCODE_TABLE[((high >> 31) & 0x1F) as usize]);
    buf.push(ENCODE_TABLE[((high >> 26) & 0x1F) as usize]);
    buf.push(ENCODE_TABLE[((high >> 21) & 0x1F) as usize]);
    buf.push(ENCODE_TABLE[((high >> 16) & 0x1F) as usize]);
    buf.push(ENCODE_TABLE[((high >> 11) & 0x1F) as usize]);
    buf.push(ENCODE_TABLE[((high >> 6) & 0x1F) as usize]);
    buf.push(ENCODE_TABLE[((high >> 1) & 0x1F) as usize]);
    let split = ((high << 4) & 0x1F) | ((low >> 60) & 0x1F);
    buf.push(ENCODE_TABLE[usize::try_from(split).expect("this should be literally less than 32. wtf?")]);
    buf.push(ENCODE_TABLE[((low >> 55) & 0x1F) as usize]);
    buf.push(ENCODE_TABLE[((low >> 50) & 0x1F) as usize]);
    buf.push(ENCODE_TABLE[((low >> 45) & 0x1F) as usize]);
    buf.push(ENCODE_TABLE[((low >> 40) & 0x1F) as usize]);
    buf.push(ENCODE_TABLE[((low >> 35) & 0x1F) as usize]);
    buf.push(ENCODE_TABLE[((low >> 30) & 0x1F) as usize]);
    buf.push(ENCODE_TABLE[((low >> 25) & 0x1F) as usize]);
    buf.push(ENCODE_TABLE[((low >> 20) & 0x1F) as usize]);
    buf.push(ENCODE_TABLE[((low >> 15) & 0x1F) as usize]);
    buf.push(ENCODE_TABLE[((low >> 10) & 0x1F) as usize]);
    buf.push(ENCODE_TABLE[((low >> 5) & 0x1F) as usize]);
    buf.push(ENCODE_TABLE[(low & 0x1F) as usize]);
}

fn random_string(timestamp: u64) -> String {
    let mut buf = String::with_capacity(26);
    let mut rng = rand::thread_rng();
    assert!(
        (timestamp & 0xFFFF_0000_0000_0000) == 0,
        "currently cannot generate tokens from August 10,889 onwards."
    );
    let high = (timestamp << 16) | u64::from(rng.gen::<u16>());
    let low = rng.gen::<u64>();
    encode(high, low, &mut buf);
    buf
}

pub fn generate(id: Snowflake) -> String {
    let mut dst = [0u8; 11];
    let now = chrono::Utc::now();
    let enc_id = Base64Unpadded::encode(&id.id().to_be_bytes(), &mut dst).expect("encoded base64 exceeded 11 bytes");
    let random_string = random_string(u64::try_from(now.timestamp_millis()).expect("It is now the year 292,278,994"));
    format!("{enc_id}.{random_string}")
}
