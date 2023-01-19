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
