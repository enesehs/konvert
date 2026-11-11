pub mod audio;
pub mod document;
pub mod image;
pub mod video;

use anyhow::{bail, Context, Result};
use rayon::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResizeSpec {
    Percent(u32),
    Fit1080p,
    Fit720p,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TargetAction {
    Convert(String),
    Compress,
    TargetSize(u64),
    WebOpt,
    StripMetadata,
    Resize(ResizeSpec),
    ExtractAudio,
    Frames,
    Gif,
    AudioToVideo,
}

impl TargetAction {
    pub fn parse(target: &str) -> Result<Self> {
        let t = target.trim().to_lowercase();
        match t.as_str() {
            "compress" => Ok(TargetAction::Compress),
            "web_opt" | "web-opt" => Ok(TargetAction::WebOpt),
            "strip-metadata" | "strip_metadata" => Ok(TargetAction::StripMetadata),
            "discord_8mb" | "target_8mb" => Ok(TargetAction::TargetSize(8 * 1024 * 1024)),
            "email_25mb" | "target_25mb" => Ok(TargetAction::TargetSize(25 * 1024 * 1024)),
            "audio-to-video" | "audio_to_video" => Ok(TargetAction::AudioToVideo),
            "extract-audio" | "extract_audio" => Ok(TargetAction::ExtractAudio),
            "frames" | "png-frames" => Ok(TargetAction::Frames),
            "gif" => Ok(TargetAction::Gif),
            _ if t.starts_with("resize_") => {
                let spec = t.trim_start_matches("resize_");
                match spec {
                    "25" | "25%" => Ok(TargetAction::Resize(ResizeSpec::Percent(25))),
                    "50" | "50%" => Ok(TargetAction::Resize(ResizeSpec::Percent(50))),
                    "75" | "75%" => Ok(TargetAction::Resize(ResizeSpec::Percent(75))),
                    "1080p" | "1080" => Ok(TargetAction::Resize(ResizeSpec::Fit1080p)),
                    "720p" | "720" => Ok(TargetAction::Resize(ResizeSpec::Fit720p)),
                    _ => bail!("Geçersiz boyutlandırma parametresi: {}", spec),
                }
            }
            _ => Ok(TargetAction::Convert(t)),
        }
    }
}

pub fn get_output_dir(input_path: &Path, custom_output_dir: Option<&Path>) -> PathBuf {
    if let Some(dir) = custom_output_dir {
        return dir.to_path_buf();
    }
    match input_path.parent() {
        Some(p) if !p.as_os_str().is_empty() => p.to_path_buf(),
        _ => PathBuf::from("."),
    }
}

pub fn ensure_parent_dir(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent)
            .with_context(|| format!("Çıktı klasörü oluşturulamadı: {}", parent.display()))?;
    }
    Ok(())
}

pub fn determine_output_path(
    input_path: &Path,
    target_ext: &str,
    custom_output_dir: Option<&Path>,
) -> PathBuf {
    let file_stem = input_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("converted");

    let target_dir = get_output_dir(input_path, custom_output_dir);
    let candidate = target_dir.join(format!("{}.{}", file_stem, target_ext));
    if candidate == input_path {
        target_dir.join(format!("{}_konvert.{}", file_stem, target_ext))
    } else {
        candidate
    }
}

pub fn determine_suffixed_output_path(
    input_path: &Path,
    suffix: &str,
    target_ext: &str,
    custom_output_dir: Option<&Path>,
) -> PathBuf {
    let file_stem = input_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("file");

    let target_dir = get_output_dir(input_path, custom_output_dir);
    let candidate_name = if file_stem.ends_with(&format!("_{}", suffix)) {
        format!("{}_new.{}", file_stem, target_ext)
    } else {
        format!("{}_{}.{}", file_stem, suffix, target_ext)
    };
    let candidate = target_dir.join(candidate_name);
    if candidate == input_path {
        target_dir.join(format!("{}_{}_konvert.{}", file_stem, suffix, target_ext))
    } else {
        candidate
    }
}

