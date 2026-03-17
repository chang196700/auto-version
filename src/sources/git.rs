use std::process::Command;

use anyhow::{Context, Result};
use regex::Regex;

use crate::VersionInfo;
use crate::config::{Config, GitSourceConfig};

/// Return type for `find_version_from_tags`: (major, minor, patch, tag_sha, commits_since)
type TagResult = (u64, u64, u64, Option<String>, Option<u64>);

pub fn resolve(config: &Config) -> Result<VersionInfo> {
    let cfg = &config.source.git;

    // ── 1. Gather raw git information ────────────────────────────────────────
    let sha = git_output(&["rev-parse", "HEAD"]).ok();
    let short_sha = git_output(&["rev-parse", "--short", "HEAD"]).ok();
    let branch = current_branch();
    let dirty = is_dirty();
    let commit_date = git_output(&["log", "-1", "--format=%cs"]).ok(); // YYYY-MM-DD

    // ── 2. Find the nearest version tag ──────────────────────────────────────
    let (base_major, base_minor, base_patch, version_source_sha, commits_since_tag) =
        find_version_from_tags(cfg)?;

    // ── 3. Determine branch rule ──────────────────────────────────────────────
    let branch_rule = branch.as_deref().and_then(|b| match_branch_rule(cfg, b));

    // ── 4. Apply Conventional Commits bump (if enabled) ──────────────────────
    let (final_major, final_minor, final_patch) = if cfg.conventional_commits.enabled {
        apply_conventional_commits_bump(
            cfg,
            base_major,
            base_minor,
            base_patch,
            commits_since_tag.unwrap_or(0),
        )?
    } else {
        (base_major, base_minor, base_patch)
    };

    // ── 5. Build pre-release label from branch rule ───────────────────────────
    let pre_release = build_pre_release_label(
        branch_rule.as_ref().map(|(_, r)| r.label.as_str()),
        branch.as_deref(),
        short_sha.as_deref(),
        commits_since_tag,
        &cfg.dirty_suffix,
        dirty,
    );

    // ── 6. Build metadata (commit count since tag) ────────────────────────────
    let build_metadata = commits_since_tag.filter(|&c| c > 0).map(|c| c.to_string());

    // ── 7. Assemble VersionInfo ───────────────────────────────────────────────
    let branch_name_slug = branch.as_deref().map(slugify);
    let uncommitted_changes = if dirty {
        count_uncommitted_changes()
    } else {
        Some(0)
    };

    let info = VersionInfo {
        major: final_major,
        minor: final_minor,
        patch: final_patch,
        pre_release,
        build_metadata,
        major_minor_patch: String::new(), // filled by finalize()
        sem_ver: String::new(),
        full_sem_ver: String::new(),
        informational_version: String::new(),
        branch_name: branch.clone(),
        branch_name_slug,
        sha,
        short_sha,
        commits_since_tag,
        uncommitted_changes,
        version_source_sha,
        commit_date,
        build_date: chrono::Utc::now().format("%Y-%m-%d").to_string(),
        source: "git".into(),
    };

    Ok(info.finalize())
}

// ─── Git helpers ─────────────────────────────────────────────────────────────

