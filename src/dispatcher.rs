use anyhow::{Context, Result, bail};
use std::path::{Path, PathBuf};

use crate::cli::Cli;
use crate::config::Config;
use crate::converter::{self, ConvertOptions};
use crate::desktop;
use crate::history::{self, HistoryEntry, add_history_entry, format_bytes};
use crate::i18n::t;
use crate::notification::{JobProgressUpdate, Notifier};
use crate::process::{remove_pid_lock, write_pid_lock};

pub fn calculate_savings_str(orig: u64, out: u64) -> String {
    if orig > out && orig > 0 {
        let saved = orig - out;
        let pct = (saved as f64 / orig as f64) * 100.0;
        format!(
            "{} -> {} (-{:.0}%)",
            format_bytes(orig),
            format_bytes(out),
            pct
        )
    } else if out > orig && orig > 0 {
        let inc = out - orig;
        let pct = (inc as f64 / orig as f64) * 100.0;
        format!(
            "{} -> {} (+{:.0}%)",
            format_bytes(orig),
            format_bytes(out),
            pct
        )
    } else {
        format!("{} -> {}", format_bytes(orig), format_bytes(out))
    }
}

pub fn calculate_savings_pct(orig: u64, out: u64) -> f64 {
    if orig > out && orig > 0 {
        ((orig - out) as f64 / orig as f64) * 100.0
    } else {
        0.0
    }
}

pub fn format_action_headline(target: &str, target_display: &str) -> String {
    match target {
        "compress" => t("notify_status_compressing").to_string(),
        "discord_8mb" | "email_25mb" => {
            format!("{}: {}", t("notify_status_compressing"), target_display)
        }
        s if s.starts_with("target_") => {
            format!("{}: {}", t("notify_status_compressing"), target_display)
        }
        "web_opt" => t("notify_status_optimizing").to_string(),
        s if s.starts_with("resize_") => format!(
            "{}: {}",
            t("notify_status_resizing"),
            s.trim_start_matches("resize_")
        ),
        "strip-metadata" => t("notify_status_stripping").to_string(),
        "frames" | "images" => format!("{}: {}", t("notify_status_extracting"), target_display),
        "merge-pdf" => t("notify_status_merging").to_string(),
        _ => format!("{}: {}", t("notify_status_converting"), target_display),
    }
}