pub fn run_imagemagick<I, S>(args: I) -> Result<()>
where
    I: IntoIterator<Item = S> + Clone,
    S: AsRef<std::ffi::OsStr>,
{
    let status = Command::new("magick")
        .args(args.clone())
        .status()
        .or_else(|_| Command::new("convert").args(args).status())
        .with_context(|| "ImageMagick (magick veya convert) çalıştırılamadı")?;

    if !status.success() {
        bail!("ImageMagick komutu başarısız oldu (çıkış kodu 0 değil)");
    }
    Ok(())
}

pub struct BatchResult {
    pub succeeded: Vec<PathBuf>,
    pub failed: Vec<(PathBuf, String)>,
}

#[derive(Clone)]
pub struct ConvertOptions {
    pub target: String,
    pub quality: Option<u8>,
    pub lossless: bool,
    pub crf: Option<u8>,
    pub preset: Option<String>,
    pub bitrate: Option<String>,
    pub dpi: Option<u32>,
    pub custom_output_dir: Option<PathBuf>,
}

pub fn convert_single(input_path: &Path, opts: &ConvertOptions) -> Result<PathBuf> {
    if !input_path.exists() {
        bail!("Girdi dosyası mevcut değil: {}", input_path.display());
    }

    let action = TargetAction::parse(&opts.target)?;
    let ext = input_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    let out_dir = opts.custom_output_dir.as_deref();

    match action {
        TargetAction::StripMetadata => {
            if is_image_ext(&ext) {
                image::strip_metadata(input_path, out_dir)
            } else if is_video_ext(&ext) {
                video::strip_video_metadata(input_path, out_dir)
            } else if is_audio_ext(&ext) {
                audio::strip_audio_metadata(input_path, out_dir)
            } else {
                bail!("Metadata temizleme sadece görsel, video ve ses dosyaları için desteklenir: .{}", ext)
            }
        }
        TargetAction::Resize(spec) => {
            if is_image_ext(&ext) {
                image::resize_image(input_path, &spec, out_dir)
            } else {
                bail!("Yeniden boyutlandırma sadece görseller için geçerlidir (.{})", ext)
            }
        }
        TargetAction::Compress => {
            if is_video_ext(&ext) {
                video::compress_video(input_path, out_dir)
            } else if is_image_ext(&ext) {
                image::compress_image(input_path, opts.quality, out_dir)
            } else if ext == "pdf" {
                document::compress_pdf(input_path, out_dir)
            } else {
                bail!("Bu format için sıkıştırma desteklenmiyor: .{}", ext)
            }
        }
        TargetAction::TargetSize(target_bytes) => {
            if is_video_ext(&ext) {
                video::compress_video_to_size(input_path, target_bytes, out_dir)
            } else if is_image_ext(&ext) {
                image::compress_image_to_size(input_path, target_bytes, out_dir)
            } else if ext == "pdf" {
                document::compress_pdf_to_size(input_path, target_bytes, out_dir)
            } else {
                bail!("Bu format için hedef boyut sıkıştırması desteklenmiyor: .{}", ext)
            }
        }
        TargetAction::WebOpt => {
            if is_video_ext(&ext) {
                video::web_optimize_video(input_path, out_dir)
            } else if is_image_ext(&ext) {
                image::web_optimize_image(input_path, out_dir)
            } else {
                bail!("Web optimizasyonu sadece görsel ve videolar için geçerlidir (.{})", ext)
            }
        }
        TargetAction::ExtractAudio => {
            if is_video_ext(&ext) {
                video::extract_audio(input_path, out_dir)
            } else {
                bail!("Videodan ses ayıklama sadece video dosyaları için geçerlidir (.{})", ext)
            }
        }
        TargetAction::Frames => {
            if is_video_ext(&ext) {
                video::extract_frames(input_path, out_dir)
            } else {
                bail!("Kare çıkarma sadece video dosyaları için geçerlidir (.{})", ext)
            }
        }
        TargetAction::Gif => {
            if is_video_ext(&ext) {
                video::convert_to_gif(input_path, out_dir)
            } else {
                bail!("GIF dönüşümü sadece video dosyaları için geçerlidir (.{})", ext)
            }
        }
        TargetAction::AudioToVideo => {
            if is_audio_ext(&ext) {
                audio::convert_audio_to_video(input_path, out_dir)
            } else {
                bail!("Ses dalgası videosu oluşturmak için bir ses dosyası gereklidir (.{})", ext)
            }
        }
        TargetAction::Convert(target_fmt) => {
            if ext == "pdf" && (target_fmt == "png" || target_fmt == "jpg" || target_fmt == "jpeg") {
                document::convert_pdf_to_images(input_path, &target_fmt, opts.dpi.unwrap_or(300), out_dir)
            } else if ext == "udf" {
                document::convert_udf(input_path, &target_fmt, out_dir)
            } else if ext == "docx" {
                document::convert_docx(input_path, &target_fmt, out_dir)
            } else if is_image_ext(&ext) && image::is_supported_image_target(&target_fmt) {
                image::convert_image(input_path, &target_fmt, opts.quality, opts.lossless, out_dir)
            } else if is_video_ext(&ext) && video::is_supported_video_target(&target_fmt) {
                video::convert_video(input_path, &target_fmt, opts.crf, opts.preset.as_deref(), opts.lossless, out_dir)
            } else if is_audio_ext(&ext) && audio::is_supported_audio_target(&target_fmt) {
                audio::convert_audio(input_path, &target_fmt, opts.bitrate.as_deref(), out_dir)
            } else {
                bail!("Desteklenmeyen dönüştürme: .{} -> .{}", ext, target_fmt)
            }
        }
    }
}

