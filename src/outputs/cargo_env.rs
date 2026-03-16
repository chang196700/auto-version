use anyhow::Result;
use crate::VersionInfo;

/// Emit cargo:rustc-env lines for use in build.rs.
pub fn render(info: &VersionInfo) -> Result<String> {
    let mut out = String::new();
    out.push_str(&format!("cargo:rustc-env=VERSION={}\n", info.sem_ver));
    out.push_str(&format!("cargo:rustc-env=VERSION_MAJOR={}\n", info.major));
    out.push_str(&format!("cargo:rustc-env=VERSION_MINOR={}\n", info.minor));
    out.push_str(&format!("cargo:rustc-env=VERSION_PATCH={}\n", info.patch));
    out.push_str(&format!("cargo:rustc-env=VERSION_FULL={}\n", info.full_sem_ver));
    out.push_str(&format!("cargo:rustc-env=VERSION_INFO={}\n", info.informational_version));
    out.push_str(&format!("cargo:rustc-env=BUILD_DATE={}\n", info.build_date));
    if let Some(ref sha) = info.short_sha {
        out.push_str(&format!("cargo:rustc-env=GIT_SHORT_SHA={}\n", sha));
    }
    if let Some(ref branch) = info.branch_name {
        out.push_str(&format!("cargo:rustc-env=GIT_BRANCH={}\n", branch));
    }
    // Instruct cargo to re-run if .git/HEAD or refs change
    out.push_str("cargo:rerun-if-changed=.git/HEAD\n");
    out.push_str("cargo:rerun-if-changed=.git/refs\n");
    Ok(out)
}
