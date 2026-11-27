use anyhow::{Context, Result, bail};
use std::fs::{self, File};
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

static TEMP_DOC_COUNTER: AtomicUsize = AtomicUsize::new(0);
static SOFFICE_MUTEX: Mutex<()> = Mutex::new(());

struct TempDirGuard(PathBuf);

impl Drop for TempDirGuard {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

struct TempFileGuard(PathBuf);

impl Drop for TempFileGuard {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

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
    fs::create_dir_all(&base_dir)?;
    let pages_dir =
        crate::converter::claim_output_path(base_dir.join(format!("{}_pages", file_stem)));

    let count = TEMP_DOC_COUNTER.fetch_add(1, Ordering::Relaxed);
    let staging_dir = base_dir.join(format!(
        ".konvert_tmp_pages_{}_{}",
        std::process::id(),
        count
    ));
    fs::create_dir_all(&staging_dir)?;
    let staging_guard = TempDirGuard(staging_dir.clone());

    let output_prefix = staging_dir.join("page");

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

    let status = crate::process::run_tracked_status(&mut cmd).map_err(|e| {
        if crate::process::is_cancelled_error(&e) {
            e
        } else {
            e.context("pdftoppm bulunamadı")
        }
    })?;
    if !status.success() {
        bail!("{}", crate::i18n::t("err_pdf_render_images"));
    }

    if pages_dir.exists() {
        let _ = fs::remove_dir_all(&pages_dir);
    }
    fs::rename(&staging_dir, &pages_dir)
        .with_context(|| format!("Sayfa klasörü taşınamadı: {}", pages_dir.display()))?;
    std::mem::forget(staging_guard);

    Ok(pages_dir)
}

pub fn convert_to_pdf_via_soffice(
    input_path: &Path,
    output_path: &Path,
    _target_dir: &Path,
) -> Result<PathBuf> {
    let count = TEMP_DOC_COUNTER.fetch_add(1, Ordering::Relaxed);
    let ts = chrono::Local::now().timestamp_nanos_opt().unwrap_or(0);
    let temp_profile = std::env::temp_dir().join(format!(
        "konvert_soffice_{}_{}_{}",
        std::process::id(),
        ts,
        count
    ));
    let temp_outdir = std::env::temp_dir().join(format!(
        "konvert_soffice_out_{}_{}_{}",
        std::process::id(),
        ts,
        count
    ));
    fs::create_dir_all(&temp_profile)?;
    fs::create_dir_all(&temp_outdir)?;
    let _profile_guard = TempDirGuard(temp_profile.clone());
    let _outdir_guard = TempDirGuard(temp_outdir.clone());

    // Paralel LibreOffice kilit çatışmasını önlemek için süreç içi kilit ve bağımsız profil
    let lock = SOFFICE_MUTEX.lock();
    let mut cmd = Command::new("soffice");
    cmd.arg(format!(
        "-env:UserInstallation=file://{}",
        temp_profile.display()
    ))
    .arg("--headless")
    .arg("--convert-to")
    .arg("pdf")
    .arg("--outdir")
    .arg(&temp_outdir)
    .arg(input_path);
    let output = crate::process::run_tracked_output(&mut cmd);
    drop(lock);

    let out = match output {
        Ok(o) if o.status.success() => o,
        Ok(o) => {
            let detail = String::from_utf8_lossy(&o.stderr);
            let trimmed = detail.trim();
            if trimmed.is_empty() {
                bail!("{}", crate::i18n::t("err_soffice_failed"));
            }
            bail!("{}: {}", crate::i18n::t("err_soffice_failed"), trimmed);
        }
        Err(err) => {
            if crate::process::is_cancelled_error(&err) {
                return Err(err);
            }
            bail!("{}", crate::i18n::t("err_soffice_failed"));
        }
    };
    let _ = out;

    let input_stem = input_path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| anyhow::anyhow!("Geçersiz dosya adı"))?;
    let expected_pdf = temp_outdir.join(format!("{}.pdf", input_stem));
    let generated = if expected_pdf.exists() {
        expected_pdf
    } else {
        fs::read_dir(&temp_outdir)?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .find(|p| {
                p.extension()
                    .and_then(|s| s.to_str())
                    .is_some_and(|e| e.eq_ignore_ascii_case("pdf"))
            })
            .ok_or_else(|| anyhow::anyhow!("{}", crate::i18n::t("err_soffice_failed")))?
    };

