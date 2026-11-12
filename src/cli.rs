use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "konvert",
    author = "enesehs <enesehs@protonmail.com>",
    version,
    about = "Fast, lightweight, native right-click file converter and optimizer for Linux and KDE Plasma"
)]
pub struct Cli {
    /// Target format (e.g. webp, png, jpg, avif, jxl, mp4, gif, mp3, txt, pdf)
    #[arg(short = 't', long = "target")]
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
    #[arg(long = "merge-pdf")]
    pub merge_pdf: bool,

    /// Smart compression to target file size (e.g. 8MB, 25MB)
    #[arg(long = "target-size")]
    pub target_size: Option<String>,

    /// Strip EXIF, GPS, and privacy metadata
    #[arg(long = "strip-metadata")]
    pub strip_metadata: bool,

    /// Resize images (e.g. 50%, 1080p, 720p)
    #[arg(long = "resize")]
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

    /// Audio bitrate value (e.g. 128k, 192k, 320k)
    #[arg(short = 'b', long = "bitrate", value_parser = parse_bitrate)]
    pub bitrate: Option<String>,

    /// Document / PDF rendering DPI resolution (default: 300)
    #[arg(long = "dpi", value_parser = clap::value_parser!(u32).range(36..=1200))]
    pub dpi: Option<u32>,

    /// Extract video frames to folder as PNG images
    #[arg(long = "extract-frames")]
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

    /// Cancel running background conversion process
    #[arg(long = "cancel")]
    pub cancel: bool,

    /// Generate shell auto-completion script (bash, zsh, fish)
    #[arg(long = "completions", value_name = "SHELL")]
    pub completions: Option<clap_complete::Shell>,

    /// Check external tool dependencies (ffmpeg, ImageMagick, ghostscript, ...)
    #[arg(long = "check-deps")]
    pub check_deps: bool,

    /// Open Qt6 Settings GUI
    #[arg(short = 's', long = "settings")]
    pub settings: bool,

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

    /// Install integrations for all supported file managers
    #[arg(long = "install-all")]
    pub install_all: bool,

    /// Uninstall integrations from all file managers
    #[arg(long = "uninstall-all")]
    pub uninstall_all: bool,
}

/// `128k`, `192k`, `1.5m` gibi ses bit hızlarını doğrular.
fn parse_bitrate(value: &str) -> Result<String, String> {
    let trimmed = value.trim().to_lowercase();
    if trimmed.is_empty() {
        return Err("bit hızı boş olamaz".to_string());
    }
    let number_part = trimmed
        .strip_suffix('k')
        .or_else(|| trimmed.strip_suffix('m'))
        .unwrap_or(&trimmed);
    match number_part.parse::<f64>() {
        Ok(n) if n.is_finite() && n > 0.0 && n <= 1024.0 => Ok(trimmed),
        _ => Err(format!(
            "geçersiz bit hızı: {value} (örnek: 128k, 192k, 320k)"
        )),
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
