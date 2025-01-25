use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum HeartbeatError {
    #[error("could not remove '{name}' directory: '{}'", .path.display())]
    RemovingDirectory { name: &'static str, path: PathBuf },
}
