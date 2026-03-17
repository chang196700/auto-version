#![cfg(feature = "cli")]

use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};

// ── Compile-time version constants (set by build.rs via auto-version) ─────────
//
// On first build (no binary yet) build.rs falls back to raw git subprocess.
// From the second build onward, auto-version reads auto-version.toml and sets
// these from git tags + branch rules.

/// Short semver, e.g. `0.1.0` or `0.1.0-alpha.3`
const VERSION_STR: &str = match option_env!("VERSION") {
    Some(v) => v,
    None => env!("CARGO_PKG_VERSION"),
};

/// Full semver including build metadata, e.g. `0.1.0+abc1234`
const VERSION_FULL: &str = match option_env!("VERSION_FULL") {
    Some(v) => v,
    None => VERSION_STR,
};

/// Informational version: `0.1.0.Branch.main.Sha.<full-sha>`
const VERSION_INFO: &str = match option_env!("VERSION_INFO") {
    Some(v) => v,
    None => VERSION_STR,
};

const GIT_SHA: &str = match option_env!("GIT_SHORT_SHA") {
    Some(v) => v,
    None => "unknown",
};
const GIT_BRANCH: &str = match option_env!("GIT_BRANCH") {
    Some(v) => v,
    None => "unknown",
};
const BUILD_DATE: &str = match option_env!("BUILD_DATE") {
    Some(v) => v,
    None => "",
};

/// Combined string shown by `--version` and in the help header: `0.1.0 (abc1234)`
const TOOL_VERSION: &str = match option_env!("TOOL_VERSION") {
    Some(v) => v,
    None => VERSION_STR,
};

// ── CLI definition ─────────────────────────────────────────────────────────────

#[derive(Parser)]
#[command(
    name    = "auto-version",
    about   = "Multi-format, multi-source automatic version generator",
    version = TOOL_VERSION,
    arg_required_else_help = true,
)]
struct Cli {
    /// Path to config file (auto-discovered if omitted)
    #[arg(short, long, global = true)]
    config: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate version from configured sources and write all outputs
    Generate,

    /// Print the resolved version string to stdout (quick check)
    Show {
        /// Output format: semver | full | info | json | kv | c_header | cmake | makefile | cargo_env
        #[arg(short, long, default_value = "semver")]
        format: String,
    },

    /// Show build version information (git tag, commit, branch, date)
    Version {
        /// Output format: brief | full | json
        #[arg(short, long, default_value = "brief")]
        format: String,
    },

    /// Dump the resolved (+ defaulted) config as TOML
    DumpConfig,

    /// Print the example config file
    InitConfig,
}

// ── Entry point ────────────────────────────────────────────────────────────────

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Handle commands that don't need a config file
    match &cli.command {
        Commands::InitConfig => {
            print!("{}", EXAMPLE_CONFIG);
            return Ok(());
        }
        Commands::Version { format } => {
            return cmd_version(format);
        }
        _ => {}
    }

    // Locate and load config
    let config_path = if let Some(ref p) = cli.config {
        p.clone()
    } else {
        let cwd = std::env::current_dir()?;
        auto_version::config::find_config_file(&cwd)
            .unwrap_or_else(|| cwd.join("auto-version.toml"))
    };

    let config = if config_path.exists() {
        auto_version::config::load(&config_path)?
    } else {
        auto_version::config::Config::default()
    };

    match cli.command {
        Commands::Generate => {
            auto_version::generate(&config)?;
        }

        Commands::Show { format } => {
            let info = auto_version::sources::resolve(&config)?;
            let output = match format.as_str() {
                "semver" => info.sem_ver.clone(),
                "full" => info.full_sem_ver.clone(),
                "info" => info.informational_version.clone(),
                "json" => auto_version::outputs::json::render(&info)?,
                "kv" => auto_version::outputs::kv::render(&info)?,
                "c_header" => auto_version::outputs::c_header::render(&info, &[])?,
                "cmake" => auto_version::outputs::cmake_vars::render(&info, &[])?,
                "makefile" => auto_version::outputs::makefile_vars::render(&info)?,
                "cargo_env" => auto_version::outputs::cargo_env::render(&info)?,
                other => anyhow::bail!("unknown format: {}", other),
            };
            print!("{}", output);
        }

        Commands::DumpConfig => {
            let toml_str = toml::to_string_pretty(&config)?;
            print!("{}", toml_str);
        }

        Commands::Version { .. } | Commands::InitConfig => unreachable!(),
    }

    Ok(())
}

// ── `version` subcommand ───────────────────────────────────────────────────────

fn cmd_version(format: &str) -> Result<()> {
    match format {
        "brief" => {
            println!("auto-version {VERSION_STR}");
            println!("  commit  {GIT_SHA}");
            println!("  branch  {GIT_BRANCH}");
            if !BUILD_DATE.is_empty() {
                println!("  date    {BUILD_DATE}");
            }
        }
        "full" => {
            println!("{VERSION_INFO}");
        }
        "json" => {
            println!(
                "{{\n  \"version\": \"{VERSION_STR}\",\n  \"full\": \"{VERSION_FULL}\",\
                 \n  \"informational\": \"{VERSION_INFO}\",\n  \"sha\": \"{GIT_SHA}\",\
                 \n  \"branch\": \"{GIT_BRANCH}\",\n  \"date\": \"{BUILD_DATE}\"\n}}"
            );
        }
        other => anyhow::bail!("unknown format '{}'; valid: brief | full | json", other),
    }
    Ok(())
}

// ── Embedded example config ────────────────────────────────────────────────────

const EXAMPLE_CONFIG: &str = r#"# auto-version.toml — Version generation configuration
# Run `auto-version init-config > auto-version.toml` to create this file.

[source]
# Ordered list of providers. First successful one wins.
providers = ["git"]

[source.git]
tag_pattern = "v{major}.{minor}.{patch}"
use_describe = true
dirty_suffix = "-dirty"

# Branch-based pre-release rules
[source.git.branch_rules]
"^(main|master)$"   = { label = "",                   increment = "patch" }
"^develop$"          = { label = "alpha.{commits}",    increment = "minor" }
"^feature[s]?[/-]"  = { label = "feat.{branch_slug}", increment = "minor" }
"^release[s]?[/-]"  = { label = "rc.{commits}",       increment = "none"  }
"^hotfix(es)?[/-]"  = { label = "hotfix.{short_sha}",  increment = "patch" }

[source.git.conventional_commits]
enabled = false
major_pattern = '^(feat|fix)!:|^BREAKING CHANGE'
minor_pattern = '^feat:'
patch_pattern = '^(fix|perf|refactor):'

# Generate a C/C++ header
[[output]]
format = "c_header"
target = "file"
path = "include/version.h"

# Also write CMake variables
[[output]]
format = "cmake_vars"
target = "file"
path = "cmake/version.cmake"

# Also print JSON to stdout
[[output]]
format = "json"
target = "stdout"
"#;
