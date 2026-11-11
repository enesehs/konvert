use anyhow::{bail, Context, Result};
use std::fs::{self, File};
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static TEMP_DOC_COUNTER: AtomicUsize = AtomicUsize::new(0);

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

    let base_dir = crate::converter::get_output_dir(input_path, custom_output_dir);
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
    let output_path = crate::converter::determine_output_path(
        input_path,
        target_format,
        custom_output_dir,
    );
    crate::converter::ensure_parent_dir(&output_path)?;

    let mut xml_content = String::new();

    let file = BufReader::new(File::open(input_path)?);
    if let Ok(mut archive) = zip::ZipArchive::new(file)
        && let Ok(zip_file) = archive.by_name("content.xml")
    {
        zip_file.take(50 * 1024 * 1024).read_to_string(&mut xml_content)?;
    }

    if xml_content.is_empty() {
        xml_content = fs::read_to_string(input_path).unwrap_or_default();
    }

    let text_content = extract_text_from_xml(&xml_content);
    if text_content.is_empty() {
        bail!("UDF belgesinden metin okunamadı");
    }

    match target_format {
        "html" => {
            let paragraphs = text_content
                .split("\n\n")
                .map(|p| format!("<p>{}</p>", html_escape::encode_text(p)))
                .collect::<Vec<_>>()
                .join("\n");

            let html = format!(
                "<!DOCTYPE html><html><head><meta charset=\"utf-8\"><title>UYAP UDF Belgesi</title>\
                <style>body{{font-family:sans-serif;padding:3rem;line-height:1.6;background:#fdfdfd;color:#222;max-width:800px;margin:auto;}}\
                .header{{border-bottom:2px solid #ccc;padding-bottom:10px;margin-bottom:20px;font-weight:bold;}}\
                p{{margin-bottom:1em;}}</style>\
                </head><body><div class=\"header\">T.C. ADALET BAKANLIĞI - UYAP DOKÜMANI</div>{}</body></html>",
                paragraphs
            );
            fs::write(&output_path, html)?;
            Ok(output_path)
        }
        "txt" | "text" => {
            fs::write(&output_path, text_content)?;
            Ok(output_path)
        }
        "pdf" => {
            let target_dir = crate::converter::get_output_dir(input_path, custom_output_dir);
            let count = TEMP_DOC_COUNTER.fetch_add(1, Ordering::Relaxed);
            let temp_html = std::env::temp_dir().join(format!(
                "udf_temp_{}_{}_{}.html",
                std::process::id(),
                chrono::Local::now().timestamp_nanos_opt().unwrap_or(0),
                count
            ));

            let paragraphs = text_content
                .split("\n\n")
                .map(|p| format!("<p>{}</p>", html_escape::encode_text(p)))
                .collect::<Vec<_>>()
                .join("\n");

            let html = format!(
                "<!DOCTYPE html><html><head><meta charset=\"utf-8\"><title>UYAP UDF Belgesi</title>\
                <style>body{{font-family:sans-serif;padding:2rem;line-height:1.5;color:#222;}}\
                .header{{border-bottom:2px solid #ccc;padding-bottom:10px;margin-bottom:20px;font-weight:bold;}}\
                p{{margin-bottom:1em;}}</style>\
                </head><body><div class=\"header\">T.C. ADALET BAKANLIĞI - UYAP DOKÜMANI</div>{}</body></html>",
                paragraphs
            );
            fs::write(&temp_html, html)?;

            let status = Command::new("soffice")
                .arg("--headless")
                .arg("--convert-to")
                .arg("pdf")
                .arg("--outdir")
                .arg(&target_dir)
                .arg(&temp_html)
                .status();

            let _ = fs::remove_file(&temp_html);

            let intermediate_pdf = target_dir.join(format!("{}.pdf", temp_html.file_stem().unwrap().to_string_lossy()));
            if let Ok(st) = status
                && st.success()
                && intermediate_pdf.exists()
            {
                fs::rename(&intermediate_pdf, &output_path)?;
                return Ok(output_path);
            }

            bail!("UDF -> PDF dönüşümü için sisteminizde LibreOffice (soffice) kurulu olmalıdır. Belgeyi doğrudan TXT veya HTML formatına dönüştürebilirsiniz.");
        }
        _ => {
            bail!("UDF dosyası için desteklenmeyen hedef format: .{}", target_format);
        }
    }
}

pub fn convert_docx(
    input_path: &Path,
    target_format: &str,
    custom_output_dir: Option<&Path>,
) -> Result<PathBuf> {
    let output_path = crate::converter::determine_output_path(
        input_path,
        target_format,
        custom_output_dir,
    );
    crate::converter::ensure_parent_dir(&output_path)?;

    if target_format == "pdf" {
        let target_dir = crate::converter::get_output_dir(input_path, custom_output_dir);
        let status = Command::new("soffice")
            .arg("--headless")
            .arg("--convert-to")
            .arg("pdf")
            .arg("--outdir")
            .arg(&target_dir)
            .arg(input_path)
            .status();

        if let Ok(st) = status
            && st.success()
            && output_path.exists()
        {
            return Ok(output_path);
        }

        bail!("DOCX -> PDF dönüşümü için sisteminizde LibreOffice (soffice) kurulu olmalıdır");
    }

    let file = BufReader::new(File::open(input_path)?);
    let mut archive = zip::ZipArchive::new(file)?;
    let mut xml_content = String::new();

    if let Ok(doc) = archive.by_name("word/document.xml") {
        doc.take(50 * 1024 * 1024).read_to_string(&mut xml_content)?;
    } else {
        bail!("DOCX belgesi içinde word/document.xml bulunamadı");
    }

    let text_content = extract_text_from_xml(&xml_content);
    if text_content.is_empty() {
        bail!("DOCX belgesinden metin okunamadı");
    }

    match target_format {
        "html" => {
            let paragraphs = text_content
                .split("\n\n")
                .map(|p| format!("<p>{}</p>", html_escape::encode_text(p)))
                .collect::<Vec<_>>()
                .join("\n");

            let html = format!(
                "<!DOCTYPE html><html><head><meta charset=\"utf-8\"><title>Doküman</title>\
                <style>body{{font-family:sans-serif;padding:3rem;line-height:1.6;background:#fdfdfd;color:#222;max-width:800px;margin:auto;}}\
                p{{margin-bottom:1em;}}</style>\
                </head><body>{}</body></html>",
                paragraphs
            );
            fs::write(&output_path, html)?;
            Ok(output_path)
        }
        "txt" | "text" => {
            fs::write(&output_path, text_content)?;
            Ok(output_path)
        }
        _ => {
            bail!("DOCX dosyası için desteklenmeyen hedef format: .{}", target_format);
        }
    }
}

