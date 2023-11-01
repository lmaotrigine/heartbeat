use std::path::{Component, Path, PathBuf};

use axum::{
    http::{header, StatusCode, Uri},
    response::{IntoResponse, Response},
};

#[derive(Clone, rust_embed::RustEmbed)]
#[folder = "static"]
pub struct Static;

pub(super) async fn serve_static_file(uri: Uri) -> StaticFile {
    let path = uri.path().trim_start_matches('/');
    StaticFile(build_and_validate_path(path))
}

pub struct StaticFile(Option<PathBuf>);

impl IntoResponse for StaticFile {
    fn into_response(self) -> Response {
        let Some(path) = self.0 else {
            return (StatusCode::NOT_FOUND, "Not found").into_response();
        };
        match Static::get(&path.as_path().to_string_lossy()) {
            Some(content) => {
                let mime = mime_guess::from_path(&path).first_or_octet_stream();
                ([(header::CONTENT_TYPE, mime.as_ref())], content.data).into_response()
            }
            None => (StatusCode::NOT_FOUND, "Not found").into_response(),
        }
    }
}

fn build_and_validate_path(requested_path: &str) -> Option<PathBuf> {
    let path_decoded = percent_encoding::percent_decode(requested_path.as_ref())
        .decode_utf8()
        .ok()?;
    let path_decoded = Path::new(&*path_decoded);
    let mut ret = PathBuf::new();
    for component in path_decoded.components() {
        match component {
            Component::Normal(comp) => {
                if Path::new(&comp).components().all(|c| matches!(c, Component::Normal(_))) {
                    ret.push(comp);
                } else {
                    return None;
                }
            }
            Component::CurDir => {}
            Component::Prefix(_) | Component::RootDir | Component::ParentDir => return None,
        }
    }
    Some(ret)
}
