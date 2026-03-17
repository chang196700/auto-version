use anyhow::{Context, Result, bail};
use regex::Regex;

use crate::VersionInfo;
use crate::config::Config;

pub fn resolve(config: &Config) -> Result<VersionInfo> {
    let cfg = &config.source.c_header;
    if cfg.path.is_empty() {
        bail!("c_header source: no path configured");
    }
    let content = std::fs::read_to_string(&cfg.path)
        .with_context(|| format!("reading C header: {}", cfg.path))?;

    // Try full version string define first
    if !cfg.version_define.is_empty() {
        let rx = Regex::new(&format!(
            r#"#\s*define\s+{}\s+"?([^\s"]+)"?"#,
            regex::escape(&cfg.version_define)
        ))?;
        if let Some(cap) = rx.captures(&content) {
            let ver = semver::Version::parse(&cap[1])
                .with_context(|| format!("parsing version from define {}", cfg.version_define))?;
            return Ok(make_info(ver, "c_header"));
        }
    }

    // Try individual major/minor/patch defines
    let major = parse_define(&content, &cfg.major_define)?;
    let minor = parse_define(&content, &cfg.minor_define)?;
    let patch = parse_define(&content, &cfg.patch_define)?;

    let info = VersionInfo {
        major,
        minor,
        patch,
        pre_release: None,
        build_metadata: None,
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
        source: "c_header".into(),
    };
    Ok(info.finalize())
}

fn parse_define(content: &str, name: &str) -> Result<u64> {
    if name.is_empty() {
        return Ok(0);
    }
    let rx = Regex::new(&format!(r"#\s*define\s+{}\s+(\d+)", regex::escape(name)))?;
    let cap = rx
        .captures(content)
        .with_context(|| format!("define '{}' not found in C header", name))?;
    cap[1]
        .parse::<u64>()
        .with_context(|| format!("parsing define '{}'", name))
}

fn make_info(ver: semver::Version, source: &str) -> VersionInfo {
    let info = VersionInfo {
        major: ver.major,
        minor: ver.minor,
        patch: ver.patch,
        pre_release: if ver.pre.is_empty() {
            None
        } else {
            Some(ver.pre.to_string())
        },
        build_metadata: if ver.build.is_empty() {
            None
        } else {
            Some(ver.build.to_string())
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
        source: source.into(),
    };
    info.finalize()
}
