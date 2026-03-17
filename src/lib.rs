pub mod build_rs;
pub mod config;
pub mod formats;
pub mod outputs;
pub mod sources;
pub mod writers;

use anyhow::Result;
use config::Config;

/// The resolved version information, containing all computed variables.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VersionInfo {
    // Core numeric parts
    pub major: u64,
    pub minor: u64,
    pub patch: u64,

    // SemVer parts
    pub pre_release: Option<String>,
    pub build_metadata: Option<String>,

    // Derived string forms
    pub major_minor_patch: String,
    pub sem_ver: String,
    pub full_sem_ver: String,
    pub informational_version: String,

    // Git context
    pub branch_name: Option<String>,
    pub branch_name_slug: Option<String>,
    pub sha: Option<String>,
    pub short_sha: Option<String>,
    pub commits_since_tag: Option<u64>,
    pub uncommitted_changes: Option<u64>,
    pub version_source_sha: Option<String>,

    // Date
    pub commit_date: Option<String>,
    pub build_date: String,

    // Source metadata
    pub source: String,
}

impl VersionInfo {
    /// Build derived fields from the core major/minor/patch values.
    pub fn finalize(mut self) -> Self {
        self.major_minor_patch = format!("{}.{}.{}", self.major, self.minor, self.patch);
        self.sem_ver = match &self.pre_release {
            Some(pre) => format!("{}-{}", self.major_minor_patch, pre),
            None => self.major_minor_patch.clone(),
        };
        self.full_sem_ver = match &self.build_metadata {
            Some(meta) => format!("{}+{}", self.sem_ver, meta),
            None => self.sem_ver.clone(),
        };
        let branch_part = self
            .branch_name
            .as_deref()
            .map(|b| format!(".Branch.{}", b))
            .unwrap_or_default();
        let sha_part = self
            .sha
            .as_deref()
            .map(|s| format!(".Sha.{}", s))
            .unwrap_or_default();
        self.informational_version = format!("{}{}{}", self.full_sem_ver, branch_part, sha_part);
        self
    }

    /// Render a template string using VersionInfo fields as context.
    /// Delegates to `outputs::template::render_template`, which uses the full
    /// Tera engine when the `template` feature is enabled, or the built-in
    /// lightweight `{{ var }}` substitution otherwise.
    pub fn render_template(&self, template: &str) -> Result<String> {
        outputs::template::render_template(self, template)
    }
}

/// Top-level entry point: load config, resolve version, run all outputs.
pub fn generate(config: &Config) -> Result<VersionInfo> {
    let info = sources::resolve(config)?;
    outputs::run_all(config, &info)?;
    Ok(info)
}
