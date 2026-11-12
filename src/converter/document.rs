use anyhow::{Context, Result, bail};
use std::fs::{self, File};
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

static TEMP_DOC_COUNTER: AtomicUsize = AtomicUsize::new(0);
static SOFFICE_MUTEX: Mutex<()> = Mutex::new(());

pub fn convert_pdf_to_images(
    input_path: &Path,
    img_format: &str,
    dpi: u32,
    max_pages: u32,
    custom_output_dir: Option<&Path>,
) -> Result<PathBuf> {
    let file_stem = input_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("pdf_pages");

    let base_dir = crate::converter::get_output_dir(input_path, custom_output_dir);
    let pages_dir =
        crate::converter::claim_output_path(base_dir.join(format!("{}_pages", file_stem)));
    fs::create_dir_all(&pages_dir)?;

    let output_prefix = pages_dir.join("page");

    let mut cmd = Command::new("pdftoppm");
    if img_format.eq_ignore_ascii_case("png") {
        cmd.arg("-png");
    } else {
        cmd.arg("-jpeg");
    }
    cmd.arg("-r").arg(dpi.to_string());

    // Sayfa sayısı sınırsız büyümesin: 0 = sınırsız, aksi halde ilk N sayfa.
    if max_pages > 0 {
        cmd.arg("-l").arg(max_pages.to_string());
    }

    cmd.arg(input_path).arg(output_prefix);

    let status = cmd.status().with_context(|| "pdftoppm bulunamadı")?;
    if !status.success() {
        bail!("PDF sayfaları görsele dönüştürülemedi");
    }

    Ok(pages_dir)
}

pub fn convert_to_pdf_via_soffice(
    input_path: &Path,
    output_path: &Path,
    target_dir: &Path,
) -> Result<PathBuf> {
    let count = TEMP_DOC_COUNTER.fetch_add(1, Ordering::Relaxed);
    let temp_profile = std::env::temp_dir().join(format!(
        "konvert_soffice_{}_{}_{}",
        std::process::id(),
        chrono::Local::now().timestamp_nanos_opt().unwrap_or(0),
        count
    ));
    let _ = fs::create_dir_all(&temp_profile);

    // Paralel LibreOffice kilit çatışmasını önlemek için süreç içi kilit ve bağımsız profil
    let _lock = SOFFICE_MUTEX.lock();
    let status = Command::new("soffice")
        .arg(format!(
            "-env:UserInstallation=file://{}",
            temp_profile.display()
        ))
        .arg("--headless")
        .arg("--convert-to")
        .arg("pdf")
        .arg("--outdir")
        .arg(target_dir)
        .arg(input_path)
        .status();

    let _ = fs::remove_dir_all(&temp_profile);

    if let Ok(st) = status
        && st.success()
        && output_path.exists()
    {
        return Ok(output_path.to_path_buf());
    }

    bail!(
        "LibreOffice (soffice) ile PDF dönüşümü başarısız oldu veya soffice sistemde kurulu değil"
    )
}

pub fn write_html_document(
    title: &str,
    header: Option<&str>,
    text_content: &str,
    output_path: &Path,
) -> Result<PathBuf> {
    let paragraphs = text_content
        .split("\n\n")
        .map(|p| format!("<p>{}</p>", html_escape::encode_text(p)))
        .collect::<Vec<_>>()
        .join("\n");

    let header_html = if let Some(h) = header {
        format!(
            "<div class=\"header\">{}</div>",
            html_escape::encode_text(h)
        )
    } else {
        String::new()
    };

    let html = format!(
        "<!DOCTYPE html><html><head><meta charset=\"utf-8\"><title>{}</title>\
        <style>body{{font-family:sans-serif;padding:3rem;line-height:1.6;background:#fdfdfd;color:#222;max-width:800px;margin:auto;}}\
        .header{{border-bottom:2px solid #ccc;padding-bottom:10px;margin-bottom:20px;font-weight:bold;}}\
        p{{margin-bottom:1em;}}</style>\
        </head><body>{}{}</body></html>",
        html_escape::encode_text(title),
        header_html,
        paragraphs
    );
    let pending = crate::converter::PendingOutput::new(output_path);
    fs::write(pending.path(), html)?;
    pending.commit()
}

