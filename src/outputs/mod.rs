pub mod c_header;
pub mod cargo_env;
pub mod cmake_vars;
pub mod json;
pub mod kv;
pub mod makefile_vars;
pub mod template;

use std::path::Path;

use anyhow::{bail, Context, Result};

use crate::config::{Config, OutputConfig};
use crate::VersionInfo;

/// Run all configured output blocks.
pub fn run_all(config: &Config, info: &VersionInfo) -> Result<()> {
    if config.output.is_empty() {
        // Default: JSON to stdout
        let rendered = json::render(info)?;
        write_stdout(&rendered);
        return Ok(());
    }
    for (i, output_cfg) in config.output.iter().enumerate() {
        run_one(output_cfg, info).with_context(|| format!("output block #{}", i))?;
    }
    Ok(())
}

fn run_one(cfg: &OutputConfig, info: &VersionInfo) -> Result<()> {
    let rendered = match cfg.format.as_str() {
        "json" => json::render(info)?,
        "kv" => kv::render(info)?,
        "c_header" => c_header::render(info, &cfg.variables)?,
        "cmake_vars" => cmake_vars::render(info, &cfg.variables)?,
        "makefile_vars" => makefile_vars::render(info)?,
        "cargo_env" => cargo_env::render(info)?,
        "template" => template::render(info, cfg)?,
        other => bail!("unknown output format: {}", other),
    };

    match cfg.target.as_str() {
        "stdout" => write_stdout(&rendered),
        "file" => write_file(cfg.path.as_deref(), &rendered)?,
        other => bail!("unknown output target: {}", other),
    }
    Ok(())
}

fn write_stdout(content: &str) {
    print!("{}", content);
}

/// Atomic-ish file write: skip if content unchanged to avoid spurious rebuilds.
fn write_file(path: Option<&str>, content: &str) -> Result<()> {
    let path = path.with_context(|| "output target is 'file' but no path specified")?;
    let path = Path::new(path);

    // Create parent dirs
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("creating parent dirs for {}", path.display()))?;
    }

    // Skip write if content is identical (preserves file mtime for incremental builds)
    if let Ok(existing) = std::fs::read_to_string(path) && existing == content {
        return Ok(());
    }

    std::fs::write(path, content)
        .with_context(|| format!("writing output file: {}", path.display()))
}
