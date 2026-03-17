use crate::config::Config;
use crate::VersionInfo;
use anyhow::{bail, Context, Result};

pub fn resolve(config: &Config) -> Result<VersionInfo> {
    let cfg = &config.source.file;
    if cfg.path.is_empty() {
        bail!("file source: no path configured");
    }
    let content = std::fs::read_to_string(&cfg.path)
        .with_context(|| format!("reading version file: {}", cfg.path))?;
    let version_str = content.trim();
    let parsed = semver::Version::parse(version_str)
        .with_context(|| format!("parsing version '{}' from file '{}'", version_str, cfg.path))?;

    let info = VersionInfo {
        major: parsed.major,
        minor: parsed.minor,
        patch: parsed.patch,
        pre_release: if parsed.pre.is_empty() {
            None
        } else {
            Some(parsed.pre.to_string())
        },
        build_metadata: if parsed.build.is_empty() {
            None
        } else {
            Some(parsed.build.to_string())
        },
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
        source: "file".into(),
    };
    Ok(info.finalize())
}
