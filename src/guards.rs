// Copyright (c) 2023 VJ <root@5ht2.me>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::{error::Error, models::AuthInfo, AppState, CONFIG};
use axum::{
    extract::{FromRef, FromRequestParts},
    http::{request::Parts, StatusCode},
};
use sqlx::PgPool;

#[axum::async_trait]
impl FromRequestParts<AppState> for AuthInfo {
    type Rejection = Error;

    async fn from_request_parts(req: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let Some(token) = req.headers.get("Authorization") else {
            return Err(Error::new(req.uri.path(), &req.method, StatusCode::UNAUTHORIZED).with_reason("No token provided.".into()));
        };
        let pool = PgPool::from_ref(state);
        let mut conn = pool.acquire().await.unwrap();
        sqlx::query_as!(
            AuthInfo,
            "SELECT id, name FROM devices WHERE token = $1;",
            token.to_str().unwrap()
        )
        .fetch_one(&mut *conn)
        .await
        .map_err(|_| {
            Error::new(req.uri.path(), &req.method, StatusCode::UNAUTHORIZED).with_reason("Invalid token.".into())
        })
    }
}

#[derive(Debug)]
pub struct Authorized(pub String);

#[axum::async_trait]
impl<A: Send + Sync> FromRequestParts<A> for Authorized {
    type Rejection = Error;

    async fn from_request_parts(req: &mut Parts, _: &A) -> Result<Self, Self::Rejection> {
        let expected = CONFIG.secret_key.clone();
        if expected.is_none() {
            return Err(Error::new(req.uri.path(), &req.method, StatusCode::NOT_FOUND).with_reason("Not found.".into()));
        }
        let token = match req.headers.get("Authorization") {
            Some(token) => token.to_str().ok().map(ToOwned::to_owned),
            None => {
                return Err(Error::new(req.uri.path(), &req.method, StatusCode::UNAUTHORIZED)
                    .with_reason("No token provided.".into()))
            }
        };
        if token.as_ref() == expected.as_ref() {
            Ok(Self(token.unwrap_or_default()))
        } else {
            Err(Error::new(req.uri.path(), &req.method, StatusCode::UNAUTHORIZED).with_reason("Invalid token.".into()))
        }
    }
}
