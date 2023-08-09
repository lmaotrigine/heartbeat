// Copyright (c) 2023 VJ <root@5ht2.me>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use super::colour::brightness;
use super::font::measure;
use super::xml::{Content, Element, ElementList, Render};
use super::{FONT_FAMILY, FONT_SCALE_DOWN_VALUE, FONT_SCALE_UP_FACTOR};

fn colours_for_background(colour: &str) -> (&str, &str) {
    let brightness_threshold = 0.69;
    if brightness(colour) <= brightness_threshold {
        ("#fff", "#010101")
    } else {
        ("#333", "#ccc")
    }
}

fn round_up_to_odd(val: f32) -> f32 {
    if val.rem_euclid(2.0) == 0.0 {
        val + 1.0
    } else {
        val
    }
}

fn preferred_width(text: &str) -> f32 {
    if text.is_empty() {
        return 0.0;
    }
    round_up_to_odd(measure(text))
}

fn get_accessible_text(label: Option<&str>, message: &str) -> String {
    label.map_or_else(|| message.to_string(), |label| format!("{label}: {message}"))
}
fn get_logo_element(
    logo: Option<&str>,
    horiz_padding: impl std::fmt::Display,
    badge_height: f32,
    logo_width: impl std::fmt::Display,
) -> Content {
    let logo_height = 14.0;
    logo.map_or(Content::Text(""), |logo| {
        Content::Element(
            Element::new("image")
                .attr("x", horiz_padding)
                .attr("y", 0.5 * (badge_height - logo_height))
                .attr("width", logo_width)
                .attr("height", logo_height)
                .attr("xlink:href", logo),
        )
    })
}

fn render_badge(
    content: Vec<Content>,
    left_width: f32,
    right_width: f32,
    height: f32,
    accessible_text: &str,
) -> String {
    let width = left_width + right_width;
    let title = Element::new("title").content(vec![Content::Text(accessible_text)]);
    let body = ElementList::new(content);
    let svg = Element::new("svg")
        .content(vec![Content::Element(title), Content::List(body)])
        .attr("xmlns", "http://www.w3.org/2000/svg")
        .attr("xmlns:xlink", "http://www.w3.org/1999/xlink")
        .attr("width", width)
        .attr("height", height)
        .attr("role", "img")
        .attr("aria-label", accessible_text);
    svg.render()
}

#[derive(Debug)]
pub struct Badge<'a> {
    horiz_padding: f32,
    label_margin: f32,
    message_margin: f32,
    label_width: f32,
    message_width: f32,
    left_width: f32,
    right_width: f32,
    width: f32,
    label_colour: &'a str,
    colour: &'a str,
    label: &'a str,
    message: &'a str,
    accessible_text: String,
    logo_element: Content<'a>,
}

static HEIGHT: f32 = 20.0;
static VERTICAL_MARGIN: f32 = 0.0;
static SHADOW: bool = true;

impl<'a> Badge<'a> {
    pub fn new(
        label: Option<&'a str>,
        message: &'a str,
        logo: Option<&'a str>,
        logo_width: f32,
        logo_padding: f32,
        colour: Option<&'a str>,
        label_colour: Option<&'a str>,
    ) -> Self {
        let colour = colour.unwrap_or("#4c1");
        let horiz_padding = 5.0;
        let has_logo = !logo.unwrap_or_default().is_empty();
        let total_logo_width = logo_width + logo_padding;
        let accessible_text = get_accessible_text(label, message);
        let has_label = !label.unwrap_or_default().is_empty();
        let label_colour = if has_label || has_logo {
            label_colour.unwrap_or("#555")
        } else {
            colour
        };
        let label_margin = total_logo_width + 1.0;
        let label_width = preferred_width(label.unwrap_or(""));
        let left_width = if has_label {
            2.0f32.mul_add(horiz_padding, label_width) + total_logo_width
        } else {
            0.0
        };
        let message_width = preferred_width(message);
        let mut message_margin = left_width - message.len().min(1) as f32;
        if !has_label {
            if has_logo {
                message_margin += total_logo_width + horiz_padding;
            } else {
                message_margin += 1.0;
            }
        }
        let mut right_width = 2.0f32.mul_add(horiz_padding, message_width);
        if has_logo && !has_label {
            right_width += total_logo_width + horiz_padding - 1.0;
        }
        let width = left_width + right_width;
        Self {
            horiz_padding,
            label_margin,
            message_margin,
            label_width,
            message_width,
            left_width,
            right_width,
            width,
            label_colour,
            colour,
            label: label.unwrap_or(""),
            message,
            accessible_text,
            logo_element: get_logo_element(logo, horiz_padding, HEIGHT, logo_width),
        }
    }

