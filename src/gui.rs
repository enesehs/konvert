use anyhow::{Context, Result};
use std::io::IsTerminal;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

fn is_executable_file(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::metadata(path)
            .map(|m| (m.permissions().mode() & 0o111) != 0)
            .unwrap_or(false)
    }
    #[cfg(not(unix))]
    {
        true
    }
}

/// Konvert Qt6 Ayarlar uygulamasını (`konvert-settings`) başlatır.
/// Çalıştırılabilir dosyayı sırasıyla mevcut ikilinin yanında, yerel dizinlerde ve sistem yollarında arar.
pub fn launch_settings_gui() -> Result<()> {
    let mut candidates = Vec::new();

    // 1. Mevcut ikili dosyanın kardeşi veya lib dizini
    if let Ok(exe) = std::env::current_exe() {
        candidates.push(exe.with_file_name("konvert-settings"));
        if let Some(parent) = exe.parent().and_then(|p| p.parent()) {
            candidates.push(parent.join("lib").join("konvert").join("konvert-settings"));
        }
    }

    // 2. Kullanıcı yerel dizinleri (~/.local/bin ve ~/.local/lib/konvert)
    if let Ok(home) = std::env::var("HOME") {
        let p = PathBuf::from(home);
        candidates.push(p.join(".local/bin/konvert-settings"));
        candidates.push(p.join(".local/lib/konvert/konvert-settings"));
    }

    // 3. Sistem dizinleri
    candidates.push(PathBuf::from("/usr/lib/konvert/konvert-settings"));
    candidates.push(PathBuf::from("/usr/local/bin/konvert-settings"));
    candidates.push(PathBuf::from("/usr/bin/konvert-settings"));

    let target = candidates.into_iter().find(|p| is_executable_file(p));

    let mut cmd = match target {
        Some(path) => Command::new(path),
        None => Command::new("konvert-settings"),
    };

    let stderr_cfg = if std::io::stderr().is_terminal() {
        Stdio::inherit()
    } else {
        Stdio::null()
    };

    cmd.stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(stderr_cfg);

    cmd.spawn()
        .context("Failed to launch konvert settings GUI")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_executable_file_checks_type_and_mode() {
        let dir =
            std::env::temp_dir().join(format!("konvert_gui_exec_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        assert!(!is_executable_file(&dir));

        let plain = dir.join("plain.txt");
        std::fs::write(&plain, b"echo hi\n").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&plain, std::fs::Permissions::from_mode(0o644)).unwrap();
            assert!(!is_executable_file(&plain));

            std::fs::set_permissions(&plain, std::fs::Permissions::from_mode(0o755)).unwrap();
            assert!(is_executable_file(&plain));
        }

        let _ = std::fs::remove_dir_all(&dir);
    }
}
