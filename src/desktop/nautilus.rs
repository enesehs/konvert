use anyhow::{Context, Result};
use directories::BaseDirs;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

pub fn get_nautilus_scripts_dir() -> Result<PathBuf> {
    let base_dirs = BaseDirs::new().context("Kullanıcı ev dizini bulunamadı")?;
    let path = base_dirs
        .data_local_dir()
        .join("nautilus")
        .join("scripts")
        .join("Konvert");
    Ok(path)
}

#[allow(dead_code)]
pub fn is_nautilus_installed() -> bool {
    get_nautilus_scripts_dir().map(|p| p.exists()).unwrap_or(false)
}

pub fn install_nautilus_scripts(custom_exec: Option<&str>) -> Result<PathBuf> {
    let dir = get_nautilus_scripts_dir()?;
    fs::create_dir_all(&dir)?;

    let exec = match custom_exec {
        Some(e) => e.to_string(),
        None => std::env::current_exe()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| "konvert".to_string()),
    };

    let actions = [
        ("WebP'ye Dönüştür", "webp"),
        ("PNG'ye Dönüştür", "png"),
        ("JPG'ye Dönüştür", "jpg"),
        ("AVIF'e Dönüştür", "avif"),
        ("MP3'e Dönüştür", "mp3"),
        ("GIF'e Dönüştür", "gif"),
    ];

    for (name, format) in actions {
        let script_path = dir.join(name);
        let content = format!(
            r#"#!/usr/bin/env bash
if [ $# -gt 0 ]; then
    "{exec}" -t {format} "$@"
elif [ -n "$NAUTILUS_SCRIPT_SELECTED_FILE_PATHS" ]; then
    mapfile -t files <<< "$NAUTILUS_SCRIPT_SELECTED_FILE_PATHS"
    "{exec}" -t {format} "${{files[@]}}"
fi
"#,
            exec = exec,
            format = format
        );
        fs::write(&script_path, content)?;

        let mut perms = fs::metadata(&script_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script_path, perms)?;
    }

    Ok(dir)
}

pub fn uninstall_nautilus_scripts() -> Result<bool> {
    let dir = get_nautilus_scripts_dir()?;
    if dir.exists() {
        fs::remove_dir_all(&dir)?;
        Ok(true)
    } else {
        Ok(false)
    }
}