pub fn merge_images_to_pdf(images: &[PathBuf], custom_output_dir: Option<&Path>) -> Result<PathBuf> {
    image::merge_images_to_pdf(images, custom_output_dir)
}

pub fn convert_batch_with_progress<F>(
    files: &[PathBuf],
    opts: &ConvertOptions,
    progress_callback: F,
) -> BatchResult
where
    F: FnMut(usize, usize, &Path, i32) + Send,
{
    let total = files.len();
    if total == 0 {
        return BatchResult {
            succeeded: Vec::new(),
            failed: Vec::new(),
        };
    }

    let progress_state = Mutex::new((0usize, progress_callback));

    let results: Vec<Result<PathBuf, (PathBuf, String)>> = files
        .par_iter()
        .map(|file| {
            let res = convert_single(file, opts);

            if let Ok(mut lock) = progress_state.lock() {
                lock.0 += 1;
                let curr = lock.0;
                let percent = ((curr as f32 / total as f32) * 100.0) as i32;
                (lock.1)(curr, total, file, percent);
            }

            match res {
                Ok(p) => Ok(p),
                Err(e) => Err((file.clone(), format!("{:#}", e))),
            }
        })
        .collect();

    let mut succeeded = Vec::with_capacity(total);
    let mut failed = Vec::new();

    for r in results {
        match r {
            Ok(p) => succeeded.push(p),
            Err(f) => failed.push(f),
        }
    }

    BatchResult { succeeded, failed }
}

pub fn is_image_ext(ext: &str) -> bool {
    matches!(
        ext.to_lowercase().as_str(),
        "png" | "jpg" | "jpeg" | "webp" | "avif" | "jxl" | "bmp" | "ico" | "tiff" | "tif" | "qoi" | "svg"
    )
}

pub fn is_video_ext(ext: &str) -> bool {
    matches!(
        ext.to_lowercase().as_str(),
        "mp4" | "mkv" | "webm" | "avi" | "mov" | "flv" | "wmv" | "m4v"
    )
}