    let pending = crate::converter::PendingOutput::new(output_path);
    fs::rename(&generated, pending.path()).or_else(|_| {
        fs::copy(&generated, pending.path())?;
        Ok::<(), anyhow::Error>(())
    })?;
    pending.commit()
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
            "{}: {} (max: {} MB)",
            crate::i18n::t("err_doc_size_limit"),
            path_hint.display(),
            max_bytes / (1024 * 1024)
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
        bail!("{}", crate::i18n::t("err_udf_read_text"));
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
            let _html_guard = TempFileGuard(temp_html.clone());

            write_html_document(
                "UYAP UDF Belgesi",
                Some("T.C. ADALET BAKANLIĞI - UYAP DOKÜMANI"),
                &text_content,
                &temp_html,
            )?;

            convert_to_pdf_via_soffice(&temp_html, &output_path, &target_dir).with_context(
                || "UDF -> PDF dönüşümü için sisteminizde LibreOffice (soffice) kurulu olmalıdır",
            )
        }
        _ => {
            bail!(
                "{}: UDF -> .{}",
                crate::i18n::t("err_unsupported_conversion"),
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

    let text_content = extract_docx_text_with_budget(&mut archive, input_path, MAX_DOC_XML_BYTES)?;

    match target_format.to_lowercase().as_str() {
        "html" => write_html_document("Doküman", None, &text_content, &output_path),
        "txt" | "text" => {
            let pending = crate::converter::PendingOutput::new(&output_path);
            fs::write(pending.path(), text_content)?;
            pending.commit()
        }
        _ => {
            bail!(
                "{}: DOCX -> .{}",
                crate::i18n::t("err_unsupported_conversion"),
                target_format
            );
        }
    }
}

fn extract_docx_text_with_budget<R: Read + std::io::Seek>(
    archive: &mut zip::ZipArchive<R>,
    input_path: &Path,
    total_budget: u64,
) -> Result<String> {
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
    let mut remaining_budget = total_budget;

    // 1. Üstbilgiler (Headers)
    for h_name in &header_names {
        let h_xml = read_docx_part(archive, h_name, &mut remaining_budget, input_path)?;
        let h_text = extract_text_from_xml(&h_xml);
        if !h_text.is_empty() {
            full_text_sections.push(h_text);
        }
    }

    // 2. Ana Belge (word/document.xml)
    let main_xml = read_docx_part(
        archive,
        "word/document.xml",
        &mut remaining_budget,
        input_path,
    )
    .with_context(|| "DOCX belgesi içinde word/document.xml bulunamadı")?;
    let main_text = extract_text_from_xml(&main_xml);
    if main_text.is_empty() {
        bail!(
            "{}: {}",
            crate::i18n::t("err_docx_read_text"),
            input_path.display()
        );
    }
    full_text_sections.push(main_text);

    // 3. Dipnotlar (Footnotes)
    for fn_name in &footnote_names {
        let fn_xml = read_docx_part(archive, fn_name, &mut remaining_budget, input_path)?;
        let fn_text = extract_text_from_xml(&fn_xml);
        if !fn_text.is_empty() {
            full_text_sections.push(format!("--- Dipnotlar ---\n{}", fn_text));
        }
    }

    // 4. Altbilgiler (Footers)
    for f_name in &footer_names {
        let f_xml = read_docx_part(archive, f_name, &mut remaining_budget, input_path)?;
        let f_text = extract_text_from_xml(&f_xml);
        if !f_text.is_empty() {
            full_text_sections.push(f_text);
        }
    }

    Ok(full_text_sections.join("\n\n"))
}

fn read_docx_part<R: Read + std::io::Seek>(
    archive: &mut zip::ZipArchive<R>,
    part_name: &str,
    remaining_budget: &mut u64,
    input_path: &Path,
) -> Result<String> {
    let entry = archive
        .by_name(part_name)
        .with_context(|| format!("DOCX bileşeni okunamadı: {}", part_name))?;
    let xml = read_limited_xml(entry, *remaining_budget, input_path)?;
    *remaining_budget = remaining_budget.saturating_sub(xml.len() as u64);
    Ok(xml)
}

fn decode_xml_entity(entity: &str) -> Option<char> {
    if entity.eq_ignore_ascii_case("amp") {
        return Some('&');
    }
    if entity.eq_ignore_ascii_case("lt") {
        return Some('<');
    }
    if entity.eq_ignore_ascii_case("gt") {
        return Some('>');
    }
    if entity.eq_ignore_ascii_case("quot") {
        return Some('"');
    }
    if entity.eq_ignore_ascii_case("apos") {
        return Some('\'');
    }
    if entity.eq_ignore_ascii_case("nbsp") {
        return Some(' ');
    }
    if let Some(hex) = entity
        .strip_prefix("#x")
        .or_else(|| entity.strip_prefix("#X"))
    {
        return u32::from_str_radix(hex, 16).ok().and_then(char::from_u32);
    }
    if let Some(dec) = entity.strip_prefix('#') {
        return dec.parse::<u32>().ok().and_then(char::from_u32);
    }
    None
}

fn unescape_xml(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut cursor = 0;

    while let Some(rel_amp) = input[cursor..].find('&') {
        let amp = cursor + rel_amp;
        out.push_str(&input[cursor..amp]);

        let rest = &input[amp + 1..];
        let mut semi_byte = None;
        for (idx, ch) in rest.char_indices() {
            if idx > 16 {
                break;
            }
            if ch == ';' {
                semi_byte = Some(idx);
                break;
            }
            if ch.is_whitespace() || ch == '<' || ch == '&' {
                break;
            }
        }

        if let Some(semi) = semi_byte
            && let Some(decoded) = decode_xml_entity(&rest[..semi])
        {
            out.push(decoded);
            cursor = amp + 1 + semi + 1;
            continue;
        }

        out.push('&');
        cursor = amp + 1;
    }
    out.push_str(&input[cursor..]);
    out
}

fn is_xml_skipped_tag(tag_token: &str) -> bool {
    if matches!(
        tag_token,
        "w:del"
            | "del"
            | "w:deltext"
            | "deltext"
            | "w:instrtext"
            | "instrtext"
            | "wp:posoffset"
            | "posoffset"
            | "style"
            | "script"
            | "head"
    ) {
        return true;
    }
    tag_token.ends_with("pr") && !matches!(tag_token, "w:tr" | "tr")
}

pub fn extract_text_from_xml(xml: &str) -> String {
    let mut result = String::with_capacity(xml.len() / 2);
    let mut raw_text = String::new();
    let mut chars = xml.chars().peekable();
    let mut skip_stack: Vec<String> = Vec::new();
    let mut table_cell_depth: usize = 0;
    let mut need_cell_space = false;

    let flush_raw = |raw: &mut String, out: &mut String, need_space: &mut bool| {
        if !raw.is_empty() {
            let decoded = unescape_xml(raw);
            if *need_space && !decoded.trim().is_empty() {
                if !out.is_empty() && !out.ends_with([' ', '\t', '\n']) {
                    out.push(' ');
                }
                *need_space = false;
            }
            out.push_str(&decoded);
            raw.clear();
        }
    };

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
                let mut end_count = 0usize;
                for c in chars.by_ref() {
                    if c == ']' {
                        end_count += 1;
                    } else if c == '>' && end_count >= 2 {
                        while end_count > 2 {
                            cdata.push(']');
                            end_count -= 1;
                        }
                        break;
                    } else {
                        while end_count > 0 {
                            cdata.push(']');
                            end_count -= 1;
                        }
                        cdata.push(c);
                    }
                }
                if skip_stack.is_empty() && !cdata.is_empty() {
                    flush_raw(&mut raw_text, &mut result, &mut need_cell_space);
                    if need_cell_space && !cdata.trim().is_empty() {
                        if !result.is_empty() && !result.ends_with([' ', '\t', '\n']) {
                            result.push(' ');
                        }
                        need_cell_space = false;
                    }
                    result.push_str(&cdata);
                }
                continue;
            }

            if tag_peek.starts_with("!--") {
                let tail: String = tag_peek.chars().skip(3).collect();
                let mut dash_count = tail.chars().rev().take_while(|&c| c == '-').count();
                let already_closed = tail.ends_with("--") && chars.peek() == Some(&'>');
                if already_closed {
                    chars.next();
                    continue;
                }
                for c in chars.by_ref() {
                    if c == '-' {
                        dash_count += 1;
                    } else if c == '>' && dash_count >= 2 {
                        break;
                    } else {
                        dash_count = 0;
                    }
                }
                continue;
            }

            let mut tag_name = tag_peek;
            let mut quote: Option<char> = None;
            for c in tag_name.chars() {
                match (quote, c) {
                    (None, '"' | '\'') => quote = Some(c),
                    (Some(q), ch) if ch == q => quote = None,
                    _ => {}
                }
            }
            if chars.peek() == Some(&'>') && quote.is_none() {
                chars.next();
            } else {
                for c in chars.by_ref() {
                    match (quote, c) {
                        (None, '"' | '\'') => {
                            quote = Some(c);
                            tag_name.push(c);
                        }
                        (Some(q), ch) if ch == q => {
                            quote = None;
                            tag_name.push(c);
                        }
                        (None, '>') => break,
                        _ => tag_name.push(c),
                    }
                }
            }

            let tag_body = tag_name.trim();
            if tag_body.starts_with('?') || tag_body.starts_with('!') {
                continue;
            }

            let is_closing = tag_body.starts_with('/');
            let is_self_closing = !is_closing && tag_body.ends_with('/');
            let tag_token = tag_body
                .trim_start_matches('/')
                .trim_end_matches('/')
                .trim()
                .split(|c: char| c.is_whitespace())
                .next()
                .unwrap_or("")
                .to_ascii_lowercase();

            if tag_token.is_empty() {
                continue;
            }

            if !skip_stack.is_empty() {
                if is_closing {
                    if let Some(pos) = skip_stack.iter().rposition(|t| t == &tag_token) {
                        skip_stack.truncate(pos);
                    }
                } else if !is_self_closing && is_xml_skipped_tag(&tag_token) {
                    skip_stack.push(tag_token);
                }
                continue;
            }

            flush_raw(&mut raw_text, &mut result, &mut need_cell_space);

            if is_xml_skipped_tag(&tag_token) {
                if !is_closing && !is_self_closing {
                    skip_stack.push(tag_token);
                }
                continue;
            }

            let is_cell = matches!(tag_token.as_str(), "w:tc" | "tc" | "td" | "th");
            let is_row = matches!(tag_token.as_str(), "w:tr" | "tr");
            let is_para = matches!(
                tag_token.as_str(),
                "w:p"
                    | "p"
                    | "div"
                    | "li"
                    | "h1"
                    | "h2"
                    | "h3"
                    | "h4"
                    | "h5"
                    | "h6"
                    | "text:p"
                    | "text:h"
            );
            let is_break = matches!(tag_token.as_str(), "br" | "w:br" | "text:line-break");

            if is_cell {
                if is_closing {
                    table_cell_depth = table_cell_depth.saturating_sub(1);
                    need_cell_space = false;
                    if !result.ends_with('\t') && !result.ends_with('\n') {
                        result.push('\t');
                    }
                } else if is_self_closing {
                    if !result.ends_with('\t') && !result.ends_with('\n') {
                        result.push('\t');
                    }
                } else {
                    table_cell_depth += 1;
                    need_cell_space = false;
                }
            } else if is_row {
                if is_closing {
                    need_cell_space = false;
                    if !result.ends_with('\n') {
                        result.push('\n');
                    }
                }
            } else if is_para {
                if is_closing {
                    if table_cell_depth == 0 {
                        if !result.ends_with('\n') {
                            result.push('\n');
                        }
                    } else {
                        need_cell_space = true;
                    }
                } else if is_self_closing
                    && table_cell_depth == 0
                    && !result.is_empty()
                    && !result.ends_with('\n')
                {
                    result.push('\n');
                }
            } else if is_break && !is_closing {
                if table_cell_depth == 0 {
                    result.push('\n');
                } else {
                    need_cell_space = true;
                }
            } else if matches!(tag_token.as_str(), "w:tab" | "text:tab") && !is_closing {
                result.push('\t');
                need_cell_space = false;
            }
        } else if skip_stack.is_empty() {
            raw_text.push(ch);
        }
    }

    if skip_stack.is_empty() {
        flush_raw(&mut raw_text, &mut result, &mut need_cell_space);
    }

    result
        .lines()
        .map(|l| l.trim_end())
        .filter(|l| !l.trim().is_empty())
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn build_ghostscript_pdf_command(
    input_path: &Path,
    output_path: &Path,
    setting: &str,
    downsample_dpi: Option<u32>,
) -> Command {
    let mut cmd = Command::new("gs");
    cmd.args([
        "-dSAFER",
        "-sDEVICE=pdfwrite",
        "-dCompatibilityLevel=1.4",
        setting,
        "-dNOPAUSE",
        "-dQUIET",
        "-dBATCH",
    ]);

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

    let safe_input = crate::converter::sanitize_cli_path(input_path);
    cmd.arg(format!("-sOutputFile={}", output_path.display()))
        .arg("-f")
        .arg(safe_input);
    cmd
}

