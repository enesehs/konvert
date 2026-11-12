use anyhow::{Context, Result, bail};
use directories::BaseDirs;
use std::fs;
use std::path::{Path, PathBuf};

pub fn get_thunar_uca_path() -> Result<PathBuf> {
    let base_dirs = BaseDirs::new().context("Kullanıcı ev dizini bulunamadı")?;
    let path = base_dirs.config_dir().join("Thunar").join("uca.xml");
    Ok(path)
}

#[allow(dead_code)]
pub fn is_thunar_installed() -> bool {
    if let Ok(path) = get_thunar_uca_path()
        && path.exists()
        && let Ok(content) = fs::read_to_string(&path)
    {
        return content.contains("<unique-id>konvert-");
    }
    false
}

fn xml_escape(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn generate_thunar_actions(exec: &str) -> String {
    use crate::desktop::dolphin::{QUICK_ACTION_IDS, find_action};

    let mut out = String::new();
    for id in QUICK_ACTION_IDS {
        let Some(action) = find_action(id) else {
            continue;
        };
        let file_type = match action.category {
            "photos" => "    <image-files/>\n",
            "videos" => "    <video-files/>\n",
            "music" => "    <audio-files/>\n    <video-files/>\n",
            _ => "",
        };
        let name = xml_escape(action.name_en);
        out.push_str(&format!(
            "  <action>\n    <icon>{icon}</icon>\n    <name>{name}</name>\n    <submenu>Konvert</submenu>\n    <unique-id>konvert-{id}</unique-id>\n    <command>{exec} {args}</command>\n    <description>Konvert: {name}</description>\n    <patterns>*</patterns>\n{file_type}  </action>\n",
            icon = action.icon,
            name = name,
            id = action.id,
            exec = exec,
            args = action.cmd_args,
            file_type = file_type,
        ));
    }
    out
}

/// `<action>` etiketinin başlangıç konumunu döndürür.
///
/// Yalnızca tam `<action>` / `<action ...>` etiketlerini eşler; `<actions>`
/// kök etiketiyle karışmaz.
fn find_action_start(s: &str) -> Option<usize> {
    let mut search_from = 0;
    while let Some(rel) = s[search_from..].find("<action") {
        let abs = search_from + rel;
        let after = s[abs + "<action".len()..].chars().next();
        let is_tag = match after {
            Some('>') | Some('/') => true,
            Some(c) => c.is_whitespace(),
            None => false,
        };
        if is_tag {
            return Some(abs);
        }
        search_from = abs + 1;
    }
    None
}

/// `unique-id` değeri `konvert-` ile başlayan `<action>` bloklarını çıkarır.
///
/// Blok dışındaki her şey (XML bildirimi, `<actions>` kökü, kullanıcı
/// action'ları ve boş satırlar) olduğu gibi korunur. Kapanış etiketi eksik
/// bozuk bir blokla karşılaşılırsa kalan içerik dokunulmadan bırakılır.
fn strip_konvert_actions(content: &str) -> String {
    let mut out = String::with_capacity(content.len());
    let mut cursor = 0usize;

    while let Some(start_rel) = find_action_start(&content[cursor..]) {
        let start = cursor + start_rel;
        out.push_str(&content[cursor..start]);

        let Some(end_rel) = content[start..].find("</action>") else {
            out.push_str(&content[start..]);
            return out;
        };
        let end = start + end_rel + "</action>".len();
        let block = &content[start..end];
        if !block.contains("<unique-id>konvert-") {
            out.push_str(block);
        }
        cursor = end;
    }

    out.push_str(&content[cursor..]);
    out
}

fn atomic_write(path: &Path, content: &str) -> Result<()> {
    let tmp = path.with_extension("xml.tmp");
    fs::write(&tmp, content)
        .with_context(|| format!("Geçici dosya yazılamadı: {}", tmp.display()))?;
    fs::rename(&tmp, path).with_context(|| format!("Dosya güncellenemedi: {}", path.display()))?;
    Ok(())
}

pub fn install_thunar_actions(custom_exec: Option<&str>) -> Result<PathBuf> {
    let path = get_thunar_uca_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let exec = match custom_exec {
        Some(e) => e.to_string(),
        None => std::env::current_exe()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| "konvert".to_string()),
    };

    let actions_snippet = generate_thunar_actions(&exec);

    if path.exists() {
        let content = fs::read_to_string(&path)?;
        if !content.contains("</actions>") {
            bail!(
                "Thunar yapılandırması beklenen biçimde değil (</actions> bulunamadı): {}",
                path.display()
            );
        }

        // Güncelleme durumunda eski Konvert action'larını temizleyip yenilerini ekle;
        // böylece komut yolu değiştiğinde kayıtlar güncel kalır.
        let cleaned = strip_konvert_actions(&content);
        let insert_at = cleaned
            .rfind("</actions>")
            .context("Thunar yapılandırmasında </actions> konumu bulunamadı")?;

        let mut new_content = String::with_capacity(cleaned.len() + actions_snippet.len() + 1);
        new_content.push_str(&cleaned[..insert_at]);
        new_content.push_str(&actions_snippet);
        new_content.push('\n');
        new_content.push_str(&cleaned[insert_at..]);
        atomic_write(&path, &new_content)?;
    } else {
        let new_content = format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<actions>\n{}\n</actions>\n",
            actions_snippet
        );
        atomic_write(&path, &new_content)?;
    }

    Ok(path)
}