pub fn is_audio_ext(ext: &str) -> bool {
    matches!(
        ext.to_lowercase().as_str(),
        "mp3" | "wav" | "flac" | "ogg" | "m4a" | "aac" | "opus" | "wma" | "aiff"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_image_ext() {
        assert!(is_image_ext("png"));
        assert!(is_image_ext("PNG"));
        assert!(is_image_ext("jpg"));
        assert!(is_image_ext("JPG"));
        assert!(is_image_ext("webp"));
        assert!(is_image_ext("avif"));
        assert!(is_image_ext("jxl"));
        assert!(!is_image_ext("mp4"));
        assert!(!is_image_ext("pdf"));
    }

    #[test]
    fn test_is_video_ext() {
        assert!(is_video_ext("mp4"));
        assert!(is_video_ext("MP4"));
        assert!(is_video_ext("mkv"));
        assert!(is_video_ext("webm"));
        assert!(!is_video_ext("png"));
        assert!(!is_video_ext("mp3"));
    }

    #[test]
    fn test_is_audio_ext() {
        assert!(is_audio_ext("mp3"));
        assert!(is_audio_ext("MP3"));
        assert!(is_audio_ext("flac"));
        assert!(is_audio_ext("wav"));
        assert!(!is_audio_ext("mp4"));
        assert!(!is_audio_ext("png"));
    }

    #[test]
    fn test_get_output_dir_empty_parent() {
        let path = Path::new("test.docx");
        let dir = get_output_dir(path, None);
        assert_eq!(dir, PathBuf::from("."));
        assert!(!dir.as_os_str().is_empty());
    }

    #[test]
    fn test_determine_suffixed_output_path() {
        let path = Path::new("my_photo.jpg");
        let out = determine_suffixed_output_path(path, "clean", "jpg", None);
        assert_eq!(out, PathBuf::from("./my_photo_clean.jpg"));

        let already_clean = Path::new("my_photo_clean.jpg");
        let out2 = determine_suffixed_output_path(already_clean, "clean", "jpg", None);
        assert_eq!(out2, PathBuf::from("./my_photo_clean_new.jpg"));
    }

    #[test]
    fn test_target_action_parse() {
        assert_eq!(TargetAction::parse("compress").unwrap(), TargetAction::Compress);
        assert_eq!(TargetAction::parse("web_opt").unwrap(), TargetAction::WebOpt);
        assert_eq!(TargetAction::parse("web-opt").unwrap(), TargetAction::WebOpt);
        assert_eq!(TargetAction::parse("discord_8mb").unwrap(), TargetAction::TargetSize(8 * 1024 * 1024));
        assert_eq!(TargetAction::parse("email_25mb").unwrap(), TargetAction::TargetSize(25 * 1024 * 1024));
        assert_eq!(TargetAction::parse("strip-metadata").unwrap(), TargetAction::StripMetadata);
        assert_eq!(TargetAction::parse("resize_50%").unwrap(), TargetAction::Resize(ResizeSpec::Percent(50)));
        assert_eq!(TargetAction::parse("resize_1080p").unwrap(), TargetAction::Resize(ResizeSpec::Fit1080p));
        assert!(TargetAction::parse("resize_invalid").is_err());
        assert_eq!(TargetAction::parse("webp").unwrap(), TargetAction::Convert("webp".to_string()));
    }

    #[test]
    fn test_batch_empty() {
        let opts = ConvertOptions {
            target: "webp".to_string(),
            quality: None,
            lossless: false,
            crf: None,
            preset: None,
            bitrate: None,
            dpi: None,
            custom_output_dir: None,
        };
        let result = convert_batch_with_progress(&[], &opts, |_, _, _, _| {});
        assert!(result.succeeded.is_empty());
        assert!(result.failed.is_empty());
    }
}