const MAX_DOC_XML_BYTES: u64 = 50 * 1024 * 1024;

fn read_limited_xml<R: Read>(mut reader: R, max_bytes: u64, path_hint: &Path) -> Result<String> {
    let mut buf = String::new();
    reader
        .by_ref()
        .take(max_bytes + 1)
        .read_to_string(&mut buf)?;
    if buf.len() as u64 > max_bytes {
        bail!(
            "Belge boyutu güvenlik sınırını aşıyor (maksimum {} MB): {}",
            max_bytes / (1024 * 1024),
            path_hint.display()
        );
    }
    Ok(buf)
}

pub fn convert_udf(
    input_path: &Path,
    target_format: &str,
    custom_output_dir: Option<&Path>,
) -> Result<PathBuf> {
    let output_path = crate::converter::claim_output_path(crate::converter::determine_output_path(
        input_path,
        target_format,
        custom_output_dir,
    ));
    crate::converter::ensure_parent_dir(&output_path)?;

    let mut xml_content = String::new();

    let file = BufReader::new(File::open(input_path)?);
    if let Ok(mut archive) = zip::ZipArchive::new(file)
        && let Ok(zip_file) = archive.by_name("content.xml")
    {
        xml_content = read_limited_xml(zip_file, MAX_DOC_XML_BYTES, input_path)?;
    }

    // ZIP olmayan düz dosya yedeğinde DoS saldırısını önlemek için 50 MB sınır koy
    if xml_content.is_empty() {
        let f = BufReader::new(File::open(input_path)?);
        xml_content = read_limited_xml(f, MAX_DOC_XML_BYTES, input_path)?;
    }

    let text_content = extract_text_from_xml(&xml_content);
    if text_content.is_empty() {
        bail!("UDF belgesinden metin okunamadı");
    }

    match target_format.to_lowercase().as_str() {
        "html" => write_html_document(
            "UYAP UDF Belgesi",
            Some("T.C. ADALET BAKANLIĞI - UYAP DOKÜMANI"),
            &text_content,
            &output_path,
        ),
        "txt" | "text" => {
            let pending = crate::converter::PendingOutput::new(&output_path);
            fs::write(pending.path(), text_content)?;
            pending.commit()
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

            write_html_document(
                "UYAP UDF Belgesi",
                Some("T.C. ADALET BAKANLIĞI - UYAP DOKÜMANI"),
                &text_content,
                &temp_html,
            )?;

            let res = convert_to_pdf_via_soffice(&temp_html, &output_path, &target_dir);
            let _ = fs::remove_file(&temp_html);

            let intermediate_pdf = target_dir.join(format!(
                "{}.pdf",
                temp_html.file_stem().unwrap().to_string_lossy()
            ));
            if intermediate_pdf.exists() && intermediate_pdf != output_path {
                let _ = fs::rename(&intermediate_pdf, &output_path);
            }

            if output_path.exists() {
                Ok(output_path)
            } else {
                res.with_context(|| "UDF -> PDF dönüşümü için sisteminizde LibreOffice (soffice) kurulu olmalıdır")
            }
        }
        _ => {
            bail!(
                "UDF dosyası için desteklenmeyen hedef format: .{}",
                target_format
            );
        }
    }
}

