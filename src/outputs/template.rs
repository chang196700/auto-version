use anyhow::{Context, Result, bail};

use crate::VersionInfo;
use crate::config::OutputConfig;

pub fn render(info: &VersionInfo, cfg: &OutputConfig) -> Result<String> {
    let template: String = if let Some(ref t) = cfg.template {
        t.clone()
    } else if let Some(ref path) = cfg.template_file {
        std::fs::read_to_string(path).with_context(|| format!("reading template file: {}", path))?
    } else {
        bail!("output format 'template' requires either 'template' or 'template_file' to be set");
    };

    render_template(info, &template)
}

/// Render a template string.
///
/// With the `template` feature enabled: uses the full Tera engine (supports
/// loops, conditionals, filters, etc.).
///
/// Without the `template` feature: uses a built-in lightweight engine that
/// handles `{{ variable }}` substitution only — sufficient for the common case
/// of generating version header files or source files.
pub fn render_template(info: &VersionInfo, template: &str) -> Result<String> {
    #[cfg(feature = "template")]
    {
        _render_tera(info, template)
    }
    #[cfg(not(feature = "template"))]
    {
        _render_simple(info, template)
    }
}

/// Lightweight `{{ var }}` substitution — zero extra dependencies.
/// Supported variables match VersionInfo fields (snake_case).
pub fn _render_simple(info: &VersionInfo, template: &str) -> Result<String> {
    let vars: &[(&str, String)] = &[
        ("major", info.major.to_string()),
        ("minor", info.minor.to_string()),
        ("patch", info.patch.to_string()),
        ("pre_release", info.pre_release.clone().unwrap_or_default()),
        (
            "build_metadata",
            info.build_metadata.clone().unwrap_or_default(),
        ),
        ("major_minor_patch", info.major_minor_patch.clone()),
        ("sem_ver", info.sem_ver.clone()),
        ("full_sem_ver", info.full_sem_ver.clone()),
        ("informational_version", info.informational_version.clone()),
        ("branch_name", info.branch_name.clone().unwrap_or_default()),
        (
            "branch_name_slug",
            info.branch_name_slug.clone().unwrap_or_default(),
        ),
        ("sha", info.sha.clone().unwrap_or_default()),
        ("short_sha", info.short_sha.clone().unwrap_or_default()),
        (
            "commits_since_tag",
            info.commits_since_tag
                .map(|v| v.to_string())
                .unwrap_or_default(),
        ),
        (
            "uncommitted_changes",
            info.uncommitted_changes
                .map(|v| v.to_string())
                .unwrap_or_default(),
        ),
        ("commit_date", info.commit_date.clone().unwrap_or_default()),
        ("build_date", info.build_date.clone()),
        ("source", info.source.clone()),
    ];

    let mut out = template.to_string();
    for (key, value) in vars {
        // Support both {{ key }} and {{key}} with optional whitespace
        out = out
            .replace(&format!("{{{{ {} }}}}", key), value)
            .replace(&format!("{{{{{}}}}}", key), value);
    }
    Ok(out)
}

/// Full Tera render — only compiled when the `template` feature is enabled.
#[cfg(feature = "template")]
fn _render_tera(info: &VersionInfo, template: &str) -> Result<String> {
    let mut ctx = tera::Context::new();
    ctx.insert("major", &info.major);
    ctx.insert("minor", &info.minor);
    ctx.insert("patch", &info.patch);
    ctx.insert("pre_release", &info.pre_release);
    ctx.insert("build_metadata", &info.build_metadata);
    ctx.insert("major_minor_patch", &info.major_minor_patch);
    ctx.insert("sem_ver", &info.sem_ver);
    ctx.insert("full_sem_ver", &info.full_sem_ver);
    ctx.insert("informational_version", &info.informational_version);
    ctx.insert("branch_name", &info.branch_name);
    ctx.insert("branch_name_slug", &info.branch_name_slug);
    ctx.insert("sha", &info.sha);
    ctx.insert("short_sha", &info.short_sha);
    ctx.insert("commits_since_tag", &info.commits_since_tag);
    ctx.insert("uncommitted_changes", &info.uncommitted_changes);
    ctx.insert("commit_date", &info.commit_date);
    ctx.insert("build_date", &info.build_date);
    ctx.insert("source", &info.source);
    tera::Tera::one_off(template, &ctx, false).map_err(Into::into)
}
