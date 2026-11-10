pub mod audio;
pub mod document;
pub mod image;
pub mod video;

use anyhow::{bail, Result};
use std::path::{Path, PathBuf};

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
    mut progress_callback: F,
) -> BatchResult
where
    F: FnMut(usize, usize, &Path, i32),
{
    let total = files.len();
    let mut succeeded = Vec::new();
    let mut failed = Vec::new();

    for (index, file) in files.iter().enumerate() {
        let percent = ((index as f32 / total as f32) * 100.0) as i32;
        progress_callback(index + 1, total, file, percent);

        match convert_single(file, opts) {
            Ok(p) => succeeded.push(p),
            Err(e) => failed.push((file.clone(), e.to_string())),
        }
    }

    let final_percent = 100;
    if let Some(last_file) = files.last() {
        progress_callback(total, total, last_file, final_percent);
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
