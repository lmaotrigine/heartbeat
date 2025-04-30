use super::raw;
use std::{io, path::Path};

/// creates a directory if it doesn't exist, and invokes a callback while
/// creating it.
///
/// # Errors
///
/// returns an error if directory creation fails.
pub fn ensure_dir_exists<F: FnOnce(&Path)>(path: &Path, callback: F) -> io::Result<bool> {
    // TODO: better API?
    raw::ensure_dir_exists(path, callback)
}
