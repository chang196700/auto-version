use std::collections::HashMap;
use serde::{Deserialize, Serialize};

// ─── Top-level config ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Config {
    pub source: SourceConfig,
    pub format: FormatConfig,
    #[serde(default)]
    pub output: Vec<OutputConfig>,
}

// ─── Source ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SourceConfig {
    /// Ordered list of providers to try. First successful one wins.
    pub providers: Vec<String>,
    pub git: GitSourceConfig,
    pub file: FileSourceConfig,
    pub c_header: CHeaderSourceConfig,
    pub env: EnvSourceConfig,
    pub toml_field: TomlFieldSourceConfig,
}

impl Default for SourceConfig {
    fn default() -> Self {
        Self {
            providers: vec!["git".into()],
            git: Default::default(),
            file: Default::default(),
            c_header: Default::default(),
            env: Default::default(),
            toml_field: Default::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GitSourceConfig {
    /// Glob pattern for version tags, e.g. "v{major}.{minor}.{patch}"
    pub tag_pattern: String,
    /// Whether to fall back to `git describe` for pre-release suffix
    pub use_describe: bool,
    /// Suffix appended when working tree is dirty
    pub dirty_suffix: String,
    /// Branch-based pre-release rules: map of regex → BranchRule
    pub branch_rules: HashMap<String, BranchRule>,
    /// Conventional commits configuration
    pub conventional_commits: ConventionalCommitsConfig,
}

impl Default for GitSourceConfig {
    fn default() -> Self {
        let mut branch_rules = HashMap::new();
        branch_rules.insert(
            r"^(main|master)$".into(),
            BranchRule { label: String::new(), increment: "patch".into() },
        );
        branch_rules.insert(
            r"^develop$".into(),
            BranchRule { label: "alpha.{commits}".into(), increment: "minor".into() },
        );
        branch_rules.insert(
            r"^feature[s]?[/\-]".into(),
            BranchRule { label: "feat.{branch_slug}".into(), increment: "minor".into() },
        );
        branch_rules.insert(
            r"^release[s]?[/\-]".into(),
            BranchRule { label: "rc.{commits}".into(), increment: "none".into() },
        );
        branch_rules.insert(
            r"^hotfix(es)?[/\-]".into(),
            BranchRule { label: "hotfix.{short_sha}".into(), increment: "patch".into() },
        );
        Self {
            tag_pattern: "v{major}.{minor}.{patch}".into(),
            use_describe: true,
            dirty_suffix: "-dirty".into(),
            branch_rules,
            conventional_commits: Default::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchRule {
    /// Pre-release label template. Supports {commits}, {branch_slug}, {short_sha}.
    /// Empty string = no pre-release (release branch).
    pub label: String,
    /// Which version component to auto-increment: "major", "minor", "patch", "none", "inherit"
    pub increment: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ConventionalCommitsConfig {
    pub enabled: bool,
    pub major_pattern: String,
    pub minor_pattern: String,
    pub patch_pattern: String,
}

impl Default for ConventionalCommitsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            major_pattern: r"^(feat|fix|refactor|perf)!:|^BREAKING CHANGE".into(),
            minor_pattern: r"^feat:".into(),
            patch_pattern: r"^(fix|perf|refactor):".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct FileSourceConfig {
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct CHeaderSourceConfig {
    pub path: String,
    pub major_define: String,
    pub minor_define: String,
    pub patch_define: String,
    /// Optional single-string define like `#define VERSION "1.2.3"`
    pub version_define: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct EnvSourceConfig {
    /// Env var name containing full version string, e.g. "APP_VERSION"
    pub version_var: String,
    pub major_var: String,
    pub minor_var: String,
    pub patch_var: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct TomlFieldSourceConfig {
    pub path: String,
    /// Dot-separated field path, e.g. "package.version"
    pub field: String,
}

// ─── Format ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct FormatConfig {
    pub hex: HexFormatConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct HexFormatConfig {
    /// Bit layout: e.g. "major:8,minor:8,patch:16"
    pub layout: String,
    pub prefix: String,
}

impl Default for HexFormatConfig {
    fn default() -> Self {
        Self {
            layout: "major:8,minor:8,patch:16".into(),
            prefix: "0x".into(),
        }
    }
}

// ─── Output ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    /// Output format: "json" | "kv" | "c_header" | "cmake_vars" | "makefile_vars" |
    ///                "cargo_env" | "template"
    pub format: String,
    /// Output target: "stdout" | "file"
    #[serde(default = "default_target")]
    pub target: String,
    /// File path (required when target = "file")
    pub path: Option<String>,
    /// Template string (for format = "template")
    pub template: Option<String>,
    /// Path to external template file (for format = "template")
    pub template_file: Option<String>,
    /// Variables to include (for c_header / cmake_vars). Empty = include all.
    #[serde(default)]
    pub variables: Vec<String>,
}

fn default_target() -> String {
    "stdout".into()
}