fn git_output(args: &[&str]) -> Result<String> {
    let out = Command::new("git")
        .args(args)
        .output()
        .with_context(|| format!("running git {}", args.join(" ")))?;
    if !out.status.success() {
        anyhow::bail!(
            "git {} exited {}: {}",
            args.join(" "),
            out.status,
            String::from_utf8_lossy(&out.stderr)
        );
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn current_branch() -> Option<String> {
    // Prefer symbolic-ref (works in normal branches)
    if let Ok(b) = git_output(&["symbolic-ref", "--short", "HEAD"]) {
        return Some(b);
    }
    // Fallback: check GITHUB_REF_NAME env (CI detached HEAD)
    std::env::var("GITHUB_REF_NAME").ok()
}

fn is_dirty() -> bool {
    Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .ok()
        .map(|o| !o.stdout.is_empty())
        .unwrap_or(false)
}

fn count_uncommitted_changes() -> Option<u64> {
    let out = Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .ok()?;
    Some(String::from_utf8_lossy(&out.stdout).lines().count() as u64)
}

/// Find the nearest SemVer tag and return (major, minor, patch, tag_sha, commits_since).
fn find_version_from_tags(cfg: &GitSourceConfig) -> Result<TagResult> {
    // Build a regex from tag_pattern like "v{major}.{minor}.{patch}"
    let pattern_rx = tag_pattern_to_regex(&cfg.tag_pattern);
    let rx = Regex::new(&pattern_rx).with_context(|| "compiling tag pattern regex")?;

    // List all tags sorted by version (newest first)
    let tags_raw = git_output(&["tag", "--list", "--sort=-version:refname"]).unwrap_or_default();

    for tag in tags_raw.lines() {
        if let Some(cap) = rx.captures(tag.trim()) {
            let major: u64 = cap["major"].parse().unwrap_or(0);
            let minor: u64 = cap["minor"].parse().unwrap_or(0);
            let patch: u64 = cap["patch"].parse().unwrap_or(0);

            let tag_sha = git_output(&["rev-list", "-n1", tag.trim()]).ok();

            // Count commits since this tag
            let commits = git_output(&["rev-list", &format!("{}..HEAD", tag.trim()), "--count"])
                .ok()
                .and_then(|s| s.parse::<u64>().ok());

            return Ok((major, minor, patch, tag_sha, commits));
        }
    }

    // No tag found — start from 0.1.0
    let commits = git_output(&["rev-list", "--count", "HEAD"])
        .ok()
        .and_then(|s| s.parse::<u64>().ok());
    Ok((0, 1, 0, None, commits))
}

/// Convert "v{major}.{minor}.{patch}" → a named-capture regex.
fn tag_pattern_to_regex(pattern: &str) -> String {
    let mut rx = regex::escape(pattern);
    // Use [0-9]+ instead of \d+ so the pattern works without the unicode feature.
    rx = rx.replace(r"\{major\}", r"(?P<major>[0-9]+)");
    rx = rx.replace(r"\{minor\}", r"(?P<minor>[0-9]+)");
    rx = rx.replace(r"\{patch\}", r"(?P<patch>[0-9]+)");
    format!("^{}$", rx)
}

// ─── Branch rule matching ─────────────────────────────────────────────────────

/// Returns the matched (pattern, BranchRule) or None.
fn match_branch_rule<'a>(
    cfg: &'a GitSourceConfig,
    branch: &str,
) -> Option<(String, &'a crate::config::schema::BranchRule)> {
    for (pattern, rule) in &cfg.branch_rules {
        if let Ok(rx) = Regex::new(pattern)
            && rx.is_match(branch)
        {
            return Some((pattern.to_string(), rule));
        }
    }
    None
}

// ─── Pre-release label builder ────────────────────────────────────────────────

fn build_pre_release_label(
    label_template: Option<&str>,
    branch: Option<&str>,
    short_sha: Option<&str>,
    commits: Option<u64>,
    dirty_suffix: &str,
    dirty: bool,
) -> Option<String> {
    let template = label_template?;
    if template.is_empty() {
        return if dirty {
            Some(dirty_suffix.trim_start_matches('-').to_string())
        } else {
            None
        };
    }

    let branch_slug = branch.map(slugify).unwrap_or_default();
    let mut label = template
        .replace("{commits}", &commits.unwrap_or(0).to_string())
        .replace("{branch_slug}", &branch_slug)
        .replace("{short_sha}", short_sha.unwrap_or("unknown"));

    if dirty {
        label.push_str(dirty_suffix);
    }
    Some(label)
}

// ─── Conventional Commits bump ────────────────────────────────────────────────

fn apply_conventional_commits_bump(
    cfg: &GitSourceConfig,
    major: u64,
    minor: u64,
    patch: u64,
    commits_since_tag: u64,
) -> Result<(u64, u64, u64)> {
    if commits_since_tag == 0 {
        return Ok((major, minor, patch));
    }

    // Get commit messages since last tag
    let log = git_output(&[
        "log",
        &format!("HEAD~{}..HEAD", commits_since_tag),
        "--format=%s",
    ])
    .unwrap_or_default();

    let cc = &cfg.conventional_commits;
    let major_rx = Regex::new(&cc.major_pattern).ok();
    let minor_rx = Regex::new(&cc.minor_pattern).ok();
    let patch_rx = Regex::new(&cc.patch_pattern).ok();

    let mut bump = BumpLevel::None;
    for line in log.lines() {
        if major_rx.as_ref().map(|r| r.is_match(line)).unwrap_or(false) {
            bump = BumpLevel::Major;
            break;
        } else if minor_rx.as_ref().map(|r| r.is_match(line)).unwrap_or(false)
            && bump < BumpLevel::Minor
        {
            bump = BumpLevel::Minor;
        } else if patch_rx.as_ref().map(|r| r.is_match(line)).unwrap_or(false)
            && bump < BumpLevel::Patch
        {
            bump = BumpLevel::Patch;
        }
    }

    Ok(match bump {
        BumpLevel::Major => (major + 1, 0, 0),
        BumpLevel::Minor => (major, minor + 1, 0),
        BumpLevel::Patch => (major, minor, patch + 1),
        BumpLevel::None => (major, minor, patch),
    })
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
enum BumpLevel {
    None,
    Patch,
    Minor,
    Major,
}

// ─── Utilities ────────────────────────────────────────────────────────────────

/// Convert a branch name to a slug safe for use in pre-release identifiers.
pub fn slugify(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .trim_matches('-')
        .to_lowercase()
}
