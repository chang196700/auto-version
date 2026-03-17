use crate::VersionInfo;
use anyhow::Result;

pub fn render(info: &VersionInfo) -> Result<String> {
    let mut out = String::new();
    out.push_str(&format!("MAJOR={}\n", info.major));
    out.push_str(&format!("MINOR={}\n", info.minor));
    out.push_str(&format!("PATCH={}\n", info.patch));
    if let Some(ref pre) = info.pre_release {
        out.push_str(&format!("PRE_RELEASE={}\n", pre));
    }
    out.push_str(&format!("MAJOR_MINOR_PATCH={}\n", info.major_minor_patch));
    out.push_str(&format!("SEM_VER={}\n", info.sem_ver));
    out.push_str(&format!("FULL_SEM_VER={}\n", info.full_sem_ver));
    out.push_str(&format!(
        "INFORMATIONAL_VERSION={}\n",
        info.informational_version
    ));
    if let Some(ref b) = info.branch_name {
        out.push_str(&format!("BRANCH_NAME={}\n", b));
    }
    if let Some(ref s) = info.short_sha {
        out.push_str(&format!("SHORT_SHA={}\n", s));
    }
    if let Some(ref s) = info.sha {
        out.push_str(&format!("SHA={}\n", s));
    }
    if let Some(c) = info.commits_since_tag {
        out.push_str(&format!("COMMITS_SINCE_TAG={}\n", c));
    }
    out.push_str(&format!("BUILD_DATE={}\n", info.build_date));
    Ok(out)
}
