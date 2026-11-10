use anyhow::Result;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub general: GeneralConfig,
    pub context_menu: ContextMenuConfig,
    pub image: ImageConfig,
    pub video: VideoConfig,
    pub audio: AudioConfig,
    pub document: DocumentConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GeneralConfig {
    pub notification_enabled: bool,
    pub output_directory: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContextMenuConfig {
    pub enabled_actions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ImageConfig {
    pub quality: u8,
    pub lossless: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VideoConfig {
    pub crf: u8,
    pub preset: String,
    pub lossless: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AudioConfig {
    pub bitrate: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DocumentConfig {
    pub pdf_dpi: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            general: GeneralConfig {
                notification_enabled: true,
                output_directory: None,
            },
            context_menu: ContextMenuConfig {
                enabled_actions: vec![
                    "compress_photo".into(),
                    "to_webp".into(),
                    "to_png".into(),
                    "to_jpg".into(),
                    "to_avif".into(),
                    "to_jxl".into(),
                    "to_jxl_lossless".into(),
                    "merge_to_pdf".into(),
                    "strip_metadata".into(),
                    "resize_50".into(),
                    "resize_1080p".into(),
                    "web_opt".into(),
                    "discord_8mb_photo".into(),
                    "email_25mb_photo".into(),
                    "compress_video".into(),
                    "to_mp4".into(),
                    "to_mkv".into(),
                    "to_webm".into(),
                    "to_gif".into(),
                    "extract_frames".into(),
                    "extract_audio".into(),
                    "discord_8mb_video".into(),
                    "email_25mb_video".into(),
                    "to_mp3".into(),
                    "to_wav".into(),
                    "audio_to_video".into(),
                    "compress_doc".into(),
                    "udf_to_pdf".into(),
                    "udf_to_txt".into(),
                    "pdf_to_images".into(),
                ],
            },
            image: ImageConfig {
                quality: 85,
                lossless: false,
            },
            video: VideoConfig {
                crf: 23,
                preset: "medium".into(),
                lossless: false,
            },
            audio: AudioConfig {
                bitrate: "320k".into(),
            },
            document: DocumentConfig {
                pdf_dpi: 300,
            },
        }
    }
}

impl Config {
    pub fn config_path() -> Option<PathBuf> {
        let proj_dirs = ProjectDirs::from("", "", "konvert")?;
        Some(proj_dirs.config_dir().join("config.toml"))
    }

    pub fn load() -> Self {
        if let Some(path) = Self::config_path() {
            if path.exists() {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(cfg) = toml::from_str(&content) {
                        return cfg;
                    }
                }
            }
        }
        let default_config = Self::default();
        let _ = default_config.save();
        default_config
    }

    pub fn save(&self) -> Result<()> {
        if let Some(path) = Self::config_path() {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            let toml_str = toml::to_string_pretty(self)?;
            fs::write(path, toml_str)?;
        }
        Ok(())
    }
}