pub fn uninstall_thunar_actions() -> Result<bool> {
    let path = get_thunar_uca_path()?;
    if !path.exists() {
        return Ok(false);
    }

    let content = fs::read_to_string(&path)?;
    if !content.contains("<unique-id>konvert-") {
        return Ok(false);
    }

    let cleaned = strip_konvert_actions(&content);
    if cleaned == content {
        return Ok(false);
    }

    atomic_write(&path, &cleaned)?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<actions>
  <action>
    <icon>utilities-terminal</icon>
    <name>Open Terminal Here</name>
    <unique-id>user-terminal-1</unique-id>
    <command>exo-open --launch TerminalEmulator</command>
    <patterns>*</patterns>
    <directories/>
  </action>
  <action>
    <icon>image-x-generic-symbolic</icon>
    <name>WebP'ye Dönüştür</name>
    <unique-id>konvert-webp</unique-id>
    <command>/usr/bin/konvert -t webp %F</command>
    <patterns>*</patterns>
    <image-files/>
  </action>
  <action>
    <icon>audio-x-generic-symbolic</icon>
    <name>MP3'e Dönüştür</name>
    <unique-id>konvert-mp3</unique-id>
    <command>/usr/bin/konvert -t mp3 %F</command>
    <patterns>*</patterns>
    <audio-files/>
  </action>
</actions>
"#;

    #[test]
    fn test_strip_konvert_actions_preserves_xml_root_and_user_actions() {
        let cleaned = strip_konvert_actions(FIXTURE);

        assert!(cleaned.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
        assert!(cleaned.contains("<actions>"));
        assert!(cleaned.contains("</actions>"));
        assert!(cleaned.contains("user-terminal-1"));
        assert!(cleaned.contains("Open Terminal Here"));

        assert!(!cleaned.contains("konvert-webp"));
        assert!(!cleaned.contains("konvert-mp3"));
        assert!(!cleaned.contains("<unique-id>konvert-"));
    }

    #[test]
    fn test_strip_konvert_actions_only_konvert_left() {
        let only_konvert = FIXTURE
            .replace("user-terminal-1", "konvert-x")
            .replace("Open Terminal Here", "X");
        let cleaned = strip_konvert_actions(&only_konvert);

        // Kök ve XML bildirimi korunmalı, içeride Konvert action'ı kalmamalı.
        assert!(cleaned.contains("<actions>"));
        assert!(cleaned.contains("</actions>"));
        assert!(!cleaned.contains("<unique-id>konvert-"));
    }

    #[test]
    fn test_strip_handles_missing_closing_tag() {
        let broken = "<actions>\n<action>\n<unique-id>konvert-x</unique-id>\n";
        let cleaned = strip_konvert_actions(broken);
        // Kapanış yoksa veri kaybetmemek için içerik dokunulmadan bırakılır.
        assert_eq!(cleaned, broken);
    }

    #[test]
    fn test_find_action_start_ignores_actions_root() {
        assert_eq!(find_action_start("<actions>\n<action>"), Some(10));
        assert_eq!(find_action_start("<actions></actions>"), None);
    }
}
