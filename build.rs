// auto-version versions itself (dogfooding).
//
// Strategy:
//   1. If a previously-compiled auto-version binary exists in target/, run it.
//      It reads auto-version.toml and emits `cargo:rustc-env=` lines to stdout,
//      which we pass through verbatim to Cargo.
//   2. Fallback (first build / no binary yet): resolve version via raw git
//      subprocess and emit the same env vars manually.

use std::process::Command;

fn main() {
    // Always re-run when git state or config changes
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/refs");
    println!("cargo:rerun-if-changed=auto-version.toml");

    if !try_self_binary() {
        fallback_git_version();
    }
}

/// Try to run the previously-compiled auto-version binary.
/// Returns true if the binary ran successfully and emitted version env vars.
fn try_self_binary() -> bool {
    let exe = if cfg!(windows) { "auto-version.exe" } else { "auto-version" };
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_default();

    for rel in &[
        format!("target/release/{exe}"),
        format!("target/debug/{exe}"),
    ] {
        let path = std::path::Path::new(&manifest_dir).join(rel);
        if !path.exists() {
            continue;
        }
        let Ok(out) = Command::new(&path).arg("generate").current_dir(&manifest_dir).output()
        else {
            continue;
        };
        if !out.status.success() {
            continue;
        }

        // Parse the cargo_env output so we can also emit TOOL_VERSION (version + sha).
        let text = String::from_utf8_lossy(&out.stdout);
        let mut version = String::new();
        let mut sha = String::new();
        for line in text.lines() {
            if let Some(v) = line.strip_prefix("cargo:rustc-env=VERSION=") {
                version = v.to_owned();
            } else if let Some(v) = line.strip_prefix("cargo:rustc-env=GIT_SHORT_SHA=") {
                sha = v.to_owned();
            }
            println!("{line}");
        }
        // Emit a combined display string used by the --version flag and help header.
        match (version.is_empty(), sha.is_empty()) {
            (false, false) => println!("cargo:rustc-env=TOOL_VERSION={version} ({sha})"),
            (false, true)  => println!("cargo:rustc-env=TOOL_VERSION={version}"),
            _              => {}
        }
        return true;
    }
    false
}

/// Fallback used on first build (no binary compiled yet).
fn fallback_git_version() {
    let short_sha = git(&["rev-parse", "--short", "HEAD"]).unwrap_or_else(|| "unknown".into());
    let branch    = git(&["symbolic-ref", "--short", "HEAD"]).unwrap_or_else(|| "unknown".into());
    let dirty     = is_dirty();
    let date      = git(&["log", "-1", "--format=%cd", "--date=short"]).unwrap_or_default();

    let version_base = env!("CARGO_PKG_VERSION");
    let version = if dirty {
        format!("{version_base}-dirty")
    } else {
        version_base.to_owned()
    };
    let version_info = format!("{version}.Branch.{branch}.Sha.{short_sha}");
    let tool_version = format!("{version} ({short_sha})");

    println!("cargo:rustc-env=VERSION={version}");
    println!("cargo:rustc-env=VERSION_FULL={version}");
    println!("cargo:rustc-env=VERSION_INFO={version_info}");
    println!("cargo:rustc-env=GIT_SHORT_SHA={short_sha}");
    println!("cargo:rustc-env=GIT_BRANCH={branch}");
    println!("cargo:rustc-env=BUILD_DATE={date}");
    println!("cargo:rustc-env=TOOL_VERSION={tool_version}");
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