/// Masaüstü entegrasyonu (Dolphin, Nemo, Nautilus, Thunar) ve bağlam menüsü kurulum komutlarını işler.
pub fn handle_desktop_integrations(
    cli: &Cli,
    config: &Config,
    notifier: &Notifier,
) -> Result<bool> {
    if cli.sync_menu {
        let menu_exec = cli.menu_exec.as_deref();
        let synced = desktop::detector::sync_installed_menus(
            &config.context_menu.enabled_actions,
            menu_exec,
            cli.menu_dir.clone(),
        )
        .map_err(|e| anyhow::anyhow!("Failed to sync context menu: {:#}", e))?;
        for path in &synced {
            println!("Context menu synchronized with config: {}", path.display());
        }
        return Ok(true);
    }

    if cli.install_all {
        let report =
            desktop::detector::install_all_with_config(&config.context_menu.enabled_actions);
        for item in &report.affected {
            println!("Installed: {}", item);
        }
        for (name, err) in &report.failed {
            eprintln!("Installation failed ({}): {}", name, err);
        }
        if report.affected.is_empty() {
            notifier.error(t("notify_failed"), t("err_no_integrations_installed"));
            bail!(
                "{} ({} errors)",
                t("err_no_integrations_installed"),
                report.failed.len()
            );
        }
        if report.failed.is_empty() {
            notifier.success(
                t("notify_integ_title"),
                &format!(
                    "Integrations installed for {} desktop managers.",
                    report.affected.len()
                ),
            );
        } else {
            notifier.error(
                t("notify_failed"),
                &format!(
                    "{} integration(s) installed, {} failed.",
                    report.affected.len(),
                    report.failed.len()
                ),
            );
        }
        return Ok(true);
    }

    if cli.install_dolphin {
        let path = desktop::dolphin::install_dolphin_service(None)
            .map_err(|e| anyhow::anyhow!("Dolphin installation error: {:#}", e))?;
        println!("Dolphin service menu installed: {}", path.display());
        notifier.success(t("notify_integ_title"), "Dolphin context menu installed.");
        return Ok(true);
    }

    if cli.uninstall_dolphin {
        let ok = desktop::dolphin::uninstall_dolphin_service()
            .map_err(|e| anyhow::anyhow!("Dolphin uninstall error: {:#}", e))?;
        if ok {
            println!("Dolphin service menu removed.");
            notifier.success(t("notify_integ_title"), "Dolphin context menu removed.");
        } else {
            println!("Dolphin service menu was not found.");
        }
        return Ok(true);
    }

    if cli.install_nemo {
        let path = desktop::nemo::install_nemo_actions_with_options(
            cli.menu_exec.as_deref(),
            cli.menu_dir.clone(),
        )
        .map_err(|e| anyhow::anyhow!("Nemo installation error: {:#}", e))?;
        println!("Nemo actions installed: {}", path.display());
        notifier.success(t("notify_integ_title"), "Nemo context menu installed.");
        return Ok(true);
    }

    if cli.uninstall_nemo {
        let ok = desktop::nemo::uninstall_nemo_actions_with_options(cli.menu_dir.clone())
            .map_err(|e| anyhow::anyhow!("Nemo uninstall error: {:#}", e))?;
        if ok {
            println!("Nemo actions removed.");
            notifier.success(t("notify_integ_title"), "Nemo context menu removed.");
        } else {
            println!("Nemo actions were not found.");
        }
        return Ok(true);
    }

    if cli.uninstall_all {
        let report = desktop::detector::uninstall_all();
        for item in &report.affected {
            println!("Removed: {}", item);
        }
        for (name, err) in &report.failed {
            eprintln!("Removal failed ({}): {}", name, err);
        }
        if report.affected.is_empty() && !report.failed.is_empty() {
            notifier.error(t("notify_failed"), "No integration could be removed.");
            bail!(
                "{} ({} {})",
                t("err_no_integrations_installed"),
                report.failed.len(),
                t("label_failed")
            );
        }
        if report.affected.is_empty() {
            println!("No integrations were installed.");
        } else {
            notifier.success(
                t("notify_integ_title"),
                &format!(
                    "Integrations removed for {} desktop managers.",
                    report.affected.len()
                ),
            );
        }
        return Ok(true);
    }

    Ok(false)
}

