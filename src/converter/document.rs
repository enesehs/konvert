use anyhow::{bail, Context, Result};
use std::fs::{self, File};
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static TEMP_DOC_COUNTER: AtomicUsize = AtomicUsize::new(0);

#[allow(dead_code)]
pub fn is_supported_document_target(target: &str) -> bool {
    matches!(
        target.to_lowercase().as_str(),
        "pdf" | "txt" | "html" | "docx" | "png" | "jpg"
    )
}

pub fn convert_document(
    input_path: &Path,
    target_ext: &str,
    dpi: Option<u32>,
    custom_output_dir: Option<&Path>,
) -> Result<PathBuf> {
    let target = target_ext.to_lowercase();
    let ext = input_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    // 1. PDF -> Görseller (pdftoppm)
    if ext == "pdf" && (target == "png" || target == "jpg") {
        return convert_pdf_to_images(input_path, &target, dpi.unwrap_or(300), custom_output_dir);
    }

    // 2. UDF (UYAP Dokümanı) Dönüştürme
    if ext == "udf" {
        return convert_udf(input_path, &target, custom_output_dir);
    }

    // 3. DOCX Dönüştürme
    if ext == "docx" {
        return convert_docx(input_path, &target, custom_output_dir);
    }

    // 4. Genel Metin / HTML
    let output_path = crate::converter::image::determine_output_path(
        input_path,
        &target,
        custom_output_dir,
    );
    crate::converter::image::ensure_parent_dir(&output_path)?;

    let content = fs::read_to_string(input_path)
        .with_context(|| format!("Belge okunamadı: {}", input_path.display()))?;

    fs::write(&output_path, content)?;
    Ok(output_path)
}

pub fn convert_pdf_to_images(
    input_path: &Path,
    img_format: &str,
    dpi: u32,
    custom_output_dir: Option<&Path>,
) -> Result<PathBuf> {
    let file_stem = input_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("pdf_pages");

    let base_dir = match custom_output_dir {
        Some(d) => d.to_path_buf(),
        None => input_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf(),
    };

    let pages_dir = base_dir.join(format!("{}_sayfalar", file_stem));
    fs::create_dir_all(&pages_dir)?;

    let output_prefix = pages_dir.join("sayfa");

    let mut cmd = Command::new("pdftoppm");
    if img_format == "png" {
        cmd.arg("-png");
    } else {
        cmd.arg("-jpeg");
    }
    cmd.arg("-r")
        .arg(dpi.to_string())
        .arg(input_path)
        .arg(output_prefix);

    let status = cmd.status().with_context(|| "pdftoppm bulunamadı")?;
    if !status.success() {
        bail!("PDF sayfaları görsele dönüştürülemedi");
    }

    Ok(pages_dir)
}

pub fn convert_udf(
    input_path: &Path,
    target_format: &str,
    custom_output_dir: Option<&Path>,
) -> Result<PathBuf> {
    let output_path = crate::converter::image::determine_output_path(
        input_path,
        target_format,
        custom_output_dir,
    );
    crate::converter::image::ensure_parent_dir(&output_path)?;

    let mut xml_content = String::new();

    let file = BufReader::new(File::open(input_path)?);
    if let Ok(mut archive) = zip::ZipArchive::new(file)
        && let Ok(mut zip_file) = archive.by_name("content.xml")
    {
        let _ = zip_file.read_to_string(&mut xml_content);
    }

    if xml_content.is_empty() {
        xml_content = fs::read_to_string(input_path).unwrap_or_default();
    }

    let text_content = extract_text_from_xml(&xml_content);

    match target_format {
        "pdf" => {
            // UDF içeriğini doğrudan PDF dosyasına bas
            let count = TEMP_DOC_COUNTER.fetch_add(1, Ordering::Relaxed);
            let temp_txt = std::env::temp_dir().join(format!("udf_temp_{}_{}.txt", std::process::id(), count));
            fs::write(&temp_txt, &text_content)?;

            let status = Command::new("magick")
                .arg("-size")
                .arg("800x1100")
                .arg("xc:white")
                .arg("-pointsize")
                .arg("12")
                .arg("-fill")
                .arg("black")
                .arg("-annotate")
                .arg("+50+60")
                .arg(format!("@{}", temp_txt.display()))
                .arg(&output_path)
                .status()
                .or_else(|_| {
                    Command::new("convert")
                        .arg("-size")
                        .arg("800x1100")
                        .arg("xc:white")
                        .arg("-pointsize")
                        .arg("12")
                        .arg("-fill")
                        .arg("black")
                        .arg("-annotate")
                        .arg("+50+60")
                        .arg(format!("@{}", temp_txt.display()))
                        .arg(&output_path)
                        .status()
                });

            let _ = fs::remove_file(temp_txt);

            match status {
                Ok(s) if s.success() => Ok(output_path),
                _ => {
                    // Fallback to txt
                    let txt_path = output_path.with_extension("txt");
                    fs::write(&txt_path, text_content)?;
                    Ok(txt_path)
                }
            }
        }
        "html" => {
            let html = format!(
                "<!DOCTYPE html><html><head><meta charset=\"utf-8\"><title>UYAP UDF Belgesi</title>\
                <style>body{{font-family:sans-serif;padding:3rem;line-height:1.6;white-space:pre-wrap;background:#fdfdfd;color:#222;}}\
                .header{{border-bottom:2px solid #ccc;padding-bottom:10px;margin-bottom:20px;font-weight:bold;}}</style>\
                </head><body><div class=\"header\">T.C. ADALET BAKANLIĞI - UYAP DOKÜMANI</div>{}</body></html>",
                html_escape::encode_text(&text_content)
            );
            fs::write(&output_path, html)?;
            Ok(output_path)
        }
        _ => {
            fs::write(&output_path, text_content)?;
            Ok(output_path)
        }
    }
}

