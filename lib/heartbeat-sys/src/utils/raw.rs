#![allow(clippy::missing_errors_doc)]

use std::{fs, io, path::Path};

pub(crate) fn ensure_dir_exists<P: AsRef<Path>, F: FnOnce(&Path)>(
    path: P,
    callback: F,
) -> io::Result<bool> {
    if is_directory(&path) {
        Ok(false)
    } else {
        callback(path.as_ref());
        fs::create_dir_all(path).map(|()| true)
    }
}

pub(crate) fn is_directory<P: AsRef<Path>>(path: P) -> bool {
    fs::metadata(path).ok().as_ref().map(fs::Metadata::is_dir) == Some(true)
}

#[cfg(feature = "fs")]
pub fn remove_dir(path: &Path) -> io::Result<()> {
    if fs::symlink_metadata(path)?.file_type().is_symlink() {
        if cfg!(windows) {
            fs::remove_dir(path)
        } else {
            fs::remove_file(path)
        }
    } else {
        crate::fs::remove_dir_all(path)
    }
}

#[cfg(all(feature = "fs", windows))]
pub(crate) mod windows {
    use std::{ffi::OsStr, io, os::windows::ffi::OsStrExt};

    pub fn to_u16s<S: AsRef<OsStr>>(s: S) -> io::Result<Vec<u16>> {
        fn inner(s: &OsStr) -> io::Result<Vec<u16>> {
            let mut maybe_result = s.encode_wide().collect::<Vec<_>>();
            if maybe_result.iter().any(|&u| u == 0) {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "strings passed to WinAPI cannot contain NULs",
                ));
            }
            maybe_result.push(0);
            Ok(maybe_result)
        }
        inner(s.as_ref())
    }
}
