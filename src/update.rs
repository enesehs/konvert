use anyhow::{Context, Result, bail};

fn parse_semver(version: &str) -> Option<(u64, u64, u64)> {
    let clean = version.trim().trim_start_matches('v');
    let core = clean.split(['-', '+']).next()?;
    let mut parts = core.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next().unwrap_or("0").parse().ok()?;
    let patch = parts.next().unwrap_or("0").parse().ok()?;
    if parts.next().is_some() {
        return None;
    }
    Some((major, minor, patch))
}

fn is_newer_version(remote: &str, current: &str) -> bool {
    match (parse_semver(remote), parse_semver(current)) {
        (Some(r), Some(c)) => r > c,
        _ => false,
    }
}

fn validate_tag(tag: &str) -> bool {
    !tag.is_empty()
        && tag.len() <= 64
        && tag
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_'))
}

/// GitHub Releases üzerinden yeni bir Konvert sürümü olup olmadığını sorgular.
pub fn check_for_updates() -> Result<()> {
    let current_version = env!("CARGO_PKG_VERSION");
    println!(
        "Checking for updates... (Current version: v{})",
        current_version
    );

    let output = std::process::Command::new("curl")
        .args([
            "-fsSL",
            "--connect-timeout",
            "5",
            "--max-time",
            "10",
            "-H",
            "User-Agent: konvert",
            "https://api.github.com/repos/enesehs/konvert/releases/latest",
        ])
        .output()
        .with_context(|| "Failed to execute curl for update check")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!(
            "Failed to query GitHub releases ({}): {}",
            output.status,
            stderr.trim()
        );
    }

    let payload: serde_json::Value = serde_json::from_slice(&output.stdout)
        .with_context(|| "Invalid JSON response from GitHub releases API")?;
    let tag = payload
        .get("tag_name")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| validate_tag(s))
        .ok_or_else(|| anyhow::anyhow!("Release tag_name missing or malformed in response"))?;

    if is_newer_version(tag, current_version) {
        println!(
            "\n[UPDATE] A new version of Konvert is available: {} (current: v{})",
            tag, current_version
        );
        println!("Download and verify release from:");
        println!("  https://github.com/enesehs/konvert/releases/tag/{}", tag);
    } else {
        println!("Konvert is up to date (v{}).", current_version);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semver_parsing_and_comparison() {
        assert_eq!(parse_semver("v0.4.0"), Some((0, 4, 0)));
        assert_eq!(parse_semver("0.5.1-rc1"), Some((0, 5, 1)));
        assert!(is_newer_version("v0.5.0", "0.4.0"));
        assert!(!is_newer_version("v0.4.0", "0.4.0"));
        assert!(!is_newer_version("v0.3.9", "0.4.0"));
        assert!(!is_newer_version("v0.4.0", "0.5.0"));
    }

    #[test]
    fn test_validate_tag_rejects_control_chars() {
        assert!(validate_tag("v0.4.0"));
        assert!(validate_tag("v0.5.0-rc.1"));
        assert!(!validate_tag(""));
        assert!(!validate_tag("v0.4.0\nrm -rf /"));
    }
}
