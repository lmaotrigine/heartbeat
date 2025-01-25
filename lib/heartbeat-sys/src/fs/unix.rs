use std::{
    ffi::{CStr, CString, OsStr, OsString},
    fs::{File, OpenOptions},
    io,
    marker::PhantomData,
    os::unix::{
        ffi::OsStrExt,
        fs::OpenOptionsExt,
        io::{AsRawFd, FromRawFd},
    },
    path::{Path, PathBuf},
    ptr,
};

#[cfg(any(
    target_os = "aix",
    target_os = "macos",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "ios",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "illumos"
))]
use libc::openat as openat64;
#[cfg(not(any(
    target_os = "aix",
    target_os = "macos",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "ios",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "illumos"
)))]
use libc::openat64;

#[cfg(target_os = "aix")]
use libc::_Errno as errno_location;
#[cfg(any(target_os = "illumos", target_os = "solaris"))]
use libc::___errno as errno_location;
#[cfg(any(target_os = "android", target_os = "netbsd", target_os = "openbsd"))]
use libc::__errno as errno_location;
#[cfg(any(
    target_os = "linux",
    target_os = "redox",
    target_os = "dragonfly",
    target_os = "fuchsia"
))]
use libc::__errno_location as errno_location;
#[cfg(any(target_os = "freebsd", target_os = "ios", target_os = "macos"))]
use libc::__error as errno_location;
#[cfg(target_os = "haiku")]
use libc::_errnop as errno_location;

fn clear_errno() {
    unsafe {
        *errno_location() = 0;
    }
}

#[cfg(not(target_os = "solaris"))]
pub(super) fn flock(file: &File) -> io::Result<()> {
    let ret = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) };
    if ret < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

#[cfg(target_os = "solaris")]
pub(super) fn flock(file: &File) -> io::Result<()> {
    let fl = libc::flock {
        l_whence: 0,
        l_start: 0,
        l_len: 0,
        l_type: libc::F_WRLCK,
        l_pad: [0; 4],
        l_pid: 0,
        l_sysid: 0,
    };
    let ret = unsafe { libc::fcntl(file.as_raw_fd(), libc::F_SETLK, &fl) };
    match ret {
        -1 => match io::Error::last_os_error().raw_os_error() {
            Some(libc::EACCES) => return Err(io::Error::from_raw_os_error(libc::EWOULDBLOCK)),
            _ => return Err(io::Error::last_os_error()),
        },
        _ => Ok(()),
    }
}

pub(super) fn normalize(path: &Path) -> io::Result<PathBuf> {
    path.canonicalize()
}

pub(super) fn duplicate_fd(f: &File) -> io::Result<File> {
    let source_fd = f.as_raw_fd();
    let fd = unsafe { libc::fcntl(source_fd, libc::F_DUPFD_CLOEXEC, 0) };
    if fd == -1 {
        return Err(io::Error::last_os_error());
    }
    Ok(unsafe { File::from_raw_fd(fd) })
}

pub(super) fn open_dir(p: &Path) -> io::Result<File> {
    let mut options = OpenOptions::new();
    options.read(true);
    options.custom_flags(libc::O_NOFOLLOW);
    options.open(p)
}

pub(super) fn open_dir_at(d: &File, p: &Path) -> io::Result<File> {
    let flags = libc::O_RDONLY | libc::O_NOFOLLOW | libc::O_CLOEXEC | libc::O_NOCTTY;
    let path = CString::new(p.as_os_str().as_bytes())
        .map_err(|_| io::Error::from(io::ErrorKind::InvalidInput))?;
    let fd = loop {
        match unsafe { openat64(d.as_raw_fd(), path.as_ptr(), flags, 0o777) } {
            -1 => {
                let err = io::Error::last_os_error();
                if err.kind() != io::ErrorKind::Interrupted {
                    return Err(err);
                }
            }
            n => break n,
        }
    };
    Ok(unsafe { File::from_raw_fd(fd) })
}

pub(super) fn rmdir_at<P: AsRef<Path>>(d: &File, p: P) -> io::Result<()> {
    _unlinkat(d, p.as_ref(), libc::AT_REMOVEDIR)
}

pub(super) fn unlink_at<P: AsRef<Path>>(d: &File, p: P) -> io::Result<()> {
    _unlinkat(d, p.as_ref(), 0)
}

fn _unlinkat(d: &File, p: &Path, flags: libc::c_int) -> io::Result<()> {
    let path = CString::new(p.as_os_str().as_bytes())
        .map_err(|_| io::Error::from(io::ErrorKind::InvalidInput))?;
    loop {
        match unsafe { libc::unlinkat(d.as_raw_fd(), path.as_ptr(), flags) } {
            -1 => {
                let err = io::Error::last_os_error();
                if err.kind() != io::ErrorKind::Interrupted {
                    return Err(err);
                }
            }
            _ => return Ok(()),
        }
    }
}

pub(super) struct ReadDir<'a> {
    _marker: PhantomData<&'a mut File>,
    dir: Option<ptr::NonNull<libc::DIR>>,
}

unsafe impl<'a> Send for ReadDir<'a> where Box<libc::DIR>: Send {}
unsafe impl<'a> Sync for ReadDir<'a> where Box<libc::DIR>: Sync {}

impl<'a> ReadDir<'a> {
    pub(super) fn new(d: &'a mut File) -> io::Result<Self> {
        let new_fd = loop {
            match unsafe { libc::fcntl(d.as_raw_fd(), libc::F_DUPFD_CLOEXEC, 0) } {
                -1 => {
                    let err = io::Error::last_os_error();
                    if err.kind() != io::ErrorKind::Interrupted {
                        return Err(err);
                    }
                }
                n => break n,
            }
        };
        let mut dir = Some(
            ptr::NonNull::new(unsafe { libc::fdopendir(new_fd) }).ok_or_else(|| {
                let _ = unsafe { File::from_raw_fd(new_fd) };
                io::Error::last_os_error()
            })?,
        );
        if let Some(d) = dir.as_mut() {
            unsafe { libc::rewinddir(d.as_mut()) };
        }
        Ok(Self {
            _marker: PhantomData,
            dir,
        })
    }

    fn close(&mut self) -> io::Result<()> {
        if let Some(mut dir) = self.dir {
            let res = unsafe { libc::closedir(dir.as_mut()) };
            self.dir = None;
            match res {
                -1 => Err(io::Error::last_os_error()),
                _ => Ok(()),
            }
        } else {
            Ok(())
        }
    }
}

impl Drop for ReadDir<'_> {
    fn drop(&mut self) {
        let _ = self.close();
    }
}

impl Iterator for ReadDir<'_> {
    type Item = io::Result<DirEntry>;

    fn next(&mut self) -> Option<Self::Item> {
        let dir = unsafe { self.dir?.as_mut() };
        clear_errno();
        ptr::NonNull::new(unsafe { libc::readdir(dir) })
            .map(|e| {
                Ok(DirEntry {
                    name: unsafe {
                        let c_str = CStr::from_ptr(e.as_ref().d_name.as_ptr());
                        let os_str = OsStr::from_bytes(c_str.to_bytes());
                        os_str.to_os_string()
                    },
                })
            })
            .or_else(|| {
                let err = io::Error::last_os_error();
                if err.raw_os_error() == Some(0) {
                    None
                } else {
                    Some(Err(err))
                }
            })
    }
}

pub(super) struct DirEntry {
    name: OsString,
}

impl DirEntry {
    pub(super) fn name(&self) -> &OsStr {
        &self.name
    }
}
