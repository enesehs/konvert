use anyhow::{Context, Result, bail};
use image::{
    DynamicImage, GenericImageView, ImageDecoder, ImageEncoder, ImageFormat, RgbImage,
    codecs::{avif::AvifEncoder, jpeg::JpegEncoder},
    imageops,
};
use std::fs::{self, File};
use std::io::{BufRead, Seek};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub use crate::converter::{
    claim_output_path, determine_output_path, determine_suffixed_output_path, ensure_parent_dir,
    get_output_dir,
};

pub fn is_supported_image_target(target: &str) -> bool {
    let t = target.to_lowercase();
    t == "jxl" || t == "svg" || t == "heic" || t == "heif" || parse_image_format(&t).is_some()
}

pub(crate) fn flatten_to_rgb8(img: &DynamicImage) -> RgbImage {
    if !img.color().has_alpha() {
        return img.to_rgb8();
    }
    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    let mut rgb = RgbImage::new(w, h);
    for (x, y, px) in rgba.enumerate_pixels() {
        let a = px[3] as u16;
        let inv_a = 255 - a;
        let r = ((px[0] as u16 * a + 255 * inv_a + 127) / 255) as u8;
        let g = ((px[1] as u16 * a + 255 * inv_a + 127) / 255) as u8;
        let b = ((px[2] as u16 * a + 255 * inv_a + 127) / 255) as u8;
        rgb.put_pixel(x, y, image::Rgb([r, g, b]));
    }
    rgb
}

fn encode_avif_with_quality<W: std::io::Write>(
    img: &DynamicImage,
    quality: u8,
    writer: W,
) -> Result<()> {
    let encoder = AvifEncoder::new_with_speed_quality(writer, 6, quality.clamp(1, 100));
    if img.color().has_alpha() {
        let rgba = img.to_rgba8();
        encoder
            .write_image(
                rgba.as_raw(),
                rgba.width(),
                rgba.height(),
                image::ExtendedColorType::Rgba8,
            )
            .context("AVIF kodlama hatası")?;
    } else {
        let rgb = img.to_rgb8();
        encoder
            .write_image(
                rgb.as_raw(),
                rgb.width(),
                rgb.height(),
                image::ExtendedColorType::Rgb8,
            )
            .context("AVIF kodlama hatası")?;
    }
    Ok(())
}

struct TempFileGuard(PathBuf);

