use anyhow::{Context, Result, bail};
use std::path::{Path, PathBuf};

use crate::cli::Cli;
use crate::config::Config;
use crate::converter::{self, ConvertOptions};
use crate::desktop;
use crate::history::{self, HistoryEntry, add_history_entry, format_bytes};
use crate::i18n::t;
use crate::notification::Notifier;
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

/// Masaüstü entegrasyonu (Dolphin, Nemo, Nautilus, Thunar) ve bağlam menüsü kurulum komutlarını işler.
pub fn handle_desktop_integrations(
    cli: &Cli,
    config: &Config,
    notifier: &Notifier,
) -> Result<bool> {
    if cli.sync_menu {
        let menu_exec = cli.menu_exec.as_deref();
        let path = desktop::dolphin::install_dolphin_service_with_options(
            &config.context_menu.enabled_actions,
            menu_exec,
            cli.menu_dir.clone(),
        )
        .map_err(|e| anyhow::anyhow!("Failed to sync context menu: {:#}", e))?;
        println!("Context menu synchronized with config: {}", path.display());
        return Ok(true);
    }

    if cli.install_all {
        let report = desktop::detector::install_all();
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
    let progress_title = "Konvert: Merging PDF...".to_string();
    let stable_body = format!("Scanning {} images...", files.len());
    let mut progress_handle =
        notifier.start_progress(&progress_title, &stable_body, 15, Some("15%"));

    let (tx, rx) = std::sync::mpsc::channel();
    let merge_files = files.to_vec();
    let merge_out_dir = custom_output_dir.map(PathBuf::from);
    std::thread::spawn(move || {
        let res = converter::merge_images_to_pdf(&merge_files, merge_out_dir.as_deref());
        let _ = tx.send(res);
    });

    let start_time = std::time::Instant::now();
    let mut progress_val = 15;

    let merge_res = loop {
        match rx.recv_timeout(std::time::Duration::from_millis(300)) {
            Ok(res) => break res,
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                let elapsed_secs = start_time.elapsed().as_secs();
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
                    notifier.update_progress(
                        &mut progress_handle,
                        &progress_title,
                        &stable_body,
                        progress_val,
                        Some(&prog_text),
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
                "PDF Merge Completed",
                &format!(
                    "{}\n{} images ({} -> {})",
                    out_name,
                    files.len(),
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
        Err(err) => {
            remove_pid_lock();
            let msg = format!("Failed to merge PDF:\n{:#}", err);
            eprintln!("{}", msg);
            notifier.error_progress(&mut progress_handle, "PDF Merge Failed", &msg);

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

/// Stdin üzerinden gelen baytları hedef formata dönüştürür.
pub fn run_stdin_conversion(
    opts: &ConvertOptions,
    dry_run: bool,
    output_dir: Option<&Path>,
) -> Result<()> {
    use std::io::Read;
    let mut input_bytes = Vec::new();
    std::io::stdin()
        .lock()
        .read_to_end(&mut input_bytes)
        .context("Failed to read from stdin")?;

    if input_bytes.is_empty() {
        bail!("Error: stdin was empty.");
    }

    let detected = converter::detect_extension_from_bytes(&input_bytes);
    if dry_run {
        println!(
            "[Dry Run] Planning stdin conversion: detected format .{} -> target: {}",
            detected, opts.target
        );
        return Ok(());
    }

    let temp_dir = std::env::temp_dir();
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let temp_file = temp_dir.join(format!(
        "konvert_stdin_{}_{}.{}",
        std::process::id(),
        unique,
        detected
    ));
    std::fs::write(&temp_file, &input_bytes)?;

    let convert_res = converter::convert_single(&temp_file, opts);
    let _ = std::fs::remove_file(&temp_file);

    match convert_res {
        Ok(output_path) => {
            use std::io::IsTerminal;
            if !std::io::stdout().is_terminal() && output_dir.is_none() {
                let out_data = std::fs::read(&output_path)?;
                let _ = std::fs::remove_file(&output_path);
                let mut out = std::io::stdout().lock();
                std::io::Write::write_all(&mut out, &out_data)?;
                std::io::Write::flush(&mut out)?;
            } else {
                println!("Converted stdin -> {}", output_path.display());
            }
            Ok(())
        }
        Err(e) => {
            bail!("Stdin conversion failed: {:#}", e);
        }
    }
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
    for (i, file) in files.iter().enumerate() {
        match converter::plan_output_path(file, opts, force) {
            Ok((predicted, exists)) => {
                let status = if exists {
                    collisions += 1;
                    if force {
                        "(overwriting existing)"
                    } else {
                        "(collision resolved with new name)"
                    }
                } else {
                    "(new file)"
                };
                println!(
                    "  {:2}. {} -> {} {}",
                    i + 1,
                    file.display(),
                    predicted.display(),
                    status
                );
            }
            Err(err) => {
                println!("  {:2}. {} -> [Hata: {}]", i + 1, file.display(), err);
            }
        }
    }
    println!();
    println!(
        "[Dry Run] Completed: {} file(s) planned, {} collision(s), 0 files modified.",
        files.len(),
        collisions
    );
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

    let progress_title = "Konvert".to_string();
    let stable_body = format!("{} -> {}", file_name, target_display);

    let mut progress_handle =
        notifier.start_progress(&progress_title, &stable_body, 10, Some("10%"));

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
    let mut has_real_progress = false;

    let convert_res = loop {
        match rx.recv_timeout(std::time::Duration::from_millis(250)) {
            Ok(SingleProgressMsg::Done(res)) => break res,
            Ok(SingleProgressMsg::Progress(pct, detail_text)) => {
                has_real_progress = true;
                progress_val = pct;
                notifier.update_progress(
                    &mut progress_handle,
                    &progress_title,
                    &stable_body,
                    pct,
                    Some(&detail_text),
                );
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                if has_real_progress {
                    continue;
                }

                let elapsed_secs = start_time.elapsed().as_secs();
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
                    notifier.update_progress(
                        &mut progress_handle,
                        &progress_title,
                        &stable_body,
                        progress_val,
                        Some(&progress_text),
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
            notifier.finish_progress(&mut progress_handle, out_name, &savings_str);

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
    let batch_title = t("notify_batch_in_progress");
    let initial_batch_body = format!("[0/{}] {}", total_count, target_display);
    let mut progress_handle =
        notifier.start_progress(batch_title, &initial_batch_body, 0, Some("0%"));

    let batch_start = std::time::Instant::now();
    let batch_result =
        converter::convert_batch_with_progress(files, opts, |curr, total, file, percent| {
            let file_name = file.file_name().and_then(|s| s.to_str()).unwrap_or("file");
            let elapsed = batch_start.elapsed().as_secs_f64();
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
            let summary = format!("[{}/{}]", curr, total);
            let body = format!("{} -> {}", file_name, target_display);
            let progress_text = format!("{}%{}", percent, eta_str);
            notifier.update_progress(
                &mut progress_handle,
                &summary,
                &body,
                percent,
                Some(&progress_text),
            );
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
        status: if fail_count == 0 {
            "Success".into()
        } else {
            "Partial".into()
        },
        output_path: batch_result
            .succeeded
            .first()
            .map(|p| p.to_string_lossy().to_string()),
    });

    if fail_count == 0 {
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
