/// Build-script integration helpers.
///
/// Use this module in your `build.rs` to inject version information into your
/// Rust crate at compile time.
///
/// # Example `build.rs`
///
/// ```rust,no_run
/// fn main() {
///     auto_version::build_rs::run_default();
/// }
/// ```
///
/// Then in your crate source:
///
/// ```rust,ignore
/// const VERSION: &str = env!("VERSION");
/// const VERSION_FULL: &str = env!("VERSION_FULL");
/// const GIT_SHA: &str = env!("GIT_SHORT_SHA");
/// ```

use crate::{config::Config, generate};

/// Run auto-version with the default config (auto-discovered from workspace root)
/// and emit `cargo:rustc-env` + `cargo:rerun-if-changed` directives to stdout.
///
/// This is the simplest integration — just call it from `build.rs`.
pub fn run_default() {
    let config = find_and_load_config();
    run_with_config(&config);
}

/// Run auto-version with an explicit config and emit cargo directives.
pub fn run_with_config(config: &Config) {
    match generate(config) {
        Ok(info) => {
            // Version env vars
            println!("cargo:rustc-env=VERSION={}", info.sem_ver);
            println!("cargo:rustc-env=VERSION_MAJOR={}", info.major);
            println!("cargo:rustc-env=VERSION_MINOR={}", info.minor);
            println!("cargo:rustc-env=VERSION_PATCH={}", info.patch);
            println!("cargo:rustc-env=VERSION_FULL={}", info.full_sem_ver);
            println!("cargo:rustc-env=VERSION_INFO={}", info.informational_version);
            println!("cargo:rustc-env=BUILD_DATE={}", info.build_date);

            if let Some(ref sha) = info.short_sha {
                println!("cargo:rustc-env=GIT_SHORT_SHA={}", sha);
            }
            if let Some(ref sha) = info.sha {
                println!("cargo:rustc-env=GIT_SHA={}", sha);
            }
            if let Some(ref b) = info.branch_name {
                println!("cargo:rustc-env=GIT_BRANCH={}", b);
            }
            if let Some(c) = info.commits_since_tag {
                println!("cargo:rustc-env=GIT_COMMITS_SINCE_TAG={}", c);
            }

            // Re-run triggers
            println!("cargo:rerun-if-changed=.git/HEAD");
            println!("cargo:rerun-if-changed=.git/refs");
            println!("cargo:rerun-if-changed=auto-version.toml");
        }
        Err(e) => {
            // Emit a warning but don't fail the build — fall back to Cargo.toml version
            println!(
                "cargo:warning=auto-version failed ({}); version info may be incomplete",
                e
            );
            println!("cargo:rerun-if-changed=.git/HEAD");
        }
    }
}

fn find_and_load_config() -> Config {
    // CARGO_MANIFEST_DIR is set by cargo when running build.rs
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .unwrap_or_else(|_| ".".into());
    let start = std::path::Path::new(&manifest_dir);

    if let Some(path) = crate::config::find_config_file(start) {
        match crate::config::load(&path) {
            Ok(cfg) => return cfg,
            Err(e) => eprintln!("auto-version: failed to load config: {}", e),
        }
    }
    Config::default()
}
