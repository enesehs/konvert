use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "konvert",
    author = "enesehs (https://enesehs.dev)",
    version,
    about = "Fast, lightweight, native right-click file converter and optimizer for Linux and KDE Plasma"
)]
pub struct Cli {
    /// Target format (e.g. webp, png, jpg, avif, jxl, mp4, gif, mp3, txt, pdf)
    #[arg(
        short = 't',
        long = "target",
        conflicts_with_all = ["resize", "target_size", "strip_metadata", "extract_frames", "merge_pdf"]
    )]
    pub target: Option<String>,

    /// Files to convert (single or batch)
    #[arg(value_name = "FILES")]
    pub files: Vec<PathBuf>,

    /// Output quality (1-100)
    #[arg(short = 'q', long = "quality", value_parser = clap::value_parser!(u8).range(1..=100))]
    pub quality: Option<u8>,

    /// Lossless compression / conversion mode
    #[arg(short = 'l', long = "lossless")]
    pub lossless: bool,

    /// Overwrite existing output files instead of creating name_2, name_3, ...
    #[arg(short = 'f', long = "force")]
    pub force: bool,

    /// Create a .bak backup before overwriting existing files
    #[arg(long = "backup")]
    pub backup: bool,

    /// Preview conversion plan and output destinations without modifying disk
    #[arg(long = "dry-run")]
    pub dry_run: bool,

    /// Recursively scan directories for input files
    #[arg(short = 'r', long = "recursive")]
    pub recursive: bool,

    /// Maximum number of files converted in parallel (default: CPU cores, capped at 4)
    #[arg(short = 'j', long = "jobs", value_name = "N", value_parser = parse_jobs)]
    pub jobs: Option<usize>,

    /// Merge multiple images into a single PDF document
    #[arg(
        long = "merge-pdf",
        conflicts_with_all = ["target", "resize", "target_size", "strip_metadata", "extract_frames"]
    )]
    pub merge_pdf: bool,

    /// Smart compression to target file size (e.g. 8MB, 25MB)
    #[arg(
        long = "target-size",
        conflicts_with_all = ["target", "resize", "strip_metadata", "extract_frames", "merge_pdf"]
    )]
    pub target_size: Option<String>,

    /// Conversion profile preset (web, balanced, archive, lossless, or custom)
    #[arg(long = "profile")]
    pub profile: Option<String>,

    /// Save current CLI settings as a named preset profile in config
    #[arg(long = "save-profile")]
    pub save_profile: Option<String>,

    /// List all available conversion profiles (built-in and custom)
    #[arg(long = "list-profiles")]
    pub list_profiles: bool,

    /// Check for online updates from GitHub releases
    #[arg(long = "check-update")]
    pub check_update: bool,

    /// Strip EXIF, GPS, and privacy metadata
    #[arg(
        long = "strip-metadata",
        conflicts_with_all = ["target", "resize", "target_size", "extract_frames", "merge_pdf"]
    )]
    pub strip_metadata: bool,

    /// Resize images (e.g. 50%, 1080p, 720p)
    #[arg(
        long = "resize",
        conflicts_with_all = ["target", "target_size", "strip_metadata", "extract_frames", "merge_pdf"]
    )]
    pub resize: Option<String>,

    /// Video CRF value (0: Lossless, 18-28: Balanced, 35+: High compression)
    #[arg(long = "crf", value_parser = clap::value_parser!(u8).range(0..=51))]
    pub crf: Option<u8>,

    /// Video encoding preset (ultrafast … veryslow)
    #[arg(
        long = "preset",
        value_parser = ["ultrafast", "superfast", "veryfast", "faster", "fast", "medium", "slow", "slower", "veryslow", "placebo"]
    )]
    pub preset: Option<String>,

    /// Hardware acceleration mode for video encoding (auto, vaapi, nvenc, none, cpu)
    #[arg(
        long = "hwaccel",
        value_parser = ["auto", "vaapi", "nvenc", "none", "cpu"]
    )]
    pub hwaccel: Option<String>,

    /// Audio bitrate value (e.g. 128k, 192k, 320k)
    #[arg(short = 'b', long = "bitrate", value_parser = parse_bitrate)]
    pub bitrate: Option<String>,

    /// Document / PDF rendering DPI resolution (default: 300)
    #[arg(long = "dpi", value_parser = clap::value_parser!(u32).range(36..=1200))]
    pub dpi: Option<u32>,

    /// Extract video frames to folder as PNG images
    #[arg(
        long = "extract-frames",
        conflicts_with_all = ["target", "resize", "target_size", "strip_metadata", "merge_pdf"]
    )]
    pub extract_frames: bool,

    /// GIF length in seconds (0 = entire video; default: 15)
    #[arg(long = "gif-duration", value_name = "SECONDS", value_parser = parse_gif_duration)]
    pub gif_duration: Option<f64>,

    /// Maximum pages to render from a PDF (0 = unlimited, default: 200)
    #[arg(long = "max-pages", value_name = "N", value_parser = clap::value_parser!(u32).range(0..=10000))]
    pub max_pages: Option<u32>,

    /// GIF frame rate (1-60, default: 15)
    #[arg(long = "gif-fps", value_parser = clap::value_parser!(u32).range(1..=60))]
    pub gif_fps: Option<u32>,

    /// Output directory (default: same directory as input file)
    #[arg(short = 'o', long = "output-dir")]
    pub output_dir: Option<PathBuf>,

    /// Disable desktop system notifications
    #[arg(long = "no-notify")]
    pub no_notify: bool,

    /// Enable verbose diagnostic logging to terminal and ~/.cache/konvert/konvert.log
    #[arg(short = 'v', long = "verbose")]
    pub verbose: bool,

    /// Pause running background conversion process
    #[arg(long = "pause", conflicts_with_all = ["resume", "cancel"])]
    pub pause: bool,

    /// Resume paused background conversion process
    #[arg(long = "resume", conflicts_with_all = ["pause", "cancel"])]
    pub resume: bool,

    /// Cancel running background conversion process
    #[arg(long = "cancel", conflicts_with_all = ["pause", "resume"])]
    pub cancel: bool,

    /// Generate shell auto-completion script (bash, zsh, fish)
    #[arg(long = "completions", value_name = "SHELL")]
    pub completions: Option<clap_complete::Shell>,

    /// Run system diagnostic doctor and check external tool dependencies
    #[arg(long = "doctor")]
    pub doctor: bool,

    /// Check external tool dependencies (ffmpeg, ImageMagick, ghostscript, ...)
    #[arg(long = "check-deps")]
    pub check_deps: bool,

    /// Override interface language (en, tr, de, fr, es, it, ru, zh, ja)
    #[arg(long = "lang", value_name = "CODE")]
    pub lang: Option<String>,

    /// Open Qt6 Settings GUI
    #[arg(short = 's', long = "settings")]
    pub settings: bool,

    /// Set a configuration value (e.g. image.quality=90, general.notification_enabled=false)
    #[arg(long = "config-set", value_name = "KEY=VALUE")]
    pub config_set: Option<String>,

    /// Get a configuration value by key (e.g. image.quality)
    #[arg(long = "config-get", value_name = "KEY")]
    pub config_get: Option<String>,

    /// Save configuration payload from JSON string or stdin ('-') and sync menus
    #[arg(long = "save-config-json", value_name = "JSON")]
    pub save_config_json: Option<String>,

    /// Dump active configuration as JSON (Single Source of Truth for GUI)
    #[arg(long = "dump-config-json")]
    pub dump_config_json: bool,

    /// Re-synchronize right-click context menu according to config
    #[arg(long = "sync-menu")]
    pub sync_menu: bool,

    /// Target directory for generated menu files (for packaging)
    #[arg(long = "menu-dir")]
    pub menu_dir: Option<PathBuf>,

    /// Binary command to use in desktop actions (e.g. /usr/bin/konvert or konvert)
    #[arg(long = "menu-exec")]
    pub menu_exec: Option<String>,

    /// Install Dolphin context menu integration
    #[arg(long = "install-dolphin")]
    pub install_dolphin: bool,

    /// Uninstall Dolphin context menu integration
    #[arg(long = "uninstall-dolphin")]
    pub uninstall_dolphin: bool,

    /// Install Nemo context menu integration
    #[arg(long = "install-nemo")]
    pub install_nemo: bool,

    /// Uninstall Nemo context menu integration
    #[arg(long = "uninstall-nemo")]
    pub uninstall_nemo: bool,

    /// Install integrations for all supported file managers
    #[arg(long = "install-all")]
    pub install_all: bool,

    /// Uninstall integrations from all file managers
    #[arg(long = "uninstall-all")]
    pub uninstall_all: bool,
}