/// Çoklu görsel veya PDF dosyalarını tek bir PDF belgesinde birleştirir.
pub fn run_merge_pdf(
    files: &[PathBuf],
    custom_output_dir: Option<&Path>,
    dry_run: bool,
    notifier: &Notifier,
) -> Result<()> {
    if dry_run {
        println!(
            "[Dry Run] Planning merge into single PDF ({} input files):",
            files.len()
        );
        let first_stem = files
            .first()
            .and_then(|f| f.file_stem())
            .and_then(|s| s.to_str())
            .unwrap_or("images");
        let target_dir = converter::get_output_dir(&files[0], custom_output_dir);
        let out_pdf = target_dir.join(format!("{}_merged.pdf", first_stem));
        for (i, f) in files.iter().enumerate() {
            println!("  {:2}. {}", i + 1, f.display());
        }
        println!("  -> Output: {}", out_pdf.display());
        println!(
            "  -> Collision: {}",
            if out_pdf.exists() {
                "yes (file exists)"
            } else {
                "none"
            }
        );
        println!("[Dry Run] Completed: 1 output file planned, 0 written.");
        return Ok(());
    }

    write_pid_lock();
    let orig_bytes: u64 = files
        .iter()
        .map(|f| f.metadata().map(|m| m.len()).unwrap_or(0))
        .sum();
    let headline = t("notify_status_merging");
    let source_label = format!("{} ({})", files.len(), format_bytes(orig_bytes));
    let mut progress_handle = notifier.start_job(&JobProgressUpdate {
        headline,
        source_value: &source_label,
        target_value: "PDF",
        percent: 15,
        detail_text: Some("15%"),
        batch_progress: Some((0, files.len() as u64)),
    });

    let (tx, rx) = std::sync::mpsc::channel();
    let merge_files = files.to_vec();
    let merge_out_dir = custom_output_dir.map(PathBuf::from);
    std::thread::spawn(move || {
        let res = converter::merge_images_to_pdf(&merge_files, merge_out_dir.as_deref());
        let _ = tx.send(res);
    });

    let start_time = std::time::Instant::now();
    let mut progress_val = 15;
    let mut was_paused = false;

    let merge_res = loop {
        match rx.recv_timeout(std::time::Duration::from_millis(300)) {
            Ok(res) => break res,
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                let is_paused = crate::process::is_job_paused();
                if is_paused {
                    if !was_paused {
                        was_paused = true;
                        let paused_text = format!("{}% ({})", progress_val, t("notify_paused"));
                        notifier.update_job(
                            &mut progress_handle,
                            &JobProgressUpdate {
                                headline,
                                source_value: &source_label,
                                target_value: "PDF",
                                percent: progress_val,
                                detail_text: Some(&paused_text),
                                batch_progress: Some((0, files.len() as u64)),
                            },
                        );
                    }
                    continue;
                }
                if was_paused {
                    was_paused = false;
                    let prog_text = format!("{}%", progress_val);
                    notifier.update_job(
                        &mut progress_handle,
                        &JobProgressUpdate {
                            headline,
                            source_value: &source_label,
                            target_value: "PDF",
                            percent: progress_val,
                            detail_text: Some(&prog_text),
                            batch_progress: Some((0, files.len() as u64)),
                        },
                    );
                }

                let elapsed_secs = start_time
                    .elapsed()
                    .saturating_sub(crate::process::job_paused_duration())
                    .as_secs();
                let target_pct = if elapsed_secs <= 5 {
                    15 + (elapsed_secs as i32 * 6)
                } else if elapsed_secs <= 15 {
                    45 + ((elapsed_secs - 5) as i32 * 25 / 10)
                } else if elapsed_secs <= 30 {
                    70 + ((elapsed_secs - 15) as i32 * 15 / 15)
                } else {
                    85 + (((elapsed_secs - 30) as f32 / (elapsed_secs + 30) as f32) * 10.0) as i32
                }
                .min(95);

                if target_pct > progress_val {
                    progress_val = target_pct;
                    let prog_text = format!("{}%", progress_val);
                    notifier.update_job(
                        &mut progress_handle,
                        &JobProgressUpdate {
                            headline,
                            source_value: &source_label,
                            target_value: "PDF",
                            percent: progress_val,
                            detail_text: Some(&prog_text),
                            batch_progress: Some((0, files.len() as u64)),
                        },
                    );
                }
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                break Err(anyhow::anyhow!(
                    "PDF birleştirme işlemi beklenmeyen şekilde sonlandı"
                ));
            }
        }
    };

    match merge_res {
        Ok(out_pdf) => {
            remove_pid_lock();
            let out_bytes = history::path_size_bytes(&out_pdf);
            let out_name = out_pdf
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("merged.pdf");
            let msg = format!(
                "{} images merged into single PDF:\n{}\n({} -> {})",
                files.len(),
                out_pdf.display(),
                format_bytes(orig_bytes),
                format_bytes(out_bytes)
            );
            println!("{}", msg);
            notifier.finish_progress(
                &mut progress_handle,
                t("notify_done_single"),
                &format!(
                    "{}\n{} -> {}",
                    out_name,
                    format_bytes(orig_bytes),
                    format_bytes(out_bytes)
                ),
            );

            let _ = add_history_entry(HistoryEntry {
                timestamp: history::current_timestamp(),
                target: "merge-pdf".into(),
                file_count: files.len(),
                original_size_bytes: orig_bytes,
                output_size_bytes: out_bytes,
                savings_percent: 0.0,
                status: "Success".into(),
                output_path: Some(out_pdf.to_string_lossy().to_string()),
            });
            Ok(())
        }
        Err(err) if crate::process::is_cancelled_error(&err) => {
            remove_pid_lock();
            let cancel_msg = t("err_job_cancelled");
            eprintln!("{}", cancel_msg);
            notifier.cancel_progress(&mut progress_handle, t("notify_cancelled"), &source_label);

            let _ = add_history_entry(HistoryEntry {
                timestamp: history::current_timestamp(),
                target: "merge-pdf".into(),
                file_count: files.len(),
                original_size_bytes: orig_bytes,
                output_size_bytes: 0,
                savings_percent: 0.0,
                status: "Cancelled".into(),
                output_path: None,
            });
            bail!("{}", cancel_msg);
        }
        Err(err) => {
            remove_pid_lock();
            let msg = format!("Failed to merge PDF:\n{:#}", err);
            eprintln!("{}", msg);
            notifier.error_progress(&mut progress_handle, t("notify_failed"), &msg);

            let _ = add_history_entry(HistoryEntry {
                timestamp: history::current_timestamp(),
                target: "merge-pdf".into(),
                file_count: files.len(),
                original_size_bytes: orig_bytes,
                output_size_bytes: 0,
                savings_percent: 0.0,
                status: "Failed".into(),
                output_path: None,
            });
            bail!("PDF merge failed");
        }
    }
}

