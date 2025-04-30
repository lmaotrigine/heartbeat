#![no_std]

extern crate alloc;

use core::fmt::{self, Arguments, Display, Write};

use alloc::{borrow::Cow, boxed::Box, string::String, sync::Arc};

pub use html_macros::html;

mod escape;

pub struct Escaper<'a>(&'a mut String);

impl<'a> Escaper<'a> {
    pub fn new(buffer: &'a mut String) -> Self {
        Self(buffer)
    }
}

impl<'a> fmt::Write for Escaper<'a> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        escape::escape_to_string(s, self.0);
        Ok(())
    }
}

pub trait Render {
    fn render(&self) -> Markup {
        let mut buffer = String::new();
        self.render_to(&mut buffer);
        PreEscaped(buffer)
    }

    fn render_to(&self, w: &mut String) {
        w.push_str(&self.render().into_string());
    }
}

impl Render for str {
    fn render_to(&self, w: &mut String) {
        escape::escape_to_string(self, w)
    }
}

impl Render for String {
    fn render_to(&self, w: &mut String) {
        str::render_to(self, w)
    }
}

impl<'a> Render for Cow<'a, str> {
    fn render_to(&self, w: &mut String) {
        str::render_to(self, w)
    }
}

impl<'a> Render for Arguments<'a> {
    fn render_to(&self, w: &mut String) {
        let _ = Escaper::new(w).write_fmt(*self);
    }
}

impl<'a, T: Render + ?Sized> Render for &'a T {
    fn render_to(&self, w: &mut String) {
        T::render_to(self, w);
    }
}

impl<'a, T: Render + ?Sized> Render for &'a mut T {
    fn render_to(&self, w: &mut String) {
        T::render_to(self, w);
    }
}

impl<T: Render + ?Sized> Render for Box<T> {
    fn render_to(&self, w: &mut String) {
        T::render_to(self, w)
    }
}

impl<T: Render + ?Sized> Render for Arc<T> {
    fn render_to(&self, w: &mut String) {
        T::render_to(self, w)
    }
}

macro_rules! impl_render_with_display {
    ($($ty:ty)*) => {
        $(
            impl Render for $ty {
                fn render_to(&self, w: &mut String) {
                    format_args!("{self}").render_to(w);
                }
            }
        )*
    };
}

impl_render_with_display! {
    char f32 f64
}

macro_rules! impl_render_with_itoa {
    ($($ty:ty)*) => {
        $(
            impl Render for $ty {
                fn render_to(&self, w: &mut String) {
                    w.push_str(itoa::Buffer::new().format(*self));
                }
            }
        )*
    };
}

impl_render_with_itoa! {
    i8 i16 i32 i64 i128 isize
    u8 u16 u32 u64 u128 usize
}

pub fn display(value: impl Display) -> impl Render {
    struct DisplayWrapper<T>(T);

    impl<T: Display> Render for DisplayWrapper<T> {
        fn render_to(&self, w: &mut String) {
            format_args!("{}", self.0).render_to(w);
        }
    }

    DisplayWrapper(value)
}

#[derive(Debug, Clone, Copy)]
pub struct PreEscaped<T>(pub T);

impl<T: AsRef<str>> Render for PreEscaped<T> {
    fn render_to(&self, w: &mut String) {
        w.push_str(self.0.as_ref());
    }
}

pub type Markup = PreEscaped<String>;

impl<T: Into<String>> PreEscaped<T> {
    pub fn into_string(self) -> String {
        self.0.into()
    }
}

impl<T: Into<String>> From<PreEscaped<T>> for String {
    fn from(value: PreEscaped<T>) -> Self {
        value.into_string()
    }
}

impl<T: Default> Default for PreEscaped<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

pub const DOCTYPE: PreEscaped<&'static str> = PreEscaped("<!DOCTYPE html>");

#[cfg(feature = "rocket")]
mod rocket_support {
    extern crate std;
    use crate::PreEscaped;
    use alloc::string::String;
    use rocket::{
        http::ContentType,
        request::Request,
        response::{Responder, Response},
    };
    use std::io::Cursor;

    impl Responder<'static> for PreEscaped<String> {
        fn respond_to(self, _: &Request) -> rocket::response::Result<'static> {
            Response::build()
                .header(ContentType::HTML)
                .sized_body(Cursor::new(self.0))
                .ok()
        }
    }
}

#[cfg(feature = "actix-web")]
mod actix_support {
    use crate::PreEscaped;
    use actix_web_dep::{http::header, HttpRequest, HttpResponse, Responder};
    use alloc::string::String;

    impl Responder for PreEscaped<String> {
        type Body = String;

        fn respond_to(self, _: &HttpRequest) -> HttpResponse<Self::Body> {
            HttpResponse::Ok()
                .content_type(header::ContentType::html())
                .message_body(self.0)
                .unwrap()
        }
    }
}

#[cfg(feature = "tide")]
mod tide_support {
    use crate::PreEscaped;
    use alloc::string::String;
    use tide::{http::mime, Response, StatusCode};

    impl From<PreEscaped<String>> for Response {
        fn from(value: PreEscaped<String>) -> Self {
            Response::builder(StatusCode::Ok)
                .body(value.into_string())
                .content_type(mime::HTML)
                .build()
        }
    }
}

#[cfg(feature = "axum")]
mod axum_support {
    use crate::PreEscaped;
    use alloc::string::String;
    use axum_core::{response::IntoResponse, response::Response};
    use http::{header, HeaderMap, HeaderValue};

    impl IntoResponse for PreEscaped<String> {
        fn into_response(self) -> Response {
            let mut headers = HeaderMap::new();
            headers.insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static("text/html; charset=utf-8"),
            );
            (headers, self.0).into_response()
        }
    }
}

pub mod macro_private {
    #[macro_export]
    macro_rules! render_to {
        ($x:expr, $buffer:expr) => {{
            use $crate::macro_private::*;
            match ChooseRenderOrDisplay($x) {
                x => (&&x).implements_render_or_display().render_to(x.0, $buffer),
            }
        }};
    }

    use core::fmt::Display;

    use alloc::string::String;
    pub use render_to;

    use crate::{display, Render};

    pub struct ChooseRenderOrDisplay<T>(pub T);

    pub struct ViaRenderTag;
    pub struct ViaDisplayTag;

    pub trait ViaRender {
        fn implements_render_or_display(&self) -> ViaRenderTag {
            ViaRenderTag
        }
    }

    pub trait ViaDisplay {
        fn implements_render_or_display(&self) -> ViaDisplayTag {
            ViaDisplayTag
        }
    }

    impl<T: Render> ViaRender for &ChooseRenderOrDisplay<T> {}
    impl<T: Display> ViaDisplay for ChooseRenderOrDisplay<T> {}

    impl ViaRenderTag {
        pub fn render_to<T: Render + ?Sized>(self, value: &T, buffer: &mut String) {
            value.render_to(buffer);
        }
    }

    impl ViaDisplayTag {
        pub fn render_to<T: Display + ?Sized>(self, value: &T, buffer: &mut String) {
            display(value).render_to(buffer);
        }
    }
}
