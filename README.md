# auto-version

Multi-format, multi-source automatic version generator — cross-platform, cross-language, zero runtime dependencies.

## Features

- **Multiple version sources**: Git (branch-aware), VERSION file, C/C++ header `#define`, environment variables, TOML/JSON fields
- **Branch-aware Git versioning**: Automatic pre-release labels based on branch type (GitFlow, GitHub Flow, custom)
- **Conventional Commits support**: Auto-bump major/minor/patch from commit messages
- **Multiple output formats**: JSON, key=value, C/C++ header, CMake variables, Makefile variables, Cargo env, Tera templates
- **Multiple integrations**: CMake submodule, Cargo build dependency, standalone binary, Git submodule
- **Zero runtime dependencies**: Compiles to a single self-contained binary

## Quick Start

### As a Git Submodule (recommended)

```sh
git submodule add https://github.com/yourname/auto-version vendor/auto-version
```

Create `auto-version.toml` in your project root (or run `auto-version init-config > auto-version.toml`).

### CMake Integration

```cmake
include(vendor/auto-version/cmake/AutoVersion.cmake)

auto_version_setup()   # Compiles the tool via Cargo (once)
auto_version_generate()  # Runs generation, writes all configured outputs
auto_version_target(my_app)  # Ensures version is generated before building my_app
```

### Cargo / Rust Integration

```toml
# Cargo.toml
[build-dependencies]
auto-version = { path = "vendor/auto-version" }
```

```rust
// build.rs
fn main() {
    let config = auto_version::config::Config::default();
    let info = auto_version::generate(&config).expect("auto-version failed");
    println!("cargo:rustc-env=VERSION={}", info.sem_ver);
    println!("cargo:rerun-if-changed=.git/HEAD");
}
```

### Standalone CLI

```sh
cargo install auto-version   # or download from GitHub Releases
cd your-project
auto-version show            # Print SemVer to stdout
auto-version show -f json    # Print full JSON
auto-version generate        # Run all outputs from auto-version.toml
```

## Configuration

See [`auto-version.example.toml`](auto-version.example.toml) for a fully annotated config.

### Branch-aware versions

When using the Git source, versions are automatically adapted per branch:

| Branch           | Example Output         |
|-----------------|------------------------|
| `main`/`master` | `1.2.3`                |
| `develop`        | `1.3.0-alpha.5`        |
| `feature/foo`    | `1.3.0-feat.foo`       |
| `release/1.3`    | `1.3.0-rc.2`           |
| `hotfix/crash`   | `1.2.4-hotfix.abc1234` |

### HEX version

The C header output includes a packed HEX version suitable for embedded firmware:

```c
#define VERSION_HEX  0x01020003  // major=1, minor=2, patch=3
```

## Output formats

| Format          | Description                           |
|-----------------|---------------------------------------|
| `json`          | Full JSON object with all variables   |
| `kv`            | `KEY=value` lines                     |
| `c_header`      | C/C++ `#define` header (+ HEX)        |
| `cmake_vars`    | CMake `set()` calls                   |
| `makefile_vars` | Makefile `VAR := value` assignments   |
| `cargo_env`     | `cargo:rustc-env=` lines for build.rs |
| `template`      | Tera template (fully custom output)   |

## Version sources

| Source        | Description                              |
|---------------|------------------------------------------|
| `git`         | Git tags + branch detection              |
| `file`        | Plain text `VERSION` file                |
| `c_header`    | C/C++ `#define` macros                   |
| `env`         | Environment variables                    |
| `toml_field`  | Field in a TOML file (e.g. Cargo.toml)   |

## License

MIT