/// `128k`, `192k`, `1.5m` gibi ses bit hızlarını doğrular (`k` veya `m` soneki zorunludur).
fn parse_bitrate(value: &str) -> Result<String, String> {
    let trimmed = value.trim().to_lowercase();
    if trimmed.is_empty() {
        return Err("bit hızı boş olamaz".to_string());
    }
    let valid = if let Some(k_str) = trimmed.strip_suffix('k') {
        matches!(k_str.parse::<f64>(), Ok(n) if n.is_finite() && n > 0.0 && n <= 1024.0)
    } else if let Some(m_str) = trimmed.strip_suffix('m') {
        matches!(m_str.parse::<f64>(), Ok(n) if n.is_finite() && n > 0.0 && n <= 10.0)
    } else {
        false
    };
    if valid {
        Ok(trimmed)
    } else {
        Err(format!(
            "geçersiz bit hızı: {value} ('k' veya 'm' soneki gerekli, örnek: 128k, 192k, 1.5m)"
        ))
    }
}

/// `--jobs` için 1-64 aralığında tam sayı doğrulaması.
fn parse_jobs(value: &str) -> Result<usize, String> {
    match value.trim().parse::<usize>() {
        Ok(n) if (1..=64).contains(&n) => Ok(n),
        _ => Err(format!("geçersiz iş sayısı: {value} (1-64 arası olmalı)")),
    }
}

