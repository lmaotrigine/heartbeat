// Copyright (c) 2023 VJ <root@5ht2.me>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use axum::{
    http::{Method, Request, Response, StatusCode},
    middleware::Next,
    response::IntoResponse,
    RequestExt,
};
use axum_realip::RealIp;
use core::fmt;
use tracing::warn;

use crate::templates::error;

#[derive(Debug)]
pub struct Error {
    path: String,
    method: Method,
    message: String,
    status: StatusCode,
}

impl Error {
    pub fn new(path: &str, method: &Method, status: StatusCode) -> Self {
        Self {
            path: path.into(),
            method: method.clone(),
            status,
            message: status.canonical_reason().unwrap_or_default().to_string(),
        }
    }

    #[allow(clippy::missing_const_for_fn)] // false positive
    pub fn with_reason(self, message: String) -> Self {
        Self { message, ..self }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let markup = error(&self.message, self.method.as_str(), &self.path);
        write!(f, "{}", markup.0)
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        let markup = error(&self.message, self.method.as_str(), &self.path);
        let res = markup.into_response();
        Response::builder().status(self.status).body(res.into_body()).unwrap()
    }
}

pub async fn handle_errors<B: Send + 'static>(mut req: Request<B>, next: Next<B>) -> impl IntoResponse {
    let path = req.uri().path().to_owned();
    let method = req.method().clone();
    let RealIp(ip) = req.extract_parts::<RealIp>().await.unwrap();
    let headers = req.headers().clone();
    let auth = headers
        .get("Authorization")
        .map_or("", |auth| auth.to_str().unwrap_or_default());
    let resp = next.run(req).await;
    let status = resp.status();
    match status {
        StatusCode::METHOD_NOT_ALLOWED | StatusCode::NOT_FOUND => {
            let code = status.as_u16();
            warn!("returned {code} to {ip} - tried to {method} {path} with Authorization {auth}");
            Err(Error::new(&path, &method, status))
        }
        _ => Ok(resp),
    }
}
