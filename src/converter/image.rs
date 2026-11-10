use anyhow::{bail, Context, Result};
use image::{codecs::jpeg::JpegEncoder, imageops, DynamicImage, GenericImageView, ImageFormat};
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn is_supported_image_target(target: &str) -> bool {
    let t = target.to_lowercase();
    t == "jxl"
        || t == "strip-metadata"
        || t == "strip_metadata"
        || t == "web_opt"
        || t == "web-opt"
        || t == "discord_8mb"
        || t == "target_8mb"
        || t == "email_25mb"
        || t == "target_25mb"
        || t.starts_with("resize_")
        || parse_image_format(&t).is_some()
}

pub fn parse_image_format(target: &str) -> Option<ImageFormat> {
    match target.to_lowercase().as_str() {
        "png" => Some(ImageFormat::Png),
        "jpg" | "jpeg" => Some(ImageFormat::Jpeg),
        "webp" => Some(ImageFormat::WebP),
        "bmp" => Some(ImageFormat::Bmp),
        "ico" => Some(ImageFormat::Ico),
        "tiff" | "tif" => Some(ImageFormat::Tiff),
        "qoi" => Some(ImageFormat::Qoi),
        "avif" => Some(ImageFormat::Avif),
        _ => None,
    }
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

    let target_dir = match custom_output_dir {
        Some(dir) => dir.to_path_buf(),
        None => input_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf(),
    };

    let candidate = target_dir.join(format!("{}.{}", file_stem, target_ext));
    if candidate == input_path {
        target_dir.join(format!("{}_konvert.{}", file_stem, target_ext))
    } else {
        candidate
    }
}

pub fn convert_image(
    input_path: &Path,
    target_ext: &str,
    quality: Option<u8>,
    lossless: bool,
    custom_output_dir: Option<&Path>,
) -> Result<PathBuf> {
    let target_lower = target_ext.to_lowercase();

    // 1. Metadata / EXIF Temizleme
    if target_lower == "strip-metadata" || target_lower == "strip_metadata" {
        return strip_metadata(input_path, custom_output_dir);
    }

    // 2. Yeniden Boyutlandırma
    if target_lower.starts_with("resize_") {
        let param = target_lower.trim_start_matches("resize_");
        return resize_image(input_path, param, custom_output_dir);
    }

    let output_path = determine_output_path(input_path, &target_lower, custom_output_dir);

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // JPEG XL (.jxl)
    if target_lower == "jxl" {
        return convert_to_jxl(input_path, &output_path, quality, lossless);
    }

    // JXL okuma desteği
    let is_input_jxl = input_path
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.eq_ignore_ascii_case("jxl"))
        .unwrap_or(false);

    let img: DynamicImage = if is_input_jxl {
        let temp_png = std::env::temp_dir().join(format!("konvert_jxl_{}.png", std::process::id()));
        let status = Command::new("djxl")
            .arg(input_path)
            .arg(&temp_png)
            .status()
            .with_context(|| "djxl komutu çalıştırılamadı")?;

        if !status.success() {
            bail!("JXL görseli çözülemedi: {}", input_path.display());
        }

        let loaded = image::open(&temp_png)
            .with_context(|| "djxl çıktısı okunamadı")?;
        let _ = fs::remove_file(temp_png);
        loaded
    } else {
        image::open(input_path)
            .with_context(|| format!("Görsel açılamadı: {}", input_path.display()))?
    };

    let format = parse_image_format(&target_lower)
        .with_context(|| format!("Desteklenmeyen görsel formatı: {}", target_lower))?;

    match format {
        ImageFormat::Jpeg => {
            let q = quality.unwrap_or(85).clamp(1, 100);
            let mut file = File::create(&output_path)
                .with_context(|| format!("Dosya oluşturulamadı: {}", output_path.display()))?;
            let mut encoder = JpegEncoder::new_with_quality(&mut file, q);
            encoder.encode_image(&img)
                .with_context(|| "JPEG kodlama hatası")?;
        }
        ImageFormat::WebP => {
            img.save_with_format(&output_path, ImageFormat::WebP)
                .with_context(|| "WebP kaydetme hatası")?;
        }
        _ => {
            img.save_with_format(&output_path, format)
                .with_context(|| format!("Görsel kaydedilemedi: {}", output_path.display()))?;
        }
    }

    Ok(output_path)
}

