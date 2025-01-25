use std::{
    net::{IpAddr, SocketAddr},
    str::FromStr,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Protocol {
    Http,
    Https,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Identifier {
    SocketAddr(SocketAddr),
    IpAddr(IpAddr),
    String(String),
    Unknown,
}

impl FromStr for Protocol {
    type Err = ForwardedHeaderValueParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "http" => Ok(Self::Http),
            "https" => Ok(Self::Https),
            _ => Err(ForwardedHeaderValueParseError::InvalidProtocol),
        }
    }
}

impl Identifier {
    #[must_use]
    pub const fn ip(&self) -> Option<IpAddr> {
        match self {
            Self::SocketAddr(s) => Some(s.ip()),
            Self::IpAddr(i) => Some(*i),
            _ => None,
        }
    }
}

impl FromStr for Identifier {
    type Err = ForwardedHeaderValueParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim().trim_matches('"').trim_matches('\'');
        if s == "unknown" {
            return Ok(Self::Unknown);
        }
        s.parse::<SocketAddr>().map_or_else(
            |_| {
                s.parse::<IpAddr>().map_or_else(
                    |_| {
                        if s.starts_with('[') && s.ends_with(']') {
                            s[1..(s.len() - 1)].parse::<IpAddr>().map_or_else(
                                |_| Err(ForwardedHeaderValueParseError::InvalidAddress),
                                |ip_addr| Ok(Self::IpAddr(ip_addr)),
                            )
                        } else if s.starts_with('_') {
                            Ok(Self::String(s.to_string()))
                        } else {
                            Err(ForwardedHeaderValueParseError::InvalidObfuscatedNode(s.to_string()))
                        }
                    },
                    |ip_addr| Ok(Self::IpAddr(ip_addr)),
                )
            },
            |socket_addr| Ok(Self::SocketAddr(socket_addr)),
        )
    }
}

fn values_from_header(header_value: &str) -> impl Iterator<Item = &str> {
    header_value.trim().split(',').filter_map(|i| {
        let trimmed = i.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    })
}

#[derive(Debug, Default)]
pub struct ForwardedStanza {
    pub forwarded_by: Option<Identifier>,
    pub forwarded_for: Option<Identifier>,
    pub forwarded_host: Option<String>,
    pub forwarded_proto: Option<Protocol>,
}

impl FromStr for ForwardedStanza {
    type Err = ForwardedHeaderValueParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut rv = Self::default();
        let s = s.trim();
        for part in s.split(';') {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }
            if let Some((key, value)) = part.split_once('=') {
                match key.to_ascii_lowercase().as_str() {
                    "by" => rv.forwarded_by = Some(value.parse()?),
                    "for" => rv.forwarded_for = Some(value.parse()?),
                    "host" => {
                        rv.forwarded_host = {
                            if value.starts_with('"') && value.ends_with('"') {
                                Some(value[1..(value.len() - 1)].replace("\\\"", "\"").replace("\\\\", "\\"))
                            } else {
                                Some(value.to_string())
                            }
                        }
                    }
                    "proto" => rv.forwarded_proto = Some(value.parse()?),
                    _ => continue,
                }
            } else {
                return Err(ForwardedHeaderValueParseError::InvalidPart(part.to_owned()));
            }
        }
        Ok(rv)
    }
}
#[derive(Debug)]
pub struct ForwardedHeaderValue {
    values: Vec<ForwardedStanza>,
}

impl ForwardedHeaderValue {
    pub fn from_forwarded(header_value: &str) -> Result<Self, ForwardedHeaderValueParseError> {
        let this = values_from_header(header_value)
            .map(str::parse)
            .collect::<Result<Vec<_>, _>>()
            .map(|values| Self { values })?;
        if this.values.is_empty() {
            Err(ForwardedHeaderValueParseError::EmptyHeader)
        } else {
            Ok(this)
        }
    }

    pub fn iter(&self) -> ForwardedHeaderValueIter {
        ForwardedHeaderValueIter {
            head: self.values.first(),
            tail: &self.values[1..],
        }
    }
}

pub struct ForwardedHeaderValueIter<'a> {
    head: Option<&'a ForwardedStanza>,
    tail: &'a [ForwardedStanza],
}

impl<'a> Iterator for ForwardedHeaderValueIter<'a> {
    type Item = &'a ForwardedStanza;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(head) = self.head.take() {
            Some(head)
        } else if let Some((first, rest)) = self.tail.split_first() {
            self.tail = rest;
            Some(first)
        } else {
            None
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ForwardedHeaderValueParseError {
    #[error("header is empty")]
    EmptyHeader,
    #[error("stanza contained invalid part: {0}")]
    InvalidPart(String),
    #[error("Stanza specified an invalid protocol")]
    InvalidProtocol,
    #[error("identifier specified an invalid or malformed IP address")]
    InvalidAddress,
    #[error("identifier uses an obfuscated node ({0:?}) that is invalid")]
    InvalidObfuscatedNode(String),
}
