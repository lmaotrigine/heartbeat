/**
 * Copyright (c) 2023 VJ <root@5ht2.me>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
use std::collections::HashMap;

use super::Regex;

pub trait Render {
    fn render(&self) -> String;
}

#[inline]
fn strip_xml_whitespace(xml: &str) -> String {
    let init = Regex::new(r">\s+").unwrap();
    let _final = Regex::new(r"<\s+").unwrap();
    let s = init.replace_all(xml, ">");
    _final.replace_all(&s, "<").to_string()
}

#[inline]
fn escape_xml(s: &str) -> String {
    s.replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace("\"", "&quot;")
        .replace("'", "&apos;")
}

#[derive(Debug, Clone)]
pub enum XmlContent<'a> {
    Text(&'a str),
    Element(XmlElement<'a>),
    List(ElementList<'a>),
}

#[derive(Debug, Clone)]
pub struct XmlElement<'a> {
    name: &'a str,
    content: Vec<XmlContent<'a>>,
    attrs: HashMap<&'a str, String>,
}

impl<'a> XmlElement<'a> {
    pub fn new(name: &'a str) -> Self {
        Self {
            name,
            content: Vec::new(),
            attrs: HashMap::new(),
        }
    }

    pub fn content(mut self, content: Vec<XmlContent<'a>>) -> Self {
        self.content = content;
        self
    }

    pub fn attr(mut self, key: &'a str, value: impl std::fmt::Display) -> Self {
        let r = format!("{}", value);
        let ins = match r.parse::<f32>() {
            Ok(f) => format!("{:.2}", f),
            Err(_) => r,
        };
        self.attrs.insert(key, ins);
        self
    }
}

impl<'a> Render for XmlElement<'a> {
    fn render(&self) -> String {
        let attrs_str = self
            .attrs
            .iter()
            .map(|(k, v)| format!(" {}=\"{}\"", k, escape_xml(v)))
            .collect::<String>();
        if self.content.len() > 0 {
            let content = self.content.iter().map(|x| x.render()).collect::<String>();
            strip_xml_whitespace(&format!("<{}{}>{}</{}>", self.name, attrs_str, content, self.name))
        } else {
            strip_xml_whitespace(&format!("<{}{}/>", self.name, attrs_str))
        }
    }
}

impl Render for XmlContent<'_> {
    fn render(&self) -> String {
        match self {
            XmlContent::Text(s) => escape_xml(s),
            XmlContent::Element(e) => e.render(),
            XmlContent::List(l) => l.render(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ElementList<'a> {
    content: Vec<XmlContent<'a>>,
}

impl<'a> ElementList<'a> {
    pub fn new(content: Vec<XmlContent<'a>>) -> Self {
        Self { content }
    }
}

impl Render for ElementList<'_> {
    fn render(&self) -> String {
        self.content.iter().map(|x| x.render()).collect::<String>()
    }
}
