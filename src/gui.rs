use anyhow::{Context, Result};
use std::path::PathBuf;
use std::process::{Command, Stdio};

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

    let target = candidates.into_iter().find(|p| p.exists());

    let mut cmd = match target {
        Some(path) => Command::new(path),
        None => Command::new("konvert-settings"),
    };

    cmd.stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    cmd.spawn()
        .context("Failed to launch konvert settings GUI")?;
    Ok(())
}
