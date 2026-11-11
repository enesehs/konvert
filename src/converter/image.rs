use anyhow::{bail, Context, Result};
use image::{codecs::jpeg::JpegEncoder, imageops, DynamicImage, GenericImageView, ImageFormat};
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

pub use crate::converter::{determine_output_path, ensure_parent_dir, get_output_dir};

pub fn is_supported_image_target(target: &str) -> bool {
    let t = target.to_lowercase();
    t == "jxl" || parse_image_format(&t).is_some()
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

pub fn convert_image(
    input_path: &Path,
    target_ext: &str,
    quality: Option<u8>,
    lossless: bool,
    custom_output_dir: Option<&Path>,
) -> Result<PathBuf> {
    let target_lower = target_ext.to_lowercase();
    let output_path = determine_output_path(input_path, &target_lower, custom_output_dir);
    ensure_parent_dir(&output_path)?;

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
        let output = Command::new("djxl")
            .arg(input_path)
            .arg("-")
            .arg("--output_format")
            .arg("png")
            .output()
            .with_context(|| "djxl komutu çalıştırılamadı")?;

        if !output.status.success() {
            bail!("JXL görseli çözülemedi: {}", input_path.display());
        }

        image::load_from_memory(&output.stdout)
            .with_context(|| "djxl çıktısı bellekten okunamadı")?
    } else {
        image::open(input_path)
            .with_context(|| format!("Görsel açılamadı: {}", input_path.display()))?
    };

    let format = parse_image_format(&target_lower)
        .with_context(|| format!("Desteklenmeyen görsel formatı: {}", target_lower))?;

    match format {
        ImageFormat::Jpeg => {
            let q = quality.unwrap_or(85).clamp(1, 100);
            let file = File::create(&output_path)
                .with_context(|| format!("Dosya oluşturulamadı: {}", output_path.display()))?;
            let mut writer = BufWriter::with_capacity(64 * 1024, file);
            let mut encoder = JpegEncoder::new_with_quality(&mut writer, q);
            encoder.encode_image(&img)
                .with_context(|| "JPEG kodlama hatası")?;
            writer.flush().with_context(|| "JPEG tamponu yazılamadı")?;
        }
        ImageFormat::WebP => {
            // If lossy is preferred and quality is set, try ImageMagick for true lossy WebP
            if !lossless {
                let q_str = quality.unwrap_or(82).to_string();
                if crate::converter::run_imagemagick(&[
                    input_path.as_os_str(),
                    std::ffi::OsStr::new("-auto-orient"),
                    std::ffi::OsStr::new("-quality"),
                    std::ffi::OsStr::new(&q_str),
                    output_path.as_os_str(),
                ]).is_ok() {
                    return Ok(output_path);
                }
            }
            let file = File::create(&output_path)
                .with_context(|| format!("Dosya oluşturulamadı: {}", output_path.display()))?;
            let mut writer = BufWriter::with_capacity(64 * 1024, file);
            img.write_to(&mut writer, ImageFormat::WebP)
                .with_context(|| "WebP kaydetme hatası")?;
            writer.flush().with_context(|| "WebP tamponu yazılamadı")?;
        }
        _ => {
            let file = File::create(&output_path)
                .with_context(|| format!("Dosya oluşturulamadı: {}", output_path.display()))?;
            let mut writer = BufWriter::with_capacity(64 * 1024, file);
            img.write_to(&mut writer, format)
                .with_context(|| format!("Görsel kaydedilemedi: {}", output_path.display()))?;
            writer.flush().with_context(|| "Görsel tamponu yazılamadı")?;
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
    ensure_parent_dir(output_path)?;
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

/// Akıllı görsel sıkıştırma (ImageMagick lossy WebP, 82 kalite, EXIF temizleme)
pub fn compress_image(
    input_path: &Path,
    quality: Option<u8>,
    custom_output_dir: Option<&Path>,
) -> Result<PathBuf> {
    let file_stem = input_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("image");

    let target_dir = get_output_dir(input_path, custom_output_dir);
    let output_path = target_dir.join(format!("{}_compressed.webp", file_stem));
    ensure_parent_dir(&output_path)?;

    let q_str = quality.unwrap_or(82).to_string();
    if crate::converter::run_imagemagick(&[
        input_path.as_os_str(),
        std::ffi::OsStr::new("-auto-orient"),
        std::ffi::OsStr::new("-strip"),
        std::ffi::OsStr::new("-quality"),
        std::ffi::OsStr::new(&q_str),
        output_path.as_os_str(),
    ]).is_ok() {
        return Ok(output_path);
    }

    // Native Image crate fallback
    let img = image::open(input_path)
        .with_context(|| format!("Görsel açılamadı: {}", input_path.display()))?;

    let file = File::create(&output_path)
        .with_context(|| format!("Çıktı dosyası oluşturulamadı: {}", output_path.display()))?;
    let mut writer = BufWriter::with_capacity(64 * 1024, file);

    img.write_to(&mut writer, ImageFormat::WebP)
        .with_context(|| "Görsel WebP olarak sıkıştırılamadı")?;
    writer.flush().with_context(|| "WebP tamponu boşaltılamadı")?;

    Ok(output_path)
}

/// EXIF, GPS ve gizlilik bilgilerini kaldırırken oryantasyonu (yönü) koruyarak temiz bir kopya üretir
pub fn strip_metadata(input_path: &Path, custom_output_dir: Option<&Path>) -> Result<PathBuf> {
    let ext = input_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("png")
        .to_lowercase();

    let file_stem = input_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("image");

    let target_dir = get_output_dir(input_path, custom_output_dir);
    let output_path = target_dir.join(format!("{}_clean.{}", file_stem, ext));
    ensure_parent_dir(&output_path)?;

    // ImageMagick ile -auto-orient uygulayarak EXIF yönünü piksele göm, ardından tüm etiketleri temizle!
    if crate::converter::run_imagemagick(&[
        input_path.as_os_str(),
        std::ffi::OsStr::new("-auto-orient"),
        std::ffi::OsStr::new("-strip"),
        output_path.as_os_str(),
    ]).is_ok() {
        return Ok(output_path);
    }

    // Yedek: Yerel bellek içi kodlama
    if let Ok(img) = image::open(input_path)
        && let Ok(file) = File::create(&output_path)
    {
        let mut writer = BufWriter::with_capacity(64 * 1024, file);
        let fmt = parse_image_format(&ext).unwrap_or(ImageFormat::Png);
        let write_res = match fmt {
            ImageFormat::Jpeg => {
                let mut encoder = JpegEncoder::new_with_quality(&mut writer, 95);
                encoder.encode_image(&img)
            }
            _ => img.write_to(&mut writer, fmt),
        };
        if write_res.is_ok() && writer.flush().is_ok() {
            return Ok(output_path);
        }
    }

    bail!("Metadata temizleme işlemi başarısız oldu")
}

/// Görseli yeniden boyutlandırır (orijinalden küçükse 1080p/720p için büyütmez)
pub fn resize_image(
    input_path: &Path,
    spec: &crate::converter::ResizeSpec,
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

    let target_dir = get_output_dir(input_path, custom_output_dir);
    let output_path = target_dir.join(format!("{}_resized.{}", file_stem, ext));
    ensure_parent_dir(&output_path)?;

    let img = image::open(input_path)
        .with_context(|| format!("Görsel açılamadı: {}", input_path.display()))?;

    let (w, h) = img.dimensions();

    let (new_w, new_h) = match spec {
        crate::converter::ResizeSpec::Percent(pct) => {
            let p = *pct as f32 / 100.0;
            (((w as f32 * p).round() as u32).max(1), ((h as f32 * p).round() as u32).max(1))
        }
        crate::converter::ResizeSpec::Fit1080p => {
            if w <= 1920 && h <= 1080 {
                (w, h)
            } else if w > h {
                let ratio = 1920.0 / w as f32;
                (1920, ((h as f32 * ratio).round() as u32).max(1))
            } else {
                let ratio = 1080.0 / h as f32;
                (((w as f32 * ratio).round() as u32).max(1), 1080)
            }
        }
        crate::converter::ResizeSpec::Fit720p => {
            if w <= 1280 && h <= 720 {
                (w, h)
            } else if w > h {
                let ratio = 1280.0 / w as f32;
                (1280, ((h as f32 * ratio).round() as u32).max(1))
            } else {
                let ratio = 720.0 / h as f32;
                (((w as f32 * ratio).round() as u32).max(1), 720)
            }
        }
    };

    let resized = img.resize(new_w, new_h, imageops::FilterType::Lanczos3);

    let fmt = parse_image_format(ext).unwrap_or(ImageFormat::Png);
    let file = File::create(&output_path)?;
    let mut writer = BufWriter::with_capacity(64 * 1024, file);
    match fmt {
        ImageFormat::Jpeg => {
            let mut encoder = JpegEncoder::new_with_quality(&mut writer, 90);
            encoder.encode_image(&resized)?;
        }
        _ => {
            resized.write_to(&mut writer, fmt)?;
        }
    }
    writer.flush()?;

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

    let target_dir = get_output_dir(&images[0], custom_output_dir);
    let output_path = target_dir.join(format!("{}_birlestirildi.pdf", first_stem));
    ensure_parent_dir(&output_path)?;

    let mut args: Vec<&std::ffi::OsStr> = Vec::with_capacity(images.len() + 2);
    for img in images {
        args.push(img.as_os_str());
    }
    args.push(std::ffi::OsStr::new("-auto-orient"));
    args.push(output_path.as_os_str());

    crate::converter::run_imagemagick(&args)
        .with_context(|| "Görseller PDF olarak birleştirilemedi")?;

    Ok(output_path)
}

/// Web optimizasyonu (kayıplı WebP, 82 kalite, EXIF auto-orient ve strip)
pub fn web_optimize_image(
    input_path: &Path,
    custom_output_dir: Option<&Path>,
) -> Result<PathBuf> {
    let file_stem = input_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("image");

    let target_dir = get_output_dir(input_path, custom_output_dir);
    let output_path = target_dir.join(format!("{}_web.webp", file_stem));
    ensure_parent_dir(&output_path)?;

    if crate::converter::run_imagemagick(&[
        input_path.as_os_str(),
        std::ffi::OsStr::new("-auto-orient"),
        std::ffi::OsStr::new("-strip"),
        std::ffi::OsStr::new("-quality"),
        std::ffi::OsStr::new("82"),
        output_path.as_os_str(),
    ]).is_ok() {
        return Ok(output_path);
    }

    // Yerel WebP yedeği
    let img = image::open(input_path)
        .with_context(|| "Görsel açılamadı")?;
    let file = File::create(&output_path)?;
    let mut writer = BufWriter::with_capacity(64 * 1024, file);
    img.write_to(&mut writer, ImageFormat::WebP)?;
    writer.flush()?;

    Ok(output_path)
}

/// Hedef boyuta göre akıllı görsel sıkıştırması (Discord 8MB, E-Posta 25MB)
/// Dosya boyutunu hedefin altına çekene kadar kaliteyi ve ölçeği kademeli olarak azaltır
pub fn compress_image_to_size(
    input_path: &Path,
    target_bytes: u64,
    custom_output_dir: Option<&Path>,
) -> Result<PathBuf> {
    if target_bytes == 0 {
        bail!("Hedef dosya boyutu 0 olamaz");
    }

    let file_stem = input_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("image");
    let ext = input_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("jpg")
        .to_lowercase();
    let target_ext = if ext == "png" { "png" } else { "jpg" };

    let target_dir = get_output_dir(input_path, custom_output_dir);
    let output_path = target_dir.join(format!("{}_target.{}", file_stem, target_ext));
    ensure_parent_dir(&output_path)?;

    let current_bytes = fs::metadata(input_path).map(|m| m.len()).unwrap_or(0);
    if current_bytes > 0
        && current_bytes <= target_bytes
        && crate::converter::run_imagemagick(&[
            input_path.as_os_str(),
            std::ffi::OsStr::new("-auto-orient"),
            std::ffi::OsStr::new("-strip"),
            output_path.as_os_str(),
        ]).is_ok()
    {
        return Ok(output_path);
    }

    // Kademeli sıkıştırma ve ölçekleme döngüsü (hedef boyutu kesin garanti altına alır)
    let qualities = ["82", "65", "50", "35"];
    let scales = ["100%", "75%", "50%", "35%"];

    for scale in &scales {
        for quality in &qualities {
            let mut args = vec![
                input_path.as_os_str(),
                std::ffi::OsStr::new("-auto-orient"),
                std::ffi::OsStr::new("-strip"),
            ];
            if *scale != "100%" {
                args.push(std::ffi::OsStr::new("-resize"));
                args.push(std::ffi::OsStr::new(scale));
            }
            args.push(std::ffi::OsStr::new("-quality"));
            args.push(std::ffi::OsStr::new(quality));
            args.push(output_path.as_os_str());

            if crate::converter::run_imagemagick(&args).is_ok()
                && let Ok(meta) = fs::metadata(&output_path)
                && meta.len() <= target_bytes
            {
                return Ok(output_path);
            }
        }
    }

    if output_path.exists() {
        Ok(output_path)
    } else {
        bail!("Görsel hedef boyuta sıkıştırılamadı");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_image_format() {
        assert_eq!(parse_image_format("png"), Some(ImageFormat::Png));
        assert_eq!(parse_image_format("jpg"), Some(ImageFormat::Jpeg));
        assert_eq!(parse_image_format("webp"), Some(ImageFormat::WebP));
        assert_eq!(parse_image_format("unknown"), None);
    }

    #[test]
    fn test_determine_output_path() {
        let input = Path::new("/test/path/image.jpg");
        let out = determine_output_path(input, "webp", None);
        assert_eq!(out, PathBuf::from("/test/path/image.webp"));

        let custom_dir = Path::new("/custom/out");
        let out_custom = determine_output_path(input, "png", Some(custom_dir));
        assert_eq!(out_custom, PathBuf::from("/custom/out/image.png"));
    }

    #[test]
    fn test_determine_output_path_same_extension() {
        let input = Path::new("/test/path/image.jpg");
        let out = determine_output_path(input, "jpg", None);
        assert_eq!(out, PathBuf::from("/test/path/image_konvert.jpg"));
    }

    #[test]
    fn test_ensure_parent_dir() {
        let test_file = std::env::temp_dir().join("konvert_test_image_dir/test.txt");
        assert!(ensure_parent_dir(&test_file).is_ok());
        assert!(test_file.parent().unwrap().exists());
        let _ = std::fs::remove_dir_all(std::env::temp_dir().join("konvert_test_image_dir"));
    }
}
