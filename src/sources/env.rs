use anyhow::{bail, Context, Result};
use crate::config::Config;
use crate::VersionInfo;

pub fn resolve(config: &Config) -> Result<VersionInfo> {
    let cfg = &config.source.env;

    // Try full version string var first
    if !cfg.version_var.is_empty() && let Ok(val) = std::env::var(&cfg.version_var) {
        let ver = semver::Version::parse(&val)
            .with_context(|| format!("parsing version from env var '{}'", cfg.version_var))?;
        return Ok(make_info(ver));
    }

    let major: u64 = read_env_u64(&cfg.major_var, "MAJOR")?;
    let minor: u64 = read_env_u64(&cfg.minor_var, "MINOR")?;
    let patch: u64 = read_env_u64(&cfg.patch_var, "PATCH")?;

    if major == 0 && minor == 0 && patch == 0 && cfg.major_var.is_empty() {
        bail!("env source: no version variables configured or found");
    }

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
        source: "env".into(),
    };
    Ok(info.finalize())
}

fn read_env_u64(var_name: &str, _fallback: &str) -> Result<u64> {
    if var_name.is_empty() {
        return Ok(0);
    }
    let val = std::env::var(var_name)
        .with_context(|| format!("env var '{}' not set", var_name))?;
    val.parse::<u64>()
        .with_context(|| format!("parsing env var '{}' = '{}' as integer", var_name, val))
}

fn make_info(ver: semver::Version) -> VersionInfo {
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
        source: "env".into(),
    };
    info.finalize()
}