fn unescape_xml(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '&' {
            let mut entity = String::with_capacity(8);
            let mut has_semicolon = false;
            for next_ch in chars.by_ref() {
                if next_ch == ';' {
                    has_semicolon = true;
                    break;
                }
                entity.push(next_ch);
                if entity.len() > 8 || next_ch.is_whitespace() || next_ch == '<' {
                    break;
                }
            }
            if has_semicolon {
                match entity.as_str() {
                    "amp" => out.push('&'),
                    "lt" => out.push('<'),
                    "gt" => out.push('>'),
                    "quot" => out.push('"'),
                    "apos" => out.push('\''),
                    "nbsp" => out.push(' '),
                    _ if entity.starts_with('#') => {
                        if let Some(rest) = entity.strip_prefix("#x") {
                            if let Ok(code) = u32::from_str_radix(rest, 16)
                                && let Some(c) = char::from_u32(code)
                            {
                                out.push(c);
                                continue;
                            }
                        } else if let Some(rest) = entity.strip_prefix('#')
                            && let Ok(code) = rest.parse::<u32>()
                            && let Some(c) = char::from_u32(code)
                        {
                            out.push(c);
                            continue;
                        }
                        out.push('&');
                        out.push_str(&entity);
                        out.push(';');
                    }
                    _ => {
                        out.push('&');
                        out.push_str(&entity);
                        out.push(';');
                    }
                }
            } else {
                out.push('&');
                out.push_str(&entity);
            }
        } else {
            out.push(ch);
        }
    }
    out
}

pub fn extract_text_from_xml(xml: &str) -> String {
    let mut result = String::with_capacity(xml.len() / 2);
    let mut chars = xml.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '<' {
            let mut tag_peek = String::new();
            while let Some(&next_ch) = chars.peek() {
                if next_ch == '>' || tag_peek.len() >= 8 {
                    break;
                }
                tag_peek.push(next_ch);
                chars.next();
            }

            if tag_peek.starts_with("![CDATA[") {
                let mut cdata = String::new();
                let mut end_count = 0;
                for c in chars.by_ref() {
                    if c == ']' {
                        end_count += 1;
                    } else if c == '>' && end_count >= 2 {
                        break;
                    } else {
                        while end_count > 0 {
                            cdata.push(']');
                            end_count -= 1;
                        }
                        cdata.push(c);
                    }
                }
                result.push_str(&cdata);
                continue;
            }

            let mut tag_name = tag_peek;
            for c in chars.by_ref() {
                if c == '>' {
                    break;
                }
                tag_name.push(c);
            }

            let lower_tag = tag_name.to_ascii_lowercase();
            if lower_tag.starts_with("/w:p")
                || lower_tag.starts_with("/p")
                || lower_tag.starts_with("w:br")
                || lower_tag.starts_with("br")
                || lower_tag.starts_with("/tr")
            {
                result.push('\n');
            } else if lower_tag.starts_with("w:tab") {
                result.push('\t');
            }
        } else {
            result.push(ch);
        }
    }

    let unescaped = unescape_xml(&result);
    unescaped
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n")
}

pub fn compress_pdf(input_path: &Path, custom_output_dir: Option<&Path>) -> Result<PathBuf> {
    let file_stem = input_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("document");

    let target_dir = crate::converter::get_output_dir(input_path, custom_output_dir);
    let output_path = target_dir.join(format!("{}_compressed.pdf", file_stem));
    crate::converter::ensure_parent_dir(&output_path)?;

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
        .or_else(|_| {
            Command::new("convert")
                .arg("-density")
                .arg("150")
                .arg(input_path)
                .arg("-quality")
                .arg("80")
                .arg(&output_path)
                .status()
        })
        .with_context(|| "PDF sıkıştırıcı (gs veya magick) çalıştırılamadı")?;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_text_from_xml() {
        let xml = "<w:p><w:r><w:t>Hello</w:t></w:r><w:r><w:t> World</w:t></w:r></w:p><w:p><w:r><w:t>Second &amp; Line</w:t></w:r></w:p>";
        let text = extract_text_from_xml(xml);
        assert_eq!(text, "Hello World\n\nSecond & Line");
    }

    #[test]
    fn test_extract_text_cdata() {
        let xml = "<content><![CDATA[Mahkeme & Karari]]></content>";
        let text = extract_text_from_xml(xml);
        assert_eq!(text, "Mahkeme & Karari");
    }
}