struct PidLockGuard;

impl PidLockGuard {
    fn acquire() -> Self {
        write_pid_lock();
        Self
    }
}

impl Drop for PidLockGuard {
    fn drop(&mut self) {
        remove_pid_lock();
    }
}

struct TempPathGuard(PathBuf);

impl Drop for TempPathGuard {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

fn read_stdin_prefix(reader: &mut impl std::io::Read, max_bytes: usize) -> Result<Vec<u8>> {
    let mut buf = Vec::with_capacity(max_bytes);
    let mut chunk = [0u8; 1024];
    while buf.len() < max_bytes {
        let to_read = (max_bytes - buf.len()).min(chunk.len());
        let n = reader
            .read(&mut chunk[..to_read])
            .context("Failed to read from stdin")?;
        if n == 0 {
            break;
        }
        buf.extend_from_slice(&chunk[..n]);
    }
    Ok(buf)
}

fn create_private_stdin_temp(ext: &str) -> Result<(PathBuf, std::fs::File)> {
    let temp_dir = std::env::temp_dir();
    let pid = std::process::id();
    for attempt in 0..16u32 {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let path = temp_dir.join(format!(
            "konvert_stdin_{}_{}_{}.{}",
            pid, unique, attempt, ext
        ));
        let mut open_opts = std::fs::OpenOptions::new();
        open_opts.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            open_opts.mode(0o600);
        }
        match open_opts.open(&path) {
            Ok(file) => return Ok((path, file)),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(e.into()),
        }
    }
    bail!("Failed to create secure temporary file for stdin conversion")
}

