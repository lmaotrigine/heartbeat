/**
 * Copyright (c) 2023 VJ <root@5ht2.me>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

fn main() {
    if let Some(rev) = git_revision_hash() {
        println!("cargo:rustc-env=HB_GIT_COMMIT={}", rev);
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