fn convert_to_jxl(
    input_path: &Path,
    output_path: &Path,
    quality: Option<u8>,
    lossless: bool,
) -> Result<PathBuf> {
    let mut cmd = Command::new("cjxl");
    cmd.arg(input_path).arg(output_path);

    if lossless {
        cmd.arg("-d").arg("0");
    } else {
        let q = quality.unwrap_or(85).clamp(1, 100);
        cmd.arg("-q").arg(q.to_string());
    }

    let status = cmd.status().with_context(|| "cjxl komutu çalıştırılamadı")?;
    if !status.success() {
        bail!("cjxl ile JPEG XL dönüştürme başarısız oldu");
    }

    Ok(output_path.to_path_buf())
}

/// Akıllı görsel sıkıştırma (en küçük boyut, maksimum görsel kalite)
pub fn compress_image(input_path: &Path, custom_output_dir: Option<&Path>) -> Result<PathBuf> {
    let file_stem = input_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("image");

    let target_dir = match custom_output_dir {
        Some(dir) => dir.to_path_buf(),
        None => input_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf(),
    };

    let output_path = target_dir.join(format!("{}_compressed.webp", file_stem));
    let img = image::open(input_path)
        .with_context(|| format!("Görsel açılamadı: {}", input_path.display()))?;

    let mut file = File::create(&output_path)
        .with_context(|| format!("Çıktı dosyası oluşturulamadı: {}", output_path.display()))?;

    img.write_to(&mut file, ImageFormat::WebP)
        .with_context(|| "Görsel WebP olarak sıkıştırılamadı")?;

    Ok(output_path)
}

/// EXIF, GPS ve gizlilik bilgilerini kaldırarak temiz bir kopya üretir
pub fn strip_metadata(input_path: &Path, custom_output_dir: Option<&Path>) -> Result<PathBuf> {
    let ext = input_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("png");

    let file_stem = input_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("image");

    let target_dir = match custom_output_dir {
        Some(dir) => dir.to_path_buf(),
        None => input_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf(),
    };

    let output_path = target_dir.join(format!("{}_clean.{}", file_stem, ext));

    // ImageMagick ile tüm metadata'yı strip et
    let status = Command::new("magick")
        .arg(input_path)
        .arg("-strip")
        .arg(&output_path)
        .status()
        .or_else(|_| {
            Command::new("convert")
                .arg(input_path)
                .arg("-strip")
                .arg(&output_path)
                .status()
        })
        .with_context(|| "Metadata temizleyici (magick/convert) çalıştırılamadı")?;

    if !status.success() {
        bail!("Metadata temizleme işlemi başarısız oldu");
    }

    Ok(output_path)
}

/// Görseli yeniden boyutlandırır (%50, 1080p, 720p)
pub fn resize_image(
    input_path: &Path,
    param: &str,
    custom_output_dir: Option<&Path>,
) -> Result<PathBuf> {
    let ext = input_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("png");

    let file_stem = input_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("image");

    let target_dir = match custom_output_dir {
        Some(dir) => dir.to_path_buf(),
        None => input_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf(),
    };

    let img = image::open(input_path)
        .with_context(|| format!("Görsel açılamadı: {}", input_path.display()))?;

    let (w, h) = img.dimensions();

    let (new_w, new_h) = match param {
        "50" | "50%" => (w / 2, h / 2),
        "75" | "75%" => ((w * 3) / 4, (h * 3) / 4),
        "25" | "25%" => (w / 4, h / 4),
        "1080p" | "1080" => {
            if w > h {
                let ratio = 1920.0 / w as f32;
                (1920, (h as f32 * ratio) as u32)
            } else {
                let ratio = 1080.0 / h as f32;
                ((w as f32 * ratio) as u32, 1080)
            }
        }
        "720p" | "720" => {
            if w > h {
                let ratio = 1280.0 / w as f32;
                (1280, (h as f32 * ratio) as u32)
            } else {
                let ratio = 720.0 / h as f32;
                ((w as f32 * ratio) as u32, 720)
            }
        }
        _ => (w / 2, h / 2),
    };

    let resized = img.resize(new_w, new_h, imageops::FilterType::Lanczos3);

    let output_path = target_dir.join(format!("{}_{}.{}", file_stem, param, ext));
    resized.save(&output_path)
        .with_context(|| "Boyutlandırılmış görsel kaydedilemedi")?;

    Ok(output_path)
}

