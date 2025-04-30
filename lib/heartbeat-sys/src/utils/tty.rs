//! the [`isatty`][isatty] crate is abandoned and doesn't work as expected.
//! this is pretty much copied from rustc.
//!
//! [isatty]: https://docs.rs/crate/isatty/0.1.9

#[cfg(windows)]
use windows_sys::Win32::System::Console;

#[cfg(unix)]
#[must_use]
pub fn stderr_isatty() -> bool {
    isatty(libc::STDERR_FILENO)
}

#[cfg(windows)]
#[must_use]
pub fn stderr_isatty() -> bool {
    isatty(Console::STD_ERROR_HANDLE)
}

#[cfg(unix)]
#[must_use]
pub fn stdout_isatty() -> bool {
    isatty(libc::STDOUT_FILENO)
}

#[cfg(windows)]
#[must_use]
pub fn stdout_isatty() -> bool {
    isatty(Console::STD_OUTPUT_HANDLE)
}

#[inline]
#[cfg(unix)]
fn isatty(fd: libc::c_int) -> bool {
    unsafe { libc::isatty(fd) == 1 }
}

#[inline]
#[cfg(windows)]
fn isatty(fd: Console::STD_HANDLE) -> bool {
    unsafe {
        let handle = Console::GetStdHandle(fd);
        let mut out = 0;
        Console::GetConsoleMode(handle, &mut out) != 0
    }
}