impl Drop for TempFileGuard {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

fn fit_within_bounds(w: u32, h: u32, max_w: u32, max_h: u32) -> (u32, u32) {
    if w <= max_w && h <= max_h {
        return (w, h);
    }
    let ratio = (max_w as f32 / w as f32).min(max_h as f32 / h as f32);
    (
        ((w as f32 * ratio).round() as u32).max(1),
        ((h as f32 * ratio).round() as u32).max(1),
    )
}

pub(crate) fn compute_resize_dimensions(
    w: u32,
    h: u32,
    spec: &crate::converter::ResizeSpec,
) -> (u32, u32) {
    match spec {
        crate::converter::ResizeSpec::Percent(pct) => {
            let p = *pct as f32 / 100.0;
            (
                ((w as f32 * p).round() as u32).max(1),
                ((h as f32 * p).round() as u32).max(1),
            )
        }
        crate::converter::ResizeSpec::Fit1080p => fit_within_bounds(w, h, 1920, 1080),
        crate::converter::ResizeSpec::Fit720p => fit_within_bounds(w, h, 1280, 720),
    }
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
        let mut cmd = Command::new("djxl");
        cmd.arg(input_path)
            .arg("-")
            .arg("--output_format")
            .arg("png");
        let output = crate::process::run_tracked_output(&mut cmd).map_err(|e| {
            if crate::process::is_cancelled_error(&e) {
                e
            } else {
                e.context("djxl komutu çalıştırılamadı")
            }
        })?;

        if !output.status.success() {
            bail!(
                "{}: {}",
                crate::i18n::t("err_jxl_decode"),
                input_path.display()
            );
        }

        image::load_from_memory(&output.stdout).with_context(|| "djxl çıktısı bellekten okunamadı")
    } else if ext == "svg" {
        let safe_input = crate::converter::sanitize_cli_path(input_path);
        let mut magick_cmd = Command::new("magick");
        magick_cmd
            .arg("-background")
            .arg("none")
            .arg("-density")
            .arg("300")
            .arg(&safe_input)
            .arg("png:-");
        let output = match crate::process::run_tracked_output(&mut magick_cmd) {
            Ok(out) => out,
            Err(err) => {
                if crate::process::is_cancelled_error(&err) {
                    return Err(err);
                }
                let mut convert_cmd = Command::new("convert");
                convert_cmd
                    .arg("-background")
                    .arg("none")
                    .arg("-density")
                    .arg("300")
                    .arg(&safe_input)
                    .arg("png:-");
                crate::process::run_tracked_output(&mut convert_cmd)
                    .with_context(|| "SVG rasterleştirici (magick veya convert) çalıştırılamadı")?
            }
        };

        if !output.status.success() {
            bail!(
                "{}: {}",
                crate::i18n::t("err_svg_rasterize"),
                input_path.display()
            );
        }

        image::load_from_memory(&output.stdout)
            .with_context(|| "SVG raster çıktısı bellekten okunamadı")
    } else if ext == "heic" || ext == "heif" || ext == "avif" {
        let safe_input = crate::converter::sanitize_cli_path(input_path);
        let mut magick_cmd = Command::new("magick");
        magick_cmd.arg(&safe_input).arg("-auto-orient").arg("png:-");
        let output = match crate::process::run_tracked_output(&mut magick_cmd) {
            Ok(out) => out,
            Err(err) => {
                if crate::process::is_cancelled_error(&err) {
                    return Err(err);
                }
                let mut convert_cmd = Command::new("convert");
                convert_cmd
                    .arg(&safe_input)
                    .arg("-auto-orient")
                    .arg("png:-");
                crate::process::run_tracked_output(&mut convert_cmd).with_context(
                    || "AVIF/HEIC/HEIF okuyucu (magick veya convert) çalıştırılamadı",
                )?
            }
        };

        if !output.status.success() {
            bail!(
                "{}: {}",
                crate::i18n::t("err_heic_decode"),
                input_path.display()
            );
        }

        image::load_from_memory(&output.stdout)
            .with_context(|| "AVIF/HEIC/HEIF çıktısı bellekten okunamadı")
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

    // SVG (.svg) - Sistemde potrace varsa gerçek vektörizasyon (tracing), yoksa ImageMagick ile gömme.
    if target_lower == "svg" {
        let pending = crate::converter::PendingOutput::new(&output_path);

        let mut vectorized = false;
        if crate::deps::command_exists("potrace") {
            crate::process::wait_if_paused_or_cancelled()?;
            let safe_input = crate::converter::sanitize_cli_path(input_path);
            let magick_args = [
                "-background",
                "white",
                "-alpha",
                "remove",
                "-alpha",
                "off",
                "-colorspace",
                "Gray",
                "-threshold",
                "50%",
                "pbm:-",
            ];
            let mut magick_cmd = Command::new("magick");
            magick_cmd
                .arg(&safe_input)
                .args(magick_args)
                .stdout(Stdio::piped())
                .stderr(Stdio::null());
            crate::process::configure_tracked_command(&mut magick_cmd);

            let magick_child = magick_cmd.spawn().or_else(|_| {
                let mut convert_cmd = Command::new("convert");
                convert_cmd
                    .arg(&safe_input)
                    .args(magick_args)
                    .stdout(Stdio::piped())
                    .stderr(Stdio::null());
                crate::process::configure_tracked_command(&mut convert_cmd);
                convert_cmd.spawn()
            });

            if let Ok(mut m_child) = magick_child
                && let Some(m_out) = m_child.stdout.take()
            {
                let _m_guard = crate::process::register_running_child(m_child.id());
                let mut p_cmd = Command::new("potrace");
                p_cmd
                    .args(["-s", "-", "-o"])
                    .arg(pending.path())
                    .stdin(m_out)
                    .stderr(Stdio::null());
                crate::process::configure_tracked_command(&mut p_cmd);
                let potrace_status = match p_cmd.spawn() {
                    Ok(mut p_child) => {
                        let _p_guard = crate::process::register_running_child(p_child.id());
                        p_child.wait()
                    }
                    Err(e) => Err(e),
                };

                let magick_ok = matches!(m_child.wait(), Ok(st) if st.success());
                let potrace_ok = matches!(potrace_status, Ok(st) if st.success());
                if crate::process::is_job_cancelled() {
                    return Err(crate::process::cancelled_error());
                }
                let non_empty = fs::metadata(pending.path())
                    .map(|m| m.len() > 0)
                    .unwrap_or(false);

                if magick_ok && potrace_ok && non_empty {
                    crate::logger::info(&format!(
                        "Vectorized raster image to SVG via potrace: {}",
                        input_path.display()
                    ));
                    vectorized = true;
                }
            }
        }

        if !vectorized {
            let im_res = crate::converter::run_imagemagick(&[
                input_path.as_os_str(),
                pending.path().as_os_str(),
            ]);

            if let Err(ref err) = im_res
                && crate::process::is_cancelled_error(err)
            {
                return Err(crate::process::cancelled_error());
            }

            if im_res.is_err() || !pending.path().exists() {
                crate::logger::info(&format!(
                    "ImageMagick SVG delegate unavailable; falling back to native SVG bitmap embedding: {}",
                    input_path.display()
                ));
                embed_image_to_svg(input_path, pending.path()).with_context(|| {
                    format!(
                        "{}: {}",
                        crate::i18n::t("err_svg_convert"),
                        input_path.display()
                    )
                })?;
            } else {
                crate::logger::info(&format!(
                    "Converting raster image to SVG (bitmap embed) via ImageMagick: {}",
                    input_path.display()
                ));
            }
        }

        return pending.commit();
    }

    // HEIC / HEIF (.heic, .heif)
    if target_lower == "heic" || target_lower == "heif" {
        let pending = crate::converter::PendingOutput::new(&output_path);
        let q_str = quality.unwrap_or(85).to_string();
        let res = crate::converter::run_imagemagick(&[
            input_path.as_os_str(),
            std::ffi::OsStr::new("-quality"),
            std::ffi::OsStr::new(&q_str),
            pending.path().as_os_str(),
        ]);
        match res {
            Ok(()) if pending.path().exists() => return pending.commit(),
            Err(err) if crate::process::is_cancelled_error(&err) => return Err(err),
            _ => {}
        }
        bail!(
            "{}: {}",
            crate::i18n::t("err_heic_convert"),
            input_path.display()
        );
    }

    let format = parse_image_format(&target_lower)
        .with_context(|| format!("Desteklenmeyen görsel formatı: {}", target_lower))?;

    match format {
        ImageFormat::WebP => {
            if !lossless {
                let pending = crate::converter::PendingOutput::new(&output_path);
                let q_str = quality.unwrap_or(82).to_string();
                let res = crate::converter::run_imagemagick(&[
                    input_path.as_os_str(),
                    std::ffi::OsStr::new("-auto-orient"),
                    std::ffi::OsStr::new("-quality"),
                    std::ffi::OsStr::new(&q_str),
                    pending.path().as_os_str(),
                ]);
                match res {
                    Ok(()) if pending.path().exists() => return pending.commit(),
                    Err(err) if crate::process::is_cancelled_error(&err) => return Err(err),
                    _ => {}
                }
                crate::logger::warn(
                    "ImageMagick WebP encoding unavailable; falling back to lossless built-in WebP encoder",
                );
            }
            let img = open_dynamic_image(input_path)?;
            crate::converter::write_file_atomic(&output_path, |writer| {
                img.write_to(writer, ImageFormat::WebP)
                    .context("WebP kaydetme hatası")?;
                Ok(())
            })?;
        }
        ImageFormat::Png => {
            let img = open_dynamic_image(input_path)?;
            crate::converter::write_file_atomic(&output_path, |writer| {
                img.write_to(writer, ImageFormat::Png)
                    .with_context(|| format!("Görsel kaydedilemedi: {}", output_path.display()))?;
                Ok(())
            })?;
            optimize_png_with_oxipng(&output_path);
        }
        ImageFormat::Ico => {
            let pending = crate::converter::PendingOutput::new(&output_path);
            let im_input = imagemagick_first_frame_arg(input_path);
            let res = crate::converter::run_imagemagick(&[
                im_input.as_os_str(),
                std::ffi::OsStr::new("-resize"),
                std::ffi::OsStr::new("256x256>"),
                pending.path().as_os_str(),
            ]);
            match res {
                Ok(()) if pending.path().exists() => return pending.commit(),
                Err(err) if crate::process::is_cancelled_error(&err) => return Err(err),
                _ => {}
            }
            let img = open_dynamic_image(input_path)?;
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
            let im_input = imagemagick_first_frame_arg(input_path);
            let res = crate::converter::run_imagemagick(&[
                im_input.as_os_str(),
                pending.path().as_os_str(),
            ]);
            match res {
                Ok(()) if pending.path().exists() => return pending.commit(),
                Err(err) if crate::process::is_cancelled_error(&err) => return Err(err),
                _ => {}
            }
            let img = open_dynamic_image(input_path)?;
            crate::converter::write_file_atomic(&output_path, |writer| {
                img.write_to(writer, ImageFormat::Bmp).with_context(|| {
                    format!("BMP formatında kaydedilemedi: {}", output_path.display())
                })?;
                Ok(())
            })?;
        }
        ImageFormat::Jpeg => {
            let img = open_dynamic_image(input_path)?;
            let q = quality.unwrap_or(85).clamp(1, 100);
            let rgb = flatten_to_rgb8(&img);
            crate::converter::write_file_atomic(&output_path, |writer| {
                let mut encoder = JpegEncoder::new_with_quality(writer, q);
                encoder.encode_image(&rgb).context("JPEG kodlama hatası")?;
                Ok(())
            })?;
        }
        ImageFormat::Avif => {
            let img = open_dynamic_image(input_path)?;
            let q = quality.unwrap_or(80).clamp(1, 100);
            crate::converter::write_file_atomic(&output_path, |writer| {
                encode_avif_with_quality(&img, q, writer)
            })?;
        }
        _ => {
            let img = open_dynamic_image(input_path)?;
            crate::converter::write_file_atomic(&output_path, |writer| {
                img.write_to(writer, format)
                    .with_context(|| format!("Görsel kaydedilemedi: {}", output_path.display()))?;
                Ok(())
            })?;
        }
    }

    Ok(output_path)
}

fn imagemagick_first_frame_arg(input_path: &Path) -> std::ffi::OsString {
    let sanitized = crate::converter::sanitize_cli_path(input_path);
    let is_multiframe = input_path
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| matches!(e.to_ascii_lowercase().as_str(), "gif" | "tif" | "tiff"));
    let mut os = sanitized.into_os_string();
    if is_multiframe {
        os.push("[0]");
    }
    os
}

