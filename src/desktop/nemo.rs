use anyhow::{Context, Result};
use directories::BaseDirs;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use super::dolphin::ALL_ACTIONS;

fn nemo_extensions(category: &str) -> &'static str {
    match category {
        "photos" => "png;jpg;jpeg;webp;avif;jxl;bmp;ico;tiff;tif;qoi;svg;heic;heif;",
        "videos" => "mp4;mkv;webm;avi;mov;flv;wmv;m4v;ts;",
        "music" => "mp3;wav;flac;ogg;m4a;aac;opus;wma;aiff;",
        "documents" => "pdf;udf;docx;doc;txt;html;",
        _ => "any;",
    }
}

pub fn get_nemo_actions_dir() -> Result<PathBuf> {
    let base_dirs = BaseDirs::new().context("Kullanıcı ev dizini bulunamadı")?;
    Ok(base_dirs.data_local_dir().join("nemo").join("actions"))
}

fn action_file_name(id: &str) -> String {
    format!("konvert_{}.nemo_action", id)
}

pub fn install_nemo_actions(custom_exec: Option<&str>) -> Result<PathBuf> {
    let dir = get_nemo_actions_dir()?;
    fs::create_dir_all(&dir)?;

    let exec = match custom_exec {
        Some(e) => e.to_string(),
        None => std::env::current_exe()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| "konvert".to_string()),
    };

    // Önce eski kayıtları temizle: action listesi/kurulum yolu değişmiş olabilir.
    if let Ok(entries) = fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            if name.to_string_lossy().starts_with("konvert_") {
                let _ = fs::remove_file(entry.path());
            }
        }
    }

    for action in ALL_ACTIONS {
        let content = format!(
            "[Nemo Action]\n\
             Name={name}\n\
             Comment=Konvert: {name}\n\
             Exec={exec} {args}\n\
             Icon-Name={icon}\n\
             Selection=m\n\
             Extensions={extensions}\n",
            name = action.name_en,
            exec = exec,
            args = action.cmd_args,
            icon = action.icon,
            extensions = nemo_extensions(action.category),
        );

        let path = dir.join(action_file_name(action.id));
        fs::write(&path, content)?;
        if let Ok(mut perms) = fs::metadata(&path).map(|m| m.permissions()) {
            perms.set_mode(0o644);
            let _ = fs::set_permissions(&path, perms);
        }
    }

    Ok(dir)
}

pub fn uninstall_nemo_actions() -> Result<bool> {
    let dir = get_nemo_actions_dir()?;
    if !dir.exists() {
        return Ok(false);
    }

    let mut removed = false;
    if let Ok(entries) = fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            if name.to_string_lossy().starts_with("konvert_") {
                let _ = fs::remove_file(entry.path());
                removed = true;
            }
        }
    }
    Ok(removed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nemo_extensions_are_terminated() {
        for category in ["photos", "videos", "music", "documents"] {
            let exts = nemo_extensions(category);
            assert!(exts.ends_with(';'), "{category} listesi ; ile bitmeli");
            assert!(!exts.contains(' '), "{category} listesi boşluk içermemeli");
        }
    }

    #[test]
    fn test_action_file_name() {
        assert_eq!(action_file_name("to_webp"), "konvert_to_webp.nemo_action");
    }
}