pub fn convert_docx(
    input_path: &Path,
    target_format: &str,
    custom_output_dir: Option<&Path>,
) -> Result<PathBuf> {
    let output_path = crate::converter::claim_output_path(crate::converter::determine_output_path(
        input_path,
        target_format,
        custom_output_dir,
    ));
    crate::converter::ensure_parent_dir(&output_path)?;

    if target_format.eq_ignore_ascii_case("pdf") {
        let target_dir = crate::converter::get_output_dir(input_path, custom_output_dir);
        return convert_to_pdf_via_soffice(input_path, &output_path, &target_dir);
    }

    let file = BufReader::new(File::open(input_path)?);
    let mut archive = zip::ZipArchive::new(file)?;

    // Belge bileşenlerini tara: üstbilgi, ana metin, dipnotlar ve altbilgiler
    let mut header_names = Vec::new();
    let mut footer_names = Vec::new();
    let mut footnote_names = Vec::new();

    for name in archive.file_names().flatten() {
        if name.starts_with("word/header") && name.ends_with(".xml") {
            header_names.push(name.to_string());
        } else if name.starts_with("word/footer") && name.ends_with(".xml") {
            footer_names.push(name.to_string());
        } else if name.starts_with("word/footnotes") && name.ends_with(".xml") {
            footnote_names.push(name.to_string());
        }
    }
    header_names.sort();
    footer_names.sort();
    footnote_names.sort();

    let mut full_text_sections = Vec::new();

    // 1. Üstbilgiler (Headers)
    for h_name in &header_names {
        if let Ok(h_file) = archive.by_name(h_name)
            && let Ok(h_xml) = read_limited_xml(h_file, MAX_DOC_XML_BYTES, input_path)
        {
            let h_text = extract_text_from_xml(&h_xml);
            if !h_text.is_empty() {
                full_text_sections.push(h_text);
            }
        }
    }

    // 2. Ana Belge (word/document.xml)
    let doc_file = archive
        .by_name("word/document.xml")
        .context("DOCX belgesi içinde word/document.xml bulunamadı")?;
    let main_xml = read_limited_xml(doc_file, MAX_DOC_XML_BYTES, input_path)?;
    let main_text = extract_text_from_xml(&main_xml);
    if main_text.is_empty() {
        bail!("DOCX belgesinden metin okunamadı: {}", input_path.display());
    }
    full_text_sections.push(main_text);

    // 3. Dipnotlar (Footnotes)
    for fn_name in &footnote_names {
        if let Ok(fn_file) = archive.by_name(fn_name)
            && let Ok(fn_xml) = read_limited_xml(fn_file, MAX_DOC_XML_BYTES, input_path)
        {
            let fn_text = extract_text_from_xml(&fn_xml);
            if !fn_text.is_empty() {
                full_text_sections.push(format!("--- Dipnotlar ---\n{}", fn_text));
            }
        }
    }

    // 4. Altbilgiler (Footers)
    for f_name in &footer_names {
        if let Ok(f_file) = archive.by_name(f_name)
            && let Ok(f_xml) = read_limited_xml(f_file, MAX_DOC_XML_BYTES, input_path)
        {
            let f_text = extract_text_from_xml(&f_xml);
            if !f_text.is_empty() {
                full_text_sections.push(f_text);
            }
        }
    }

    let text_content = full_text_sections.join("\n\n");

    match target_format.to_lowercase().as_str() {
        "html" => write_html_document("Doküman", None, &text_content, &output_path),
        "txt" | "text" => {
            let pending = crate::converter::PendingOutput::new(&output_path);
            fs::write(pending.path(), text_content)?;
            pending.commit()
        }
        _ => {
            bail!(
                "DOCX dosyası için desteklenmeyen hedef format: .{}",
                target_format
            );
        }
    }
}

