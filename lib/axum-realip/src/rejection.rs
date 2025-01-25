use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use core::convert::Infallible;

pub struct StringReject(pub String);
pub type InfallibleReject = (StatusCode, Infallible);

impl<T: Into<String>> From<T> for StringReject {
    fn from(value: T) -> Self {
        Self(value.into())
    }
}

impl IntoResponse for StringReject {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, self.0).into_response()
    }
}
