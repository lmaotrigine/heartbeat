use crate::{
    traits::{MacExt, ToHex},
    AppState,
};
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
        Some(ref g) if !g.webhook_secret.is_empty() => g.webhook_secret.as_bytes(),
        _ => return Err((StatusCode::SERVICE_UNAVAILABLE, "No secret configured".into())),
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
