#![forbid(unsafe_code)]
#![deny(clippy::pedantic, clippy::nursery)]

use std::net::{IpAddr, SocketAddr};

mod headers;
mod local;
mod rejection;
mod rfc7239;

use axum::{
    extract::{ConnectInfo, FromRequestParts},
    http::{request::Parts, Extensions, StatusCode},
};
pub use headers::{
    CfConnectingIp, FastlyClientIp, FlyClientIp, Forwarded, LeftmostForwarded, LeftmostXForwardedFor, MultipleIpHeader,
    SingleIpHeader, TrueClientIp, XForwardedFor, XRealIp,
};
use local::IsLocalAddr;
pub use rfc7239::ForwardedHeaderValueParseError;

#[derive(Debug, Clone, Copy)]
pub struct RealIp(pub IpAddr);

#[axum::async_trait]
impl<S: Sync> FromRequestParts<S> for RealIp {
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        XForwardedFor::option_leftmost_ip(&parts.headers)
            .or_else(|| Forwarded::option_leftmost_ip(&parts.headers))
            .or_else(|| XRealIp::option_ip_from_headers(&parts.headers))
            .or_else(|| FlyClientIp::option_ip_from_headers(&parts.headers))
            .or_else(|| FastlyClientIp::option_ip_from_headers(&parts.headers))
            .or_else(|| TrueClientIp::option_ip_from_headers(&parts.headers))
            .or_else(|| CfConnectingIp::option_ip_from_headers(&parts.headers))
            .filter(|ip| !ip.is_local())
            .or_else(|| option_connect_info(&parts.extensions))
            .map(Self)
            .ok_or((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Can't extract RealIp, provide `axum::extract::ConnectInfo`",
            ))
    }
}

fn option_connect_info(extensions: &Extensions) -> Option<IpAddr> {
    extensions
        .get::<ConnectInfo<SocketAddr>>()
        .map(|ConnectInfo(addr)| addr.ip())
}