fn optimize_png_with_oxipng(path: &Path) {
    if !crate::deps::command_exists("oxipng") || crate::process::is_job_cancelled() {
        return;
    }
    let safe_path = crate::converter::sanitize_cli_path(path);
    let mut cmd = Command::new("oxipng");
    cmd.args(["-o", "2", "--strip", "safe", "--quiet"])
        .arg(&safe_path)
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    let _ = crate::process::run_tracked_status(&mut cmd);
}

fn convert_to_jxl(
    input_path: &Path,
    output_path: &Path,
    quality: Option<u8>,
    lossless: bool,
) -> Result<PathBuf> {
    ensure_parent_dir(output_path)?;
    let ext = input_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    let is_native_cjxl = matches!(
        ext.as_str(),
        "png" | "apng" | "gif" | "jpg" | "jpeg" | "ppm" | "pgm" | "pbm" | "pnm" | "pfm" | "pam"
    );

    let (_guard, cjxl_input): (Option<TempFileGuard>, PathBuf) = if is_native_cjxl {
        (None, crate::converter::sanitize_cli_path(input_path))
    } else {
        static JXL_INPUT_COUNTER: std::sync::atomic::AtomicUsize =
            std::sync::atomic::AtomicUsize::new(0);
        let cnt = JXL_INPUT_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let temp_png =
            std::env::temp_dir().join(format!("konvert_jxl_in_{}_{}.png", std::process::id(), cnt));
        let guard = TempFileGuard(temp_png.clone());
        let img = open_dynamic_image(input_path)?;
        img.save_with_format(&temp_png, ImageFormat::Png)?;
        (Some(guard), temp_png)
    };

    let pending = crate::converter::PendingOutput::new(output_path);
    let mut cmd = Command::new("cjxl");
    cmd.arg(&cjxl_input).arg(pending.path());

    if lossless {
        cmd.arg("-d").arg("0");
    } else {
        let q = quality.unwrap_or(85).clamp(1, 100);
        cmd.arg("-q").arg(q.to_string());
    }

    let status = crate::process::run_tracked_status(&mut cmd).map_err(|e| {
        if crate::process::is_cancelled_error(&e) {
            e
        } else {
            e.context("cjxl komutu çalıştırılamadı")
        }
    })?;
    if !status.success() {
        bail!("{}", crate::i18n::t("err_cjxl_failed"));
    }

    pending.commit()
}

