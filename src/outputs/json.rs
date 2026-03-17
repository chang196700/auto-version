use anyhow::Result;

use crate::VersionInfo;

pub fn render(info: &VersionInfo) -> Result<String> {
    Ok(serde_json::to_string_pretty(info)? + "\n")
}
