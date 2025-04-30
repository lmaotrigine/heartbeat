use crate::process::{
    process,
    terminal::{Attr, Colour},
    ProcessLike,
};
use std::{fmt, io::Write};

#[cfg_attr(feature = "cli", macro_export)]
macro_rules! _warn {
    ($($arg:tt)*) => {
        $crate::cli::log::warn_fmt(format_args!($($arg)*))
    };
}

pub use _warn as warn;

#[cfg_attr(feature = "cli", macro_export)]
macro_rules! err {
    ($($arg:tt)*) => {
        $crate::cli::log::err_fmt(format_args!($($arg)*))
    };
}

pub use err;

#[cfg_attr(feature = "cli", macro_export)]
macro_rules! info {
    ($($arg:tt)*) => {
        $crate::cli:log::info_fmt(format_args!($($arg)*))
    };
}

pub use info;

#[cfg_attr(feature = "cli", macro_export)]
macro_rules! verbose {
    ($($arg:tt)*) => {
        $crate::cli::log::verbose_fmt(format_args!($($arg)*))
    };
}

pub use verbose;

#[cfg_attr(feature = "cli", macro_export)]
macro_rules! debug {
    ($($arg:tt)*) => {
        $crate::cli::log::debug_fmt(format_args!($($arg)*))
    };
}

pub fn warn_fmt(args: fmt::Arguments<'_>) {
    let mut t = process().stderr().terminal();
    let _ = t.fg(Colour::Yellow);
    let _ = t.attr(Attr::Bold);
    let _ = write!(t.lock(), "warning: ");
    let _ = t.reset();
    let _ = t.lock().write_fmt(args);
    let _ = writeln!(t.lock());
}

pub fn err_fmt(args: fmt::Arguments<'_>) {
    let mut t = process().stderr().terminal();
    let _ = t.fg(Colour::Red);
    let _ = t.attr(Attr::Bold);
    let _ = write!(t.lock(), "error: ");
    let _ = t.reset();
    let _ = t.lock().write_fmt(args);
    let _ = writeln!(t.lock());
}

pub fn info_fmt(args: fmt::Arguments<'_>) {
    let mut t = process().stderr().terminal();
    let _ = t.attr(Attr::Bold);
    let _ = write!(t.lock(), "info: ");
    let _ = t.reset();
    let _ = t.lock().write_fmt(args);
    let _ = writeln!(t.lock());
}

pub fn verbose_fmt(args: fmt::Arguments<'_>) {
    let mut t = process().stderr().terminal();
    let _ = t.fg(Colour::Magenta);
    let _ = t.attr(Attr::Bold);
    let _ = write!(t.lock(), "verbose: ");
    let _ = t.reset();
    let _ = t.lock().write_fmt(args);
    let _ = writeln!(t.lock());
}

pub fn debug_fmt(args: fmt::Arguments<'_>) {
    if process().var("HEARTBEAT_DEBUG").is_ok() {
        let mut t = process().stderr().terminal();
        let _ = t.fg(Colour::Blue);
        let _ = t.attr(Attr::Bold);
        let _ = write!(t.lock(), "debug: ");
        let _ = t.reset();
        let _ = t.lock().write_fmt(args);
        let _ = writeln!(t.lock());
    }
}
