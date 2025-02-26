use super::{
    process,
    terminal::{ColourableTerminal, StreamSelector},
};
use parking_lot::{Mutex, MutexGuard};
use std::{
    io::{self, BufRead, Cursor, Read, Result, Write},
    sync::Arc,
};

/// stand-in for [`std::io::Stdin`].
pub trait Stdin {
    fn lock(&self) -> Box<dyn StdinLock + '_>;
    /// # Errors
    ///
    /// This function will return an error if the read bytes are not valid
    /// UTF-8.
    fn read_line(&self, buf: &mut String) -> Result<usize>;
}

/// stand-in for [`std::io::StdinLock`].
pub trait StdinLock: Read + BufRead {}

// ------------------------- STDIN --------------------------
// // ------------------------- OS ----------------------------

impl StdinLock for io::StdinLock<'_> {}

impl Stdin for io::Stdin {
    fn lock(&self) -> Box<dyn StdinLock> {
        Box::new(Self::lock(self))
    }

    fn read_line(&self, buf: &mut String) -> Result<usize> {
        Self::read_line(self, buf)
    }
}

// // ------------------------ TEST ---------------------------

struct TestStdinLock<'a> {
    inner: MutexGuard<'a, Cursor<String>>,
}

impl StdinLock for TestStdinLock<'_> {}

impl Read for TestStdinLock<'_> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.inner.read(buf)
    }
}

impl BufRead for TestStdinLock<'_> {
    fn fill_buf(&mut self) -> Result<&[u8]> {
        self.inner.fill_buf()
    }

    fn consume(&mut self, amt: usize) {
        self.inner.consume(amt);
    }
}

pub(super) type TestStdinInner = Arc<Mutex<Cursor<String>>>;

pub(super) struct TestStdin(pub(super) TestStdinInner);

impl Stdin for TestStdin {
    fn lock(&self) -> Box<dyn StdinLock + '_> {
        Box::new(TestStdinLock { inner: self.0.lock() })
    }

    fn read_line(&self, buf: &mut String) -> Result<usize> {
        self.lock().read_line(buf)
    }
}

/// this is a stand-in for [`std::io::StdoutLock`] and [`std::io::StderrLock`].
pub trait WriterLock: Write {}

/// this is a stand-in for [`std::io::Stdout`] and [`std::io::Stderr`].
pub trait Writer: Write + Send {
    /// this is a stand-in for [`std::io::Stdout::lock()`] and
    /// [`std::io::Stderr::lock()`].
    fn lock(&self) -> Box<dyn WriterLock + '_>;

    /// query whether a TTY is present. this may be useful for progress bars,
    /// prompts, and other things of that nature.
    fn is_a_tty(&self) -> bool;

    /// Construct a terminal on this writer.
    fn terminal(&self) -> ColourableTerminal;
}

// --------------------- STDOUT/STDERR ----------------------
// // ------------------------- OS ----------------------------

impl WriterLock for io::StdoutLock<'_> {}

impl Writer for io::Stdout {
    fn is_a_tty(&self) -> bool {
        match process() {
            super::Process::OS(p) => p.stdout_is_a_tty,
            #[cfg(feature = "test")]
            super::Process::Test(_) => unreachable!(),
        }
    }

    fn lock(&self) -> Box<dyn WriterLock + '_> {
        Box::new(Self::lock(self))
    }

    fn terminal(&self) -> ColourableTerminal {
        ColourableTerminal::new(StreamSelector::Stdout)
    }
}

impl WriterLock for io::StderrLock<'_> {}

impl Writer for io::Stderr {
    fn is_a_tty(&self) -> bool {
        match process() {
            super::Process::OS(p) => p.stderr_is_a_tty,
            #[cfg(feature = "test")]
            super::Process::Test(_) => unreachable!(),
        }
    }

    fn lock(&self) -> Box<dyn WriterLock + '_> {
        Box::new(Self::lock(self))
    }

    fn terminal(&self) -> ColourableTerminal {
        ColourableTerminal::new(StreamSelector::Stderr)
    }
}

// // ------------------------ TEST ---------------------------

pub(super) struct TestWriterLock<'a> {
    inner: MutexGuard<'a, Vec<u8>>,
}

impl WriterLock for TestWriterLock<'_> {}

impl Write for TestWriterLock<'_> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.inner.write(buf)
    }

    fn flush(&mut self) -> Result<()> {
        self.inner.flush()
    }
}

#[cfg(feature = "test")]
pub(super) type TestWriterInner = Arc<Mutex<Vec<u8>>>;

/// a thread-safe test file handle that pretends to be stdout/stderr.
#[derive(Clone, Default)]
#[cfg(feature = "test")]
pub(super) struct TestWriter(pub(super) TestWriterInner);

#[cfg(feature = "test")]
impl TestWriter {
    pub(super) fn lock(&self) -> TestWriterLock<'_> {
        TestWriterLock { inner: self.0.lock() }
    }
}

#[cfg(feature = "test")]
impl Writer for TestWriter {
    fn is_a_tty(&self) -> bool {
        false
    }

    fn lock(&self) -> Box<dyn WriterLock + '_> {
        Box::new(self.lock())
    }

    fn terminal(&self) -> ColourableTerminal {
        ColourableTerminal::new(StreamSelector::TestWriter(self.clone()))
    }
}

#[cfg(feature = "test")]
impl Write for TestWriter {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.lock().write(buf)
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}
