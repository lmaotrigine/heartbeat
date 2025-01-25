#![allow(clippy::module_name_repetitions)]

use crate::utils::tty::{stderr_isatty, stdout_isatty};
#[cfg(feature = "test")]
use parking_lot::Mutex;
use parking_lot::Once;
#[cfg(feature = "test")]
use rand::{thread_rng, Rng};
use std::{cell::RefCell, env, ffi::OsString, marker::PhantomData, panic, path::PathBuf};
#[cfg(feature = "test")]
use std::{collections::HashMap, io::Cursor, path::Path, sync::Arc};
mod io;
pub(crate) mod terminal;
use io::{Stdin, Writer};
#[cfg(feature = "test")]
use io::{TestStdinInner, TestWriterInner};

pub trait ProcessLike {
    fn id(&self) -> u64;
    fn args(&self) -> Box<dyn Iterator<Item = String>>;
    fn args_os(&self) -> Box<dyn Iterator<Item = OsString>>;
    /// # Errors
    ///
    /// This function returns an error if it fails to retrieve the current
    /// directory.
    fn current_dir(&self) -> std::io::Result<PathBuf>;
    fn stdin(&self) -> Box<dyn Stdin>;
    fn stdout(&self) -> Box<dyn Writer>;
    fn stderr(&self) -> Box<dyn Writer>;
    /// # Errors
    ///
    /// this function returns an error if the environment variable doesn't
    /// exist, or is not valid UTF-8.
    fn var(&self, key: &str) -> Result<String, env::VarError>;
    fn var_os(&self, key: &str) -> Option<OsString>;
}

#[derive(Debug, Clone)]
pub enum Process {
    OS(OSProcess),
    #[cfg(feature = "test")]
    Test(TestProcess),
}

impl From<OSProcess> for Process {
    fn from(value: OSProcess) -> Self {
        Self::OS(value)
    }
}

#[cfg(feature = "test")]
impl From<TestProcess> for Process {
    fn from(value: TestProcess) -> Self {
        Self::Test(value)
    }
}

impl ProcessLike for Process {
    fn args(&self) -> Box<dyn Iterator<Item = String>> {
        match self {
            Self::OS(p) => p.args(),
            #[cfg(feature = "test")]
            Self::Test(p) => p.args(),
        }
    }

    fn args_os(&self) -> Box<dyn Iterator<Item = OsString>> {
        match self {
            Self::OS(p) => p.args_os(),
            #[cfg(feature = "test")]
            Self::Test(p) => p.args_os(),
        }
    }

    fn current_dir(&self) -> std::io::Result<PathBuf> {
        match self {
            Self::OS(p) => p.current_dir(),
            #[cfg(feature = "test")]
            Self::Test(p) => p.current_dir(),
        }
    }

    fn id(&self) -> u64 {
        match self {
            Self::OS(p) => p.id(),
            #[cfg(feature = "test")]
            Self::Test(p) => p.id(),
        }
    }

    fn stderr(&self) -> Box<dyn Writer> {
        match self {
            Self::OS(p) => p.stderr(),
            #[cfg(feature = "test")]
            Self::Test(p) => p.stderr(),
        }
    }

    fn stdin(&self) -> Box<dyn Stdin> {
        match self {
            Self::OS(p) => p.stdin(),
            #[cfg(feature = "test")]
            Self::Test(p) => p.stdin(),
        }
    }

    fn stdout(&self) -> Box<dyn Writer> {
        match self {
            Self::OS(p) => p.stdout(),
            #[cfg(feature = "test")]
            Self::Test(p) => p.stdout(),
        }
    }

    fn var(&self, key: &str) -> Result<String, env::VarError> {
        match self {
            Self::OS(p) => p.var(key),
            #[cfg(feature = "test")]
            Self::Test(p) => p.var(key),
        }
    }

    fn var_os(&self, key: &str) -> Option<OsString> {
        match self {
            Self::OS(p) => p.var_os(key),
            #[cfg(feature = "test")]
            Self::Test(p) => p.var_os(key),
        }
    }
}

static HOOK_INSTALLED: Once = Once::new();

thread_local!(pub(crate) static PROCESS: RefCell<Option<Process>> = RefCell::new(None));

#[must_use]
pub fn process() -> Process {
    home_process()
}

pub(crate) fn home_process() -> Process {
    PROCESS
        .with(|p| p.borrow().clone())
        .map_or_else(|| panic!("no process instance"), |p| p)
}

fn clear_process() {
    PROCESS.with(|p| p.replace(None));
}

/// run a function in the context of a process definition.
///
/// # Panics
///
/// if the function panics, the process definition *in that thread* is cleared
/// by an implicitly installed global panic hook.
pub fn with<F: FnOnce() -> R, R>(process: Process, f: F) -> R {
    HOOK_INSTALLED.call_once(|| {
        let orig = panic::take_hook();
        panic::set_hook(Box::new(move |info| {
            clear_process();
            orig(info);
        }));
    });
    PROCESS.with(|p| {
        if let Some(old_p) = &*p.borrow() {
            panic!("current process already set {old_p:?}");
        }
        *p.borrow_mut() = Some(process);
        let result = f();
        *p.borrow_mut() = None;
        result
    })
}

// real process

#[derive(Debug, Clone)]
pub struct OSProcess {
    pub(self) stderr_is_a_tty: bool,
    pub(self) stdout_is_a_tty: bool,
}

