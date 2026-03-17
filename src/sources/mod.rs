pub mod c_header;
pub mod env;
pub mod file;
pub mod git;
pub mod toml_field;

use anyhow::{bail, Result};

use crate::config::Config;
use crate::VersionInfo;

/// Try each configured provider in order; return the first successful result.
pub fn resolve(config: &Config) -> Result<VersionInfo> {
    let providers = if config.source.providers.is_empty() {
        vec!["git".to_string()]
    } else {
        config.source.providers.clone()
    };

    for provider in &providers {
        let result = match provider.as_str() {
            "git" => git::resolve(config),
            "file" => file::resolve(config),
            "c_header" => c_header::resolve(config),
            "env" => env::resolve(config),
            "toml_field" => toml_field::resolve(config),
            other => bail!("unknown version source provider: {}", other),
        };
        match result {
            Ok(info) => return Ok(info),
            Err(e) => {
                eprintln!("auto-version: source '{}' failed: {:#}", provider, e);
            }
        }
    }
    bail!("all configured version sources failed")
}
