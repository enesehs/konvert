use anyhow::{bail, Context, Result};
use image::{codecs::jpeg::JpegEncoder, imageops, DynamicImage, GenericImageView, ImageFormat};
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

pub use crate::converter::{
    determine_output_path, determine_suffixed_output_path, ensure_parent_dir, get_output_dir,
};

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

/// JXL, SVG ve standart görselleri açmak için birleşik resim yükleyici
pub fn open_dynamic_image(input_path: &Path) -> Result<DynamicImage> {
    let ext = input_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    if ext == "jxl" {
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
            .with_context(|| "djxl çıktısı bellekten okunamadı")
    } else if ext == "svg" {
        let output = Command::new("magick")
            .arg("-density")
            .arg("300")
            .arg(input_path)
            .arg("png:-")
            .output()
            .or_else(|_| {
                Command::new("convert")
                    .arg("-density")
                    .arg("300")
                    .arg(input_path)
                    .arg("png:-")
                    .output()
            })
            .with_context(|| "SVG rasterleştirici (magick veya convert) çalıştırılamadı")?;

        if !output.status.success() {
            bail!("SVG görseli rasterleştirilemedi: {}", input_path.display());
        }

        image::load_from_memory(&output.stdout)
            .with_context(|| "SVG raster çıktısı bellekten okunamadı")
    } else {
        image::open(input_path)
            .with_context(|| format!("Görsel açılamadı: {}", input_path.display()))
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

    let img = open_dynamic_image(input_path)?;
    let format = parse_image_format(&target_lower)
        .with_context(|| format!("Desteklenmeyen görsel formatı: {}", target_lower))?;

    match format {
        ImageFormat::Jpeg => {
            let q = quality.unwrap_or(85).clamp(1, 100);
            let file = File::create(&output_path)
                .with_context(|| format!("Dosya oluşturulamadı: {}", output_path.display()))?;
            let mut writer = BufWriter::with_capacity(64 * 1024, file);
            let rgb = img.to_rgb8(); // RGBA verilerini güvenli RGB8 formatına dönüştür (çökmeyi önler)
            let mut encoder = JpegEncoder::new_with_quality(&mut writer, q);
            encoder.encode_image(&rgb)
                .with_context(|| "JPEG kodlama hatası")?;
            writer.flush().with_context(|| "JPEG tamponu yazılamadı")?;
        }
        ImageFormat::WebP => {
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

pub fn encode_webp_optimized(
    input_path: &Path,
    quality: Option<u8>,
    custom_output_dir: Option<&Path>,
    suffix: &str,
) -> Result<PathBuf> {
    let output_path = determine_suffixed_output_path(
        input_path,
        suffix,
        "webp",
        custom_output_dir,
    );
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

    // Yerel WebP yedeği
    let img = open_dynamic_image(input_path)?;
    let file = File::create(&output_path)?;
    let mut writer = BufWriter::with_capacity(64 * 1024, file);
    img.write_to(&mut writer, ImageFormat::WebP)?;
    writer.flush()?;

    Ok(output_path)
}

pub fn compress_image(
    input_path: &Path,
    quality: Option<u8>,
    custom_output_dir: Option<&Path>,
) -> Result<PathBuf> {
    encode_webp_optimized(input_path, quality, custom_output_dir, "compressed")
}

pub fn web_optimize_image(
    input_path: &Path,
    custom_output_dir: Option<&Path>,
) -> Result<PathBuf> {
    encode_webp_optimized(input_path, Some(82), custom_output_dir, "web")
}

pub fn strip_metadata(input_path: &Path, custom_output_dir: Option<&Path>) -> Result<PathBuf> {
    let ext = input_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("png")
        .to_lowercase();

    let output_path = determine_suffixed_output_path(input_path, "clean", &ext, custom_output_dir);
    ensure_parent_dir(&output_path)?;

    if ext == "jxl" {
        return convert_to_jxl(input_path, &output_path, None, true);
    }

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
    if let Ok(img) = open_dynamic_image(input_path)
        && let Ok(file) = File::create(&output_path)
    {
        let mut writer = BufWriter::with_capacity(64 * 1024, file);
        let fmt = parse_image_format(&ext).unwrap_or(ImageFormat::Png);
        let write_res = match fmt {
            ImageFormat::Jpeg => {
                let rgb = img.to_rgb8();
                let mut encoder = JpegEncoder::new_with_quality(&mut writer, 95);
                encoder.encode_image(&rgb)
            }
            _ => img.write_to(&mut writer, fmt),
        };
        if write_res.is_ok() && writer.flush().is_ok() {
            return Ok(output_path);
        }
    }

    bail!("Metadata temizleme işlemi başarısız oldu")
}

pub fn resize_image(
    input_path: &Path,
    spec: &crate::converter::ResizeSpec,
    custom_output_dir: Option<&Path>,
) -> Result<PathBuf> {
    let ext = input_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("png")
        .to_lowercase();

    let output_path = determine_suffixed_output_path(input_path, "resized", &ext, custom_output_dir);
    ensure_parent_dir(&output_path)?;

    let img = open_dynamic_image(input_path)?;
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

    if ext == "jxl" {
        // Resized görseli geçici PNG yapıp cjxl ile dönüştür
        let temp_png = std::env::temp_dir().join(format!("resize_temp_{}.png", std::process::id()));
        resized.save_with_format(&temp_png, ImageFormat::Png)?;
        let jxl_res = convert_to_jxl(&temp_png, &output_path, None, false);
        let _ = fs::remove_file(temp_png);
        return jxl_res;
    }

    let fmt = parse_image_format(&ext).unwrap_or(ImageFormat::Png);
    let file = File::create(&output_path)?;
    let mut writer = BufWriter::with_capacity(64 * 1024, file);
    match fmt {
        ImageFormat::Jpeg => {
            let rgb = resized.to_rgb8();
            let mut encoder = JpegEncoder::new_with_quality(&mut writer, 90);
            encoder.encode_image(&rgb)?;
        }
        _ => {
            resized.write_to(&mut writer, fmt)?;
        }
    }
    writer.flush()?;

    Ok(output_path)
}

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

pub fn compress_image_to_size(
    input_path: &Path,
    target_bytes: u64,
    custom_output_dir: Option<&Path>,
) -> Result<PathBuf> {
    if target_bytes == 0 {
        bail!("Hedef dosya boyutu 0 olamaz");
    }

    let current_bytes = fs::metadata(input_path).map(|m| m.len()).unwrap_or(0);
    if current_bytes == 0 {
        bail!("Dosya boş veya boyutu okunamadı: {}", input_path.display());
    }

    if current_bytes <= target_bytes {
        bail!(
            "Görsel zaten hedef boyuttan ({}) daha küçük ({}). Hedef boyut sıkıştırması için dosyanın {}'dan büyük olması gerekir.",
            crate::history::format_bytes(target_bytes),
            crate::history::format_bytes(current_bytes),
            crate::history::format_bytes(target_bytes)
        );
    }

    let ext = input_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("jpg")
        .to_lowercase();

    // PNG dosyaları kayıpsız olduğu için kaliteyle küçülmez; hedef çok küçükse WebP formatına geç
    let target_ext = if ext == "png" && current_bytes > target_bytes {
        "webp"
    } else {
        &ext
    };

    let output_path = determine_suffixed_output_path(input_path, "target", target_ext, custom_output_dir);
    ensure_parent_dir(&output_path)?;

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

    if let Ok(meta) = fs::metadata(&output_path)
        && meta.len() <= target_bytes
    {
        Ok(output_path)
    } else {
        let reached_size = fs::metadata(&output_path).map(|m| m.len()).unwrap_or(0);
        let _ = fs::remove_file(&output_path);
        bail!(
            "Görsel hedef boyuta ({:.1} MB) sıkıştırılamadı. Ulaşılan en küçük boyut: {:.1} MB",
            target_bytes as f64 / (1024.0 * 1024.0),
            reached_size as f64 / (1024.0 * 1024.0)
        );
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
    fn test_rgba_to_rgb_conversion_safety() {
        let rgba = image::RgbaImage::new(10, 10);
        let dyn_img = DynamicImage::ImageRgba8(rgba);
        let rgb = dyn_img.to_rgb8();
        assert_eq!(rgb.width(), 10);
        assert_eq!(rgb.height(), 10);
    }

    #[test]
    fn test_compress_image_to_size_size_guards() {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("konvert_test_small.jpg");
        std::fs::write(&test_file, b"fake small image data").unwrap();

        // 8MB hedef boyutu, dosya çok küçük -> hata vermeli
        let res = compress_image_to_size(&test_file, 8 * 1024 * 1024, None);
        assert!(res.is_err());
        let err_msg = res.unwrap_err().to_string();
        assert!(err_msg.contains("zaten hedef boyuttan"));

        // 0 hedef boyutu -> hata vermeli
        let res_zero = compress_image_to_size(&test_file, 0, None);
        assert!(res_zero.is_err());

        let _ = std::fs::remove_file(&test_file);

        // Boş dosya -> hata vermeli
        let empty_file = temp_dir.join("konvert_test_empty.jpg");
        std::fs::write(&empty_file, b"").unwrap();
        let res_empty = compress_image_to_size(&empty_file, 8 * 1024 * 1024, None);
        assert!(res_empty.is_err());
        let err_empty = res_empty.unwrap_err().to_string();
        assert!(err_empty.contains("boş") || err_empty.contains("okunamadı"));

        let _ = std::fs::remove_file(&empty_file);
    }
}
