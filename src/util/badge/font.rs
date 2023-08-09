// Copyright (c) 2023 VJ <root@5ht2.me>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use super::FONT_DATA;

struct CharWidthTable {
    pub data: &'static [(u32, u32, f32)],
}

impl CharWidthTable {
    const fn create(data: &'static [(u32, u32, f32)]) -> Self {
        Self { data }
    }

    const fn is_control_char(c: u8) -> bool {
        c <= 31 || c == 127
    }

    fn width_of_char_code(&self, char_code: u32) -> Option<f32> {
        if Self::is_control_char(u8::try_from(char_code).unwrap()) {
            return Some(0.0);
        }
        let index = self.data.binary_search_by(|(a, _, _)| a.cmp(&char_code));
        index.map_or_else(
            |_| {
                let candidate_index = index.unwrap_err() - 1;
                let (lower, upper, width) = self.data[candidate_index];
                if char_code >= lower && char_code <= upper {
                    Some(width)
                } else {
                    None
                }
            },
            |index| {
                let (_, _, width) = self.data[index];
                Some(width)
            },
        )
    }

    fn width_of(&self, text: &str) -> f32 {
        let mut width = 0.0;
        for c in text.chars() {
            match self.width_of_char_code(c as u32) {
                Some(w) => width += w,
                None => width += self.width_of_char_code('m' as u32).unwrap(),
            }
        }
        width
    }
}

const TABLE: CharWidthTable = CharWidthTable::create(FONT_DATA);

pub fn measure(text: &str) -> f32 {
    TABLE.width_of(text)
}
