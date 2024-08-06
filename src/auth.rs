// Copyright (c) 2023 Isis <root@5ht2.me>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::{config::Config, error::Error, AppState};
use axum::{
    extract::{FromRef, FromRequestParts},
    http::{request::Parts, StatusCode},
};
use sqlx::PgPool;
use tracing::error;

#[derive(Debug)]
pub struct Device {
    pub id: i64,
    pub name: Option<String>,
}

#[axum::async_trait]
impl FromRequestParts<AppState> for Device {
    type Rejection = Error;

    async fn from_request_parts(req: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let Some(token) = req.headers.get("Authorization").and_then(|t| t.to_str().ok()) else {
            return Err(Error::new(
                req.uri.path(),
                &req.method,
                StatusCode::UNAUTHORIZED,
                &state.config.server_name,
            )
            .with_reason("No token provided."));
        };
        let pool = PgPool::from_ref(state);
        let mut conn = pool.acquire().await.map_err(|e| {
            error!("Failed to acquire connection from pool. {e:?}");
            Error::new(
                req.uri.path(),
                &req.method,
                StatusCode::INTERNAL_SERVER_ERROR,
                &state.config.server_name,
            )
        })?;
        sqlx::query_as!(
            Device,
            "SELECT id, name FROM heartbeat.devices WHERE token = $1;",
            token
        )
        .fetch_one(&mut *conn)
        .await
        .map_err(|_| {
            Error::new(
                req.uri.path(),
                &req.method,
                StatusCode::UNAUTHORIZED,
                &state.config.server_name,
            )
            .with_reason("Invalid token.")
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Master;

#[axum::async_trait]
impl FromRequestParts<AppState> for Master {
    type Rejection = Error;

    async fn from_request_parts(req: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let config: &'static Config = FromRef::from_ref(state);
        let expected = &config.secret_key;
        if expected.is_empty() {
            return Err(Error::new(
                req.uri.path(),
                &req.method,
                StatusCode::NOT_FOUND,
                &state.config.server_name,
            )
            .with_reason("Not found."));
        }
        let token = req.headers.get("Authorization").map_or_else(
            || {
                return Err(Error::new(
                    req.uri.path(),
                    &req.method,
                    StatusCode::UNAUTHORIZED,
                    &state.config.server_name,
                )
                .with_reason("No token provided."));
            },
            |t| Ok(t.to_str().unwrap_or_default()),
        )?;
        if token == **expected {
            Ok(Self)
        } else {
            Err(Error::new(
                req.uri.path(),
                &req.method,
                StatusCode::UNAUTHORIZED,
                &state.config.server_name,
            )
            .with_reason("Invalid token."))
        }
    }
}