fn run_ghostscript_pdf(
    input_path: &Path,
    output_path: &Path,
    setting: &str,
    downsample_dpi: Option<u32>,
) -> Result<()> {
    // Ghostscript treats `%` in -sOutputFile as printf format specifiers.
    let gs_out = if output_path.to_string_lossy().contains('%') {
        static GS_TEMP_COUNTER: std::sync::atomic::AtomicUsize =
            std::sync::atomic::AtomicUsize::new(0);
        let cnt = GS_TEMP_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Some(std::env::temp_dir().join(format!("konvert_gs_{}_{}.pdf", std::process::id(), cnt)))
    } else {
        None
    };
    let effective_out = gs_out.as_deref().unwrap_or(output_path);

    let mut cmd = build_ghostscript_pdf_command(input_path, effective_out, setting, downsample_dpi);
    let status = match crate::process::run_tracked_status(&mut cmd) {
        Ok(s) => s,
        Err(err) => {
            if let Some(ref tmp) = gs_out {
                let _ = fs::remove_file(tmp);
            }
            if crate::process::is_cancelled_error(&err) {
                return Err(err);
            }
            return Err(
                err.context("ghostscript (gs) çalıştırılamadı; PDF sıkıştırma için kurulu olmalı")
            );
        }
    };
    if !status.success() {
        if let Some(ref tmp) = gs_out {
            let _ = fs::remove_file(tmp);
        }
        bail!("{}", crate::i18n::t("err_gs_compress_failed"));
    }

    if let Some(tmp) = gs_out {
        fs::rename(&tmp, output_path).or_else(|_| {
            fs::copy(&tmp, output_path)?;
            let _ = fs::remove_file(&tmp);
            Ok::<(), anyhow::Error>(())
        })?;
    }
    Ok(())
}