/// Stdin üzerinden gelen baytları hedef formata dönüştürür.
pub fn run_stdin_conversion(
    opts: &ConvertOptions,
    dry_run: bool,
    output_dir: Option<&Path>,
) -> Result<()> {
    use std::io::{IsTerminal, Write};

    let mut stdin_lock = std::io::stdin().lock();
    let prefix = read_stdin_prefix(&mut stdin_lock, 4096)?;
    if prefix.is_empty() {
        bail!("Error: stdin was empty.");
    }

    let detected = converter::detect_extension_from_bytes(&prefix);
    let piped_stdout = !std::io::stdout().is_terminal() && output_dir.is_none();

    if piped_stdout {
        let action = converter::TargetAction::parse(&opts.target)?;
        let produces_dir = matches!(action, converter::TargetAction::Frames)
            || matches!(
                action,
                converter::TargetAction::Convert(ref fmt)
                    if detected == "pdf" && matches!(fmt.as_str(), "png" | "jpg" | "jpeg")
            );
        if produces_dir {
            bail!(
                "Target '{}' produces a folder and cannot be streamed to stdout; pass -o <dir> instead",
                opts.target
            );
        }
    }

    if dry_run {
        println!(
            "[Dry Run] Planning stdin conversion: detected format .{} -> target: {}",
            detected, opts.target
        );
        return Ok(());
    }

    let _pid_guard = PidLockGuard::acquire();
    let (temp_file, mut temp_handle) = create_private_stdin_temp(detected)?;
    let _in_guard = TempPathGuard(temp_file.clone());

    temp_handle.write_all(&prefix)?;
    std::io::copy(&mut stdin_lock, &mut temp_handle)
        .context("Failed to stream stdin to temporary file")?;
    temp_handle.flush()?;
    drop(temp_handle);
    drop(stdin_lock);

    let mut effective_opts = opts.clone();
    effective_opts.custom_output_dir = if piped_stdout {
        None
    } else {
        output_dir.map(Path::to_path_buf)
    };

    let output_path = converter::convert_single(&temp_file, &effective_opts)
        .map_err(|e| anyhow::anyhow!("Stdin conversion failed: {:#}", e))?;

    if piped_stdout {
        if output_path.is_dir() {
            let _ = std::fs::remove_dir_all(&output_path);
            bail!("Target produced a directory which cannot be streamed to stdout");
        }
        let _out_guard = TempPathGuard(output_path.clone());
        let mut out_file = std::fs::File::open(&output_path)?;
        let mut out = std::io::stdout().lock();
        std::io::copy(&mut out_file, &mut out)?;
        out.flush()?;
    } else {
        println!("Converted stdin -> {}", output_path.display());
    }

    Ok(())
}

/// Simülasyon (dry-run) kipinde dönüştürme planını ve çakışma durumunu yazdırır.
pub fn run_dry_run(files: &[PathBuf], opts: &ConvertOptions, force: bool) -> Result<()> {
    println!("[Dry Run] Planning conversion for {} file(s):", files.len());
    println!("  Target Action : {}", opts.target);
    println!("  Parallel Jobs : {}", opts.jobs);
    if let Some(dir) = &opts.custom_output_dir {
        println!("  Output Folder : {}", dir.display());
    }
    println!();

    let mut collisions = 0;
    let mut errors = 0;
    let mut planned_outputs: std::collections::HashSet<PathBuf> = std::collections::HashSet::new();
    for (i, file) in files.iter().enumerate() {
        if file.as_os_str() != "-" && !file.exists() {
            println!(
                "  {:2}. {} -> [Hata: Dosya bulunamadı]",
                i + 1,
                file.display()
            );
            errors += 1;
            continue;
        }
        match converter::plan_output_path(file, opts, force) {
            Ok((predicted, exists_on_disk)) => {
                let intra_conflict = planned_outputs.contains(&predicted);
                let resolved = if intra_conflict {
                    converter::next_free_path(&predicted, |p| {
                        planned_outputs.contains(p) || (!force && p.exists())
                    })
                } else {
                    predicted.clone()
                };
                let collision = exists_on_disk || intra_conflict || resolved != predicted;
                let status = if exists_on_disk && force && resolved == predicted {
                    collisions += 1;
                    "(overwriting existing)"
                } else if collision {
                    collisions += 1;
                    "(collision resolved with new name)"
                } else {
                    "(new file)"
                };
                planned_outputs.insert(resolved.clone());
                println!(
                    "  {:2}. {} -> {} {}",
                    i + 1,
                    file.display(),
                    resolved.display(),
                    status
                );
            }
            Err(err) => {
                println!("  {:2}. {} -> [Hata: {}]", i + 1, file.display(), err);
                errors += 1;
            }
        }
    }
    println!();
    println!(
        "[Dry Run] Completed: {} file(s) planned, {} collision(s), 0 files modified.",
        files.len(),
        collisions
    );
    if errors > 0 {
        bail!("Dry run failed with {} error(s).", errors);
    }
    Ok(())
}