fn unescape_xml(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let chars: Vec<char> = input.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        if chars[i] == '&' {
            let mut semi_pos = None;
            for (offset, &ch) in chars[(i + 1)..len.min(i + 10)].iter().enumerate() {
                if ch == ';' {
                    semi_pos = Some(i + 1 + offset);
                    break;
                }
                if ch.is_whitespace() || ch == '<' || ch == '&' {
                    break;
                }
            }

            if let Some(pos) = semi_pos {
                let entity: String = chars[(i + 1)..pos].iter().collect();
                let lower = entity.to_ascii_lowercase();
                match lower.as_str() {
                    "amp" => {
                        out.push('&');
                        i = pos + 1;
                        continue;
                    }
                    "lt" => {
                        out.push('<');
                        i = pos + 1;
                        continue;
                    }
                    "gt" => {
                        out.push('>');
                        i = pos + 1;
                        continue;
                    }
                    "quot" => {
                        out.push('"');
                        i = pos + 1;
                        continue;
                    }
                    "apos" => {
                        out.push('\'');
                        i = pos + 1;
                        continue;
                    }
                    "nbsp" => {
                        out.push(' ');
                        i = pos + 1;
                        continue;
                    }
                    _ if lower.starts_with("#x") => {
                        if let Ok(code) = u32::from_str_radix(&lower[2..], 16)
                            && let Some(c) = char::from_u32(code)
                        {
                            out.push(c);
                            i = pos + 1;
                            continue;
                        }
                    }
                    _ if lower.starts_with('#') => {
                        if let Ok(code) = lower[1..].parse::<u32>()
                            && let Some(c) = char::from_u32(code)
                        {
                            out.push(c);
                            i = pos + 1;
                            continue;
                        }
                    }
                    _ => {}
                }
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

pub fn extract_text_from_xml(xml: &str) -> String {
    let mut result = String::with_capacity(xml.len() / 2);
    let mut chars = xml.chars().peekable();
    let mut in_table_cell = false;

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

            // Etiket sınırlarını tam eşleştir (w:pPr, w:pStyle, page, branch vb. yanlış bölmeleri engeller)
            let is_closing = tag_name.starts_with('/');
            let tag_clean = tag_name.trim_start_matches('/').trim();
            let tag_token = tag_clean
                .split(|c: char| c.is_whitespace() || c == '/' || c == '>')
                .next()
                .unwrap_or("")
                .to_ascii_lowercase();

            let is_cell =
                tag_token == "w:tc" || tag_token == "tc" || tag_token == "td" || tag_token == "th";
            let is_row = tag_token == "w:tr" || tag_token == "tr";
            let is_para = tag_token == "w:p" || tag_token == "p";
            let is_break = tag_token == "br" || tag_token == "w:br";

            if is_cell {
                if is_closing {
                    in_table_cell = false;
                    if !result.ends_with('\t') && !result.ends_with('\n') {
                        result.push('\t');
                    }
                } else {
                    in_table_cell = true;
                }
            } else if is_row {
                if is_closing {
                    result.push('\n');
                }
            } else if is_para {
                if is_closing && !in_table_cell {
                    result.push('\n');
                }
            } else if is_break {
                result.push('\n');
            } else if tag_token == "w:tab" {
                result.push('\t');
            }
        } else {
            result.push(ch);
        }
    }

    let unescaped = unescape_xml(&result);
    unescaped
        .lines()
        .map(|l| l.trim_end())
        .filter(|l| !l.trim().is_empty())
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn run_ghostscript_pdf(
    input_path: &Path,
    output_path: &Path,
    setting: &str,
    downsample_dpi: Option<u32>,
) -> Result<()> {
    let mut cmd = Command::new("gs");
    cmd.arg("-sDEVICE=pdfwrite")
        .arg("-dCompatibilityLevel=1.4")
        .arg(setting)
        .arg("-dNOPAUSE")
        .arg("-dQUIET")
        .arg("-dBATCH");

    if let Some(dpi) = downsample_dpi {
        cmd.arg("-dDownsampleColorImages=true")
            .arg("-dColorImageDownsampleType=/Bicubic")
            .arg(format!("-dColorImageResolution={}", dpi))
            .arg("-dDownsampleGrayImages=true")
            .arg("-dGrayImageDownsampleType=/Bicubic")
            .arg(format!("-dGrayImageResolution={}", dpi))
            .arg("-dDownsampleMonoImages=true")
            .arg(format!("-dMonoImageResolution={}", dpi.max(150)));
    }

    cmd.arg(format!("-sOutputFile={}", output_path.display()))
        .arg(input_path);

    let status = cmd
        .status()
        .with_context(|| "ghostscript (gs) çalıştırılamadı; PDF sıkıştırma için kurulu olmalı")?;
    if !status.success() {
        bail!("ghostscript PDF sıkıştırma başarısız oldu (çıkış kodu 0 değil)");
    }
    Ok(())
}

pub fn compress_pdf_to_size(
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
            "PDF belgesi zaten hedef boyuttan ({}) daha küçük ({}). Hedef boyut sıkıştırması için dosyanın {}'dan büyük olması gerekir.",
            crate::history::format_bytes(target_bytes),
            crate::history::format_bytes(current_bytes),
            crate::history::format_bytes(target_bytes)
        );
    }

    let output_path =
        crate::converter::claim_output_path(crate::converter::determine_suffixed_output_path(
            input_path,
            "compressed",
            "pdf",
            custom_output_dir,
        ));
    crate::converter::ensure_parent_dir(&output_path)?;

    // Hedefe ulaşana kadar kademeli olarak daha agresif ayarlar dene.
    // (Sonuç her adımda ölçülür; "başarılı" demek için gerçekten ≤ hedef olmalı.)
    let ladder: [(&str, Option<u32>); 4] = [
        ("-dPDFSETTINGS=/screen", None),
        ("-dPDFSETTINGS=/screen", Some(48)),
        ("-dPDFSETTINGS=/screen", Some(36)),
        ("-dPDFSETTINGS=/ebook", Some(72)),
    ];

    let scratch = output_path.with_file_name(format!(
        "{}.candidate.pdf",
        output_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("konvert")
    ));

    if Command::new("gs").arg("--version").output().is_err() {
        bail!("PDF hedef boyut sıkıştırması için ghostscript (gs) kurulu olmalı, ancak bulunamadı");
    }

    let mut best_size: Option<u64> = None;

    for (setting, dpi) in ladder {
        if let Err(err) = run_ghostscript_pdf(input_path, &scratch, setting, dpi) {
            eprintln!("PDF sıkıştırma adımı başarısız ({}): {:#}", setting, err);
            continue;
        }

        let Ok(meta) = fs::metadata(&scratch) else {
            continue;
        };
        let size = meta.len();

        if size <= target_bytes {
            fs::rename(&scratch, &output_path).with_context(|| {
                format!("Sıkıştırılmış PDF taşınamadı: {}", output_path.display())
            })?;
            return Ok(output_path);
        }

        // Hedefe ulaşmasa da en küçük boyutu hatırla; mevcut hedef dosyaya dokunma.
        if best_size.is_none_or(|best| size < best) {
            best_size = Some(size);
        }
    }

    let _ = fs::remove_file(&scratch);

    if let Some(size) = best_size {
        bail!(
            "PDF hedef boyuta ({}) sıkıştırılamadı. Ulaşılan en küçük boyut: {}",
            crate::history::format_bytes(target_bytes),
            crate::history::format_bytes(size)
        );
    }

    bail!(
        "PDF hedef boyuta sıkıştırılamadı: ghostscript (gs) çalıştırılamadı veya hiçbir ayar sonuç üretmedi"
    )
}

