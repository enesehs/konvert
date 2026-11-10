use anyhow::{Context, Result};
use directories::BaseDirs;
use std::fs;
use std::path::PathBuf;

#[allow(dead_code)]
pub fn get_desktop_entry_path() -> Result<PathBuf> {
    let base_dirs = BaseDirs::new().context("Kullanıcı ev dizini bulunamadı")?;
    let path = base_dirs
        .data_local_dir()
        .join("applications")
        .join("io.github.enesehs.konvert.desktop");
    Ok(path)
}

#[allow(dead_code)]
pub fn install_desktop_entry(exec_path: Option<&str>) -> Result<PathBuf> {
    let path = get_desktop_entry_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let exec = match exec_path {
        Some(e) => e.to_string(),
        None => "konvert-settings".to_string(),
    };

    let content = format!(
        r#"[Desktop Entry]
Name=Konvert
Comment=Fast, lightweight, native file conversion and processing settings
Comment[tr]=Dosya dönüştürme ve işleme ayarları
Comment[de]=Einstellungen für Dateikonvertierung und -verarbeitung
Comment[fr]=Paramètres de conversion et de traitement de fichiers
Comment[es]=Ajustes de conversión y procesamiento de archivos
Comment[it]=Impostazioni di conversione ed elaborazione dei file
Comment[ru]=Настройки конвертации и обработки файлов
Comment[zh_CN]=文件转换和处理设置
Comment[ja]=ファイル変換および処理の設定
Exec={exec}
Icon=preferences-system-symbolic
Terminal=false
Type=Application
Categories=Utility;Settings;
Keywords=convert;donusturucu;settings;ayar;konvertieren;conversion;
"#,
        exec = exec
    );

    fs::write(&path, content)?;

    if let Ok(mut perms) = fs::metadata(&path).map(|m| m.permissions()) {
        use std::os::unix::fs::PermissionsExt;
        perms.set_mode(0o755);
        let _ = fs::set_permissions(&path, perms);
    }

    Ok(path)
}
