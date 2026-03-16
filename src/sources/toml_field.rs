use anyhow::{bail, Context, Result};
use crate::config::Config;
use crate::VersionInfo;

pub fn resolve(config: &Config) -> Result<VersionInfo> {
    let cfg = &config.source.toml_field;
    if cfg.path.is_empty() {
        bail!("toml_field source: no path configured");
    }
    let content = std::fs::read_to_string(&cfg.path)
        .with_context(|| format!("reading TOML file: {}", cfg.path))?;
    let doc: toml::Value = toml::from_str(&content)
        .with_context(|| format!("parsing TOML: {}", cfg.path))?;

    let field_path = if cfg.field.is_empty() { "package.version" } else { &cfg.field };
    let version_str = navigate_toml(&doc, field_path)
        .with_context(|| format!("field '{}' not found in '{}'", field_path, cfg.path))?;

    let ver = semver::Version::parse(&version_str)
        .with_context(|| format!("parsing version '{}' from '{}'", version_str, cfg.path))?;

    let info = VersionInfo {
        major: ver.major,
        minor: ver.minor,
        patch: ver.patch,
        pre_release: if ver.pre.is_empty() { None } else { Some(ver.pre.to_string()) },
        build_metadata: if ver.build.is_empty() { None } else { Some(ver.build.to_string()) },
        major_minor_patch: String::new(),
        sem_ver: String::new(),
        full_sem_ver: String::new(),
        informational_version: String::new(),
        branch_name: None,
        branch_name_slug: None,
        sha: None,
        short_sha: None,
        commits_since_tag: None,
        uncommitted_changes: None,
        version_source_sha: None,
        commit_date: None,
        build_date: chrono::Utc::now().format("%Y-%m-%d").to_string(),
        source: "toml_field".into(),
    };
    Ok(info.finalize())
}

/// Navigate a TOML value using a dot-separated path like "package.version".
fn navigate_toml(value: &toml::Value, path: &str) -> Option<String> {
    let mut current = value;
    for key in path.split('.') {
        current = current.get(key)?;
    }
    current.as_str().map(String::from)
}
