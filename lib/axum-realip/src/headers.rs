use std::net::IpAddr;

use axum::{
    extract::FromRequestParts,
    http::{request::Parts, HeaderMap},
};

use crate::{
    rejection::{InfallibleReject, StringReject},
    rfc7239::{ForwardedHeaderValue, Identifier},
};

#[derive(Debug)]
pub struct XForwardedFor(pub Vec<IpAddr>);

#[derive(Debug)]
pub struct LeftmostXForwardedFor(pub IpAddr);

#[derive(Debug)]
pub struct Forwarded(pub Vec<IpAddr>);

#[derive(Debug)]
pub struct LeftmostForwarded(pub IpAddr);

#[derive(Debug)]
pub struct XRealIp(pub IpAddr);

#[derive(Debug)]
pub struct FastlyClientIp(pub IpAddr);

#[derive(Debug)]
pub struct FlyClientIp(pub IpAddr);

#[derive(Debug)]
pub struct TrueClientIp(pub IpAddr);

#[derive(Debug)]
pub struct CfConnectingIp(pub IpAddr);

pub trait SingleIpHeader {
    const HEADER: &'static str;

    fn option_ip_from_headers(headers: &HeaderMap) -> Option<IpAddr> {
        headers
            .get(Self::HEADER)
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<IpAddr>().ok())
    }

    fn ip_from_headers(headers: &HeaderMap) -> Result<IpAddr, StringReject> {
        Self::option_ip_from_headers(headers).ok_or_else(|| Self::rejection())
    }

    fn rejection() -> StringReject {
        format!("No `{}` header, or the IP is invalid", Self::HEADER).into()
    }
}
pub trait MultipleIpHeader {
    const HEADER: &'static str;

    fn option_ips_from_header(value: &str) -> Vec<IpAddr>;

    fn ips_from_headers(headers: &HeaderMap) -> Vec<IpAddr> {
        headers
            .get_all(Self::HEADER)
            .iter()
            .filter_map(|v| v.to_str().ok())
            .flat_map(Self::option_ips_from_header)
            .collect()
    }

    fn option_leftmost_ip(headers: &HeaderMap) -> Option<IpAddr> {
        headers
            .get_all(Self::HEADER)
            .iter()
            .filter_map(|v| v.to_str().ok())
            .flat_map(Self::option_ips_from_header)
            .next()
    }

    fn leftmost_ip(headers: &HeaderMap) -> Result<IpAddr, StringReject> {
        Self::option_leftmost_ip(headers).ok_or_else(|| Self::rejection())
    }

    fn rejection() -> StringReject {
        format!("Couldn't find a valid IP in the `{}` header", Self::HEADER).into()
    }
}

macro_rules! impl_single_header {
    ($t:ty,$header:literal) => {
        impl SingleIpHeader for $t {
            const HEADER: &'static str = $header;
        }

        #[axum::async_trait]
        impl<S: Sync> FromRequestParts<S> for $t {
            type Rejection = StringReject;

            async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
                Ok(Self(
                    Self::option_ip_from_headers(&parts.headers).ok_or_else(Self::rejection)?,
                ))
            }
        }
    };
}

impl_single_header!(XRealIp, "X-Real-IP");
impl_single_header!(FastlyClientIp, "Fastly-Client-IP");
impl_single_header!(FlyClientIp, "Fly-Client-IP");
impl_single_header!(TrueClientIp, "True-Client-IP");
impl_single_header!(CfConnectingIp, "CF-Connecting-IP");

impl MultipleIpHeader for Forwarded {
    const HEADER: &'static str = "Forwarded";

    fn option_ips_from_header(value: &str) -> Vec<IpAddr> {
        let Ok(fv) = ForwardedHeaderValue::from_forwarded(value) else {
            return Vec::default();
        };
        fv.iter()
            .filter_map(|fs| fs.forwarded_for.as_ref())
            .filter_map(Identifier::ip)
            .collect()
    }
}

impl MultipleIpHeader for XForwardedFor {
    const HEADER: &'static str = "X-Forwarded-For";

    fn option_ips_from_header(value: &str) -> Vec<IpAddr> {
        value
            .split(',')
            .filter_map(|s| s.trim().parse::<IpAddr>().ok())
            .collect()
    }
}

#[axum::async_trait]
impl<S: Sync> FromRequestParts<S> for XForwardedFor {
    type Rejection = InfallibleReject;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        Ok(Self(Self::ips_from_headers(&parts.headers)))
    }
}

#[axum::async_trait]
impl<S: Sync> FromRequestParts<S> for LeftmostXForwardedFor {
    type Rejection = StringReject;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        Ok(Self(
            XForwardedFor::option_leftmost_ip(&parts.headers).ok_or_else(XForwardedFor::rejection)?,
        ))
    }
}

#[axum::async_trait]
impl<S: Sync> FromRequestParts<S> for Forwarded {
    type Rejection = InfallibleReject;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        Ok(Self(Self::ips_from_headers(&parts.headers)))
    }
}

#[axum::async_trait]
impl<S: Sync> FromRequestParts<S> for LeftmostForwarded {
    type Rejection = StringReject;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        Ok(Self(
            Forwarded::option_leftmost_ip(&parts.headers).ok_or_else(Forwarded::rejection)?,
        ))
    }
}
