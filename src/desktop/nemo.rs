use anyhow::{Context, Result};
use directories::BaseDirs;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use super::dolphin::ALL_ACTIONS;

fn nemo_extensions(category: &str) -> &'static str {
    match category {
        "photos" => "png;jpg;jpeg;webp;avif;jxl;bmp;ico;tiff;tif;qoi;svg;heic;heif;gif;",
        "videos" => "mp4;mkv;webm;avi;mov;flv;wmv;m4v;ts;",
        "music" => "mp3;wav;flac;ogg;m4a;aac;opus;wma;aiff;",
        "pdf" => "pdf;",
        "udf" => "udf;",
        "word" => "docx;",
        "documents" => "pdf;udf;docx;",
        _ => "any;",
    }
}

fn quote_exec(exec: &str) -> String {
    if exec.contains(' ') {
        format!("\"{}\"", exec.replace('"', "\\\""))
    } else {
        exec.to_string()
    }
}

pub fn get_nemo_actions_dir() -> Result<PathBuf> {
    let base_dirs = BaseDirs::new().context("Kullanıcı ev dizini bulunamadı")?;
    Ok(base_dirs.data_local_dir().join("nemo").join("actions"))
}

pub fn is_nemo_installed() -> bool {
    let Ok(dir) = get_nemo_actions_dir() else {
        return false;
    };
    let Ok(entries) = fs::read_dir(dir) else {
        return false;
    };
    entries.flatten().any(|e| {
        let name = e.file_name();
        let s = name.to_string_lossy();
        s.starts_with("konvert_") && s.ends_with(".nemo_action")
    })
}

fn action_file_name(id: &str) -> String {
    format!("konvert_{}.nemo_action", id)
}

pub fn install_nemo_actions(custom_exec: Option<&str>) -> Result<PathBuf> {
    install_nemo_actions_with_options(custom_exec, None)
}

pub fn install_nemo_actions_with_options(
    custom_exec: Option<&str>,
    target_dir: Option<PathBuf>,
) -> Result<PathBuf> {
    let cfg = crate::config::Config::load();
    install_nemo_actions_with_actions(&cfg.context_menu.enabled_actions, custom_exec, target_dir)
}

pub fn install_nemo_actions_with_actions(
    enabled_action_ids: &[String],
    custom_exec: Option<&str>,
    target_dir: Option<PathBuf>,
) -> Result<PathBuf> {
    let dir = match target_dir {
        Some(d) => d,
        None => get_nemo_actions_dir()?,
    };
    fs::create_dir_all(&dir)?;

    let exec = match custom_exec {
        Some(e) => e.to_string(),
        None => std::env::current_exe()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| "konvert".to_string()),
    };
    let quoted_exec = quote_exec(&exec);

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
        if !enabled_action_ids.iter().any(|id| id == action.id) {
            continue;
        }
        let content = format!(
            "[Nemo Action]\n\
             Name={name_en}\n\
             Name[tr]={name_tr}\n\
             Name[de]={name_de}\n\
             Name[fr]={name_fr}\n\
             Name[es]={name_es}\n\
             Comment=Konvert: {name_en}\n\
             Exec={exec} {args}\n\
             Icon-Name=konvert-symbolic\n\
             Selection=notnone\n\
             Quote=double\n\
             Extensions={extensions}\n",
            name_en = action.name_en,
            name_tr = action.name_tr,
            name_de = action.name_de,
            name_fr = action.name_fr,
            name_es = action.name_es,
            exec = quoted_exec,
            args = action.cmd_args,
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
    uninstall_nemo_actions_with_options(None)
}

pub fn uninstall_nemo_actions_with_options(target_dir: Option<PathBuf>) -> Result<bool> {
    let dir = match target_dir {
        Some(d) => d,
        None => get_nemo_actions_dir()?,
    };
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

    #[test]
    fn test_nemo_actions_selection_quote_and_filtering() {
        let temp_dir =
            std::env::temp_dir().join(format!("konvert_test_nemo_{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp_dir);
        let enabled = vec!["to_webp".to_string(), "strip_metadata_video".to_string()];
        install_nemo_actions_with_actions(
            &enabled,
            Some("/opt/my apps/konvert"),
            Some(temp_dir.clone()),
        )
        .expect("install nemo actions");

        let webp_file = temp_dir.join("konvert_to_webp.nemo_action");
        assert!(webp_file.exists());
        assert!(!temp_dir.join("konvert_to_bmp.nemo_action").exists());

        let content = fs::read_to_string(&webp_file).unwrap();
        assert!(content.contains("Selection=notnone"));
        assert!(content.contains("Quote=double"));
        assert!(content.contains("Icon-Name=konvert-symbolic"));
        assert!(content.contains("Exec=\"/opt/my apps/konvert\" -t webp %F"));
        assert!(content.contains("Name[tr]="));

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
