use anyhow::{bail, Context, Result};
use crate::config::OutputConfig;
use crate::VersionInfo;

pub fn render(info: &VersionInfo, cfg: &OutputConfig) -> Result<String> {
    // Inline template takes priority over template_file
    let template: String = if let Some(ref t) = cfg.template {
        t.clone()
    } else if let Some(ref path) = cfg.template_file {
        std::fs::read_to_string(path)
            .with_context(|| format!("reading template file: {}", path))?
    } else {
        bail!("output format 'template' requires either 'template' or 'template_file' to be set");
    };

    info.render_template(&template)
}
