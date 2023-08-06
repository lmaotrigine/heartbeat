// Copyright (c) 2023 VJ <root@5ht2.me>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use super::Regex;

#[allow(clippy::cast_lossless)]
#[inline]
pub fn brightness(colour: &str) -> f32 {
    if !colour.is_empty() {
        if let Some(css_colour) = css_from_str(colour) {
            let (r, g, b) = css_colour.to_rgb();
            return (r as f32).mul_add(299.0, (g as f32).mul_add(587.0, b as f32 * 114.0)) / 255_000.0;
        }
    }
    0.0
}

struct Colour {
    r: u8,
    g: u8,
    b: u8,
}

impl Colour {
    fn new(r: &str, g: &str, b: &str) -> Self {
        Self {
            r: u8::from_str_radix(r, 16).unwrap(),
            g: u8::from_str_radix(g, 16).unwrap(),
            b: u8::from_str_radix(b, 16).unwrap(),
        }
    }

    const fn to_rgb(&self) -> (u8, u8, u8) {
        (self.r, self.g, self.b)
    }
}

impl std::fmt::Display for Colour {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }
}

#[inline]
fn css_from_str(s: &str) -> Option<Colour> {
    let hex = Regex::new(r"^#([0-9a-fA-F]{2})([0-9a-fA-F]{2})([0-9a-fA-F]{2})$").unwrap();
    let short_hex = Regex::new(r"^#([0-9a-fA-F])([0-9a-fA-F])([0-9a-fA-F])$").unwrap();
    if let Some(caps) = hex.captures(s) {
        let r = caps.get(1).unwrap().as_str();
        let g = caps.get(2).unwrap().as_str();
        let b = caps.get(3).unwrap().as_str();
        return Some(Colour::new(r, g, b));
    }
    if let Some(caps) = short_hex.captures(s) {
        let r = caps.get(1).unwrap().as_str();
        let g = caps.get(2).unwrap().as_str();
        let b = caps.get(3).unwrap().as_str();
        return Some(Colour::new(&format!("{r}{r}"), &format!("{g}{g}"), &format!("{b}{b}")));
    }
    None
}

impl std::str::FromStr for Colour {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        css_from_str(s).map_or_else(|| Err(format!("{s} is not a valid CSS colour")), Ok)
    }
}
