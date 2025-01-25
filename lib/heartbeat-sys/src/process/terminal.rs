use parking_lot::{Mutex, MutexGuard};
use std::{
    io::{self, Write},
    mem::MaybeUninit,
    ptr::addr_of_mut,
    sync::Arc,
};
pub use termcolor::Color as Colour;
use termcolor::{ColorChoice, ColorSpec, StandardStream, StandardStreamLock, WriteColor};

#[cfg(feature = "test")]
use super::io::{TestWriter, TestWriterLock};
use super::{process, ProcessLike};

/// select what stream to make a terminal on
pub(super) enum StreamSelector {
    Stdout,
    Stderr,
    #[cfg(feature = "test")]
    TestWriter(TestWriter),
    #[cfg(all(test, feature = "test"))]
    TestTtyWriter(TestWriter),
}

impl StreamSelector {
    fn is_a_tty(&self) -> bool {
        match self {
            Self::Stdout => match process() {
                super::Process::OS(p) => p.stdout_is_a_tty,
                #[cfg(feature = "test")]
                super::Process::Test(_) => unreachable!(),
            },
            Self::Stderr => match process() {
                super::Process::OS(p) => p.stderr_is_a_tty,
                #[cfg(feature = "test")]
                super::Process::Test(_) => unreachable!(),
            },
            #[cfg(feature = "test")]
            Self::TestWriter(_) => false,
            #[cfg(all(test, feature = "test"))]
            Self::TestTtyWriter(_) => true,
        }
    }
}

/// a colourable terminal that can be written to.
pub struct ColourableTerminal {
    // termcolor uses a lifetime on locked variants, but the API we want to emulate from std::io
    // uses a static lifetime for locked variants: so we emulate it. for test workloads this
    // results in a double-layering of Arc<Mutex<...>> which isn't great, but on the other hand
    // it is test code. locking the source is important because otherwise parallel constructed
    // terminals would not be locked out.
    inner: Arc<Mutex<TerminalInner>>,
}

/// internal state for [`ColourableTerminal`].
enum TerminalInner {
    StandardStream(StandardStream, ColorSpec),
    #[cfg(feature = "test")]
    TestWriter(TestWriter, ColorChoice),
}

pub struct ColourableTerminalLocked {
    // must drop the lock before the guard, as the guard borrows from inner.
    locked: TerminalInnerLocked,
    // must drop the guard before inner, as the guard borrows from inner.
    guard: MutexGuard<'static, TerminalInner>,
    inner: Arc<Mutex<TerminalInner>>,
}