/// Tek dosya dönüştürme akışını yürütür, anlık D-Bus ilerleme çubuğu yayınlar ve geçmiş kaydını yazar.
pub fn run_single_conversion(
    file: &Path,
    opts: &ConvertOptions,
    orig_total_bytes: u64,
    target_display: &str,
    notifier: &Notifier,
) -> Result<()> {
    let file_name = file.file_name().and_then(|s| s.to_str()).unwrap_or("dosya");

    let headline = format_action_headline(&opts.target, target_display);
    let source_label = if orig_total_bytes > 0 {
        format!("{} ({})", file_name, format_bytes(orig_total_bytes))
    } else {
        file_name.to_string()
    };

    let mut progress_handle = notifier.start_job(&JobProgressUpdate {
        headline: &headline,
        source_value: &source_label,
        target_value: target_display,
        percent: 10,
        detail_text: Some("10%"),
        batch_progress: None,
    });

    enum SingleProgressMsg {
        Progress(i32, String),
        Done(Result<PathBuf>),
    }

    let (tx, rx) = std::sync::mpsc::channel();
    let target_file = file.to_path_buf();
    let convert_opts = opts.clone();
    let progress_tx = tx.clone();
    std::thread::spawn(move || {
        let res = converter::convert_single_with_progress(
            &target_file,
            &convert_opts,
            move |pct, text| {
                let _ = progress_tx.send(SingleProgressMsg::Progress(pct, text.to_string()));
            },
        );
        let _ = tx.send(SingleProgressMsg::Done(res));
    });

    let start_time = std::time::Instant::now();
    let mut progress_val = 10;
    let mut last_detail = "10%".to_string();
    let mut has_real_progress = false;
    let mut was_paused = false;

    let convert_res = loop {
        match rx.recv_timeout(std::time::Duration::from_millis(250)) {
            Ok(SingleProgressMsg::Done(res)) => break res,
            Ok(SingleProgressMsg::Progress(pct, detail_text)) => {
                has_real_progress = true;
                progress_val = pct;
                last_detail = detail_text.clone();
                if crate::process::is_job_paused() {
                    continue;
                }
                notifier.update_job(
                    &mut progress_handle,
                    &JobProgressUpdate {
                        headline: &headline,
                        source_value: &source_label,
                        target_value: target_display,
                        percent: pct,
                        detail_text: Some(&detail_text),
                        batch_progress: None,
                    },
                );
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                let is_paused = crate::process::is_job_paused();
                if is_paused {
                    if !was_paused {
                        was_paused = true;
                        let paused_text = format!("{} ({})", last_detail, t("notify_paused"));
                        notifier.update_job(
                            &mut progress_handle,
                            &JobProgressUpdate {
                                headline: &headline,
                                source_value: &source_label,
                                target_value: target_display,
                                percent: progress_val,
                                detail_text: Some(&paused_text),
                                batch_progress: None,
                            },
                        );
                    }
                    continue;
                }
                if was_paused {
                    was_paused = false;
                    notifier.update_job(
                        &mut progress_handle,
                        &JobProgressUpdate {
                            headline: &headline,
                            source_value: &source_label,
                            target_value: target_display,
                            percent: progress_val,
                            detail_text: Some(&last_detail),
                            batch_progress: None,
                        },
                    );
                }

                if has_real_progress {
                    continue;
                }

                let elapsed_secs = start_time
                    .elapsed()
                    .saturating_sub(crate::process::job_paused_duration())
                    .as_secs();
                let target_pct = if elapsed_secs <= 5 {
                    10 + (elapsed_secs as i32 * 7)
                } else if elapsed_secs <= 15 {
                    45 + ((elapsed_secs - 5) as i32 * 25 / 10)
                } else if elapsed_secs <= 30 {
                    70 + ((elapsed_secs - 15) as i32 * 15 / 15)
                } else {
                    85 + (((elapsed_secs - 30) as f32 / (elapsed_secs + 30) as f32) * 10.0) as i32
                }
                .min(95);

                if target_pct > progress_val {
                    progress_val = target_pct;
                    let progress_text = format!("{}%", progress_val);
                    last_detail = progress_text.clone();
                    notifier.update_job(
                        &mut progress_handle,
                        &JobProgressUpdate {
                            headline: &headline,
                            source_value: &source_label,
                            target_value: target_display,
                            percent: progress_val,
                            detail_text: Some(&progress_text),
                            batch_progress: None,
                        },
                    );
                }
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                break Err(anyhow::anyhow!("{}", t("err_unexpected_exit")));
            }
        }
    };

    match convert_res {
        Ok(output) => {
            remove_pid_lock();
            let out_bytes = history::path_size_bytes(&output);
            let savings_str = calculate_savings_str(orig_total_bytes, out_bytes);
            let out_name = output
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("output");

            let msg = format!("Converted:\n{}\n{}", output.display(), savings_str);
            println!("{}", msg);
            notifier.finish_progress(
                &mut progress_handle,
                t("notify_done_single"),
                &format!("{}\n{}", out_name, savings_str),
            );

            let savings_pct = calculate_savings_pct(orig_total_bytes, out_bytes);
            let _ = add_history_entry(HistoryEntry {
                timestamp: history::current_timestamp(),
                target: opts.target.clone(),
                file_count: 1,
                original_size_bytes: orig_total_bytes,
                output_size_bytes: out_bytes,
                savings_percent: savings_pct,
                status: "Success".into(),
                output_path: Some(output.to_string_lossy().to_string()),
            });
            Ok(())
        }
        Err(err) if crate::process::is_cancelled_error(&err) => {
            remove_pid_lock();
            let cancel_msg = t("err_job_cancelled");
            eprintln!("{}", cancel_msg);
            notifier.cancel_progress(&mut progress_handle, t("notify_cancelled"), file_name);

            let _ = add_history_entry(HistoryEntry {
                timestamp: history::current_timestamp(),
                target: opts.target.clone(),
                file_count: 1,
                original_size_bytes: orig_total_bytes,
                output_size_bytes: 0,
                savings_percent: 0.0,
                status: "Cancelled".into(),
                output_path: None,
            });
            bail!("{}", cancel_msg);
        }
        Err(err) => {
            remove_pid_lock();
            let msg = format!("Failed to convert {}:\n{:#}", file.display(), err);
            eprintln!("{}", msg);
            notifier.error_progress(&mut progress_handle, file_name, &format!("{:#}", err));

            let _ = add_history_entry(HistoryEntry {
                timestamp: history::current_timestamp(),
                target: opts.target.clone(),
                file_count: 1,
                original_size_bytes: orig_total_bytes,
                output_size_bytes: 0,
                savings_percent: 0.0,
                status: "Failed".into(),
                output_path: None,
            });
            bail!("Conversion failed: {:#}", err);
        }
    }
}

