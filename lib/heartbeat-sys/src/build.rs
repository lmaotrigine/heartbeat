//! Utilities for build scripts for heartbeat projects.

use std::process::Command;

/// Appends git metadata to non-stable versions.
///
/// A non-stable version is any alpha, beta, or candidate version.
/// The `'static` lifetime bound is just a sanity check to kind-of ensure that
/// you've used [`env!`]`("CARGO_PKG_VERSION")`, as you should.
#[must_use]
pub fn long_version(pkg_ver: &'static str) -> String {
    let mut version = pkg_ver.to_owned();
    if ["a", "b", "rc"].iter().any(|s| version.ends_with(s)) {
        if let Some(count) = git_commit_count() {
            version.push('.');
            version.push_str(&count);
        }
        // I've done `git config --global init.defaultBranch mistress` since forever.
        let rev = format!(
            "+g{}",
            git_revision().unwrap_or_else(|| git_branch().unwrap_or_else(|| "mistress".into()))
        );
        version.push_str(&rev);
    }
    version
}

/// Retrieves the current git revision of the root project.
///
/// Requires `git` in PATH, doesn't link to `libgit2`.
#[must_use]
pub fn git_revision() -> Option<String> {
    run_git(&["describe", "--tags", "--exact-match"]).or_else(|| run_git(&["rev-parse", "--short", "HEAD"]))
}

/// Retrieves the count of commits on the current branch.
///
/// Requires `git` in PATH, doesn't link to `libgit2`.
#[must_use]
pub fn git_commit_count() -> Option<String> {
    run_git(&["rev-list", "--count", "HEAD"])
}

/// Retrieves the current git branch, if possible.
///
/// Returns [`None`] in cases where it can't be determined, e.g. a detached
/// HEAD.
///
/// Requires `git` in PATH, doesn't link to `libgit2`.
#[must_use]
// shouldn't panic. I wrote the regex myself. It's a good regex. I promise.
#[allow(clippy::missing_panics_doc)]
pub fn git_branch() -> Option<String> {
    let res = run_git(&["symbolic-ref", "-q", "--short", "HEAD"]);
    #[cfg(not(feature = "regex"))]
    return res;
    #[cfg(feature = "regex")]
    {
        res.map(|res| {
            use regex::Regex;
            use std::sync::OnceLock;

            static RE: OnceLock<Regex> = OnceLock::new();

            let lock = &RE;
            let re = lock.get_or_init(|| Regex::new(r"[^a-zA-Z0-9.]").expect("regex should be valid."));
            re.replace_all(&res, ".").into_owned()
        })
    }
}

fn run_git(cmd: &[&str]) -> Option<String> {
    Command::new("git").args(cmd).output().ok().and_then(|output| {
        let v = String::from_utf8_lossy(&output.stdout);
        let v = v.trim();
        if v.is_empty() {
            None
        } else {
            Some(v.to_string())
        }
    })
}