pub fn compress_pdf_to_size(
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

    let output_path =
        crate::converter::claim_output_path(crate::converter::determine_suffixed_output_path(
            input_path,
            "compressed",
            "pdf",
            custom_output_dir,
        ));
    crate::converter::ensure_parent_dir(&output_path)?;

    // Yüksek kaliteden (/ebook) başla, sığmazsa daha agresif (/screen) adımlara in.
    // Sonuç her adımda ölçülür; yalnızca <= hedef boyut sağlanınca onaylanır.
    let ladder: [(&str, Option<u32>); 5] = [
        ("-dPDFSETTINGS=/ebook", None),
        ("-dPDFSETTINGS=/ebook", Some(96)),
        ("-dPDFSETTINGS=/screen", None),
        ("-dPDFSETTINGS=/screen", Some(48)),
        ("-dPDFSETTINGS=/screen", Some(36)),
    ];

    if Command::new("gs").arg("--version").output().is_err() {
        bail!("{}", crate::i18n::t("err_gs_not_installed"));
    }

    let pending = crate::converter::PendingOutput::new(&output_path);
    let mut best_size: Option<u64> = None;

    for (setting, dpi) in ladder {
        if let Err(err) = run_ghostscript_pdf(input_path, pending.path(), setting, dpi) {
            if crate::process::is_cancelled_error(&err) {
                return Err(err);
            }
            eprintln!("PDF sıkıştırma adımı başarısız ({}): {:#}", setting, err);
            continue;
        }

        let Ok(meta) = fs::metadata(pending.path()) else {
            continue;
        };
        let size = meta.len();

        if size <= target_bytes {
            return pending.commit();
        }

        // Hedefe ulaşmasa da en küçük boyutu hatırla; mevcut hedef dosyaya dokunma.
        if best_size.is_none_or(|best| size < best) {
            best_size = Some(size);
        }
    }

    drop(pending);

    if let Some(size) = best_size {
        bail!(
            "{}: {} -> {}",
            crate::i18n::t("err_target_size_not_reached"),
            crate::history::format_bytes(target_bytes),
            crate::history::format_bytes(size)
        );
    }

    bail!("{}", crate::i18n::t("err_gs_compress_failed"))
}