pub fn compress_pdf(input_path: &Path, custom_output_dir: Option<&Path>) -> Result<PathBuf> {
    let output_path =
        crate::converter::claim_output_path(crate::converter::determine_suffixed_output_path(
            input_path,
            "compressed",
            "pdf",
            custom_output_dir,
        ));
    crate::converter::ensure_parent_dir(&output_path)?;

    let pending = crate::converter::PendingOutput::new(&output_path);
    if run_ghostscript_pdf(input_path, pending.path(), "-dPDFSETTINGS=/ebook", None).is_ok()
        && pending.path().exists()
    {
        return pending.commit();
    }
    // gs başarısız oldu: geçici dosya drop ile silinir, ImageMagick yedeğine geç.
    let pending = crate::converter::PendingOutput::new(&output_path);

    let fb_status = Command::new("magick")
        .arg("-density")
        .arg("150")
        .arg(input_path)
        .arg("-quality")
        .arg("80")
        .arg(pending.path())
        .status()
        .or_else(|_| {
            Command::new("convert")
                .arg("-density")
                .arg("150")
                .arg(input_path)
                .arg("-quality")
                .arg("80")
                .arg(pending.path())
                .status()
        })
        .with_context(|| "PDF sıkıştırıcı (gs veya magick) çalıştırılamadı")?;

    if !fb_status.success() {
        bail!("PDF sıkıştırma başarısız oldu");
    }

    pending.commit()
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
        let xml = "<w:p><w:pPr><w:pStyle w:val=\"Normal\"/></w:pPr><w:r><w:t>Hello</w:t></w:r><w:r><w:t> World</w:t></w:r></w:p><w:p><w:r><w:t>Second &amp; Line</w:t></w:r></w:p>";
        let text = extract_text_from_xml(xml);
        assert_eq!(text, "Hello World\n\nSecond & Line");
    }

    #[test]
    fn test_extract_text_not_breaking_on_subtags() {
        let xml = "<w:p><w:r><w:t>Text with </w:t></w:r><w:pPr><w:pBdr/></w:pPr><w:r><w:t>continuation</w:t></w:r></w:p>";
        let text = extract_text_from_xml(xml);
        assert_eq!(text, "Text with continuation");
    }

    #[test]
    fn test_unescape_xml_ampersand_edge_cases() {
        let raw = "a=1&b=2&amp;c=3&#x20;&#X41;";
        let unescaped = unescape_xml(raw);
        assert_eq!(unescaped, "a=1&b=2&c=3 A");
    }

    #[test]
    fn test_extract_text_cdata() {
        let xml = "<content><![CDATA[Mahkeme & Karari]]></content>";
        let text = extract_text_from_xml(xml);
        assert_eq!(text, "Mahkeme & Karari");
    }

    #[test]
    fn test_extract_text_table() {
        let xml = "<w:tbl>\
            <w:tr>\
                <w:tc><w:p><w:r><w:t>Header A</w:t></w:r></w:p></w:tc>\
                <w:tc><w:p><w:r><w:t>Header B</w:t></w:r></w:p></w:tc>\
            </w:tr>\
            <w:tr>\
                <w:tc><w:p><w:r><w:t>Data 1</w:t></w:r></w:p></w:tc>\
                <w:tc><w:p><w:r><w:t>Data 2</w:t></w:r></w:p></w:tc>\
            </w:tr>\
        </w:tbl>";
        let text = extract_text_from_xml(xml);
        assert_eq!(text, "Header A\tHeader B\n\nData 1\tData 2");
    }

    #[test]
    fn test_read_limited_xml_exceeding_guard() {
        let data = "x".repeat(200);
        let reader = data.as_bytes();
        let res = read_limited_xml(reader, 100, Path::new("test.xml"));
        assert!(res.is_err());
        assert!(
            res.unwrap_err()
                .to_string()
                .contains("güvenlik sınırını aşıyor")
        );
    }

    #[test]
    fn test_compress_pdf_to_size_size_guards() {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("konvert_test_small.pdf");
        std::fs::write(&test_file, b"fake pdf bytes").unwrap();

        // 8MB hedef boyutu, dosya çok küçük -> hata vermeli
        let res = compress_pdf_to_size(&test_file, 8 * 1024 * 1024, None);
        assert!(res.is_err());
        let err_msg = res.unwrap_err().to_string();
        assert!(err_msg.contains("zaten hedef boyuttan"));

        // 0 hedef boyutu -> hata vermeli
        assert!(compress_pdf_to_size(&test_file, 0, None).is_err());

        let _ = std::fs::remove_file(&test_file);
    }
}
