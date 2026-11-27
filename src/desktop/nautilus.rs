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
    get_nautilus_scripts_dir()
        .map(|p| p.exists())
        .unwrap_or(false)
}

fn shell_single_quote(input: &str) -> String {
    format!("'{}'", input.replace('\'', "'\\''"))
}

pub fn install_nautilus_scripts(custom_exec: Option<&str>) -> Result<PathBuf> {
    let cfg = crate::config::Config::load();
    install_nautilus_scripts_with_actions(&cfg.context_menu.enabled_actions, custom_exec)
}

pub fn install_nautilus_scripts_with_actions(
    enabled_action_ids: &[String],
    custom_exec: Option<&str>,
) -> Result<PathBuf> {
    let dir = get_nautilus_scripts_dir()?;
    install_nautilus_scripts_to_dir(&dir, enabled_action_ids, custom_exec)
}

pub fn install_nautilus_scripts_to_dir(
    dir: &PathBuf,
    enabled_action_ids: &[String],
    custom_exec: Option<&str>,
) -> Result<PathBuf> {
    fs::create_dir_all(dir)?;

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_file() {
                let _ = fs::remove_file(p);
            }
        }
    }

    let exec = match custom_exec {
        Some(e) => e.to_string(),
        None => std::env::current_exe()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| "konvert".to_string()),
    };
    let quoted_exec = shell_single_quote(&exec);

    use crate::desktop::dolphin::ALL_ACTIONS;

    for action in ALL_ACTIONS {
        if !enabled_action_ids.iter().any(|id| id == action.id) {
            continue;
        }
        // Script'e dosya listesi "$@" ile geçilir; action tanımındaki %F yer tutucusu atılır.
        let args = action.cmd_args.trim_end_matches(" %F");
        let safe_name = action.name_en.replace('/', "-");
        let script_path = dir.join(safe_name);
        let content = format!(
            r#"#!/usr/bin/env bash
if [ $# -gt 0 ]; then
    exec {exec} {args} "$@"
elif [ -n "${{NAUTILUS_SCRIPT_SELECTED_FILE_PATHS:-}}" ]; then
    files=()
    while IFS= read -r line; do
        [ -n "$line" ] && files+=("$line")
    done <<< "${{NAUTILUS_SCRIPT_SELECTED_FILE_PATHS}}"
    if [ "${{#files[@]}}" -gt 0 ]; then
        exec {exec} {args} "${{files[@]}}"
    fi
fi
"#,
            exec = quoted_exec,
            args = args
        );
        fs::write(&script_path, content)?;

        let mut perms = fs::metadata(&script_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script_path, perms)?;
    }

    Ok(dir.clone())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nautilus_scripts_avoid_empty_trailing_arg_and_sync_actions() {
        let dir =
            std::env::temp_dir().join(format!("konvert_nautilus_test_{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("stale_old_script"), b"old").unwrap();

        let actions = vec!["to_webp".to_string(), "compress_video".to_string()];
        install_nautilus_scripts_to_dir(&dir, &actions, Some("/opt/my 'bin'/konvert")).unwrap();

        assert!(!dir.join("stale_old_script").exists());
        let webp_script = fs::read_to_string(dir.join("Convert to WebP")).unwrap();
        assert!(webp_script.contains("while IFS= read -r line; do"));
        assert!(webp_script.contains("'/opt/my '\\''bin'\\''/konvert'"));
        assert!(!webp_script.contains("mapfile"));

        let _ = fs::remove_dir_all(&dir);
    }
}
