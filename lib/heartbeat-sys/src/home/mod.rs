#![allow(clippy::module_name_repetitions)]

use std::{
    env,
    ffi::OsString,
    io,
    path::{Path, PathBuf},
};

#[cfg(windows)]
mod windows;
#[cfg(windows)]
use windows::home_dir_inner;

#[cfg(any(unix, target_os = "redox"))]
fn home_dir_inner() -> Option<PathBuf> {
    #[allow(deprecated)] // configured out on non-unix
    std::env::home_dir()
}
pub trait Env {
    fn home_dir(&self) -> Option<PathBuf>;
    /// # Errors
    ///
    /// This function returns an error if it fails to retrieve the current
    /// directory.
    fn current_dir(&self) -> io::Result<PathBuf>;
    fn var_os(&self, key: &str) -> Option<OsString>;
}

pub struct OSEnv;

impl Env for OSEnv {
    fn home_dir(&self) -> Option<PathBuf> {
        home_dir_inner()
    }

    fn current_dir(&self) -> io::Result<PathBuf> {
        env::current_dir()
    }

    fn var_os(&self, key: &str) -> Option<OsString> {
        env::var_os(key)
    }
}

pub const OS_ENV: OSEnv = OSEnv;

pub fn home_dir_with_env(env: &dyn Env) -> Option<PathBuf> {
    env.home_dir()
}

/// # Errors
///
/// This function returns an error if it fails to retrieve the current
/// directory, or if the home directory cannot be determined.
pub fn heartbeat_home_with_env(env: &dyn Env) -> io::Result<PathBuf> {
    let cwd = env.current_dir()?;
    heartbeat_home_with_cwd_env(env, &cwd)
}

/// # Errors
///
/// This function returns an error if the home directory cannot be determined.
pub fn heartbeat_home_with_cwd_env(env: &dyn Env, cwd: &Path) -> io::Result<PathBuf> {
    env::var_os("HEARTBEAT_HOME")
        .filter(|h| !h.is_empty())
        .map_or_else(
            || {
                home_dir_with_env(env)
                    .map(|p| p.join(".heartbeat"))
                    .ok_or_else(|| {
                        io::Error::new(io::ErrorKind::Other, "could not find heartbeat home dir")
                    })
            },
            |home| {
                let home = PathBuf::from(home);
                if home.is_absolute() {
                    Ok(home)
                } else {
                    Ok(cwd.join(home))
                }
            },
        )
}

#[must_use]
pub fn home_dir() -> Option<PathBuf> {
    home_dir_with_env(&OS_ENV)
}

/// # Errors
///
/// This function fails if it fails to retrieve the current directory,
/// or if the home directory cannot be determined.
pub fn heartbeat_home() -> io::Result<PathBuf> {
    heartbeat_home_with_env(&OS_ENV)
}
