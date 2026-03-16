# Submodule Integration Example

This is the minimal approach — works with any build system.

## Setup

```sh
# Add auto-version as a submodule
git submodule add https://github.com/yourname/auto-version vendor/auto-version
git submodule update --init

# Create your config
cp vendor/auto-version/auto-version.example.toml auto-version.toml
```

## Build the tool once

```sh
cd vendor/auto-version
cargo build --release
# Binary at: vendor/auto-version/target/release/auto-version(.exe)
```

Or put it on PATH via:
```sh
cargo install --path vendor/auto-version
```

## Run

```sh
# Show current version
auto-version show

# Generate all outputs (writes files declared in auto-version.toml)
auto-version generate

# Quick formats
auto-version show -f semver     # 1.2.3-alpha.5
auto-version show -f full       # 1.2.3-alpha.5+5
auto-version show -f info       # 1.2.3-alpha.5+5.Branch.develop.Sha.abc1234
auto-version show -f json       # full JSON
auto-version show -f c_header   # C/C++ #define block
auto-version show -f cmake      # CMake set() block
auto-version show -f kv         # KEY=value lines
auto-version show -f cargo_env  # cargo:rustc-env= lines
```

## Integration in a Makefile

```makefile
include vendor/auto-version/cmake/version.mk  # if you generate it

# Or call directly:
VERSION := $(shell vendor/auto-version/target/release/auto-version show)

all: generate-version my-app

generate-version:
	vendor/auto-version/target/release/auto-version generate
```

## Integration in a shell script / CI

```sh
VERSION=$(auto-version show)
echo "Building version $VERSION"
```

## auto-version.toml example

See [`auto-version.example.toml`](../auto-version.example.toml) in the submodule.