/// Toplu dosya dönüştürme akışını Rayon iş parçacığı havuzuyla yürütür, anlık ilerleme çubuğu ve ETA yayınlar.
pub fn run_batch_conversion(
    files: &[PathBuf],
    opts: &ConvertOptions,
    orig_total_bytes: u64,
    target_display: &str,
    notifier: &Notifier,
) -> Result<()> {
    let total_count = files.len();
    let headline = format_action_headline(&opts.target, target_display);
    let initial_source = format!("[0/{}] ({})", total_count, format_bytes(orig_total_bytes));
    let mut progress_handle = notifier.start_job(&JobProgressUpdate {
        headline: &headline,
        source_value: &initial_source,
        target_value: target_display,
        percent: 0,
        detail_text: Some("0%"),
        batch_progress: Some((0, total_count as u64)),
    });

    let batch_start = std::time::Instant::now();
    let mut last_notify_emit = std::time::Instant::now()
        .checked_sub(std::time::Duration::from_secs(1))
        .unwrap_or_else(std::time::Instant::now);
    let batch_result =
        converter::convert_batch_with_progress(files, opts, |curr, total, file, percent| {
            let file_name = file.file_name().and_then(|s| s.to_str()).unwrap_or("file");
            let elapsed = batch_start
                .elapsed()
                .saturating_sub(crate::process::job_paused_duration())
                .as_secs_f64();
            let eta_str = if curr > 0 && curr < total {
                let avg = elapsed / curr as f64;
                let rem_secs = (avg * (total - curr) as f64).round() as u64;
                if rem_secs < 60 {
                    format!(" (ETA: {}s)", rem_secs)
                } else {
                    format!(" (ETA: {}m {}s)", rem_secs / 60, rem_secs % 60)
                }
            } else {
                String::new()
            };
            if curr == 1 || curr == total || last_notify_emit.elapsed().as_millis() >= 150 {
                let source_val = format!("[{}/{}] {}", curr, total, file_name);
                let progress_text = format!("{}%{}", percent, eta_str);
                notifier.update_job(
                    &mut progress_handle,
                    &JobProgressUpdate {
                        headline: &headline,
                        source_value: &source_val,
                        target_value: target_display,
                        percent,
                        detail_text: Some(&progress_text),
                        batch_progress: Some((curr as u64, total as u64)),
                    },
                );
                last_notify_emit = std::time::Instant::now();
            }
            println!(
                "[{}/{}] {} ({}%{})",
                curr, total, file_name, percent, eta_str
            );
        });

    remove_pid_lock();

    let success_count = batch_result.succeeded.len();
    let fail_count = batch_result.failed.len();

    let out_total_bytes: u64 = batch_result
        .succeeded
        .iter()
        .map(|path| history::path_size_bytes(path))
        .sum();
    let savings_str = calculate_savings_str(orig_total_bytes, out_total_bytes);
    let savings_pct = calculate_savings_pct(orig_total_bytes, out_total_bytes);

    println!(
        "Batch conversion finished: {} succeeded, {} failed. {}",
        success_count, fail_count, savings_str
    );

    for (file, err) in &batch_result.failed {
        eprintln!("Error ({}): {}", file.display(), err);
    }

    let _ = add_history_entry(HistoryEntry {
        timestamp: history::current_timestamp(),
        target: opts.target.clone(),
        file_count: total_count,
        original_size_bytes: orig_total_bytes,
        output_size_bytes: out_total_bytes,
        savings_percent: savings_pct,
        status: if batch_result.cancelled {
            "Cancelled".into()
        } else if fail_count == 0 {
            "Success".into()
        } else {
            "Partial".into()
        },
        output_path: batch_result
            .succeeded
            .first()
            .map(|p| p.to_string_lossy().to_string()),
    });

    if batch_result.cancelled {
        notifier.cancel_progress(
            &mut progress_handle,
            t("notify_cancelled"),
            &format!("{} {}", success_count, t("label_files_converted")),
        );
        bail!("{}", t("err_job_cancelled"));
    } else if fail_count == 0 {
        notifier.finish_progress(
            &mut progress_handle,
            t("notify_done_batch"),
            &format!(
                "{} {} • {}",
                success_count,
                t("label_files_converted"),
                savings_str
            ),
        );
    } else if success_count > 0 {
        notifier.finish_progress(
            &mut progress_handle,
            t("notify_partial_batch"),
            &format!(
                "{} {}, {} {} • {}",
                success_count,
                t("label_succeeded"),
                fail_count,
                t("label_failed"),
                savings_str
            ),
        );
    } else {
        notifier.error_progress(
            &mut progress_handle,
            t("notify_failed"),
            &format!("{} {}", fail_count, t("label_failed")),
        );
        bail!("Batch conversion failed: all items failed");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_action_headline_variants() {
        let comp = format_action_headline("compress", "Compress");
        assert!(!comp.is_empty());

        let strip = format_action_headline("strip-metadata", "Strip Metadata");
        assert!(!strip.is_empty());
        assert_ne!(strip, comp);

        let webm = format_action_headline("webm", "WEBM");
        assert!(webm.ends_with(": WEBM"));

        let resize = format_action_headline("resize_1080p", "Resize (1080p)");
        assert!(resize.ends_with(": 1080p"));
    }
}
