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
    #[arg(short = 'q', long = "quality")]
    pub quality: Option<u8>,

    /// Lossless compression / conversion mode
    #[arg(short = 'l', long = "lossless")]
    pub lossless: bool,

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
    #[arg(long = "crf")]
    pub crf: Option<u8>,

    /// Video encoding preset (ultrafast, medium, veryslow)
    #[arg(long = "preset")]
    pub preset: Option<String>,

    /// Audio bitrate value (e.g. 128k, 192k, 320k)
    #[arg(short = 'b', long = "bitrate")]
    pub bitrate: Option<String>,

    /// Document / PDF rendering DPI resolution (default: 300)
    #[arg(long = "dpi")]
    pub dpi: Option<u32>,

    /// Extract video frames to folder as PNG images
    #[arg(long = "extract-frames")]
    pub extract_frames: bool,

    /// Output directory (default: same directory as input file)
    #[arg(short = 'o', long = "output-dir")]
    pub output_dir: Option<PathBuf>,

    /// Disable desktop system notifications
    #[arg(long = "no-notify")]
    pub no_notify: bool,

    /// Cancel running background conversion process
    #[arg(long = "cancel")]
    pub cancel: bool,

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