pub fn convert_docx(
    input_path: &Path,
    target_format: &str,
    custom_output_dir: Option<&Path>,
) -> Result<PathBuf> {
    let output_path = crate::converter::image::determine_output_path(
        input_path,
        target_format,
        custom_output_dir,
    );
    crate::converter::image::ensure_parent_dir(&output_path)?;

    let file = BufReader::new(File::open(input_path)?);
    let mut archive = zip::ZipArchive::new(file)?;
    let mut xml_content = String::new();

    if let Ok(mut doc) = archive.by_name("word/document.xml") {
        doc.read_to_string(&mut xml_content)?;
    }

    let text_content = extract_text_from_xml(&xml_content);
    fs::write(&output_path, text_content)?;

    Ok(output_path)
}

fn extract_text_from_xml(xml: &str) -> String {
    let mut text = String::with_capacity(xml.len() / 2);
    let mut inside_tag = false;

    for ch in xml.chars() {
        match ch {
            '<' => inside_tag = true,
            '>' => {
                inside_tag = false;
                text.push(' ');
            }
            _ => {
                if !inside_tag {
                    text.push(ch);
                }
            }
        }
    }

    text.lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n")
}

/// Akıllı PDF sıkıştırma (Ghostscript ebook önayarı ve ImageMagick yedeklemesi)
pub fn compress_pdf(input_path: &Path, custom_output_dir: Option<&Path>) -> Result<PathBuf> {
    let file_stem = input_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("document");

    let target_dir = match custom_output_dir {
        Some(dir) => dir.to_path_buf(),
        None => input_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf(),
    };

    let output_path = target_dir.join(format!("{}_compressed.pdf", file_stem));
    crate::converter::image::ensure_parent_dir(&output_path)?;

    let gs_status = Command::new("gs")
        .arg("-sDEVICE=pdfwrite")
        .arg("-dCompatibilityLevel=1.4")
        .arg("-dPDFSETTINGS=/ebook")
        .arg("-dNOPAUSE")
        .arg("-dQUIET")
        .arg("-dBATCH")
        .arg(format!("-sOutputFile={}", output_path.display()))
        .arg(input_path)
        .status();

    if let Ok(st) = gs_status
        && st.success()
        && output_path.exists()
    {
        return Ok(output_path);
    }

    let fb_status = Command::new("magick")
        .arg("-density")
        .arg("150")
        .arg(input_path)
        .arg("-quality")
        .arg("80")
        .arg(&output_path)
        .status()
        .with_context(|| "PDF sıkıştırıcı (gs/magick) çalıştırılamadı")?;

    if !fb_status.success() {
        bail!("PDF sıkıştırma başarısız oldu");
    }

    Ok(output_path)
}

mod html_escape {
    pub fn encode_text(input: &str) -> String {
        input
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
    }
}