/// `--gif-duration` için 0-36000 saniye aralığı.
fn parse_gif_duration(value: &str) -> Result<f64, String> {
    match value.trim().parse::<f64>() {
        Ok(seconds) if seconds.is_finite() && (0.0..=36000.0).contains(&seconds) => Ok(seconds),
        _ => Err(format!("geçersiz GIF süresi: {value} (0-36000 saniye)")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_bitrate_requires_suffix_and_bounds() {
        assert_eq!(parse_bitrate("128k").as_deref(), Ok("128k"));
        assert_eq!(parse_bitrate("320K").as_deref(), Ok("320k"));
        assert_eq!(parse_bitrate("1.5m").as_deref(), Ok("1.5m"));

        assert!(parse_bitrate("192").is_err());
        assert!(parse_bitrate("0k").is_err());
        assert!(parse_bitrate("-5k").is_err());
        assert!(parse_bitrate("20m").is_err());
        assert!(parse_bitrate("nan").is_err());
    }

    #[test]
    fn test_cli_conflicting_action_flags_rejected() {
        assert!(
            Cli::try_parse_from(["konvert", "-t", "webp", "--resize", "50%", "a.png"]).is_err()
        );
        assert!(Cli::try_parse_from(["konvert", "-t", "pdf", "--merge-pdf", "a.png"]).is_err());
        assert!(
            Cli::try_parse_from(["konvert", "--strip-metadata", "--extract-frames", "a.mp4"])
                .is_err()
        );
        assert!(Cli::try_parse_from(["konvert", "-t", "webp", "a.png"]).is_ok());
    }

    #[test]
    fn test_cli_pause_resume_cancel_conflicts() {
        assert!(Cli::try_parse_from(["konvert", "--pause"]).is_ok());
        assert!(Cli::try_parse_from(["konvert", "--resume"]).is_ok());
        assert!(Cli::try_parse_from(["konvert", "--cancel"]).is_ok());
        assert!(Cli::try_parse_from(["konvert", "--pause", "--resume"]).is_err());
        assert!(Cli::try_parse_from(["konvert", "--pause", "--cancel"]).is_err());
        assert!(Cli::try_parse_from(["konvert", "--resume", "--cancel"]).is_err());
    }

    #[test]
    fn test_parse_jobs_and_gif_duration() {
        assert_eq!(parse_jobs("1"), Ok(1));
        assert_eq!(parse_jobs("64"), Ok(64));
        assert!(parse_jobs("0").is_err());
        assert!(parse_jobs("65").is_err());
        assert!(parse_jobs("abc").is_err());

        assert_eq!(parse_gif_duration("0"), Ok(0.0));
        assert_eq!(parse_gif_duration("15.5"), Ok(15.5));
        assert!(parse_gif_duration("-1").is_err());
        assert!(parse_gif_duration("NaN").is_err());
        assert!(parse_gif_duration("inf").is_err());
        assert!(parse_gif_duration("40000").is_err());
    }
}
