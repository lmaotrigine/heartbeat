// Copyright (c) 2023 VJ <root@5ht2.me>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

mod vendor;
use regex::Regex;
use vendor::verdana::FONT_DATA;
static FONT_FAMILY: &str = "Verdana,Geneva,DejaVu Sans,sans-serif";
static FONT_SCALE_UP_FACTOR: f32 = 10.0;
static FONT_SCALE_DOWN_VALUE: &str = "scale(.1)";

mod colour;
mod font;
mod renderer;
mod xml;

use renderer::Badge;
use xml::Render;

pub fn make_badge(
    label: Option<&str>,
    message: &str,
    colour: Option<&str>,
    label_colour: Option<&str>,
    logo: Option<&str>,
    logo_width: Option<f32>,
) -> String {
    let logo_width = logo_width.unwrap_or(if logo.is_some() { 14.0 } else { 0.0 });
    let logo_padding = if logo.is_some() && !label.unwrap_or_default().is_empty() {
        3.0
    } else {
        0.0
    };
    Badge::new(label, message, logo, logo_width, logo_padding, colour, label_colour).render()
}
