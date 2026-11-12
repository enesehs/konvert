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

    use crate::desktop::dolphin::{find_action, QUICK_ACTION_IDS};

    for id in QUICK_ACTION_IDS {
        let Some(action) = find_action(id) else {
            continue;
        };
        // Script'e dosya listesi "$@" ile geçilir; action tanımındaki %F yer tutucusu atılır.
        let args = action.cmd_args.trim_end_matches(" %F");
        let script_path = dir.join(action.name_en);
        let content = format!(
            r#"#!/usr/bin/env bash
if [ $# -gt 0 ]; then
    "{exec}" {args} "$@"
elif [ -n "${{NAUTILUS_SCRIPT_SELECTED_FILE_PATHS}}" ]; then
    mapfile -t files <<< "${{NAUTILUS_SCRIPT_SELECTED_FILE_PATHS}}"
    "{exec}" {args} "${{files[@]}}"
fi
"#,
            exec = exec,
            args = args
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
