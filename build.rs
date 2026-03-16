// auto-version versions itself (dogfooding).
// build.rs cannot depend on the crate being built, so we call git directly.

use std::process::Command;

fn main() {
    // Re-run when git state changes
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/refs");
    println!("cargo:rerun-if-changed=auto-version.toml");

    let short_sha = git(&["rev-parse", "--short", "HEAD"]).unwrap_or_else(|| "unknown".into());
    let branch    = git(&["symbolic-ref", "--short", "HEAD"]).unwrap_or_else(|| "unknown".into());
    let dirty     = is_dirty();

    let version_base = env!("CARGO_PKG_VERSION");
    let version = if dirty {
        format!("{}-{}-dirty", version_base, short_sha)
    } else {
        format!("{}-{}", version_base, short_sha)
    };

    println!("cargo:rustc-env=GIT_VERSION={}", version);
    println!("cargo:rustc-env=GIT_SHORT_SHA={}", short_sha);
    println!("cargo:rustc-env=GIT_BRANCH={}", branch);
}

fn git(args: &[&str]) -> Option<String> {
    let out = Command::new("git").args(args).output().ok()?;
    if out.status.success() {
        Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
    } else {
        None
    }
}

fn is_dirty() -> bool {
    Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .map(|o| !o.stdout.is_empty())
        .unwrap_or(false)
}
