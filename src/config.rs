use anyhow::Result;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
pub struct ProfileConfig {
    pub quality: Option<u8>,
    pub crf: Option<u8>,
    pub preset: Option<String>,
    pub bitrate: Option<String>,
    pub dpi: Option<u32>,
    pub lossless: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub general: GeneralConfig,
    pub context_menu: ContextMenuConfig,
    pub image: ImageConfig,
    pub video: VideoConfig,
    pub audio: AudioConfig,
    pub document: DocumentConfig,
    #[serde(default)]
    pub profiles: std::collections::HashMap<String, ProfileConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GeneralConfig {
    pub notification_enabled: bool,
    pub output_directory: Option<String>,
    /// GUI'de seçilen arayüz dili (ör. "tr", "en"). Boşsa sistem yereli kullanılır.
    #[serde(default)]
    pub language: Option<String>,
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
                language: None,
            },
            context_menu: ContextMenuConfig {
                // Tek kaynak: desktop::dolphin::ALL_ACTIONS
                enabled_actions: crate::desktop::dolphin::ALL_ACTIONS
                    .iter()
                    .map(|action| action.id.to_string())
                    .collect(),
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
            document: DocumentConfig { pdf_dpi: 300 },
            profiles: std::collections::HashMap::new(),
        }
    }
}

impl Config {
    pub fn get_profile(&self, name: &str) -> Option<ProfileConfig> {
        if let Some(custom) = self.profiles.get(name) {
            return Some(custom.clone());
        }
        match name {
            "web" => Some(ProfileConfig {
                quality: Some(80),
                crf: Some(28),
                preset: Some("fast".into()),
                bitrate: Some("128k".into()),
                dpi: Some(150),
                lossless: Some(false),
            }),
            "balanced" => Some(ProfileConfig {
                quality: Some(85),
                crf: Some(23),
                preset: Some("medium".into()),
                bitrate: Some("192k".into()),
                dpi: Some(300),
                lossless: Some(false),
            }),
            "archive" => Some(ProfileConfig {
                quality: Some(95),
                crf: Some(18),
                preset: Some("slow".into()),
                bitrate: Some("320k".into()),
                dpi: Some(600),
                lossless: Some(false),
            }),
            "lossless" => Some(ProfileConfig {
                quality: None,
                crf: Some(0),
                preset: None,
                bitrate: Some("320k".into()),
                dpi: None,
                lossless: Some(true),
            }),
            _ => None,
        }
    }
    pub fn config_path() -> Option<PathBuf> {
        let proj_dirs = ProjectDirs::from("", "", "konvert")?;
        Some(proj_dirs.config_dir().join("config.toml"))
    }

    pub fn load() -> Self {
        if let Some(path) = Self::config_path()
            && path.exists()
        {
            match fs::read_to_string(&path) {
                Ok(content) => match toml::from_str::<Config>(&content) {
                    Ok(cfg) => return cfg,
                    Err(err) => {
                        // Bozuk ayar dosyasını sessizce ezmek yerine yedekle.
                        eprintln!(
                            "Uyarı: {} ayrıştırılamadı ({}). Yedekleniyor: {}",
                            path.display(),
                            err,
                            path.with_extension("toml.bak").display()
                        );
                        let _ = fs::rename(&path, path.with_extension("toml.bak"));
                    }
                },
                Err(err) => {
                    eprintln!(
                        "Uyarı: {} okunamadı ({}). Varsayılan ayarlar kullanılacak.",
                        path.display(),
                        err
                    );
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
            let tmp = path.with_extension("toml.tmp");
            fs::write(&tmp, toml_str)?;
            fs::rename(&tmp, &path)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn minimal_toml(extra_general: &str) -> String {
        format!(
            "[general]\nnotification_enabled = true\n{extra_general}\n\
             [context_menu]\nenabled_actions = []\n\
             [image]\nquality = 85\nlossless = false\n\
             [video]\ncrf = 23\npreset = \"medium\"\nlossless = false\n\
             [audio]\nbitrate = \"320k\"\n\
             [document]\npdf_dpi = 300\n"
        )
    }

    #[test]
    fn test_config_parses_language_and_output_directory() {
        let raw = minimal_toml("language = \"de\"\noutput_directory = \"/tmp/out\"");
        let cfg: Config = toml::from_str(&raw).unwrap();
        assert_eq!(cfg.general.language.as_deref(), Some("de"));
        assert_eq!(cfg.general.output_directory.as_deref(), Some("/tmp/out"));
    }

    #[test]
    fn test_default_actions_exist_in_registry() {
        let cfg = Config::default();
        assert_eq!(
            cfg.context_menu.enabled_actions.len(),
            crate::desktop::dolphin::ALL_ACTIONS.len()
        );
        for id in &cfg.context_menu.enabled_actions {
            assert!(
                crate::desktop::dolphin::find_action(id).is_some(),
                "kayıtta olmayan varsayılan action: {id}"
            );
        }
    }

    #[test]
    fn test_config_language_is_optional() {
        let cfg: Config = toml::from_str(&minimal_toml("")).unwrap();
        assert_eq!(cfg.general.language, None);
    }

    #[test]
    fn test_config_profiles_builtins_and_custom() {
        let mut cfg = Config::default();
        let web_profile = cfg.get_profile("web").expect("web profile must exist");
        assert_eq!(web_profile.quality, Some(80));
        assert_eq!(web_profile.crf, Some(28));

        // Custom override or new profile
        cfg.profiles.insert(
            "custom_photo".into(),
            ProfileConfig {
                quality: Some(99),
                ..Default::default()
            },
        );
        let custom = cfg
            .get_profile("custom_photo")
            .expect("custom_photo must exist");
        assert_eq!(custom.quality, Some(99));
        assert!(cfg.get_profile("non_existent").is_none());
    }
}
