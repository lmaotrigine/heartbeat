// Copyright (c) 2023 VJ <root@5ht2.me>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use super::Regex;
use std::collections::HashMap;

pub trait Render {
    fn render(&self) -> String;
}

#[inline]
fn strip_xml_whitespace(xml: &str) -> String {
    let init = Regex::new(r">\s+").unwrap();
    let final_ = Regex::new(r"<\s+").unwrap();
    let s = init.replace_all(xml, ">");
    final_.replace_all(&s, "<").to_string()
}

#[inline]
fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[derive(Debug, Clone)]
pub enum Content<'a> {
    Text(&'a str),
    Element(Element<'a>),
    List(ElementList<'a>),
}

#[derive(Debug, Clone)]
pub struct Element<'a> {
    name: &'a str,
    content: Vec<Content<'a>>,
    attrs: HashMap<&'a str, String>,
}

impl<'a> Element<'a> {
    pub fn new(name: &'a str) -> Self {
        Self {
            name,
            content: Vec::new(),
            attrs: HashMap::new(),
        }
    }

    pub fn content(mut self, content: Vec<Content<'a>>) -> Self {
        self.content = content;
        self
    }

    pub fn attr(mut self, key: &'a str, value: impl std::fmt::Display) -> Self {
        let r = format!("{value}");
        let ins = r.parse::<f32>().map_or(r, |f| format!("{f:.2}"));
        self.attrs.insert(key, ins);
        self
    }
}

impl<'a> Render for Element<'a> {
    fn render(&self) -> String {
        let attrs_str = self
            .attrs
            .iter()
            .map(|(k, v)| format!(" {}=\"{}\"", k, escape_xml(v)))
            .collect::<String>();
        if self.content.is_empty() {
            strip_xml_whitespace(&format!("<{}{}/>", self.name, attrs_str))
        } else {
            let content = self.content.iter().map(Render::render).collect::<String>();
            strip_xml_whitespace(&format!("<{}{}>{}</{}>", self.name, attrs_str, content, self.name))
        }
    }
}

impl Render for Content<'_> {
    fn render(&self) -> String {
        match self {
            Content::Text(s) => escape_xml(s),
            Content::Element(e) => e.render(),
            Content::List(l) => l.render(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ElementList<'a> {
    content: Vec<Content<'a>>,
}

impl<'a> ElementList<'a> {
    pub fn new(content: Vec<Content<'a>>) -> Self {
        Self { content }
    }
}

impl Render for ElementList<'_> {
    fn render(&self) -> String {
        self.content.iter().map(Render::render).collect::<String>()
    }
}
