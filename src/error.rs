// Copyright (c) 2023 Isis <root@5ht2.me>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::{templates::error, AppState};
use axum::{
    body::Body,
    extract::State,
    http::{Method, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    RequestExt,
};
use axum_realip::RealIp;
use tracing::{error, warn};

#[derive(Debug)]
pub struct Error {
    path: String,
    method: Method,
    message: &'static str,
    status: StatusCode,
    server_name: String,
}

impl Error {
    pub fn new(path: &str, method: &Method, status: StatusCode, server_name: &str) -> Self {
        Self {
            path: path.into(),
            method: method.clone(),
            status,
            server_name: server_name.into(),
            message: status.canonical_reason().unwrap_or_default(),
        }
    }

    #[allow(clippy::missing_const_for_fn)] // false positive
    pub fn with_reason(self, message: &'static str) -> Self {
        Self { message, ..self }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        if self.path.starts_with("/api") {
            return (self.status, self.message).into_response();
        }
        let markup = error(self.message, self.method.as_str(), &self.path, &self.server_name);
        (self.status, markup).into_response()
    }
}

/// An Axum middleware that serves error pages for unhandled client errors.
/// This also logs the client IP and the attempted request for debugging
/// purposes.
///
/// # Errors
///
/// This function returns an error if there was an unhandled client error status code,
/// or if the client IP could not be determined.
pub async fn handle_errors(
    State(state): State<AppState>,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, Error> {
    let path = req.uri().path().to_owned();
    let method = req.method().clone();
    let server_name = &state.config.server_name;
    let ip = match req.extract_parts::<RealIp>().await {
        Ok(RealIp(ip)) => ip,
        Err(e) => {
            error!("Failed to get Real IP from request: {e:?}\n{req:#?}");
            return Err(Error::new(
                &path,
                &method,
                StatusCode::INTERNAL_SERVER_ERROR,
                server_name,
            ));
        }
    };
    let headers = req.headers().clone();
    let auth = headers
        .get("Authorization")
        .map_or("", |auth| auth.to_str().unwrap_or_default());
    let resp = next.run(req).await;
    let status = resp.status();
    match status {
        StatusCode::METHOD_NOT_ALLOWED | StatusCode::NOT_FOUND | StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
            let code = status.as_u16();
            warn!("returned {code} to {ip} - tried to {method} {path} with Authorization {auth}");
            Err(Error::new(&path, &method, status, server_name))
        }
        _ => Ok(resp),
    }
}
