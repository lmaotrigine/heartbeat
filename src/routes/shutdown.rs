use crate::AppState;
use axum::{
    body::Bytes,
    extract::{FromRequestParts, State},
    http::{request::Parts, StatusCode},
};
use axum_shutdown::Shutdown;
use hmac::{Hmac, Mac};
use serde_json::Value;
use sha2::Sha256;
use std::{borrow::Cow, str::FromStr};
use tracing::error;

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

trait ToHex {
    fn encode_hex<T: std::iter::FromIterator<char>>(&self) -> T;
}

impl<T: AsRef<[u8]>> ToHex for T {
    #[inline]
    fn encode_hex<U: std::iter::FromIterator<char>>(&self) -> U {
        BytesToHexChars::new(self.as_ref()).collect()
    }
}

trait MacExt {
    fn with_data(self, data: &[u8]) -> Self;
}

impl<M: Mac> MacExt for M {
    fn with_data(mut self, data: &[u8]) -> Self {
        self.update(data);
        self
    }
}

pub struct Secret(String);

impl Secret {
    fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    fn value(&self) -> &str {
        &self.0
    }
}

#[axum::async_trait]
impl<S: Send + Sync> FromRequestParts<S> for Secret {
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        let raw_sig = parts
            .headers
            .get("X-Hub-Signature-256")
            .ok_or((StatusCode::UNAUTHORIZED, "Missing signature"))?
            .as_bytes();
        std::str::from_utf8(raw_sig)
            .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid UTF-8"))?
            .trim()
            .strip_prefix("sha256=")
            .ok_or((StatusCode::BAD_REQUEST, "Malformed signature"))
            .map(Self::new)
    }
}

#[axum::debug_handler]
pub async fn deploy(
    State(AppState { config, .. }): State<AppState>,
    shutdown: Shutdown,
    request_secret: Secret,
    body: Bytes,
) -> Result<(StatusCode, Cow<'static, str>), (StatusCode, Cow<'static, str>)> {
    let secret = match config.github {
        Some(ref g) => g.webhook_secret.as_bytes(),
        None => return Err((StatusCode::SERVICE_UNAVAILABLE, "No secret configured".into())),
    };
    let sha = Hmac::<Sha256>::new_from_slice(secret)
        .map_err(|e| {
            error!("Failed to create HMAC: {e:?}");
            (StatusCode::INTERNAL_SERVER_ERROR, "An internal error occurred.".into())
        })?
        .with_data(body.as_ref())
        .finalize()
        .into_bytes()
        .encode_hex::<String>();
    if sha != request_secret.value() {
        return Err((StatusCode::UNAUTHORIZED, "Invalid signature".into()));
    }
    let raw = String::from_utf8_lossy(body.as_ref());
    let payload = match Value::from_str(&raw) {
        Err(_) => return Err((StatusCode::BAD_REQUEST, "Invalid JSON in request body".into())),
        Ok(v) => v,
    };
    if payload["action"] == "completed" && payload["workflow_run"]["name"] == ".github/workflows/docker.yml" {
        shutdown.notify();
    }
    Ok((StatusCode::OK, "".into()))
}
