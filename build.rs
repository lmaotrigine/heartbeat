// Copyright (c) 2023 Isis <root@5ht2.me>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

fn main() {
    let rev = git_revision_hash().unwrap_or("main".into());
    println!("cargo:rustc-env=HB_GIT_COMMIT={}", rev);
    if let Some((major, minor, _)) = get_rustc_version() {
        if (major, minor) < (1, 70) {
            println!("cargo:rustc-cfg=let_underscore_untyped_pedantic");
        }
    }
}

fn git_revision_hash() -> Option<String> {
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
}

fn get_rustc_version() -> Option<(u8, u8, u8)> {
    std::process::Command::new(std::env::var("RUSTC").unwrap())
        .args(["--version"])
        .output()
        .ok()
        .and_then(|output| {
            let v = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if v.is_empty() {
                None
            } else {
                let mut v = v.split(' ').nth(1)?.split('.');
                let major = v.next()?.parse::<u8>().ok()?;
                let minor = v.next()?.parse::<u8>().ok()?;
                let patch = v.next()?.parse::<u8>().ok()?;
                Some((major, minor, patch))
            }
        })
}