pub fn compress_pdf(input_path: &Path, custom_output_dir: Option<&Path>) -> Result<PathBuf> {
    let orig_bytes = fs::metadata(input_path).map(|m| m.len()).unwrap_or(0);
    let output_path =
        crate::converter::claim_output_path(crate::converter::determine_suffixed_output_path(
            input_path,
            "compressed",
            "pdf",
            custom_output_dir,
        ));
    crate::converter::ensure_parent_dir(&output_path)?;

    let pending = crate::converter::PendingOutput::new(&output_path);
    run_ghostscript_pdf(input_path, pending.path(), "-dPDFSETTINGS=/ebook", None)?;
    if !pending.path().exists() {
        bail!("{}", crate::i18n::t("err_gs_compress_failed"));
    }

    let out_bytes = fs::metadata(pending.path()).map(|m| m.len()).unwrap_or(0);
    if orig_bytes > 0 && out_bytes >= orig_bytes {
        // /ebook PDF'i büyüttüyse /screen profilini dene; yine büyüme olursa orijinali koru.
        let _ = run_ghostscript_pdf(input_path, pending.path(), "-dPDFSETTINGS=/screen", None);
        let retry_bytes = fs::metadata(pending.path())
            .map(|m| m.len())
            .unwrap_or(out_bytes);
        if retry_bytes >= orig_bytes {
            fs::copy(input_path, pending.path())
                .with_context(|| format!("PDF kopyalanamadı: {}", pending.path().display()))?;
        }
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
    fn test_extract_text_skips_deleted_instr_properties_and_handles_cells() {
        let xml = "<w:document><w:body>\
            <!-- ignored comment -->\
            <w:p><w:pPr/><w:del><w:r><w:delText>Deleted</w:delText></w:r></w:del>\
            <w:r><w:rPr><w:b/></w:rPr><w:instrText>HYPERLINK</w:instrText><w:t>Keep</w:t></w:r></w:p>\
            <style>.a{color:red}</style><script>bad()</script>\
            <w:tbl><w:tr>\
                <w:tc><w:p><w:r><w:t>Line 1</w:t></w:r></w:p><w:p><w:r><w:t>Line 2</w:t></w:r></w:p></w:tc>\
                <w:tc><w:p><w:r><w:t>Col 2</w:t></w:r></w:p></w:tc>\
            </w:tr></w:tbl>\
            <content><![CDATA[Literal &amp; CDATA]]></content>\
        </w:body></w:document>";
        let text = extract_text_from_xml(xml);
        assert_eq!(text, "Keep\n\nLine 1 Line 2\tCol 2\n\nLiteral &amp; CDATA");
    }

    #[test]
    fn test_read_limited_xml_exceeding_guard() {
        let data = "x".repeat(200);
        let reader = data.as_bytes();
        let res = read_limited_xml(reader, 100, Path::new("test.xml"));
        assert!(res.is_err());
        let err_str = res.unwrap_err().to_string();
        assert!(
            err_str.contains("güvenlik sınırını aşıyor")
                || err_str.contains("security limit")
                || err_str.contains("limit")
        );
    }

    #[test]
    fn test_compress_pdf_to_size_size_guards() {
        let temp_dir = std::env::temp_dir().join(format!(
            "konvert_pdf_size_test_{}_{}",
            std::process::id(),
            chrono::Local::now().timestamp_nanos_opt().unwrap_or(0)
        ));
        std::fs::create_dir_all(&temp_dir).unwrap();
        let test_file = temp_dir.join("small.pdf");
        std::fs::write(&test_file, b"fake pdf bytes").unwrap();

        // 8MB hedef boyutu, dosya çok küçük -> hata vermeli
        let res = compress_pdf_to_size(&test_file, 8 * 1024 * 1024, None);
        assert!(res.is_err());
        let err_msg = res.unwrap_err().to_string();
        assert!(err_msg.contains("zaten hedef boyuttan") || err_msg.contains("already smaller"));

        // 0 hedef boyutu -> hata vermeli
        assert!(compress_pdf_to_size(&test_file, 0, None).is_err());

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_build_ghostscript_pdf_command_uses_safer_and_f_guard() {
        let cmd = build_ghostscript_pdf_command(
            Path::new("-evil.pdf"),
            Path::new("/tmp/safe.pdf"),
            "-dPDFSETTINGS=/ebook",
            None,
        );
        let args: Vec<String> = cmd
            .get_args()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();
        assert!(args.contains(&"-dSAFER".to_string()));
        assert!(args.windows(2).any(|w| w == ["-f", "./-evil.pdf"]));
    }

    #[test]
    fn test_docx_cumulative_budget_enforced_across_parts() {
        use std::io::Write;
        let mut cursor = std::io::Cursor::new(Vec::new());
        {
            let mut zw = zip::ZipWriter::new(&mut cursor);
            let opts = zip::write::SimpleFileOptions::default();
            zw.start_file("word/header1.xml", opts).unwrap();
            zw.write_all(b"<w:t>Header content padding data here</w:t>")
                .unwrap();
            zw.start_file("word/document.xml", opts).unwrap();
            zw.write_all(b"<w:t>Main body content exceeding cumulative budget</w:t>")
                .unwrap();
            zw.finish().unwrap();
        }
        cursor.set_position(0);
        let mut archive = zip::ZipArchive::new(cursor).unwrap();
        let res = extract_docx_text_with_budget(&mut archive, Path::new("bomb.docx"), 70);
        assert!(res.is_err());
    }
}
