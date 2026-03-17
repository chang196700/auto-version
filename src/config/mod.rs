pub mod schema;

pub use schema::{
    BranchRule, CHeaderSourceConfig, Config, ConventionalCommitsConfig, EnvSourceConfig,
    FileSourceConfig, FormatConfig, GitSourceConfig, HexFormatConfig, OutputConfig, SourceConfig,
    TomlFieldSourceConfig,
};

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

#[cfg(feature = "yaml-config")]
const CONFIG_FILE_NAMES: &[&str] = &[
    "auto-version.toml",
    "auto-version.yaml",
    "auto-version.json",
];
#[cfg(not(feature = "yaml-config"))]
const CONFIG_FILE_NAMES: &[&str] = &["auto-version.toml", "auto-version.json"];

/// Search for a config file starting from `start_dir`, walking up to the filesystem root.
pub fn find_config_file(start_dir: &Path) -> Option<PathBuf> {
    let mut dir = start_dir.to_path_buf();
    loop {
        for name in CONFIG_FILE_NAMES {
            let candidate = dir.join(name);
            if candidate.exists() {
                return Some(candidate);
            }
        }
        if !dir.pop() {
            break;
        }
    }
    None
}

/// Load and parse a config file. Detects format from file extension.
pub fn load(path: &Path) -> Result<Config> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("reading config file: {}", path.display()))?;
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("toml");
    match ext {
        "toml" => toml::from_str(&content)
            .with_context(|| format!("parsing TOML config: {}", path.display())),
        "yaml" | "yml" => {
            #[cfg(feature = "yaml-config")]
            {
                serde_yaml::from_str(&content)
                    .with_context(|| format!("parsing YAML config: {}", path.display()))
            }
            #[cfg(not(feature = "yaml-config"))]
            {
                anyhow::bail!(
                    "YAML config support is not enabled. \
                     Rebuild with `--features yaml-config`, or use a .toml config file."
                )
            }
        }
        "json" => serde_json::from_str(&content)
            .with_context(|| format!("parsing JSON config: {}", path.display())),
        other => anyhow::bail!("unsupported config format: .{}", other),
    }
}