impl OSProcess {
    #[must_use]
    pub fn new() -> Self {
        Self {
            stderr_is_a_tty: stderr_isatty(),
            stdout_is_a_tty: stdout_isatty(),
        }
    }
}

impl Default for OSProcess {
    fn default() -> Self {
        Self::new()
    }
}

impl ProcessLike for OSProcess {
    fn args(&self) -> Box<dyn Iterator<Item = String>> {
        Box::new(env::args())
    }

    fn args_os(&self) -> Box<dyn Iterator<Item = OsString>> {
        Box::new(env::args_os())
    }

    fn current_dir(&self) -> std::io::Result<PathBuf> {
        env::current_dir()
    }

    fn stdin(&self) -> Box<dyn Stdin> {
        Box::new(std::io::stdin())
    }

    fn stdout(&self) -> Box<dyn Writer> {
        Box::new(std::io::stdout())
    }

    fn stderr(&self) -> Box<dyn Writer> {
        Box::new(std::io::stderr())
    }

    fn id(&self) -> u64 {
        u64::from(std::process::id())
    }

    fn var(&self, key: &str) -> Result<String, env::VarError> {
        env::var(key)
    }

    fn var_os(&self, key: &str) -> Option<OsString> {
        env::var_os(key)
    }
}

// test process

#[cfg(feature = "test")]
#[derive(Debug, Clone, Default)]
pub struct TestProcess {
    pub cwd: PathBuf,
    pub args: Vec<String>,
    pub vars: HashMap<String, String>,
    pub id: u64,
    pub stdin: TestStdinInner,
    pub stdout: TestWriterInner,
    pub stderr: TestWriterInner,
}

#[cfg(feature = "test")]
impl TestProcess {
    pub fn new<P: AsRef<Path>, A: AsRef<str>>(
        cwd: P,
        args: &[A],
        vars: HashMap<String, String>,
        stdin: &str,
    ) -> Self {
        Self {
            cwd: cwd.as_ref().to_path_buf(),
            args: args.iter().map(|s| s.as_ref().to_string()).collect(),
            vars,
            id: Self::new_id(),
            stdin: Arc::new(Mutex::new(Cursor::new(stdin.to_string()))),
            stdout: Arc::new(Mutex::new(Vec::new())),
            stderr: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn new_id() -> u64 {
        let low_bits = u64::from(std::process::id());
        let mut rng = thread_rng();
        let high_bits = u64::from(rng.gen_range(0..u32::MAX));
        high_bits << 32 | low_bits
    }

    /// extracts the stdout from the process.
    #[must_use]
    pub fn get_stdout(&self) -> Vec<u8> {
        self.stdout.lock().clone()
    }

    /// extracts the stderr from the process.
    #[must_use]
    pub fn get_stderr(&self) -> Vec<u8> {
        self.stderr.lock().clone()
    }
}

struct VecArgs<T> {
    v: Vec<String>,
    i: usize,
    _marker: PhantomData<T>,
}

impl<T> From<&Vec<String>> for VecArgs<T> {
    fn from(value: &Vec<String>) -> Self {
        let v = value.clone();
        Self {
            v,
            i: 0,
            _marker: PhantomData,
        }
    }
}

impl<T: From<String>> Iterator for VecArgs<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i == self.v.len() {
            return None;
        }
        let i = self.i;
        self.i += 1;
        Some(T::from(self.v[i].clone()))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.v.len(), Some(self.v.len()))
    }
}

#[cfg(feature = "test")]
impl ProcessLike for TestProcess {
    fn args(&self) -> Box<dyn Iterator<Item = String>> {
        Box::new(VecArgs::from(&self.args))
    }

    fn args_os(&self) -> Box<dyn Iterator<Item = OsString>> {
        Box::new(VecArgs::from(&self.args))
    }

    fn current_dir(&self) -> std::io::Result<PathBuf> {
        Ok(self.cwd.clone())
    }

    fn stdin(&self) -> Box<dyn Stdin> {
        Box::new(io::TestStdin(self.stdin.clone()))
    }

    fn stdout(&self) -> Box<dyn Writer> {
        Box::new(io::TestWriter(self.stdout.clone()))
    }

    fn stderr(&self) -> Box<dyn Writer> {
        Box::new(io::TestWriter(self.stderr.clone()))
    }

    fn id(&self) -> u64 {
        self.id
    }

    fn var(&self, key: &str) -> Result<String, env::VarError> {
        match self.var_os(key) {
            None => Err(env::VarError::NotPresent),
            // should technically never error, but anyway.
            Some(key) => Ok(key.into_string().map_err(env::VarError::NotUnicode)?),
        }
    }

    fn var_os(&self, key: &str) -> Option<OsString> {
        self.vars.get(key).map(|s| OsString::from(s.clone()))
    }
}

#[cfg(all(test, feature = "test"))]
mod tests {
    use super::{process, with, ProcessLike, TestProcess};
    use std::{collections::HashMap, env};

    #[test]
    fn test_instance() {
        let proc = TestProcess::new(
            env::current_dir().expect("couldn't get current dir."),
            &["foo", "bar", "baz"],
            HashMap::new(),
            "",
        );
        with(proc.clone().into(), || {
            assert_eq!(proc.id(), process().id(), "{:?} != {:?}", proc, process());
        });
    }
}
