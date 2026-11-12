use anyhow::{Context, Result, bail};
use image::{
    DynamicImage, GenericImageView, ImageDecoder, ImageFormat, codecs::jpeg::JpegEncoder, imageops,
};
use std::fs::{self, File};
use std::io::{BufRead, Seek};
use std::path::{Path, PathBuf};
use std::process::Command;

pub use crate::converter::{
    claim_output_path, determine_output_path, determine_suffixed_output_path, ensure_parent_dir,
    get_output_dir,
};

pub fn is_supported_image_target(target: &str) -> bool {
    let t = target.to_lowercase();
    t == "jxl" || t == "svg" || t == "heic" || t == "heif" || parse_image_format(&t).is_some()
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

        image::load_from_memory(&output.stdout).with_context(|| "djxl çıktısı bellekten okunamadı")
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
    } else if ext == "heic" || ext == "heif" {
        let output = Command::new("magick")
            .arg("-auto-orient")
            .arg(input_path)
            .arg("png:-")
            .output()
            .or_else(|_| {
                Command::new("convert")
                    .arg("-auto-orient")
                    .arg(input_path)
                    .arg("png:-")
                    .output()
            })
            .with_context(|| "HEIC/HEIF okuyucu (magick veya convert) çalıştırılamadı")?;

        if !output.status.success() {
            bail!("HEIC/HEIF görseli çözülemedi: {}", input_path.display());
        }

        image::load_from_memory(&output.stdout)
            .with_context(|| "HEIC/HEIF çıktısı bellekten okunamadı")
    } else {
        let file = File::open(input_path)
            .with_context(|| format!("Görsel açılamadı: {}", input_path.display()))?;
        decode_with_orientation(std::io::BufReader::new(file))
            .with_context(|| format!("Görsel açılamadı: {}", input_path.display()))
    }
}

/// EXIF yön etiketini uygulayarak görseli çözer.
///
/// `image` crate yön etiketini otomatik uygulamaz; uygulanmazsa telefon
/// fotoğrafları 90° yanlış görünür. Yön, çözücüden okunup piksele gömülür.
fn decode_with_orientation<R>(reader: R) -> Result<DynamicImage>
where
    R: BufRead + Seek,
{
    let reader = image::ImageReader::new(reader)
        .with_guessed_format()
        .context("Görsel formatı tanınamadı")?;
    let mut decoder = reader
        .into_decoder()
        .context("Görsel çözücü başlatılamadı")?;
    let orientation = decoder
        .orientation()
        .unwrap_or(image::metadata::Orientation::NoTransforms);
    let mut img = DynamicImage::from_decoder(decoder).context("Görsel çözülemedi")?;
    img.apply_orientation(orientation);
    Ok(img)
}

