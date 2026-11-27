use anyhow::Result;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

pub fn validate_custom_profile(name: &str, profile: &ProfileConfig) -> Result<String> {
    let trimmed = name.trim();
    anyhow::ensure!(!trimmed.is_empty(), "Profil adı boş olamaz");
    anyhow::ensure!(
        trimmed.len() <= 64,
        "Profil adı 64 karakteri aşamaz: '{}'",
        trimmed
    );
    anyhow::ensure!(
        trimmed
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-'),
        "Profil adı yalnızca harf, rakam, '_' ve '-' içerebilir: '{}'",
        trimmed
    );
    let lower = trimmed.to_ascii_lowercase();
    anyhow::ensure!(
        !matches!(lower.as_str(), "web" | "balanced" | "archive" | "lossless"),
        "Yerleşik profil adı ('{}') üzerine yazılamaz",
        trimmed
    );
    anyhow::ensure!(
        profile.quality.is_some()
            || profile.crf.is_some()
            || profile.preset.is_some()
            || profile.bitrate.is_some()
            || profile.dpi.is_some()
            || profile.lossless.is_some(),
        "Profil en az bir ayar içermelidir (--quality, --crf, --preset, --bitrate, --dpi veya --lossless)"
    );
    Ok(trimmed.to_string())
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
pub struct ProfileConfig {
    pub quality: Option<u8>,
    pub crf: Option<u8>,
    pub preset: Option<String>,
    pub bitrate: Option<String>,
    pub dpi: Option<u32>,
    pub lossless: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct Config {
    pub general: GeneralConfig,
    pub context_menu: ContextMenuConfig,
    pub image: ImageConfig,
    pub video: VideoConfig,
    pub audio: AudioConfig,
    pub document: DocumentConfig,
    pub profiles: std::collections::HashMap<String, ProfileConfig>,
    #[serde(flatten)]
    pub extra: std::collections::BTreeMap<String, toml::Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct GeneralConfig {
    pub notification_enabled: bool,
    pub output_directory: Option<String>,
    /// GUI'de seçilen arayüz dili (ör. "tr", "en"). Boşsa sistem yereli kullanılır.
    pub language: Option<String>,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            notification_enabled: true,
            output_directory: None,
            language: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct ContextMenuConfig {
    pub enabled_actions: Vec<String>,
}

impl Default for ContextMenuConfig {
    fn default() -> Self {
        Self {
            // Tek kaynak: desktop::dolphin::ALL_ACTIONS (yalnızca default_enabled olanlar)
            enabled_actions: crate::desktop::dolphin::ALL_ACTIONS
                .iter()
                .filter(|action| action.default_enabled)
                .map(|action| action.id.to_string())
                .collect(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct ImageConfig {
    pub quality: u8,
    pub lossless: bool,
}

impl Default for ImageConfig {
    fn default() -> Self {
        Self {
            quality: 85,
            lossless: false,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct VideoConfig {
    pub crf: u8,
    pub preset: String,
    pub lossless: bool,
    pub hwaccel: String,
}

impl Default for VideoConfig {
    fn default() -> Self {
        Self {
            crf: 23,
            preset: "medium".into(),
            lossless: false,
            hwaccel: "auto".into(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct AudioConfig {
    pub bitrate: String,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            bitrate: "320k".into(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct DocumentConfig {
    pub pdf_dpi: u32,
}

impl Default for DocumentConfig {
    fn default() -> Self {
        Self { pdf_dpi: 300 }
    }
}

fn validate_quality(q: u8) -> Result<u8> {
    anyhow::ensure!(
        (1..=100).contains(&q),
        "Kalite 1 ile 100 arasında olmalıdır"
    );
    Ok(q)
}

fn validate_crf(crf: u8) -> Result<u8> {
    anyhow::ensure!(crf <= 51, "CRF 0 ile 51 arasında olmalıdır");
    Ok(crf)
}

fn validate_preset(preset: &str) -> Result<String> {
    let trimmed = preset.trim().to_ascii_lowercase();
    anyhow::ensure!(
        matches!(
            trimmed.as_str(),
            "ultrafast"
                | "superfast"
                | "veryfast"
                | "faster"
                | "fast"
                | "medium"
                | "slow"
                | "slower"
                | "veryslow"
                | "placebo"
        ),
        "Geçersiz video önayarı: '{}'",
        preset
    );
    Ok(trimmed)
}

fn validate_hwaccel(hwaccel: &str) -> Result<String> {
    let trimmed = hwaccel.trim().to_ascii_lowercase();
    anyhow::ensure!(
        matches!(
            trimmed.as_str(),
            "auto" | "vaapi" | "nvenc" | "none" | "cpu"
        ),
        "Geçersiz donanım hızlandırma kipi: '{}'",
        hwaccel
    );
    Ok(trimmed)
}

fn validate_bitrate(bitrate: &str) -> Result<String> {
    let trimmed = bitrate.trim().to_ascii_lowercase();
    let num_part = trimmed
        .strip_suffix('k')
        .or_else(|| trimmed.strip_suffix('m'))
        .unwrap_or(&trimmed);
    let parsed: f64 = num_part
        .parse()
        .map_err(|_| anyhow::anyhow!("Geçersiz bit hızı değeri: '{}'", bitrate))?;
    anyhow::ensure!(
        parsed.is_finite() && parsed > 0.0 && parsed <= 1024.0,
        "Geçersiz bit hızı aralığı: '{}'",
        bitrate
    );
    Ok(trimmed)
}

fn validate_pdf_dpi(dpi: u32) -> Result<u32> {
    anyhow::ensure!(
        (1..=2400).contains(&dpi),
        "DPI 1 ile 2400 arasında olmalıdır"
    );
    Ok(dpi)
}

fn validate_language(lang: Option<String>) -> Result<Option<String>> {
    match lang {
        None => Ok(None),
        Some(raw) => {
            let trimmed = raw.trim();
            if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("auto") {
                Ok(None)
            } else {
                anyhow::ensure!(
                    trimmed.len() <= 16
                        && trimmed
                            .chars()
                            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'),
                    "Geçersiz dil kodu: '{}'",
                    raw
                );
                Ok(Some(trimmed.to_ascii_lowercase()))
            }
        }
    }
}

fn deserialize_double_option<'de, D>(deserializer: D) -> Result<Option<Option<String>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Option::<String>::deserialize(deserializer).map(Some)
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

    pub fn to_toml_string(&self) -> Result<String> {
        let mut scalar_extra = toml::Table::new();
        let mut table_extra = std::collections::BTreeMap::new();
        for (k, v) in &self.extra {
            let is_table_like = v.is_table()
                || matches!(v, toml::Value::Array(arr) if arr.iter().any(|item| item.is_table()));
            if is_table_like {
                table_extra.insert(k.clone(), v.clone());
            } else {
                scalar_extra.insert(k.clone(), v.clone());
            }
        }

        let clean_copy = Config {
            general: self.general.clone(),
            context_menu: self.context_menu.clone(),
            image: self.image.clone(),
            video: self.video.clone(),
            audio: self.audio.clone(),
            document: self.document.clone(),
            profiles: self.profiles.clone(),
            extra: table_extra,
        };
        let body = toml::to_string_pretty(&clean_copy)?;
        if scalar_extra.is_empty() {
            Ok(body)
        } else {
            let header = toml::to_string_pretty(&scalar_extra)?;
            Ok(format!("{}\n{}", header.trim_end(), body))
        }
    }

    pub fn load() -> Self {
        Self::load_inner(true)
    }

    pub fn load_readonly() -> Self {
        Self::load_inner(false)
    }

    fn load_inner(allow_writes: bool) -> Self {
        if let Some(path) = Self::config_path()
            && path.exists()
        {
            match fs::read_to_string(&path) {
                Ok(content) => match toml::from_str::<Config>(&content) {
                    Ok(cfg) => return cfg,
                    Err(err) => {
                        if !allow_writes {
                            eprintln!(
                                "Uyarı: {} ayrıştırılamadı ({}). Varsayılan ayarlar kullanılacak.",
                                path.display(),
                                err
                            );
                            return Self::default();
                        }
                        // Bozuk ayar dosyasını sessizce ezmek yerine yedekle.
                        let bak = path.with_extension("toml.bak");
                        eprintln!(
                            "Uyarı: {} ayrıştırılamadı ({}). Yedekleniyor: {}",
                            path.display(),
                            err,
                            bak.display()
                        );
                        if fs::rename(&path, &bak).is_err() {
                            return Self::default();
                        }
                    }
                },
                Err(err) => {
                    eprintln!(
                        "Uyarı: {} okunamadı ({}). Mevcut dosya korunarak varsayılan ayarlar kullanılacak.",
                        path.display(),
                        err
                    );
                    return Self::default();
                }
            }
        }

        let default_config = Self::default();
        if allow_writes {
            let _ = default_config.save();
        }
        default_config
    }

    pub fn save(&self) -> Result<()> {
        if let Some(path) = Self::config_path() {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            let toml_str = self.to_toml_string()?;

            #[cfg(unix)]
            let _lock_file = {
                use std::os::unix::io::AsRawFd;
                let lock_path = path.with_extension("toml.lock");
                let file = fs::OpenOptions::new()
                    .create(true)
                    .read(true)
                    .write(true)
                    .truncate(false)
                    .open(&lock_path)?;
                // SAFETY: Valid file descriptor locked exclusively for config write; unlocked on drop.
                unsafe {
                    libc::flock(file.as_raw_fd(), libc::LOCK_EX);
                }
                file
            };

            let tmp = path.with_extension(format!("toml.{}.tmp", std::process::id()));
            fs::write(&tmp, toml_str)?;
            fs::rename(&tmp, &path)?;
        }
        Ok(())
    }

    pub fn update_from_json(&mut self, json_str: &str) -> Result<()> {
        let payload: ConfigUpdatePayload = serde_json::from_str(json_str)
            .map_err(|e| anyhow::anyhow!("Geçersiz ayar JSON verisi: {:#}", e))?;
        self.apply_update(payload)
    }

    pub fn apply_update(&mut self, update: ConfigUpdatePayload) -> Result<()> {
        let mut next = self.clone();

        if let Some(general_cfg) = update.general {
            if let Some(notif) = general_cfg.notification_enabled {
                next.general.notification_enabled = notif;
            }
            if let Some(out_dir) = general_cfg.output_directory {
                next.general.output_directory = out_dir.and_then(|s| {
                    let t = s.trim();
                    if t.is_empty() {
                        None
                    } else {
                        Some(t.to_string())
                    }
                });
            }
            if let Some(lang) = general_cfg.language {
                next.general.language = validate_language(lang)?;
            }
        }
        if let Some(ctx) = update.context_menu
            && let Some(actions) = ctx.enabled_actions
        {
            next.context_menu.enabled_actions = actions;
        }
        if let Some(img) = update.image {
            if let Some(q) = img.quality {
                next.image.quality = validate_quality(q)?;
            }
            if let Some(l) = img.lossless {
                next.image.lossless = l;
            }
        }
        if let Some(vid) = update.video {
            if let Some(crf) = vid.crf {
                next.video.crf = validate_crf(crf)?;
            }
            if let Some(preset) = vid.preset {
                next.video.preset = validate_preset(&preset)?;
            }
            if let Some(lossless) = vid.lossless {
                next.video.lossless = lossless;
            }
            if let Some(hwaccel) = vid.hwaccel {
                next.video.hwaccel = validate_hwaccel(&hwaccel)?;
            }
        }
        if let Some(aud) = update.audio
            && let Some(bitrate) = aud.bitrate
        {
            next.audio.bitrate = validate_bitrate(&bitrate)?;
        }
        if let Some(doc) = update.document
            && let Some(dpi) = doc.pdf_dpi
        {
            next.document.pdf_dpi = validate_pdf_dpi(dpi)?;
        }

        *self = next;
        Ok(())
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
                self.general.language = validate_language(Some(trimmed_val.to_string()))?;
            }
            "image.quality" => {
                let q = trimmed_val
                    .parse::<u8>()
                    .map_err(|_| anyhow::anyhow!("Geçersiz kalite değeri (1-100): '{}'", value))?;
                self.image.quality = validate_quality(q)?;
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
                self.video.crf = validate_crf(crf)?;
            }
            "video.preset" => {
                self.video.preset = validate_preset(trimmed_val)?;
            }
            "video.lossless" => {
                self.video.lossless = trimmed_val
                    .parse::<bool>()
                    .map_err(|_| anyhow::anyhow!("Geçersiz boolean değer: '{}'", value))?;
            }
            "video.hwaccel" => {
                self.video.hwaccel = validate_hwaccel(trimmed_val)?;
            }
            "audio.bitrate" => {
                self.audio.bitrate = validate_bitrate(trimmed_val)?;
            }
            "document.pdf_dpi" => {
                let dpi = trimmed_val
                    .parse::<u32>()
                    .map_err(|_| anyhow::anyhow!("Geçersiz DPI değeri: '{}'", value))?;
                self.document.pdf_dpi = validate_pdf_dpi(dpi)?;
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
    #[serde(default, deserialize_with = "deserialize_double_option")]
    pub output_directory: Option<Option<String>>,
    #[serde(default, deserialize_with = "deserialize_double_option")]
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
        let expected_len = crate::desktop::dolphin::ALL_ACTIONS
            .iter()
            .filter(|a| a.default_enabled)
            .count();
        assert_eq!(cfg.context_menu.enabled_actions.len(), expected_len);
        for id in &cfg.context_menu.enabled_actions {
            assert!(
                crate::desktop::dolphin::find_action(id).is_some(),
                "kayıtta olmayan varsayılan action: {id}"
            );
        }
        assert!(
            cfg.context_menu
                .enabled_actions
                .contains(&"strip_metadata_video".to_string())
        );
        assert!(
            !cfg.context_menu
                .enabled_actions
                .contains(&"extract_frames".to_string())
        );
        assert!(
            !cfg.context_menu
                .enabled_actions
                .contains(&"to_bmp".to_string())
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

    #[test]
    fn test_config_json_null_clears_optional_fields_and_rejects_invalid_values() {
        let mut cfg = Config::default();
        cfg.general.output_directory = Some("/tmp/out".to_string());
        cfg.general.language = Some("tr".to_string());

        // Null resets both fields without wiping sibling keys
        cfg.update_from_json(r#"{"general": {"output_directory": null, "language": null}}"#)
            .unwrap();
        assert_eq!(cfg.general.output_directory, None);
        assert_eq!(cfg.general.language, None);

        // Omitting them does not alter newly set values
        cfg.general.output_directory = Some("/tmp/keep".to_string());
        cfg.update_from_json(r#"{"general": {"notification_enabled": false}}"#)
            .unwrap();
        assert_eq!(cfg.general.output_directory.as_deref(), Some("/tmp/keep"));

        // Invalid values fail atomically without partial mutation
        let prev_q = cfg.image.quality;
        assert!(
            cfg.update_from_json(r#"{"image": {"quality": 0}, "video": {"crf": 99}}"#)
                .is_err()
        );
        assert_eq!(cfg.image.quality, prev_q);
        assert!(cfg.set_key_value("video.preset", "invalid_preset").is_err());
        assert!(cfg.set_key_value("video.hwaccel", "vulkan").is_err());
        assert!(cfg.set_key_value("audio.bitrate", "0k").is_err());
    }

    #[test]
    fn test_config_partial_toml_and_top_level_scalar_extra_roundtrip() {
        let partial = r#"
z_scalar = 42
alpha_flag = true

[image]
quality = 77
"#;
        let cfg: Config = toml::from_str(partial).expect("Partial TOML must parse");
        assert_eq!(cfg.image.quality, 77);
        assert_eq!(cfg.video.preset, "medium");
        assert_eq!(cfg.document.pdf_dpi, 300);
        assert_eq!(cfg.extra.get("z_scalar"), Some(&toml::Value::Integer(42)));

        let serialized = cfg
            .to_toml_string()
            .expect("Top-level scalar extra must serialize before tables");
        let reloaded: Config = toml::from_str(&serialized).expect("Roundtrip parse failed");
        assert_eq!(reloaded.image.quality, 77);
        assert_eq!(
            reloaded.extra.get("z_scalar"),
            Some(&toml::Value::Integer(42))
        );
        assert_eq!(
            reloaded.extra.get("alpha_flag"),
            Some(&toml::Value::Boolean(true))
        );
    }

    #[test]
    fn test_validate_custom_profile_rejects_invalid_and_accepts_valid() {
        let valid_profile = ProfileConfig {
            quality: Some(88),
            crf: None,
            preset: None,
            bitrate: None,
            dpi: None,
            lossless: None,
        };
        assert!(validate_custom_profile("", &valid_profile).is_err());
        assert!(validate_custom_profile("   ", &valid_profile).is_err());
        assert!(validate_custom_profile("bad/name", &valid_profile).is_err());
        assert!(validate_custom_profile("has space", &valid_profile).is_err());
        assert!(validate_custom_profile(&"a".repeat(65), &valid_profile).is_err());

        for builtin in ["web", "balanced", "archive", "lossless", "WEB", "Balanced"] {
            assert!(validate_custom_profile(builtin, &valid_profile).is_err());
        }

        let empty_profile = ProfileConfig::default();
        assert!(validate_custom_profile("my_profile", &empty_profile).is_err());

        assert_eq!(
            validate_custom_profile("  social-hq_1  ", &valid_profile).unwrap(),
            "social-hq_1"
        );
    }
}