fn base64_encode(data: &[u8]) -> String {
    const CHARSET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as usize;
        let b1 = chunk.get(1).copied().unwrap_or(0) as usize;
        let b2 = chunk.get(2).copied().unwrap_or(0) as usize;
        let triple = (b0 << 16) | (b1 << 8) | b2;
        result.push(CHARSET[(triple >> 18) & 0x3F] as char);
        result.push(CHARSET[(triple >> 12) & 0x3F] as char);
        if chunk.len() > 1 {
            result.push(CHARSET[(triple >> 6) & 0x3F] as char);
        } else {
            result.push('=');
        }
        if chunk.len() > 2 {
            result.push(CHARSET[triple & 0x3F] as char);
        } else {
            result.push('=');
        }
    }
    result
}

fn embed_image_to_svg(input_path: &Path, output_path: &Path) -> Result<()> {
    let ext = input_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    let img = open_dynamic_image(input_path)?;
    let (width, height) = (img.width(), img.height());
    let mut buf = std::io::Cursor::new(Vec::new());
    let mime = if ext == "jpg" || ext == "jpeg" {
        let rgb = flatten_to_rgb8(&img);
        let mut encoder = JpegEncoder::new_with_quality(&mut buf, 92);
        encoder.encode_image(&rgb).context("JPEG kodlama hatası")?;
        "image/jpeg"
    } else {
        img.write_to(&mut buf, ImageFormat::Png)?;
        "image/png"
    };

    let b64 = base64_encode(&buf.into_inner());
    let svg = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{w}\" height=\"{h}\" viewBox=\"0 0 {w} {h}\">\n  <image width=\"{w}\" height=\"{h}\" href=\"data:{mime};base64,{b64}\"/>\n</svg>\n",
        w = width,
        h = height,
        mime = mime,
        b64 = b64
    );
    fs::write(output_path, svg)?;
    Ok(())
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
    let im_res = crate::converter::run_imagemagick(&[
        input_path.as_os_str(),
        std::ffi::OsStr::new("-auto-orient"),
        std::ffi::OsStr::new("-strip"),
        std::ffi::OsStr::new("-quality"),
        std::ffi::OsStr::new(&q_str),
        pending.path().as_os_str(),
    ]);
    if let Err(ref err) = im_res
        && crate::process::is_cancelled_error(err)
    {
        return Err(crate::process::cancelled_error());
    }
    if im_res.is_ok() && pending.path().exists() {
        return pending.commit();
    }

    let ext = input_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    if crate::deps::command_exists("cwebp")
        && matches!(
            ext.as_str(),
            "png" | "jpg" | "jpeg" | "tif" | "tiff" | "webp"
        )
    {
        let mut cmd = Command::new("cwebp");
        cmd.args(["-q", &q_str, "-mt"])
            .arg(crate::converter::sanitize_cli_path(input_path))
            .arg("-o")
            .arg(pending.path())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        let status = crate::process::run_tracked_status(&mut cmd);
        match status {
            Ok(st) if st.success() && pending.path().exists() => return pending.commit(),
            Err(err) if crate::process::is_cancelled_error(&err) => return Err(err),
            _ => {}
        }
    }

    bail!("Kayıplı WebP sıkıştırma için ImageMagick (magick/convert) veya cwebp kurulu olmalıdır")
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
        static JXL_STRIP_COUNTER: std::sync::atomic::AtomicUsize =
            std::sync::atomic::AtomicUsize::new(0);
        let cnt = JXL_STRIP_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let temp_png = std::env::temp_dir().join(format!(
            "konvert_jxl_clean_{}_{}.png",
            std::process::id(),
            cnt
        ));
        let _guard = TempFileGuard(temp_png.clone());

        let mut cmd = Command::new("djxl");
        cmd.arg(crate::converter::sanitize_cli_path(input_path))
            .arg(&temp_png)
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        let status = crate::process::run_tracked_status(&mut cmd).map_err(|e| {
            if crate::process::is_cancelled_error(&e) {
                e
            } else {
                e.context("djxl komutu çalıştırılamadı (JXL metadata temizleme)")
            }
        })?;
        if !status.success() || !temp_png.exists() {
            bail!("{}", crate::i18n::t("err_jxl_decode"));
        }
        return convert_to_jxl(&temp_png, &output_path, None, true);
    }

    // ImageMagick ile -auto-orient uygulayarak EXIF yönünü piksele göm, ardından tüm etiketleri temizle!
    let pending = crate::converter::PendingOutput::new(&output_path);
    let im_res = crate::converter::run_imagemagick(&[
        input_path.as_os_str(),
        std::ffi::OsStr::new("-auto-orient"),
        std::ffi::OsStr::new("-strip"),
        pending.path().as_os_str(),
    ]);
    if let Err(ref err) = im_res
        && crate::process::is_cancelled_error(err)
    {
        return Err(crate::process::cancelled_error());
    }
    if im_res.is_ok() && pending.path().exists() {
        if ext == "png" {
            optimize_png_with_oxipng(pending.path());
        }
        return pending.commit();
    }

    // Yedek: Yerel bellek içi kodlama
    let Some(fmt) = parse_image_format(&ext) else {
        bail!("Desteklenmeyen hedef formatı: .{}", ext);
    };
    let img = open_dynamic_image(input_path)?;
    crate::converter::write_file_atomic(&output_path, |writer| {
        match fmt {
            ImageFormat::Jpeg => {
                let rgb = flatten_to_rgb8(&img);
                let mut encoder = JpegEncoder::new_with_quality(writer, 95);
                encoder.encode_image(&rgb)?;
            }
            ImageFormat::Avif => encode_avif_with_quality(&img, 85, writer)?,
            _ => {
                img.write_to(writer, fmt)?;
            }
        }
        Ok(())
    })?;
    if fmt == ImageFormat::Png {
        optimize_png_with_oxipng(&output_path);
    }

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
    let out_ext = if ext == "svg" { "png" } else { ext.as_str() };

    let output_path = claim_output_path(determine_suffixed_output_path(
        input_path,
        "resized",
        out_ext,
        custom_output_dir,
    ));
    ensure_parent_dir(&output_path)?;

    if matches!(out_ext, "webp" | "heic" | "heif" | "gif" | "avif") {
        let im_resize_arg = match spec {
            crate::converter::ResizeSpec::Percent(pct) => format!("{}%", pct),
            crate::converter::ResizeSpec::Fit1080p => "1920x1080>".to_string(),
            crate::converter::ResizeSpec::Fit720p => "1280x720>".to_string(),
        };
        let pending = crate::converter::PendingOutput::new(&output_path);
        let im_res = crate::converter::run_imagemagick(&[
            input_path.as_os_str(),
            std::ffi::OsStr::new("-auto-orient"),
            std::ffi::OsStr::new("-resize"),
            std::ffi::OsStr::new(&im_resize_arg),
            pending.path().as_os_str(),
        ]);
        if let Err(ref err) = im_res
            && crate::process::is_cancelled_error(err)
        {
            return Err(crate::process::cancelled_error());
        }
        if im_res.is_ok() && pending.path().exists() {
            return pending.commit();
        }
    }

    let img = open_dynamic_image(input_path)?;
    let (w, h) = img.dimensions();
    let (new_w, new_h) = compute_resize_dimensions(w, h, spec);
    let resized = img.resize(new_w, new_h, imageops::FilterType::Lanczos3);

    if out_ext == "jxl" {
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
        let _guard = TempFileGuard(temp_png.clone());
        resized.save_with_format(&temp_png, ImageFormat::Png)?;
        return convert_to_jxl(&temp_png, &output_path, None, false);
    }

    let Some(fmt) = parse_image_format(out_ext) else {
        bail!("Desteklenmeyen hedef formatı: .{}", out_ext);
    };
    crate::converter::write_file_atomic(&output_path, |writer| {
        match fmt {
            ImageFormat::Jpeg => {
                let rgb = flatten_to_rgb8(&resized);
                let mut encoder = JpegEncoder::new_with_quality(writer, 90);
                encoder.encode_image(&rgb)?;
            }
            ImageFormat::Avif => encode_avif_with_quality(&resized, 80, writer)?,
            _ => {
                resized.write_to(writer, fmt)?;
            }
        }
        Ok(())
    })?;
    if fmt == ImageFormat::Png {
        optimize_png_with_oxipng(&output_path);
    }

    Ok(output_path)
}

