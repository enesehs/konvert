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
    #[serde(default, flatten)]
    pub extra: std::collections::BTreeMap<String, toml::Value>,
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

fn default_hwaccel() -> String {
    "auto".to_string()
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VideoConfig {
    pub crf: u8,
    pub preset: String,
    pub lossless: bool,
    #[serde(default = "default_hwaccel")]
    pub hwaccel: String,
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
                // Tek kaynak: desktop::dolphin::ALL_ACTIONS (extract_frames hariç)
                enabled_actions: crate::desktop::dolphin::ALL_ACTIONS
                    .iter()
                    .filter(|action| action.id != "extract_frames")
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
                hwaccel: "auto".into(),
            },
            audio: AudioConfig {
                bitrate: "320k".into(),
            },
            document: DocumentConfig { pdf_dpi: 300 },
            profiles: std::collections::HashMap::new(),
            extra: std::collections::BTreeMap::new(),
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

    pub fn update_from_json(&mut self, json_str: &str) -> Result<()> {
        let payload: ConfigUpdatePayload = serde_json::from_str(json_str)
            .map_err(|e| anyhow::anyhow!("Geçersiz ayar JSON verisi: {:#}", e))?;
        self.apply_update(payload);
        Ok(())
    }

    pub fn apply_update(&mut self, update: ConfigUpdatePayload) {
        if let Some(general_cfg) = update.general {
            if let Some(notif) = general_cfg.notification_enabled {
                self.general.notification_enabled = notif;
            }
            if let Some(out_dir) = general_cfg.output_directory {
                self.general.output_directory = out_dir;
            }
            if let Some(lang) = general_cfg.language {
                self.general.language = lang;
            }
        }
        if let Some(ctx) = update.context_menu
            && let Some(actions) = ctx.enabled_actions
        {
            self.context_menu.enabled_actions = actions;
        }
        if let Some(img) = update.image {
            if let Some(q) = img.quality {
                self.image.quality = q;
            }
            if let Some(l) = img.lossless {
                self.image.lossless = l;
            }
        }
        if let Some(vid) = update.video {
            if let Some(crf) = vid.crf {
                self.video.crf = crf;
            }
            if let Some(preset) = vid.preset {
                self.video.preset = preset;
            }
            if let Some(lossless) = vid.lossless {
                self.video.lossless = lossless;
            }
            if let Some(hwaccel) = vid.hwaccel {
                self.video.hwaccel = hwaccel;
            }
        }
        if let Some(aud) = update.audio
            && let Some(bitrate) = aud.bitrate
        {
            self.audio.bitrate = bitrate;
        }
        if let Some(doc) = update.document
            && let Some(dpi) = doc.pdf_dpi
        {
            self.document.pdf_dpi = dpi;
        }
    }

    pub fn set_key_value(&mut self, key: &str, value: &str) -> Result<()> {
        let trimmed_val = value.trim();
        match key {
            "general.notification_enabled" => {
                self.general.notification_enabled = trimmed_val
                    .parse::<bool>()
                    .map_err(|_| anyhow::anyhow!("Geçersiz boolean değer: '{}'", value))?;
            }
            "general.output_directory" => {
                self.general.output_directory = if trimmed_val.is_empty() {
                    None
                } else {
                    Some(trimmed_val.to_string())
                };
            }
            "general.language" => {
                self.general.language = if trimmed_val.is_empty() || trimmed_val == "auto" {
                    None
                } else {
                    Some(trimmed_val.to_string())
                };
            }
            "image.quality" => {
                let q = trimmed_val
                    .parse::<u8>()
                    .map_err(|_| anyhow::anyhow!("Geçersiz kalite değeri (1-100): '{}'", value))?;
                anyhow::ensure!(
                    (1..=100).contains(&q),
                    "Kalite 1 ile 100 arasında olmalıdır"
                );
                self.image.quality = q;
            }
            "image.lossless" => {
                self.image.lossless = trimmed_val
                    .parse::<bool>()
                    .map_err(|_| anyhow::anyhow!("Geçersiz boolean değer: '{}'", value))?;
            }
            "video.crf" => {
                let crf = trimmed_val
                    .parse::<u8>()
                    .map_err(|_| anyhow::anyhow!("Geçersiz CRF değeri (0-51): '{}'", value))?;
                anyhow::ensure!(crf <= 51, "CRF 0 ile 51 arasında olmalıdır");
                self.video.crf = crf;
            }
            "video.preset" => {
                self.video.preset = trimmed_val.to_string();
            }
            "video.lossless" => {
                self.video.lossless = trimmed_val
                    .parse::<bool>()
                    .map_err(|_| anyhow::anyhow!("Geçersiz boolean değer: '{}'", value))?;
            }
            "video.hwaccel" => {
                self.video.hwaccel = trimmed_val.to_string();
            }
            "audio.bitrate" => {
                self.audio.bitrate = trimmed_val.to_string();
            }
            "document.pdf_dpi" => {
                let dpi = trimmed_val
                    .parse::<u32>()
                    .map_err(|_| anyhow::anyhow!("Geçersiz DPI değeri: '{}'", value))?;
                anyhow::ensure!(dpi > 0 && dpi <= 2400, "DPI 1 ile 2400 arasında olmalıdır");
                self.document.pdf_dpi = dpi;
            }
            "context_menu.enabled_actions" => {
                self.context_menu.enabled_actions = trimmed_val
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
            }
            _ => {
                anyhow::bail!("Bilinmeyen ayar anahtarı: '{}'", key);
            }
        }
        Ok(())
    }

    pub fn get_key_value(&self, key: &str) -> Option<String> {
        match key {
            "general.notification_enabled" => Some(self.general.notification_enabled.to_string()),
            "general.output_directory" => {
                Some(self.general.output_directory.clone().unwrap_or_default())
            }
            "general.language" => Some(
                self.general
                    .language
                    .clone()
                    .unwrap_or_else(|| "auto".to_string()),
            ),
            "image.quality" => Some(self.image.quality.to_string()),
            "image.lossless" => Some(self.image.lossless.to_string()),
            "video.crf" => Some(self.video.crf.to_string()),
            "video.preset" => Some(self.video.preset.clone()),
            "video.lossless" => Some(self.video.lossless.to_string()),
            "video.hwaccel" => Some(self.video.hwaccel.clone()),
            "audio.bitrate" => Some(self.audio.bitrate.clone()),
            "document.pdf_dpi" => Some(self.document.pdf_dpi.to_string()),
            "context_menu.enabled_actions" => Some(self.context_menu.enabled_actions.join(",")),
            _ => None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct ConfigUpdatePayload {
    pub general: Option<GeneralConfigUpdate>,
    pub context_menu: Option<ContextMenuConfigUpdate>,
    pub image: Option<ImageConfigUpdate>,
    pub video: Option<VideoConfigUpdate>,
    pub audio: Option<AudioConfigUpdate>,
    pub document: Option<DocumentConfigUpdate>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct GeneralConfigUpdate {
    pub notification_enabled: Option<bool>,
    pub output_directory: Option<Option<String>>,
    pub language: Option<Option<String>>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct ContextMenuConfigUpdate {
    pub enabled_actions: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct ImageConfigUpdate {
    pub quality: Option<u8>,
    pub lossless: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct VideoConfigUpdate {
    pub crf: Option<u8>,
    pub preset: Option<String>,
    pub lossless: Option<bool>,
    pub hwaccel: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct AudioConfigUpdate {
    pub bitrate: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct DocumentConfigUpdate {
    pub pdf_dpi: Option<u32>,
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
            crate::desktop::dolphin::ALL_ACTIONS.len() - 1
        );
        for id in &cfg.context_menu.enabled_actions {
            assert!(
                crate::desktop::dolphin::find_action(id).is_some(),
                "kayıtta olmayan varsayılan action: {id}"
            );
        }
        assert!(
            !cfg.context_menu
                .enabled_actions
                .contains(&"extract_frames".to_string())
        );
    }

    #[test]
    fn test_config_language_is_optional() {
        let cfg: Config = toml::from_str(&minimal_toml("")).unwrap();
        assert_eq!(cfg.general.language, None);
    }

    #[test]
    fn test_config_video_hwaccel() {
        let def = Config::default();
        assert_eq!(def.video.hwaccel, "auto");

        let raw = minimal_toml("language = \"en\"");
        let cfg: Config = toml::from_str(&raw).unwrap();
        assert_eq!(cfg.video.hwaccel, "auto");

        let custom_toml = "[general]\nnotification_enabled = true\n\
             [context_menu]\nenabled_actions = []\n\
             [image]\nquality = 85\nlossless = false\n\
             [video]\ncrf = 23\npreset = \"medium\"\nlossless = false\nhwaccel = \"nvenc\"\n\
             [audio]\nbitrate = \"320k\"\n\
             [document]\npdf_dpi = 300\n";
        let cfg2: Config = toml::from_str(custom_toml).unwrap();
        assert_eq!(cfg2.video.hwaccel, "nvenc");
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

    #[test]
    fn test_config_roundtrip_preserves_profiles_and_extra_sections() {
        let raw = r#"
[general]
notification_enabled = true
language = "tr"

[context_menu]
enabled_actions = ["convert_webp", "convert_png"]

[image]
quality = 80
lossless = false

[video]
crf = 26
preset = "fast"
lossless = false
hwaccel = "vaapi"

[audio]
bitrate = "192k"

[document]
pdf_dpi = 150

[profiles.discord_preset]
quality = 70
bitrate = "96k"
lossless = false

[custom_external_section]
plugin_enabled = true
author = "custom"
"#;

        let mut cfg: Config = toml::from_str(raw).expect("TOML parse failed");
        // Verify loaded correctly
        assert_eq!(cfg.general.language.as_deref(), Some("tr"));
        assert!(cfg.profiles.contains_key("discord_preset"));
        assert_eq!(cfg.profiles["discord_preset"].quality, Some(70));
        assert!(cfg.extra.contains_key("custom_external_section"));

        // Mutate one standard field (e.g. user toggles notification in GUI)
        cfg.general.notification_enabled = false;
        cfg.image.quality = 92;

        // Serialize back
        let serialized = toml::to_string_pretty(&cfg).expect("Serialization failed");

        // Deserialize again
        let reloaded: Config = toml::from_str(&serialized).expect("Reload parse failed");
        assert!(!reloaded.general.notification_enabled);
        assert_eq!(reloaded.image.quality, 92);
        assert!(reloaded.profiles.contains_key("discord_preset"));
        assert_eq!(reloaded.profiles["discord_preset"].quality, Some(70));
        assert_eq!(
            reloaded.profiles["discord_preset"].bitrate.as_deref(),
            Some("96k")
        );
        assert!(reloaded.extra.contains_key("custom_external_section"));
    }

    #[test]
    fn test_config_set_get_and_json_update() {
        let mut cfg = Config::default();

        // Test set_key_value and get_key_value
        cfg.set_key_value("image.quality", "95").unwrap();
        assert_eq!(cfg.get_key_value("image.quality"), Some("95".to_string()));
        assert_eq!(cfg.image.quality, 95);

        cfg.set_key_value("general.notification_enabled", "false")
            .unwrap();
        assert_eq!(
            cfg.get_key_value("general.notification_enabled"),
            Some("false".to_string())
        );
        assert!(!cfg.general.notification_enabled);

        cfg.set_key_value("video.hwaccel", "nvenc").unwrap();
        assert_eq!(
            cfg.get_key_value("video.hwaccel"),
            Some("nvenc".to_string())
        );
        assert_eq!(cfg.video.hwaccel, "nvenc");

        assert!(cfg.set_key_value("image.quality", "150").is_err());
        assert!(cfg.set_key_value("unknown.key", "val").is_err());

        // Test update_from_json
        let json_update = r#"{
            "image": { "quality": 88, "lossless": true },
            "video": { "crf": 18, "preset": "slow", "hwaccel": "vaapi" },
            "context_menu": { "enabled_actions": ["convert_webp", "convert_avif"] }
        }"#;

        cfg.update_from_json(json_update).unwrap();
        assert_eq!(cfg.image.quality, 88);
        assert!(cfg.image.lossless);
        assert_eq!(cfg.video.crf, 18);
        assert_eq!(cfg.video.preset, "slow");
        assert_eq!(cfg.video.hwaccel, "vaapi");
        assert_eq!(
            cfg.context_menu.enabled_actions,
            vec!["convert_webp", "convert_avif"]
        );
    }
}