/// Birden fazla görseli tek bir PDF dosyasında birleştirir
pub fn merge_images_to_pdf(
    images: &[PathBuf],
    custom_output_dir: Option<&Path>,
) -> Result<PathBuf> {
    if images.is_empty() {
        bail!("Birleştirilecek görsel bulunamadı");
    }

    let first_stem = images[0]
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("gorseller");

    let target_dir = match custom_output_dir {
        Some(dir) => dir.to_path_buf(),
        None => images[0]
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf(),
    };

    let output_path = target_dir.join(format!("{}_birlestirildi.pdf", first_stem));

    let mut cmd = Command::new("magick");
    for img in images {
        cmd.arg(img);
    }
    cmd.arg("-auto-orient");
    cmd.arg(&output_path);

    let status = cmd.status().or_else(|_| {
        let mut fallback = Command::new("convert");
        for img in images {
            fallback.arg(img);
        }
        fallback.arg("-auto-orient");
        fallback.arg(&output_path);
        fallback.status()
    }).with_context(|| "Görselleri PDF yapacak araç (magick/convert) bulunamadı")?;

    if !status.success() {
        bail!("Görseller PDF olarak birleştirilemedi");
    }

    Ok(output_path)
}

/// Akıllı görsel sıkıştırma önayarı (Web Optimize, Discord 8MB, E-Posta 25MB)
pub fn compress_image_preset(
    input_path: &Path,
    preset: &str,
    target_bytes: u64,
    custom_output_dir: Option<&Path>,
) -> Result<PathBuf> {
    let file_stem = input_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("image");

    let target_dir = match custom_output_dir {
        Some(dir) => dir.to_path_buf(),
        None => input_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf(),
    };

    if preset == "web_opt" || preset == "web-opt" {
        let output_path = target_dir.join(format!("{}_web.webp", file_stem));
        let status = Command::new("magick")
            .arg(input_path)
            .arg("-auto-orient")
            .arg("-strip")
            .arg("-quality")
            .arg("82")
            .arg(&output_path)
            .status()
            .or_else(|_| {
                Command::new("convert")
                    .arg(input_path)
                    .arg("-auto-orient")
                    .arg("-strip")
                    .arg("-quality")
                    .arg("82")
                    .arg(&output_path)
                    .status()
            })
            .with_context(|| "Görsel optimize edici (magick/convert) çalıştırılamadı")?;

        if !status.success() {
            bail!("Web optimizasyonu başarısız oldu");
        }
        return Ok(output_path);
    }

    let current_bytes = fs::metadata(input_path).map(|m| m.len()).unwrap_or(0);
    let ext = input_path.extension().and_then(|e| e.to_str()).unwrap_or("jpg").to_lowercase();
    let target_ext = if ext == "png" { "png" } else { "jpg" };
    let output_path = target_dir.join(format!("{}_target.{}", file_stem, target_ext));

    if target_bytes > 0 && current_bytes <= target_bytes {
        let status = Command::new("magick")
            .arg(input_path)
            .arg("-auto-orient")
            .arg("-strip")
            .arg(&output_path)
            .status()
            .or_else(|_| {
                Command::new("convert")
                    .arg(input_path)
                    .arg("-auto-orient")
                    .arg("-strip")
                    .arg(&output_path)
                    .status()
            })
            .with_context(|| "Görsel işleyici çalıştırılamadı")?;

        if !status.success() {
            bail!("Görsel optimize edilemedi");
        }
        return Ok(output_path);
    }

    let status = Command::new("magick")
        .arg(input_path)
        .arg("-auto-orient")
        .arg("-strip")
        .arg("-quality")
        .arg("80")
        .arg(&output_path)
        .status()
        .or_else(|_| {
            Command::new("convert")
                .arg(input_path)
                .arg("-auto-orient")
                .arg("-strip")
                .arg("-quality")
                .arg("80")
                .arg(&output_path)
                .status()
        })
        .with_context(|| "Hedef boyut sıkıştırması başarısız")?;

    if !status.success() {
        bail!("Görsel hedef boyut sıkıştırması başarısız oldu");
    }

    Ok(output_path)
}