    fn get_text_element(&'a self, left_margin: f32, content: &'a str, colour: &str, text_width: f32) -> Content {
        if content.is_empty() {
            return Content::Text("");
        }
        let (text_colour, shadow_colour) = colours_for_background(colour);
        let x = FONT_SCALE_UP_FACTOR * (0.5f32.mul_add(text_width, left_margin) + self.horiz_padding);
        let text = Element::new("text")
            .content(vec![Content::Text(content)])
            .attr("x", x)
            .attr("y", 140.0 + VERTICAL_MARGIN)
            .attr("transform", FONT_SCALE_DOWN_VALUE)
            .attr("fill", text_colour)
            .attr("textLength", FONT_SCALE_UP_FACTOR * text_width);
        let shadow_text = Element::new("text")
            .content(vec![Content::Text(content)])
            .attr("aria-hidden", true)
            .attr("x", x)
            .attr("y", 150.0 + VERTICAL_MARGIN)
            .attr("transform", FONT_SCALE_DOWN_VALUE)
            .attr("fill", shadow_colour)
            .attr("fill-opacity", ".3")
            .attr("textLength", FONT_SCALE_UP_FACTOR * text_width);
        let shadow = if SHADOW {
            Content::Element(shadow_text)
        } else {
            Content::Text("")
        };
        Content::List(ElementList::new(vec![shadow, Content::Element(text)]))
    }

    fn get_label_element(&self) -> Content {
        self.get_text_element(self.label_margin, self.label, self.label_colour, self.label_width)
    }

    fn get_message_element(&self) -> Content {
        self.get_text_element(self.message_margin, self.message, self.colour, self.message_width)
    }

    fn get_clip_path(&self, rx: f32) -> Content {
        Content::Element(
            Element::new("clipPath")
                .content(vec![Content::Element(
                    Element::new("rect")
                        .attr("width", self.width)
                        .attr("height", HEIGHT)
                        .attr("rx", rx)
                        .attr("fill", "#fff"),
                )])
                .attr("id", "r"),
        )
    }

    fn get_background_group_element(&self, with_gradient: bool) -> Element {
        let left_rect = Element::new("rect")
            .attr("width", self.left_width)
            .attr("height", HEIGHT)
            .attr("fill", self.label_colour);
        let right_rect = Element::new("rect")
            .attr("x", self.left_width)
            .attr("width", self.right_width)
            .attr("height", HEIGHT)
            .attr("fill", self.colour);
        let gradient = Element::new("rect")
            .attr("width", self.width)
            .attr("height", HEIGHT)
            .attr("fill", "url(#s)");
        let content = if with_gradient {
            vec![
                Content::Element(left_rect),
                Content::Element(right_rect),
                Content::Element(gradient),
            ]
        } else {
            vec![Content::Element(left_rect), Content::Element(right_rect)]
        };
        Element::new("g").content(content)
    }

    #[inline]
    fn get_foreground_group_element(&'a self) -> Content<'a> {
        Content::Element(
            Element::new("g")
                .content(vec![
                    self.logo_element.clone(),
                    self.get_label_element(),
                    self.get_message_element(),
                ])
                .attr("fill", "#fff")
                .attr("text-anchor", "middle")
                .attr("font-family", FONT_FAMILY)
                .attr("text-rendering", "geometricPrecision")
                .attr("font-size", 110),
        )
    }
}

impl<'a> Render for Badge<'a> {
    fn render(&self) -> String {
        let gradient = Element::new("linearGradient")
            .content(vec![
                Content::Element(
                    Element::new("stop")
                        .attr("offset", 0)
                        .attr("stop-color", "#bbb")
                        .attr("stop-opacity", ".1"),
                ),
                Content::Element(Element::new("stop").attr("offset", 1).attr("stop-opacity", ".1")),
            ])
            .attr("id", "s")
            .attr("x2", 0)
            .attr("y2", "100%");
        let clip_path = self.get_clip_path(3.0);
        let background_group = self.get_background_group_element(true).attr("clip-path", "url(#r)");
        render_badge(
            vec![
                Content::Element(gradient),
                clip_path,
                Content::Element(background_group),
                self.get_foreground_group_element(),
            ],
            self.left_width,
            self.right_width,
            HEIGHT,
            self.accessible_text.as_str(),
        )
    }
}
