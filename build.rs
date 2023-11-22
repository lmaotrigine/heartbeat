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
    let version = heartbeat_sys::build::long_version(env!("CARGO_PKG_VERSION"));
    let rev = heartbeat_sys::build::git_revision().unwrap_or_else(|| "main".into());
    println!("cargo:rustc-env=HB_VERSION={version}");
    println!("cargo:rustc-env=HB_GIT_REVISION={rev}");
}