pub fn convert_image(
    input_path: &Path,
    target_ext: &str,
    quality: Option<u8>,
    lossless: bool,
    custom_output_dir: Option<&Path>,
) -> Result<PathBuf> {
    let target_lower = target_ext.to_lowercase();

    // JPEG XL (.jxl): kayıpsız ve kayıplı action'lar aynı adı üretmesin.
    if target_lower == "jxl" {
        let candidate = if lossless {
            determine_suffixed_output_path(input_path, "jxl_lossless", "jxl", custom_output_dir)
        } else {
            determine_output_path(input_path, &target_lower, custom_output_dir)
        };
        let output_path = claim_output_path(candidate);
        return convert_to_jxl(input_path, &output_path, quality, lossless);
    }

    let output_path = claim_output_path(determine_output_path(
        input_path,
        &target_lower,
        custom_output_dir,
    ));
    ensure_parent_dir(&output_path)?;

    // SVG (.svg) - Bitmap görseller potrace üzerinden vektörleştirilir
    if target_lower == "svg" {
        crate::logger::info(&format!(
            "Tracing raster image to vector SVG via ImageMagick/potrace: {}",
            input_path.display()
        ));
        let pending = crate::converter::PendingOutput::new(&output_path);
        let status = crate::converter::run_imagemagick(&[
            input_path.as_os_str(),
            pending.path().as_os_str(),
        ]);
        if status.is_ok() && pending.path().exists() {
            return pending.commit();
        }
        bail!("SVG formatına dönüştürülemedi: {}", input_path.display());
    }

    // HEIC / HEIF (.heic, .heif)
    if target_lower == "heic" || target_lower == "heif" {
        let pending = crate::converter::PendingOutput::new(&output_path);
        let q_str = quality.unwrap_or(85).to_string();
        let status = crate::converter::run_imagemagick(&[
            input_path.as_os_str(),
            std::ffi::OsStr::new("-quality"),
            std::ffi::OsStr::new(&q_str),
            pending.path().as_os_str(),
        ]);
        if status.is_ok() && pending.path().exists() {
            return pending.commit();
        }
        bail!("HEIC formatına dönüştürülemedi: {}", input_path.display());
    }

    let img = open_dynamic_image(input_path)?;
    let format = parse_image_format(&target_lower)
        .with_context(|| format!("Desteklenmeyen görsel formatı: {}", target_lower))?;

    match format {
        ImageFormat::Jpeg => {
            let q = quality.unwrap_or(85).clamp(1, 100);
            let rgb = img.to_rgb8(); // RGBA verilerini güvenli RGB8 formatına dönüştür
            crate::converter::write_file_atomic(&output_path, |writer| {
                let mut encoder = JpegEncoder::new_with_quality(writer, q);
                encoder.encode_image(&rgb).context("JPEG kodlama hatası")?;
                Ok(())
            })?;
        }
        ImageFormat::WebP => {
            if !lossless {
                let pending = crate::converter::PendingOutput::new(&output_path);
                let q_str = quality.unwrap_or(82).to_string();
                if crate::converter::run_imagemagick(&[
                    input_path.as_os_str(),
                    std::ffi::OsStr::new("-auto-orient"),
                    std::ffi::OsStr::new("-quality"),
                    std::ffi::OsStr::new(&q_str),
                    pending.path().as_os_str(),
                ])
                .is_ok()
                    && pending.path().exists()
                {
                    return pending.commit();
                }
            }
            crate::converter::write_file_atomic(&output_path, |writer| {
                img.write_to(writer, ImageFormat::WebP)
                    .context("WebP kaydetme hatası")?;
                Ok(())
            })?;
        }
        ImageFormat::Ico => {
            let pending = crate::converter::PendingOutput::new(&output_path);
            if crate::converter::run_imagemagick(&[
                input_path.as_os_str(),
                std::ffi::OsStr::new("-resize"),
                std::ffi::OsStr::new("256x256>"),
                pending.path().as_os_str(),
            ])
            .is_ok()
                && pending.path().exists()
            {
                return pending.commit();
            }
            let resized = if img.width() > 256 || img.height() > 256 {
                img.resize(256, 256, image::imageops::FilterType::Lanczos3)
            } else {
                img
            };
            crate::converter::write_file_atomic(&output_path, |writer| {
                resized
                    .write_to(writer, ImageFormat::Ico)
                    .with_context(|| {
                        format!("ICO formatında kaydedilemedi: {}", output_path.display())
                    })?;
                Ok(())
            })?;
        }
        ImageFormat::Bmp => {
            let pending = crate::converter::PendingOutput::new(&output_path);
            if crate::converter::run_imagemagick(&[
                input_path.as_os_str(),
                pending.path().as_os_str(),
            ])
            .is_ok()
                && pending.path().exists()
            {
                return pending.commit();
            }
            crate::converter::write_file_atomic(&output_path, |writer| {
                img.write_to(writer, ImageFormat::Bmp).with_context(|| {
                    format!("BMP formatında kaydedilemedi: {}", output_path.display())
                })?;
                Ok(())
            })?;
        }
        _ => {
            crate::converter::write_file_atomic(&output_path, |writer| {
                img.write_to(writer, format)
                    .with_context(|| format!("Görsel kaydedilemedi: {}", output_path.display()))?;
                Ok(())
            })?;
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
    let pending = crate::converter::PendingOutput::new(output_path);
    let mut cmd = Command::new("cjxl");
    cmd.arg(input_path).arg(pending.path());

    if lossless {
        cmd.arg("-d").arg("0");
    } else {
        let q = quality.unwrap_or(85).clamp(1, 100);
        cmd.arg("-q").arg(q.to_string());
    }

    let status = cmd
        .status()
        .with_context(|| "cjxl komutu çalıştırılamadı")?;
    if !status.success() {
        bail!("cjxl ile JPEG XL dönüştürme başarısız oldu");
    }

    pending.commit()
}

pub fn encode_webp_optimized(
    input_path: &Path,
    quality: Option<u8>,
    custom_output_dir: Option<&Path>,
    suffix: &str,
) -> Result<PathBuf> {
    let output_path = claim_output_path(determine_suffixed_output_path(
        input_path,
        suffix,
        "webp",
        custom_output_dir,
    ));
    ensure_parent_dir(&output_path)?;

    let q_str = quality.unwrap_or(82).to_string();
    let pending = crate::converter::PendingOutput::new(&output_path);
    if crate::converter::run_imagemagick(&[
        input_path.as_os_str(),
        std::ffi::OsStr::new("-auto-orient"),
        std::ffi::OsStr::new("-strip"),
        std::ffi::OsStr::new("-quality"),
        std::ffi::OsStr::new(&q_str),
        pending.path().as_os_str(),
    ])
    .is_ok()
        && pending.path().exists()
    {
        return pending.commit();
    }

    // Yerel WebP yedeği
    let img = open_dynamic_image(input_path)?;
    crate::converter::write_file_atomic(&output_path, |writer| {
        img.write_to(writer, ImageFormat::WebP)?;
        Ok(())
    })?;

    Ok(output_path)
}

pub fn compress_image(
    input_path: &Path,
    quality: Option<u8>,
    custom_output_dir: Option<&Path>,
) -> Result<PathBuf> {
    encode_webp_optimized(input_path, quality, custom_output_dir, "compressed")
}

pub fn web_optimize_image(input_path: &Path, custom_output_dir: Option<&Path>) -> Result<PathBuf> {
    encode_webp_optimized(input_path, Some(82), custom_output_dir, "web")
}

pub fn strip_metadata(input_path: &Path, custom_output_dir: Option<&Path>) -> Result<PathBuf> {
    let ext = input_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("png")
        .to_lowercase();

    let output_path = claim_output_path(determine_suffixed_output_path(
        input_path,
        "clean",
        &ext,
        custom_output_dir,
    ));
    ensure_parent_dir(&output_path)?;

    if ext == "jxl" {
        return convert_to_jxl(input_path, &output_path, None, true);
    }

    // ImageMagick ile -auto-orient uygulayarak EXIF yönünü piksele göm, ardından tüm etiketleri temizle!
    let pending = crate::converter::PendingOutput::new(&output_path);
    if crate::converter::run_imagemagick(&[
        input_path.as_os_str(),
        std::ffi::OsStr::new("-auto-orient"),
        std::ffi::OsStr::new("-strip"),
        pending.path().as_os_str(),
    ])
    .is_ok()
        && pending.path().exists()
    {
        return pending.commit();
    }

    // Yedek: Yerel bellek içi kodlama
    let img = open_dynamic_image(input_path)?;
    let fmt = parse_image_format(&ext).unwrap_or(ImageFormat::Png);
    crate::converter::write_file_atomic(&output_path, |writer| {
        match fmt {
            ImageFormat::Jpeg => {
                let rgb = img.to_rgb8();
                let mut encoder = JpegEncoder::new_with_quality(writer, 95);
                encoder.encode_image(&rgb)?;
            }
            _ => {
                img.write_to(writer, fmt)?;
            }
        }
        Ok(())
    })?;

    Ok(output_path)
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

    let output_path = claim_output_path(determine_suffixed_output_path(
        input_path,
        "resized",
        &ext,
        custom_output_dir,
    ));
    ensure_parent_dir(&output_path)?;

    let img = open_dynamic_image(input_path)?;
    let (w, h) = img.dimensions();

    let (new_w, new_h) = match spec {
        crate::converter::ResizeSpec::Percent(pct) => {
            let p = *pct as f32 / 100.0;
            (
                ((w as f32 * p).round() as u32).max(1),
                ((h as f32 * p).round() as u32).max(1),
            )
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
        // Resized görseli geçici PNG yapıp cjxl ile dönüştür.
        // Sayaç, paralel batch'te aynı PID'nin dosyayı ezmesini engeller.
        static RESIZE_TEMP_COUNTER: std::sync::atomic::AtomicUsize =
            std::sync::atomic::AtomicUsize::new(0);
        let counter = RESIZE_TEMP_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let temp_png = std::env::temp_dir().join(format!(
            "konvert_resize_{}_{}.png",
            std::process::id(),
            counter
        ));
        resized.save_with_format(&temp_png, ImageFormat::Png)?;
        let jxl_res = convert_to_jxl(&temp_png, &output_path, None, false);
        let _ = fs::remove_file(&temp_png);
        return jxl_res;
    }

    let fmt = parse_image_format(&ext).unwrap_or(ImageFormat::Png);
    crate::converter::write_file_atomic(&output_path, |writer| {
        match fmt {
            ImageFormat::Jpeg => {
                let rgb = resized.to_rgb8();
                let mut encoder = JpegEncoder::new_with_quality(writer, 90);
                encoder.encode_image(&rgb)?;
            }
            _ => {
                resized.write_to(writer, fmt)?;
            }
        }
        Ok(())
    })?;

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
        .unwrap_or("images");

    let target_dir = get_output_dir(&images[0], custom_output_dir);
    let output_path = claim_output_path(target_dir.join(format!("{}_merged.pdf", first_stem)));
    ensure_parent_dir(&output_path)?;

    let pending = crate::converter::PendingOutput::new(&output_path);

    // Girdilerin tamamı PDF ise ImageMagick yerine doğrudan Ghostscript kullan (policy.xml engeline takılmaz).
    let all_pdf = images.iter().all(|f| {
        f.extension()
            .and_then(|e| e.to_str())
            .map(|e| e.eq_ignore_ascii_case("pdf"))
            .unwrap_or(false)
    });

    if all_pdf && crate::deps::command_exists("gs") {
        let mut cmd = Command::new("gs");
        cmd.args(["-dBATCH", "-dNOPAUSE", "-q", "-sDEVICE=pdfwrite"]);
        cmd.arg(format!("-sOutputFile={}", pending.path().display()));
        for pdf in images {
            cmd.arg(pdf);
        }
        let status = cmd
            .status()
            .with_context(|| "Ghostscript (gs) çalıştırılamadı")?;
        if !status.success() {
            bail!("Ghostscript ile PDF dosyaları birleştirilemedi");
        }
        return pending.commit();
    }

    // Yüzlerce dosya varsa ARG_MAX ve ImageMagick bellek sınırını aşmamak için 50'şerli parçalar halinde birleştir.
    if images.len() > 50 {
        eprintln!(
            "Konvert: {} dosya tespit edildi; bellek ve argüman sınırını korumak için 50'şerli parçalar halinde birleştiriliyor...",
            images.len()
        );
        static CHUNK_DIR_COUNTER: std::sync::atomic::AtomicUsize =
            std::sync::atomic::AtomicUsize::new(0);
        let chunk_id = CHUNK_DIR_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let temp_path = std::env::temp_dir().join(format!(
            "konvert_merge_chunks_{}_{}",
            std::process::id(),
            chunk_id
        ));
        std::fs::create_dir_all(&temp_path).context("Geçici klasör oluşturulamadı")?;

        struct TempChunkDir(PathBuf);
        impl Drop for TempChunkDir {
            fn drop(&mut self) {
                let _ = std::fs::remove_dir_all(&self.0);
            }
        }
        let _guard = TempChunkDir(temp_path.clone());

        let mut chunk_pdfs = Vec::new();
        for (i, chunk) in images.chunks(50).enumerate() {
            let chunk_pdf = temp_path.join(format!("chunk_{:04}.pdf", i));
            let mut chunk_args: Vec<std::ffi::OsString> = chunk
                .iter()
                .map(|img| img.as_os_str().to_os_string())
                .collect();
            chunk_args.push(std::ffi::OsStr::new("-auto-orient").to_os_string());
            chunk_args.push(chunk_pdf.as_os_str().to_os_string());
            crate::converter::run_imagemagick(&chunk_args)
                .with_context(|| format!("{}. parça PDF'e dönüştürülemedi", i + 1))?;
            chunk_pdfs.push(chunk_pdf);
        }

        if crate::deps::command_exists("gs") {
            let mut cmd = Command::new("gs");
            cmd.args(["-dBATCH", "-dNOPAUSE", "-q", "-sDEVICE=pdfwrite"]);
            cmd.arg(format!("-sOutputFile={}", pending.path().display()));
            for pdf in &chunk_pdfs {
                cmd.arg(pdf);
            }
            let status = cmd
                .status()
                .with_context(|| "Ghostscript çalıştırılamadı")?;
            if !status.success() {
                bail!("Ghostscript ile parçalar birleştirilemedi");
            }
        } else {
            let mut final_args: Vec<std::ffi::OsString> = chunk_pdfs
                .iter()
                .map(|p| p.as_os_str().to_os_string())
                .collect();
            final_args.push(pending.path().as_os_str().to_os_string());
            crate::converter::run_imagemagick(&final_args)
                .with_context(|| "Parça PDF'leri birleştirilemedi")?;
        }
        return pending.commit();
    }

    let mut args: Vec<std::ffi::OsString> = images
        .iter()
        .map(|img| img.as_os_str().to_os_string())
        .collect();
    args.push(std::ffi::OsStr::new("-auto-orient").to_os_string());
    args.push(pending.path().as_os_str().to_os_string());

    crate::converter::run_imagemagick(&args)
        .with_context(|| "Görseller PDF olarak birleştirilemedi")?;

    pending.commit()
}

pub fn image_to_pdf(input_path: &Path, custom_output_dir: Option<&Path>) -> Result<PathBuf> {
    let target_dir = get_output_dir(input_path, custom_output_dir);
    let stem = input_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("image");
    let output_path = claim_output_path(target_dir.join(format!("{}.pdf", stem)));
    ensure_parent_dir(&output_path)?;

    let pending = crate::converter::PendingOutput::new(&output_path);
    let args = [
        input_path.as_os_str().to_os_string(),
        std::ffi::OsStr::new("-auto-orient").to_os_string(),
        pending.path().as_os_str().to_os_string(),
    ];

    crate::converter::run_imagemagick(&args).with_context(|| "Görsel PDF'e dönüştürülemedi")?;

    pending.commit()
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

    let suffix = match target_bytes {
        bytes if bytes == 8 * 1024 * 1024 => "8mb",
        bytes if bytes == 25 * 1024 * 1024 => "25mb",
        _ => "target",
    };
    let output_path = claim_output_path(determine_suffixed_output_path(
        input_path,
        suffix,
        target_ext,
        custom_output_dir,
    ));
    ensure_parent_dir(&output_path)?;

    // Kademeli sıkıştırma ve ölçekleme döngüsü (hedef boyutu kesin garanti altına alır)
    let qualities = ["82", "65", "50", "35"];
    let scales = ["100%", "75%", "50%", "35%"];
    let pending = crate::converter::PendingOutput::new(&output_path);

    for scale in &scales {
        for quality in &qualities {
            let mut args: Vec<std::ffi::OsString> = vec![
                input_path.as_os_str().to_os_string(),
                std::ffi::OsStr::new("-auto-orient").to_os_string(),
                std::ffi::OsStr::new("-strip").to_os_string(),
            ];
            if *scale != "100%" {
                args.push(std::ffi::OsStr::new("-resize").to_os_string());
                args.push(std::ffi::OsStr::new(scale).to_os_string());
            }
            args.push(std::ffi::OsStr::new("-quality").to_os_string());
            args.push(std::ffi::OsStr::new(quality).to_os_string());
            args.push(pending.path().as_os_str().to_os_string());

            if crate::converter::run_imagemagick(&args).is_ok()
                && let Ok(meta) = fs::metadata(pending.path())
                && meta.len() <= target_bytes
            {
                return pending.commit();
            }
        }
    }

    // Hedefe ulaşılamadı: geçici dosya drop ile silinir, mevcut çıktı (varsa) korunur.
    let reached_size = fs::metadata(pending.path()).map(|m| m.len()).unwrap_or(0);
    drop(pending);
    bail!(
        "Görsel hedef boyuta ({:.1} MB) sıkıştırılamadı. Ulaşılan en küçük boyut: {:.1} MB",
        target_bytes as f64 / (1024.0 * 1024.0),
        reached_size as f64 / (1024.0 * 1024.0)
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_image_format() {
        assert_eq!(parse_image_format("png"), Some(ImageFormat::Png));
        assert_eq!(parse_image_format("jpg"), Some(ImageFormat::Jpeg));
        assert_eq!(parse_image_format("webp"), Some(ImageFormat::WebP));
        assert_eq!(parse_image_format("bmp"), Some(ImageFormat::Bmp));
        assert_eq!(parse_image_format("ico"), Some(ImageFormat::Ico));
        assert_eq!(parse_image_format("unknown"), None);
    }

    #[test]
    fn test_is_supported_image_target() {
        assert!(is_supported_image_target("png"));
        assert!(is_supported_image_target("webp"));
        assert!(is_supported_image_target("jxl"));
        assert!(is_supported_image_target("svg"));
        assert!(is_supported_image_target("ico"));
        assert!(is_supported_image_target("heic"));
        assert!(is_supported_image_target("bmp"));
        assert!(!is_supported_image_target("mp4"));
        assert!(!is_supported_image_target("pdf"));
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
    fn test_decode_applies_exif_orientation() {
        // 200x100, Orientation=6 (90° CW) -> piksel boyutu 100x200 olmalı.
        let bytes = include_bytes!("../../tests/fixtures/orientation_6.jpg");
        let img = decode_with_orientation(std::io::Cursor::new(&bytes[..])).unwrap();
        assert_eq!(img.dimensions(), (100, 200));
    }

    #[test]
    fn test_image_to_pdf_creates_pdf() {
        let temp_dir = std::env::temp_dir();
        let test_img = temp_dir.join("konvert_test_to_pdf.png");
        let img = DynamicImage::new_rgb8(10, 10);
        img.save(&test_img).unwrap();

        if crate::deps::imagemagick_available() {
            let res = image_to_pdf(&test_img, None);
            if let Ok(pdf_path) = res {
                assert!(pdf_path.exists());
                assert_eq!(pdf_path.extension().and_then(|e| e.to_str()), Some("pdf"));
                let _ = fs::remove_file(pdf_path);
            }
        }
        let _ = fs::remove_file(test_img);
    }

    #[test]
    fn test_merge_pdfs_via_gs_when_all_inputs_are_pdf() {
        if !crate::deps::command_exists("gs") {
            return;
        }
        let temp_dir = std::env::temp_dir();
        let img1 = temp_dir.join("konvert_test_merge_a.png");
        let img2 = temp_dir.join("konvert_test_merge_b.png");
        let _ = ::image::DynamicImage::new_rgb8(10, 10).save(&img1);
        let _ = ::image::DynamicImage::new_rgb8(10, 10).save(&img2);

        if crate::deps::imagemagick_available() {
            let pdf1 = image_to_pdf(&img1, None).unwrap();
            let pdf2 = image_to_pdf(&img2, None).unwrap();

            let merged = merge_images_to_pdf(&[pdf1.clone(), pdf2.clone()], None).unwrap();
            assert!(merged.exists());
            assert_eq!(merged.extension().and_then(|e| e.to_str()), Some("pdf"));

            let _ = fs::remove_file(merged);
            let _ = fs::remove_file(pdf1);
            let _ = fs::remove_file(pdf2);
        }

        let _ = fs::remove_file(img1);
        let _ = fs::remove_file(img2);
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
