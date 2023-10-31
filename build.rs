// Copyright (c) 2023 Isis <root@5ht2.me>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

#![forbid(
    unsafe_code,
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::unwrap_in_result,
    clippy::unwrap_used
)]

fn main() {
    let mut version = env!("CARGO_PKG_VERSION").to_owned();
    let rev = git_revision().unwrap_or_else(|| "main".into());
    if ["a", "b", "rc"].iter().any(|s| version.ends_with(s)) {
        if let Some(count) = git_commit_count() {
            version.push('.');
            version.push_str(&count);
        }
        version.push_str(&format!("+g{rev}"));
    }
    println!("cargo:rustc-env=HB_VERSION={version}");
    println!("cargo:rustc-env=HB_GIT_REVISION={rev}");
}

fn git_revision() -> Option<String> {
    std::process::Command::new("git")
        .args(["describe", "--tags", "--exact-match"])
        .output()
        .ok()
        .and_then(|output| {
            let v = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if v.is_empty() {
                None
            } else {
                Some(v)
            }
        })
        .or_else(|| {
            std::process::Command::new("git")
                .args(["rev-parse", "--short", "HEAD"])
                .output()
                .ok()
                .and_then(|output| {
                    let v = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if v.is_empty() {
                        None
                    } else {
                        Some(v)
                    }
                })
        })
}

fn git_commit_count() -> Option<String> {
    std::process::Command::new("git")
        .args(["rev-list", "--count", "HEAD"])
        .output()
        .ok()
        .and_then(|output| {
            let v = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if v.is_empty() {
                None
            } else {
                Some(v)
            }
        })
}
