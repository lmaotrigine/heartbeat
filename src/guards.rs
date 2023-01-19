use crate::models::AuthInfo;
use crate::CONFIG;
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome};
use rocket::Request;
use rocket_db_pools::Connection;

use crate::DbPool;

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthInfo {
    type Error = &'static str;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let token = match req.headers().get_one("Authorization") {
            Some(token) => token,
            None => return Outcome::Failure((Status::Unauthorized, "No token provided.")),
        };
        let mut conn = req.guard::<Connection<DbPool>>().await.unwrap();
        match sqlx::query_as!(AuthInfo, "SELECT id, name FROM devices WHERE token = $1;", token)
            .fetch_one(&mut *conn)
            .await
        {
            Ok(info) => Outcome::Success(info),
            Err(_) => Outcome::Failure((Status::Unauthorized, "Invalid token.")),
        }
    }
}

#[derive(Debug)]
pub struct Authorized(pub String);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Authorized {
    type Error = &'static str;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let expected = &CONFIG.secret_key;
        if expected.is_none() {
            return Outcome::Failure((Status::NotFound, "Not found."));
        }
        let token = match req.headers().get_one("Authorization") {
            Some(token) => token,
            None => return Outcome::Failure((Status::Unauthorized, "No token provided.")),
        };
        if token == expected.as_ref().unwrap() {
            Outcome::Success(Authorized(token.to_string()))
        } else {
            Outcome::Failure((Status::Unauthorized, "Invalid token."))
        }
    }
}
