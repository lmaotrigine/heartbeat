use std::{ffi::OsStr, fs::File, io, path::Path};

#[cfg_attr(windows, path = "windows.rs")]
#[cfg_attr(not(windows), path = "unix.rs")]
mod inner;

pub mod errors;

use rayon::iter::{ParallelBridge, ParallelIterator};

pub trait FileExt {
    /// Locks the file for shared usage.
    ///
    /// # Errors
    ///
    /// Returns an error if the file is already locked.
    fn try_lock_exclusive(&self) -> io::Result<()>;
}

impl FileExt for File {
    fn try_lock_exclusive(&self) -> io::Result<()> {
        inner::flock(self)
    }
}

enum PathComponents<'a> {
    Path(&'a Path),
    Component(&'a PathComponents<'a>, &'a Path),
}

#[allow(clippy::missing_errors_doc)]
pub fn remove_dir_all<P: AsRef<Path>>(path: P) -> io::Result<()> {
    let p = inner::normalize(path.as_ref())?;
    let d = inner::open_dir(&p)?;
    let debug_root = PathComponents::Path(if p.has_root() { &p } else { Path::new(".") });
    rmrf(d, &debug_root)?;
    std::fs::remove_dir(&p)?;
    Ok(())
}

#[allow(clippy::missing_errors_doc)]
pub fn remove_dir_contents<P: AsRef<Path>>(path: P) -> io::Result<()> {
    let d = inner::open_dir(path.as_ref())?;
    let owned_handle = inner::duplicate_fd(&d)?;
    rmrf(owned_handle, &PathComponents::Path(path.as_ref()))?;
    Ok(())
}

fn rmrf(mut d: File, debug_root: &PathComponents<'_>) -> io::Result<File> {
    let dir_fd = inner::duplicate_fd(&d)?;
    let iter = inner::ReadDir::new(&mut d)?.par_bridge();
    iter.try_for_each(|dir_entry| -> io::Result<()> {
        let dir_entry = dir_entry?;
        let name = dir_entry.name();
        if name == OsStr::new(".") || name == OsStr::new("..") {
            return Ok(());
        }
        let dir_path = Path::new(&name);
        let dir_debug_root = PathComponents::Component(debug_root, dir_path);
        #[cfg(windows)]
        {
            let child_file = inner::open_path_at(&dir_fd, Path::new(name))?;
            let metadata = child_file.metadata()?;
            let is_dir = metadata.is_dir();
            let is_symlink = metadata.is_symlink();
            if is_dir && !is_symlink {
                rmrf(inner::duplicate_fd(&child_file)?, &dir_debug_root)?;
            }
            inner::delete_by_handle(child_file).map_err(|(_, e)| e)?;
        }
        #[cfg(not(windows))]
        {
            let child_result = inner::open_dir_at(&dir_fd, Path::new(&name));
            let is_dir = match child_result {
                Err(e) if e.raw_os_error() == Some(libc::ELOOP) => return Err(e),
                Err(_) => false,
                Ok(child_file) => {
                    let metadata = child_file.metadata()?;
                    let is_dir = metadata.is_dir();
                    if is_dir {
                        rmrf(child_file, &dir_debug_root)?;
                        inner::rmdir_at(&dir_fd, name)?;
                    }
                    is_dir
                }
            };
            if !is_dir {
                inner::unlink_at(&dir_fd, name)?;
            }
        }
        Ok(())
    })?;
    Ok(dir_fd)
}
