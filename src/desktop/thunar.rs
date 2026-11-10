use anyhow::{Context, Result};
use directories::BaseDirs;
use std::fs;
use std::path::PathBuf;

pub fn get_thunar_uca_path() -> Result<PathBuf> {
    let base_dirs = BaseDirs::new().context("Kullanıcı ev dizini bulunamadı")?;
    let path = base_dirs
        .config_dir()
        .join("Thunar")
        .join("uca.xml");
    Ok(path)
}

#[allow(dead_code)]
pub fn is_thunar_installed() -> bool {
    if let Ok(path) = get_thunar_uca_path() {
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                return content.contains("<unique-id>konvert-");
            }
        }
    }
    false
}

fn generate_thunar_actions(exec: &str) -> String {
    format!(
        r#"  <action>
    <icon>image-x-generic-symbolic</icon>
    <name>WebP'ye Dönüştür</name>
    <submenu>Konvert</submenu>
    <unique-id>konvert-webp</unique-id>
    <command>{exec} -t webp %F</command>
    <description>Konvert ile WebP formatına dönüştür</description>
    <patterns>*</patterns>
    <image-files/>
  </action>
  <action>
    <icon>image-x-generic-symbolic</icon>
    <name>PNG'ye Dönüştür</name>
    <submenu>Konvert</submenu>
    <unique-id>konvert-png</unique-id>
    <command>{exec} -t png %F</command>
    <description>Konvert ile PNG formatına dönüştür</description>
    <patterns>*</patterns>
    <image-files/>
  </action>
  <action>
    <icon>image-x-generic-symbolic</icon>
    <name>JPG'ye Dönüştür</name>
    <submenu>Konvert</submenu>
    <unique-id>konvert-jpg</unique-id>
    <command>{exec} -t jpg %F</command>
    <description>Konvert ile JPG formatına dönüştür</description>
    <patterns>*</patterns>
    <image-files/>
  </action>
  <action>
    <icon>audio-x-generic-symbolic</icon>
    <name>MP3'e Dönüştür</name>
    <submenu>Konvert</submenu>
    <unique-id>konvert-mp3</unique-id>
    <command>{exec} -t mp3 %F</command>
    <description>Konvert ile MP3 formatına dönüştür</description>
    <patterns>*</patterns>
    <audio-files/>
    <video-files/>
  </action>
  <action>
    <icon>video-x-generic-symbolic</icon>
    <name>GIF'e Dönüştür</name>
    <submenu>Konvert</submenu>
    <unique-id>konvert-gif</unique-id>
    <command>{exec} -t gif %F</command>
    <description>Konvert ile GIF formatına dönüştür</description>
    <patterns>*</patterns>
    <video-files/>
  </action>"#,
        exec = exec
    )
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
        if content.contains("<unique-id>konvert-") {
            // Already installed
            return Ok(path);
        }

        if let Some(pos) = content.rfind("</actions>") {
            let mut new_content = content[..pos].to_string();
            new_content.push_str(&actions_snippet);
            new_content.push('\n');
            new_content.push_str(&content[pos..]);
            fs::write(&path, new_content)?;
        } else {
            let new_content = format!(
                "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<actions>\n{}\n</actions>\n",
                actions_snippet
            );
            fs::write(&path, new_content)?;
        }
    } else {
        let new_content = format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<actions>\n{}\n</actions>\n",
            actions_snippet
        );
        fs::write(&path, new_content)?;
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

    // Filter out <action> blocks containing konvert-
    let mut cleaned = String::new();
    let mut in_konvert_action = false;
    let mut action_buffer = String::new();

    for line in content.lines() {
        if line.contains("<action>") {
            action_buffer.clear();
            action_buffer.push_str(line);
            action_buffer.push('\n');
            in_konvert_action = false;
        } else if line.contains("</action>") {
            action_buffer.push_str(line);
            action_buffer.push('\n');
            if !in_konvert_action {
                cleaned.push_str(&action_buffer);
            }
            action_buffer.clear();
        } else {
            if line.contains("<unique-id>konvert-") {
                in_konvert_action = true;
            }
            action_buffer.push_str(line);
            action_buffer.push('\n');
        }
    }

    if cleaned.is_empty() && !content.contains("</actions>") {
        fs::remove_file(&path)?;
    } else {
        fs::write(&path, cleaned)?;
    }

    Ok(true)
}
