#![cfg(feature = "cli")]

use std::path::PathBuf;
use anyhow::Result;
use clap::{Parser, Subcommand};

// Build-time version info injected by build.rs
const TOOL_VERSION: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    " (",
    env!("GIT_SHORT_SHA", "unknown"),
    ")",
);

#[derive(Parser)]
#[command(
    name = "auto-version",
    about = "Multi-format, multi-source automatic version generator",
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
    /// Dump the resolved (+ defaulted) config as TOML
    DumpConfig,
    /// Print the example config file
    InitConfig,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // ── Locate and load config ─────────────────────────────────────────────
    let config_path = if let Some(ref p) = cli.config {
        p.clone()
    } else {
        let cwd = std::env::current_dir()?;
        auto_version::config::find_config_file(&cwd)
            .unwrap_or_else(|| cwd.join("auto-version.toml"))
    };

    match &cli.command {
        Commands::InitConfig => {
            print!("{}", EXAMPLE_CONFIG);
            return Ok(());
        }
        _ => {}
    }

    let config = if config_path.exists() {
        auto_version::config::load(&config_path)?
    } else {
        // No config file: use defaults (Git source, JSON stdout)
        auto_version::config::Config::default()
    };

    match cli.command {
        Commands::Generate => {
            auto_version::generate(&config)?;
        }

        Commands::Show { format } => {
            let info = auto_version::sources::resolve(&config)?;
            let output = match format.as_str() {
                "semver"     => info.sem_ver.clone(),
                "full"       => info.full_sem_ver.clone(),
                "info"       => info.informational_version.clone(),
                "json"       => auto_version::outputs::json::render(&info)?,
                "kv"         => auto_version::outputs::kv::render(&info)?,
                "c_header"   => auto_version::outputs::c_header::render(&info, &[])?,
                "cmake"      => auto_version::outputs::cmake_vars::render(&info, &[])?,
                "makefile"   => auto_version::outputs::makefile_vars::render(&info)?,
                "cargo_env"  => auto_version::outputs::cargo_env::render(&info)?,
                other        => anyhow::bail!("unknown format: {}", other),
            };
            print!("{}", output);
        }

        Commands::DumpConfig => {
            let toml_str = toml::to_string_pretty(&config)?;
            print!("{}", toml_str);
        }

        Commands::InitConfig => unreachable!(),
    }

    Ok(())
}

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