enum TerminalInnerLocked {
    StandardStream(StandardStreamLock<'static>),
    #[cfg(feature = "test")]
    TestWriter(TestWriterLock<'static>),
}

impl ColourableTerminal {
    /// a terminal that supports colourisation of a stream.
    /// if `HEARTBEAT_TERM_COLOUR` is set to `always`, or if the stream is a tty
    /// and `HEARTBEAT_TERM_COLOUR` is either unset or set to `auto`, then
    /// colour commands will be sent to the stream. Otherwise, colour
    /// commands are discarded.
    #[cfg_attr(not(feature = "test"), allow(clippy::needless_pass_by_value))]
    pub(super) fn new(stream: StreamSelector) -> Self {
        let env_override = process()
            .var("HEARTBEAT_TERM_COLOUR")
            .map(|it| it.to_lowercase());
        let choice = match env_override.as_deref() {
            Ok("always") => ColorChoice::Always,
            Ok("never") => ColorChoice::Never,
            _ if stream.is_a_tty() => ColorChoice::Auto,
            _ => ColorChoice::Never,
        };
        let inner = match stream {
            StreamSelector::Stdout => {
                TerminalInner::StandardStream(StandardStream::stdout(choice), ColorSpec::new())
            }
            StreamSelector::Stderr => {
                TerminalInner::StandardStream(StandardStream::stderr(choice), ColorSpec::new())
            }
            #[cfg(feature = "test")]
            StreamSelector::TestWriter(w) => TerminalInner::TestWriter(w, choice),
            #[cfg(all(test, feature = "test"))]
            StreamSelector::TestTtyWriter(w) => TerminalInner::TestWriter(w, choice),
        };
        Self {
            inner: Arc::new(Mutex::new(inner)),
        }
    }

    pub fn lock(&self) -> ColourableTerminalLocked {
        let mut uninit = MaybeUninit::<ColourableTerminalLocked>::uninit();
        let ptr = uninit.as_mut_ptr();
        // SAFETY: panics during this will leak an arc reference, or an arc
        // reference and a mutex guard, or an arc reference, mutex guard, and a
        // stream lock. drop proceeds in field order after initialization, so
        // the stream lock is dropped before the mutex guard, which is dropped
        // before the Arc<Mutex<...>>.
        unsafe {
            addr_of_mut!((*ptr).inner).write(self.inner.clone());
            addr_of_mut!((*ptr).guard).write((*ptr).inner.lock());
            addr_of_mut!((*ptr).locked).write(match &mut *(*ptr).guard {
                TerminalInner::StandardStream(s, _) => {
                    let locked = s.lock();
                    TerminalInnerLocked::StandardStream(locked)
                }
                #[cfg(feature = "test")]
                TerminalInner::TestWriter(w, _) => TerminalInnerLocked::TestWriter(w.lock()),
            });
            uninit.assume_init()
        }
    }

    pub fn fg(&mut self, colour: Colour) -> io::Result<()> {
        match &mut *self.inner.lock() {
            TerminalInner::StandardStream(s, spec) => {
                spec.set_fg(Some(colour));
                s.set_color(spec)
            }
            #[cfg(feature = "test")]
            TerminalInner::TestWriter(_, _) => Ok(()),
        }
    }

    pub fn bg(&mut self, colour: Colour) -> io::Result<()> {
        match &mut *self.inner.lock() {
            TerminalInner::StandardStream(s, spec) => {
                spec.set_bg(Some(colour));
                s.set_color(spec)
            }
            #[cfg(feature = "test")]
            TerminalInner::TestWriter(_, _) => Ok(()),
        }
    }

    pub fn attr(&mut self, attr: Attr) -> io::Result<()> {
        match &mut *self.inner.lock() {
            TerminalInner::StandardStream(s, spec) => {
                match attr {
                    Attr::Bold => spec.set_bold(true),
                    Attr::ForegroundColour(colour) => spec.set_fg(Some(colour)),
                };
                s.set_color(spec)
            }
            #[cfg(feature = "test")]
            TerminalInner::TestWriter(_, _) => Ok(()),
        }
    }

    pub fn reset(&mut self) -> io::Result<()> {
        match &mut *self.inner.lock() {
            TerminalInner::StandardStream(s, _) => s.reset(),
            #[cfg(feature = "test")]
            TerminalInner::TestWriter(_, _) => Ok(()),
        }
    }

    pub fn carriage_return(&mut self) -> io::Result<()> {
        match &mut *self.inner.lock() {
            TerminalInner::StandardStream(s, _) => s.write(b"\r")?,
            #[cfg(feature = "test")]
            TerminalInner::TestWriter(w, _) => w.write(b"\r")?,
        };
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Attr {
    Bold,
    ForegroundColour(Colour),
}

impl io::Write for ColourableTerminal {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match &mut *self.inner.lock() {
            TerminalInner::StandardStream(s, _) => s.write(buf),
            #[cfg(feature = "test")]
            TerminalInner::TestWriter(w, _) => w.write(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match &mut *self.inner.lock() {
            TerminalInner::StandardStream(s, _) => s.flush(),
            #[cfg(feature = "test")]
            TerminalInner::TestWriter(w, _) => w.flush(),
        }
    }
}

impl io::Write for ColourableTerminalLocked {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match &mut self.locked {
            TerminalInnerLocked::StandardStream(s) => s.write(buf),
            #[cfg(feature = "test")]
            TerminalInnerLocked::TestWriter(w) => w.write(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match &mut self.locked {
            TerminalInnerLocked::StandardStream(s) => s.flush(),
            #[cfg(feature = "test")]
            TerminalInnerLocked::TestWriter(w) => w.flush(),
        }
    }
}

#[cfg(all(test, feature = "test"))]
mod tests {
    use super::*;
    use crate::process;
    use std::collections::HashMap;

    #[test]
    fn term_colour_choice() {
        fn assert_colour_choice(env_val: &str, stream: StreamSelector, colour_choice: ColorChoice) {
            let mut vars = HashMap::new();
            vars.insert("HEARTBEAT_TERM_COLOUR".to_string(), env_val.to_string());
            let tp = process::TestProcess {
                vars,
                ..Default::default()
            };
            process::with(tp.into(), || {
                let term = ColourableTerminal::new(stream);
                assert!(
                    matches!(&*term.inner.lock(), &TerminalInner::TestWriter(_, choice) if choice == colour_choice)
                );
            });
        }

        assert_colour_choice(
            "aLWayS",
            StreamSelector::TestWriter(TestWriter::default()),
            ColorChoice::Always,
        );
        assert_colour_choice(
            "neVer",
            StreamSelector::TestWriter(TestWriter::default()),
            ColorChoice::Never,
        );
        assert_colour_choice(
            "AutO",
            StreamSelector::TestTtyWriter(TestWriter::default()),
            ColorChoice::Auto,
        );
        assert_colour_choice(
            "aUTo",
            StreamSelector::TestWriter(TestWriter::default()),
            ColorChoice::Never,
        );
    }
}
