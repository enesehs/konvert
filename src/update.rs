use anyhow::Result;

/// GitHub Releases üzerinden yeni bir Konvert sürümü olup olmadığını sorgular.
pub fn check_for_updates() -> Result<()> {
    let current_version = env!("CARGO_PKG_VERSION");
    println!(
        "Checking for updates... (Current version: v{})",
        current_version
    );

    let output = std::process::Command::new("curl")
        .args([
            "-sSL",
            "--connect-timeout",
            "5",
            "-H",
            "User-Agent: konvert",
            "https://api.github.com/repos/enesehs/konvert/releases/latest",
        ])
        .output();

    match output {
        Ok(out) if out.status.success() => {
            let body = String::from_utf8_lossy(&out.stdout);
            if let Some(pos) = body.find("\"tag_name\":") {
                let after = &body[pos + 11..];
                if let Some(start_quote) = after.find('"') {
                    let tag_part = &after[start_quote + 1..];
                    if let Some(end_quote) = tag_part.find('"') {
                        let tag = &tag_part[..end_quote];
                        let remote_ver = tag.trim_start_matches('v');
                        if remote_ver != current_version {
                            println!(
                                "\n[UPDATE] A new version of Konvert is available: {} (current: v{})",
                                tag, current_version
                            );
                            println!("Download release or run installer:");
                            println!(
                                "  curl -fsSL https://enesehs.dev/konvert/install.sh | bash"
                            );
                            println!("  https://github.com/enesehs/konvert/releases/tag/{}", tag);
                            return Ok(());
                        } else {
                            println!("Konvert is up to date (v{}).", current_version);
                            return Ok(());
                        }
                    }
                }
            }
            println!("Notice: Could not parse latest release information from GitHub.");
            Ok(())
        }
        _ => {
            println!("Could not connect to update server. Please check your internet connection.");
            Ok(())
        }
    }
}
