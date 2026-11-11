pub mod audio;
pub mod document;
pub mod image;
pub mod video;

use anyhow::{bail, Result};
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

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

    let target = opts.target.to_lowercase();
    let ext = input_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let out_dir = opts.custom_output_dir.as_deref();

    // 0. Akıllı Sıkıştırma (En küçük boyut, maksimum kalite)
    if target == "compress" {
        if is_image_ext(&ext) {
            return image::compress_image(input_path, out_dir);
        } else if is_video_ext(&ext) {
            return video::compress_video(input_path, out_dir);
        } else if ext == "pdf" {
            return document::compress_pdf(input_path, out_dir);
        } else {
            bail!("Bu dosya formatı için akıllı sıkıştırma desteklenmiyor: .{}", ext);
        }
    }

    // 0.1. Akıllı Hedef Boyut Sıkıştırması (Discord 8MB, E-Posta 25MB, Web Optimize)
    if target == "discord_8mb"
        || target == "target_8mb"
        || target == "email_25mb"
        || target == "target_25mb"
        || target == "web_opt"
        || target == "web-opt"
    {
        let target_bytes = if target.contains("8mb") {
            8 * 1024 * 1024
        } else if target.contains("25mb") {
            25 * 1024 * 1024
        } else {
            0
        };

        if is_video_ext(&ext) {
            return video::compress_video_to_size(input_path, target_bytes, out_dir);
        } else if is_image_ext(&ext) {
            return image::compress_image_preset(input_path, &target, target_bytes, out_dir);
        } else if ext == "pdf" {
            return document::compress_pdf(input_path, out_dir);
        } else {
            bail!("Bu dosya formatı için akıllı boyut sıkıştırması desteklenmiyor: .{}", ext);
        }
    }

    // 1. Metadata Temizleyici
    if target == "strip-metadata" || target == "strip_metadata" {
        return image::strip_metadata(input_path, out_dir);
    }

    // 2. Yeniden Boyutlandırıcı
    if target.starts_with("resize_") {
        let param = target.trim_start_matches("resize_");
        return image::resize_image(input_path, param, out_dir);
    }

    // 3. Belge dönüştürücü (UDF, DOCX, PDF -> görseller)
    if ext == "udf" || ext == "docx" || (ext == "pdf" && (target == "png" || target == "jpg")) {
        return document::convert_document(input_path, &target, opts.dpi, out_dir);
    }

    // 4. Video kare çıkarma veya GIF
    if target == "frames" || target == "png-frames" || (is_video_ext(&ext) && target == "gif") {
        return video::convert_video(
            input_path,
            &target,
            opts.crf,
            opts.preset.as_deref(),
            opts.lossless,
            out_dir,
        );
    }

    // 5. Ses -> Video (waveform)
    if target == "audio-to-video" || target == "audio_to_video" {
        return audio::convert_audio(
            input_path,
            &target,
            opts.bitrate.as_deref(),
            out_dir,
        );
    }

    // 6. Görsel dönüştürme
    if image::is_supported_image_target(&target) && is_image_ext(&ext) {
        return image::convert_image(
            input_path,
            &target,
            opts.quality,
            opts.lossless,
            out_dir,
        );
    }

    // 7. Video dönüştürme (Videodan ses ayıklama dahil)
    if video::is_supported_video_target(&target) && is_video_ext(&ext) {
        return video::convert_video(
            input_path,
            &target,
            opts.crf,
            opts.preset.as_deref(),
            opts.lossless,
            out_dir,
        );
    }

    // 8. Ses dönüştürme
    if audio::is_supported_audio_target(&target) {
        return audio::convert_audio(
            input_path,
            &target,
            opts.bitrate.as_deref(),
            out_dir,
        );
    }

    // 9. Fallback
    if image::is_supported_image_target(&target) {
        return image::convert_image(
            input_path,
            &target,
            opts.quality,
            opts.lossless,
            out_dir,
        );
    }

    if video::is_supported_video_target(&target) {
        return video::convert_video(
            input_path,
            &target,
            opts.crf,
            opts.preset.as_deref(),
            opts.lossless,
            out_dir,
        );
    }

    bail!("Desteklenmeyen dönüştürme: .{} -> .{}", ext, target)
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

    let callback = Mutex::new(progress_callback);
    let completed = AtomicUsize::new(0);

    let results: Vec<Result<PathBuf, (PathBuf, String)>> = files
        .par_iter()
        .map(|file| {
            let res = convert_single(file, opts);
            let curr = completed.fetch_add(1, Ordering::SeqCst) + 1;
            let percent = ((curr as f32 / total as f32) * 100.0) as i32;

            if let Ok(mut cb) = callback.lock() {
                cb(curr, total, file, percent);
            }

            match res {
                Ok(p) => Ok(p),
                Err(e) => Err((file.clone(), e.to_string())),
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

fn is_image_ext(ext: &str) -> bool {
    matches!(
        ext,
        "png" | "jpg" | "jpeg" | "webp" | "avif" | "jxl" | "bmp" | "ico" | "tiff" | "tif" | "qoi" | "svg"
    )
}

fn is_video_ext(ext: &str) -> bool {
    matches!(
        ext,
        "mp4" | "mkv" | "webm" | "avi" | "mov" | "flv" | "wmv" | "m4v"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_image_ext() {
        assert!(is_image_ext("png"));
        assert!(is_image_ext("jpg"));
        assert!(is_image_ext("webp"));
        assert!(is_image_ext("avif"));
        assert!(is_image_ext("jxl"));
        assert!(!is_image_ext("mp4"));
        assert!(!is_image_ext("pdf"));
    }

    #[test]
    fn test_is_video_ext() {
        assert!(is_video_ext("mp4"));
        assert!(is_video_ext("mkv"));
        assert!(is_video_ext("webm"));
        assert!(!is_video_ext("png"));
        assert!(!is_video_ext("mp3"));
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