fn build_gs_merge_command(inputs: &[PathBuf], output_path: &Path) -> Command {
    let mut cmd = Command::new("gs");
    cmd.args(["-dSAFER", "-dBATCH", "-dNOPAUSE", "-q", "-sDEVICE=pdfwrite"]);
    cmd.arg(format!("-sOutputFile={}", output_path.display()));
    cmd.arg("-f");
    for pdf in inputs {
        cmd.arg(crate::converter::sanitize_cli_path(pdf));
    }
    cmd
}

pub fn merge_images_to_pdf(
    images: &[PathBuf],
    custom_output_dir: Option<&Path>,
) -> Result<PathBuf> {
    if images.is_empty() {
        bail!("{}", crate::i18n::t("err_no_images_to_merge"));
    }

    let safe_images: Vec<PathBuf> = images
        .iter()
        .map(|p| crate::converter::sanitize_cli_path(p))
        .collect();

    let first_stem = images[0]
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("images");

    let target_dir = get_output_dir(&images[0], custom_output_dir);
    let output_path = claim_output_path(target_dir.join(format!("{}_merged.pdf", first_stem)));
    ensure_parent_dir(&output_path)?;

    let pending = crate::converter::PendingOutput::new(&output_path);

    // Girdilerin tamamı PDF ise ImageMagick yerine doğrudan Ghostscript kullan (policy.xml engeline takılmaz).
    let all_pdf = safe_images.iter().all(|f| {
        f.extension()
            .and_then(|e| e.to_str())
            .map(|e| e.eq_ignore_ascii_case("pdf"))
            .unwrap_or(false)
    });

    if all_pdf && crate::deps::command_exists("gs") {
        let mut cmd = build_gs_merge_command(&safe_images, pending.path());
        let status = crate::process::run_tracked_status(&mut cmd).map_err(|e| {
            if crate::process::is_cancelled_error(&e) {
                e
            } else {
                e.context("Ghostscript (gs) çalıştırılamadı")
            }
        })?;
        if !status.success() {
            bail!("{}", crate::i18n::t("err_gs_merge_pdf"));
        }
        return pending.commit();
    }

    // Yüzlerce dosya varsa ARG_MAX ve ImageMagick bellek sınırını aşmamak için 50'şerli parçalar halinde birleştir.
    if safe_images.len() > 50 {
        eprintln!(
            "Konvert: {} dosya tespit edildi; bellek ve argüman sınırını korumak için 50'şerli parçalar halinde birleştiriliyor...",
            safe_images.len()
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
        for (i, chunk) in safe_images.chunks(50).enumerate() {
            crate::process::wait_if_paused_or_cancelled()?;
            let chunk_pdf = temp_path.join(format!("chunk_{:04}.pdf", i));
            let mut chunk_args: Vec<std::ffi::OsString> = chunk
                .iter()
                .map(|img| img.as_os_str().to_os_string())
                .collect();
            chunk_args.push(std::ffi::OsStr::new("-auto-orient").to_os_string());
            chunk_args.push(chunk_pdf.as_os_str().to_os_string());
            crate::converter::run_imagemagick(&chunk_args).map_err(|e| {
                if crate::process::is_cancelled_error(&e) {
                    e
                } else {
                    e.context(format!("{}. parça PDF'e dönüştürülemedi", i + 1))
                }
            })?;
            chunk_pdfs.push(chunk_pdf);
        }

        if crate::deps::command_exists("gs") {
            let mut cmd = build_gs_merge_command(&chunk_pdfs, pending.path());
            let status = crate::process::run_tracked_status(&mut cmd).map_err(|e| {
                if crate::process::is_cancelled_error(&e) {
                    e
                } else {
                    e.context("Ghostscript çalıştırılamadı")
                }
            })?;
            if !status.success() {
                bail!("{}", crate::i18n::t("err_gs_merge_parts"));
            }
        } else {
            let mut final_args: Vec<std::ffi::OsString> = chunk_pdfs
                .iter()
                .map(|p| p.as_os_str().to_os_string())
                .collect();
            final_args.push(pending.path().as_os_str().to_os_string());
            crate::converter::run_imagemagick(&final_args).map_err(|e| {
                if crate::process::is_cancelled_error(&e) {
                    e
                } else {
                    e.context("Parça PDF'leri birleştirilemedi")
                }
            })?;
        }
        return pending.commit();
    }

    let mut args: Vec<std::ffi::OsString> = safe_images
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
        bail!("{}", crate::i18n::t("err_target_size_zero"));
    }

    let current_bytes = fs::metadata(input_path).map(|m| m.len()).unwrap_or(0);
    if current_bytes == 0 {
        bail!(
            "{}: {}",
            crate::i18n::t("err_file_empty_or_unreadable"),
            input_path.display()
        );
    }

    if current_bytes <= target_bytes {
        bail!(
            "{} ({} <= {}).",
            crate::i18n::t("err_file_already_smaller"),
            crate::history::format_bytes(current_bytes),
            crate::history::format_bytes(target_bytes)
        );
    }

    let ext = input_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("jpg")
        .to_lowercase();

    // Kayıpsız formatlar kaliteyle küçülmez; hedef küçükse WebP formatına geç
    let target_ext = if matches!(ext.as_str(), "png" | "bmp" | "tiff" | "tif" | "qoi")
        && current_bytes > target_bytes
    {
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

    let pending = crate::converter::PendingOutput::new(&output_path);
    let mut first_error: Option<anyhow::Error> = None;
    let mut last_reached_size: u64 = 0;

    let mut try_encode =
        |scale: Option<&str>, quality: Option<u8>, define: Option<&str>| -> Option<u64> {
            let mut args: Vec<std::ffi::OsString> = vec![
                input_path.as_os_str().to_os_string(),
                std::ffi::OsStr::new("-auto-orient").to_os_string(),
                std::ffi::OsStr::new("-strip").to_os_string(),
            ];
            if let Some(sc) = scale {
                args.push(std::ffi::OsStr::new("-resize").to_os_string());
                args.push(std::ffi::OsStr::new(sc).to_os_string());
            }
            if let Some(def) = define {
                args.push(std::ffi::OsStr::new("-define").to_os_string());
                args.push(std::ffi::OsStr::new(def).to_os_string());
            }
            if let Some(q) = quality {
                args.push(std::ffi::OsStr::new("-quality").to_os_string());
                args.push(std::ffi::OsStr::new(&q.to_string()).to_os_string());
            }
            args.push(pending.path().as_os_str().to_os_string());

            match crate::converter::run_imagemagick(&args) {
                Ok(()) => match fs::metadata(pending.path()) {
                    Ok(meta) if meta.len() > 0 => {
                        last_reached_size = meta.len();
                        Some(meta.len())
                    }
                    _ => None,
                },
                Err(err) => {
                    if first_error.is_none() {
                        first_error = Some(err);
                    }
                    None
                }
            }
        };

    // Tek geçişli boyut hedefleme (-define jpeg:extent / webp:target-size)
    let single_pass_define = if matches!(target_ext, "jpg" | "jpeg") {
        Some(format!("jpeg:extent={}B", target_bytes))
    } else if target_ext == "webp" {
        Some(format!("webp:target-size={}", target_bytes))
    } else {
        None
    };
    if let Some(ref def) = single_pass_define {
        let len_opt = try_encode(None, None, Some(def));
        if crate::process::is_job_cancelled() {
            return Err(crate::process::cancelled_error());
        }
        if let Some(len) = len_opt
            && len <= target_bytes
        {
            return pending.commit();
        }
    }

    // Yedek: ölçek basamakları içinde ikili arama ile en yüksek kaliteli sonucu bul
    let scales = [None, Some("75%"), Some("50%"), Some("35%")];
    for scale in scales {
        let high_opt = try_encode(scale, Some(82), None);
        if crate::process::is_job_cancelled() {
            return Err(crate::process::cancelled_error());
        }
        if let Some(len) = high_opt
            && len <= target_bytes
        {
            return pending.commit();
        }
        let low_opt = try_encode(scale, Some(35), None);
        if crate::process::is_job_cancelled() {
            return Err(crate::process::cancelled_error());
        }
        if let Some(len) = low_opt
            && len <= target_bytes
        {
            let mut low = 36u8;
            let mut high = 81u8;
            let mut best_q = 35u8;
            let mut last_q = 35u8;
            while low <= high {
                let mid = low + (high - low) / 2;
                last_q = mid;
                let mid_opt = try_encode(scale, Some(mid), None);
                if crate::process::is_job_cancelled() {
                    return Err(crate::process::cancelled_error());
                }
                if matches!(mid_opt, Some(l) if l <= target_bytes) {
                    best_q = mid;
                    low = mid + 1;
                } else {
                    high = mid - 1;
                }
            }
            if last_q != best_q {
                let _ = try_encode(scale, Some(best_q), None);
                if crate::process::is_job_cancelled() {
                    return Err(crate::process::cancelled_error());
                }
            }
            if fs::metadata(pending.path()).is_ok_and(|m| m.len() <= target_bytes) {
                return pending.commit();
            }
        }
    }

    // Hedefe ulaşılamadı: geçici dosya drop ile silinir, mevcut çıktı (varsa) korunur.
    if crate::process::is_job_cancelled() {
        return Err(crate::process::cancelled_error());
    }
    let reached_size = fs::metadata(pending.path())
        .map(|m| m.len())
        .unwrap_or(0)
        .max(last_reached_size);
    drop(pending);
    if reached_size == 0
        && let Some(err) = first_error
    {
        bail!(
            "{}: {:#}",
            crate::i18n::t("err_target_size_not_reached"),
            err
        );
    }
    bail!(
        "{}: {} -> {}",
        crate::i18n::t("err_target_size_not_reached"),
        crate::history::format_bytes(target_bytes),
        crate::history::format_bytes(reached_size)
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

    fn unique_test_dir(tag: &str) -> PathBuf {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let dir = std::env::temp_dir().join(format!(
            "konvert_img_test_{}_{}_{}",
            tag,
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn test_image_to_pdf_creates_pdf() {
        let temp_dir = unique_test_dir("to_pdf");
        let test_img = temp_dir.join("test_to_pdf.png");
        let img = DynamicImage::new_rgb8(10, 10);
        img.save(&test_img).unwrap();

        if crate::deps::imagemagick_available() {
            let res = image_to_pdf(&test_img, None);
            if let Ok(pdf_path) = res {
                assert!(pdf_path.exists());
                assert_eq!(pdf_path.extension().and_then(|e| e.to_str()), Some("pdf"));
            }
        }
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_merge_pdfs_via_gs_when_all_inputs_are_pdf() {
        if !crate::deps::command_exists("gs") {
            return;
        }
        let temp_dir = unique_test_dir("merge_pdfs");
        let img1 = temp_dir.join("merge_a.png");
        let img2 = temp_dir.join("merge_b.png");
        let _ = ::image::DynamicImage::new_rgb8(10, 10).save(&img1);
        let _ = ::image::DynamicImage::new_rgb8(10, 10).save(&img2);

        if crate::deps::imagemagick_available() {
            let pdf1 = image_to_pdf(&img1, None).unwrap();
            let pdf2 = image_to_pdf(&img2, None).unwrap();

            let merged = merge_images_to_pdf(&[pdf1.clone(), pdf2.clone()], None).unwrap();
            assert!(merged.exists());
            assert_eq!(merged.extension().and_then(|e| e.to_str()), Some("pdf"));
        }

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_compress_image_to_size_size_guards() {
        let temp_dir = unique_test_dir("size_guards");
        let test_file = temp_dir.join("small.jpg");
        std::fs::write(&test_file, b"fake small image data").unwrap();

        // 8MB hedef boyutu, dosya çok küçük -> hata vermeli
        let res = compress_image_to_size(&test_file, 8 * 1024 * 1024, None);
        assert!(res.is_err());
        let err_msg = res.unwrap_err().to_string();
        assert!(err_msg.contains("zaten hedef boyuttan") || err_msg.contains("already smaller"));

        // 0 hedef boyutu -> hata vermeli
        let res_zero = compress_image_to_size(&test_file, 0, None);
        assert!(res_zero.is_err());

        // Boş dosya -> hata vermeli
        let empty_file = temp_dir.join("empty.jpg");
        std::fs::write(&empty_file, b"").unwrap();
        let res_empty = compress_image_to_size(&empty_file, 8 * 1024 * 1024, None);
        assert!(res_empty.is_err());
        let err_empty = res_empty.unwrap_err().to_string();
        assert!(
            err_empty.contains("boş")
                || err_empty.contains("okunamadı")
                || err_empty.contains("empty")
        );

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_convert_image_to_svg_vectorization_or_fallback() {
        let temp_dir = std::env::temp_dir();
        let unique_id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let test_png = temp_dir.join(format!(
            "konvert_test_vector_{}_{}.png",
            std::process::id(),
            unique_id
        ));
        let img = ::image::DynamicImage::new_rgb8(32, 32);
        img.save(&test_png).expect("Failed to save test PNG");

        let svg_res = convert_image(&test_png, "svg", None, false, None);
        assert!(
            svg_res.is_ok(),
            "SVG conversion failed: {:?}",
            svg_res.err()
        );

        let svg_path = svg_res.unwrap();
        assert!(svg_path.exists());
        assert_eq!(svg_path.extension().and_then(|e| e.to_str()), Some("svg"));

        let content = std::fs::read_to_string(&svg_path).expect("Failed to read generated SVG");
        assert!(content.contains("<svg") || content.contains("SVG"));

        let _ = std::fs::remove_file(&test_png);
        let _ = std::fs::remove_file(&svg_path);
    }

    #[test]
    fn test_build_gs_merge_command_uses_safer_and_sanitizes_dash() {
        let cmd = build_gs_merge_command(
            &[PathBuf::from("-dNOSAFER"), PathBuf::from("doc.pdf")],
            Path::new("/tmp/out.pdf"),
        );
        let args: Vec<String> = cmd
            .get_args()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();
        assert!(args.contains(&"-dSAFER".to_string()));
        assert!(args.contains(&"-f".to_string()));
        assert!(args.contains(&"./-dNOSAFER".to_string()));
        assert!(!args.contains(&"-dNOSAFER".to_string()));
    }

    #[test]
    fn test_avif_decode_and_convert_if_imagemagick_present() {
        if !crate::deps::imagemagick_available() {
            return;
        }
        let temp_dir = std::env::temp_dir();
        let unique_id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let src_png = temp_dir.join(format!(
            "konvert_avif_src_{}_{}.png",
            std::process::id(),
            unique_id
        ));
        let avif_file = temp_dir.join(format!(
            "konvert_avif_in_{}_{}.avif",
            std::process::id(),
            unique_id
        ));

        let img = ::image::DynamicImage::new_rgb8(32, 32);
        img.save(&src_png).expect("save src png");
        if crate::converter::run_imagemagick(&[src_png.as_os_str(), avif_file.as_os_str()]).is_ok()
            && avif_file.exists()
        {
            let png_res = convert_image(&avif_file, "png", Some(85), false, None);
            assert!(
                png_res.is_ok(),
                "AVIF -> PNG decode failed: {:?}",
                png_res.err()
            );
            if let Ok(p) = png_res {
                let _ = std::fs::remove_file(p);
            }
        }
        let _ = std::fs::remove_file(&src_png);
        let _ = std::fs::remove_file(&avif_file);
    }

    #[test]
    fn test_flatten_to_rgb8_blends_alpha_over_white() {
        let mut rgba = image::RgbaImage::new(3, 1);
        rgba.put_pixel(0, 0, image::Rgba([0, 0, 0, 0]));
        rgba.put_pixel(1, 0, image::Rgba([200, 0, 0, 128]));
        rgba.put_pixel(2, 0, image::Rgba([255, 0, 0, 128]));
        let rgb = flatten_to_rgb8(&DynamicImage::ImageRgba8(rgba));
        assert_eq!(rgb.get_pixel(0, 0).0, [255, 255, 255]);
        let blended = rgb.get_pixel(1, 0).0;
        assert!(blended[0] > 210 && blended[1] > 110 && blended[2] > 110);
        let red_50 = rgb.get_pixel(2, 0).0;
        assert_eq!(red_50[0], 255);
        assert!((126..=129).contains(&red_50[1]));
        assert!((126..=129).contains(&red_50[2]));
    }

    #[test]
    fn test_compute_resize_dimensions_respects_both_bounds() {
        assert_eq!(
            compute_resize_dimensions(1920, 1440, &crate::converter::ResizeSpec::Fit1080p),
            (1440, 1080)
        );
        assert_eq!(
            compute_resize_dimensions(1440, 1920, &crate::converter::ResizeSpec::Fit1080p),
            (810, 1080)
        );
        assert_eq!(
            compute_resize_dimensions(6000, 4000, &crate::converter::ResizeSpec::Fit1080p),
            (1620, 1080)
        );
        assert_eq!(
            compute_resize_dimensions(4000, 6000, &crate::converter::ResizeSpec::Fit1080p),
            (720, 1080)
        );
        assert_eq!(
            compute_resize_dimensions(3840, 2160, &crate::converter::ResizeSpec::Fit1080p),
            (1920, 1080)
        );
        assert_eq!(
            compute_resize_dimensions(1080, 1920, &crate::converter::ResizeSpec::Fit1080p),
            (608, 1080)
        );
        assert_eq!(
            compute_resize_dimensions(800, 600, &crate::converter::ResizeSpec::Fit1080p),
            (800, 600)
        );
    }
}
