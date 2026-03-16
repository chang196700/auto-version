// build.rs — auto-version integration
//
// After adding to Cargo.toml:
//   [build-dependencies]
//   auto-version = { git = "https://github.com/yourname/auto-version", features = ["build-rs"] }
//
// The following env vars become available in your code via env!():
//   VERSION, VERSION_MAJOR, VERSION_MINOR, VERSION_PATCH
//   VERSION_FULL, VERSION_INFO
//   GIT_SHORT_SHA, GIT_SHA, GIT_BRANCH, GIT_COMMITS_SINCE_TAG
//   BUILD_DATE

fn main() {
    auto_version::build_rs::run_default();
}
