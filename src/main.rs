mod cli;
mod config;
mod converter;
mod deps;
mod desktop;
mod history;
mod i18n;
mod notification;

use anyhow::bail;
use clap::Parser;
use cli::Cli;
use config::Config;
use converter::ConvertOptions;
use history::{add_history_entry, format_bytes, HistoryEntry};
use i18n::t;
use notification::Notifier;
use std::fs;
use std::path::PathBuf;

fn pid_file_path() -> Option<PathBuf> {
    directories::ProjectDirs::from("", "", "konvert")
        .map(|p| p.cache_dir().join("running.pid"))
}

/// İşlem, terminale bağlı değilse (sağ tık menüsü, .desktop, servis) kendi
/// süreç grubunun lideri olur. Böylece `--cancel` sırasında ffmpeg/gs gibi
/// alt süreçler de tek sinyalle sonlandırılabilir.
#[cfg(unix)]
fn detach_process_group() {
    unsafe {
        if libc::isatty(libc::STDIN_FILENO) == 0 {
            // Zaten grup lideri ise setsid başarısız olur; yok sayılır.
            let _ = libc::setsid();
        }
    }
}

#[cfg(not(unix))]
fn detach_process_group() {}

fn write_pid_lock() {
    detach_process_group();
    converter::cleanup_recorded_temps();
    if let Some(p) = pid_file_path() {
        if let Some(parent) = p.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::write(p, std::process::id().to_string());
    }
}

fn remove_pid_lock() {
    if let Some(p) = pid_file_path() {
        let _ = fs::remove_file(p);
    }
}

/// PID dosyasındaki sürecin gerçekten Konvert olup olmadığını doğrular.
/// Bayat kayıt (PID yeniden kullanımı) durumunda ilgisiz süreç öldürülmesini önler.
fn process_is_konvert(pid: i32) -> bool {
    let comm = fs::read_to_string(format!("/proc/{}/comm", pid)).unwrap_or_default();
    if comm.trim() == "konvert" {
        return true;
    }
    match fs::read_link(format!("/proc/{}/exe", pid)) {
        Ok(exe) => exe
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("konvert")),
        Err(_) => false,
    }
}

fn process_alive(pid: i32) -> bool {
    unsafe { libc::kill(pid, 0) == 0 }
}

/// Önce SIGTERM (varsa tüm süreç grubuna), sonra gerekirse SIGKILL.
/// Süreç gerçekten sonlandırıldıysa `true` döner.
fn terminate_process(pid: i32) -> bool {
    #[cfg(unix)]
    unsafe {
        if !process_alive(pid) {
            return false;
        }

        // Süreç kendi grubunun lideriyse grubu, değilse sadece kendisini hedefle.
        let is_group_leader = libc::getpgid(pid) == pid;
        let target = if is_group_leader { -pid } else { pid };

        let mut sent = libc::kill(target, libc::SIGTERM);
        if sent != 0 && is_group_leader {
            sent = libc::kill(pid, libc::SIGTERM);
        }
        if sent != 0 {
            return false;
        }

        for _ in 0..20 {
            if !process_alive(pid) {
                return true;
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }

        let _ = libc::kill(target, libc::SIGKILL);
        std::thread::sleep(std::time::Duration::from_millis(100));
        true
    }

    #[cfg(not(unix))]
    {
        let _ = pid;
        false
    }
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let config = Config::load();
    i18n::init(config.general.language.as_deref());

    let notifier = Notifier::new(config.general.notification_enabled && !cli.no_notify);

    converter::set_force_overwrite(cli.force);

    if cli.cancel {
        let Some(path) = pid_file_path() else {
            println!("No active conversion process found.");
            return Ok(());
        };
        let Some(pid) = fs::read_to_string(&path)
            .ok()
            .and_then(|content| content.trim().parse::<i32>().ok())
        else {
            let _ = fs::remove_file(&path);
            println!("No active conversion process found.");
            return Ok(());
        };

        // Bayat/ilgisiz PID kaydını temizle, yabancı sürece dokunma.
        if !process_is_konvert(pid) {
            let _ = fs::remove_file(&path);
            println!(
                "No active Konvert process found (stale lock for PID {} cleaned).",
                pid
            );
            return Ok(());
        }

        if terminate_process(pid) {
            converter::cleanup_recorded_temps();
            let _ = fs::remove_file(&path);
            println!("Active Konvert process cancelled (PID: {}).", pid);
            notifier.success("Konvert", "Conversion process cancelled.");
        } else {
            let _ = fs::remove_file(&path);
            println!("Conversion process already finished (PID: {}).", pid);
        }
        return Ok(());
    }

    if cli.check_deps {
        let mut missing_required = false;
        for (tool, found, required, purpose) in deps::check() {
            let status = if found { "OK" } else { "EKSİK" };
            let role = if required { "gerekli" } else { "opsiyonel" };
            println!("{:<10} {:<8} ({}) - {}", tool, status, role, purpose);
            if !found && required {
                missing_required = true;
            }
        }
        if missing_required {
            bail!("Zorunlu bağımlılıklar eksik; video/ses dönüşümleri çalışmaz.");
        }
        return Ok(());
    }

    if cli.settings {
        let status = std::process::Command::new("konvert-settings")
            .spawn()
            .or_else(|_| {
                let current_exe = std::env::current_exe()?;
                let sibling = current_exe.with_file_name("konvert-settings");
                std::process::Command::new(sibling).spawn()
            });

        match status {
            Ok(_) => return Ok(()),
            Err(e) => {
                eprintln!("Failed to launch settings app: {}", e);
                std::process::exit(1);
            }
        }
    }

    if cli.sync_menu {
        let menu_exec = cli.menu_exec.as_deref();
        let path = desktop::dolphin::install_dolphin_service_with_options(
            &config.context_menu.enabled_actions,
            menu_exec,
            cli.menu_dir,
        )
        .map_err(|e| anyhow::anyhow!("Failed to sync context menu: {:#}", e))?;
        println!("Context menu synchronized with config: {}", path.display());
        return Ok(());
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
            notifier.error(t("notify_failed"), "No desktop integration could be installed.");
            bail!(
                "Hiçbir masaüstü entegrasyonu kurulamadı ({} hata)",
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
        return Ok(());
    }

    if cli.install_dolphin {
        let path = desktop::dolphin::install_dolphin_service(None)
            .map_err(|e| anyhow::anyhow!("Dolphin installation error: {:#}", e))?;
        println!("Dolphin service menu installed: {}", path.display());
        notifier.success(t("notify_integ_title"), "Dolphin context menu installed.");
        return Ok(());
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
            bail!("Hiçbir entegrasyon kaldırılamadı ({} hata)", report.failed.len());
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
        return Ok(());
    }

    if cli.uninstall_dolphin {
        match desktop::dolphin::uninstall_dolphin_service() {
            Ok(true) => {
                println!("Dolphin service menu removed.");
                notifier.success(t("notify_integ_title"), "Dolphin context menu removed.");
            }
            Ok(false) => {
                println!("Dolphin service menu was not installed.");
            }
            Err(e) => {
                notifier.error(t("notify_failed"), &format!("Removal failed: {}", e));
                return Err(anyhow::anyhow!("Dolphin removal error: {:#}", e));
            }
        }
        return Ok(());
    }

    if cli.files.is_empty() {
        eprintln!("Error: No files specified for conversion.");
        std::process::exit(1);
    }

    let custom_output_dir = cli.output_dir.clone().or_else(|| {
        config
            .general
            .output_directory
            .as_ref()
            .map(PathBuf::from)
    });

    // 1. Merge images into single PDF
    if cli.merge_pdf {
        if cli.dry_run {
            println!("[Dry Run] Planning merge into single PDF ({} input files):", cli.files.len());
            let first_stem = cli.files.first()
                .and_then(|f| f.file_stem())
                .and_then(|s| s.to_str())
                .unwrap_or("images");
            let target_dir = converter::get_output_dir(&cli.files[0], custom_output_dir.as_deref());
            let out_pdf = target_dir.join(format!("{}_merged.pdf", first_stem));
            for (i, f) in cli.files.iter().enumerate() {
                println!("  {:2}. {}", i + 1, f.display());
            }
            println!("  ➔ Output: {}", out_pdf.display());
            println!("  ➔ Collision: {}", if out_pdf.exists() { "yes (file exists)" } else { "none" });
            println!("[Dry Run] Completed: 1 output file planned, 0 written.");
            return Ok(());
        }

        write_pid_lock();
        let orig_bytes: u64 = cli.files.iter().map(|f| f.metadata().map(|m| m.len()).unwrap_or(0)).sum();
        let mut progress_handle = notifier.start_progress(
            "Konvert: Merging PDF...",
            &format!("Scanning {} images...", cli.files.len()),
            15,
        );

        let (tx, rx) = std::sync::mpsc::channel();
        let merge_files = cli.files.clone();
        let merge_out_dir = custom_output_dir.clone();
        std::thread::spawn(move || {
            let res = converter::merge_images_to_pdf(&merge_files, merge_out_dir.as_deref());
            let _ = tx.send(res);
        });

        let mut progress_val = 20;
        let progress_title = "Konvert: PDF Birleştiriliyor...".to_string();
        let progress_body = format!("{} görsel birleştiriliyor...", cli.files.len());

        let merge_res = loop {
            match rx.recv_timeout(std::time::Duration::from_millis(300)) {
                Ok(res) => break res,
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    if progress_val < 90 {
                        progress_val += 5;
                        notifier.update_progress(
                            &mut progress_handle,
                            &progress_title,
                            &progress_body,
                            progress_val,
                        );
                    }
                }
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                    break Err(anyhow::anyhow!("PDF birleştirme işlemi beklenmeyen şekilde sonlandı"));
                }
            }
        };

        match merge_res {
            Ok(out_pdf) => {
                remove_pid_lock();
                let out_bytes = history::path_size_bytes(&out_pdf);
                let out_name = out_pdf.file_name().and_then(|s| s.to_str()).unwrap_or("merged.pdf");
                let msg = format!(
                    "{} images merged into single PDF:\n{}\n({} ➔ {})",
                    cli.files.len(),
                    out_pdf.display(),
                    format_bytes(orig_bytes),
                    format_bytes(out_bytes)
                );
                println!("{}", msg);
                notifier.finish_progress(
                    &mut progress_handle,
                    "PDF Merge Completed",
                    &format!("{}\n{} images ({} ➔ {})", out_name, cli.files.len(), format_bytes(orig_bytes), format_bytes(out_bytes)),
                );

                let _ = add_history_entry(HistoryEntry {
                    timestamp: history::current_timestamp(),
                    target: "merge-pdf".into(),
                    file_count: cli.files.len(),
                    original_size_bytes: orig_bytes,
                    output_size_bytes: out_bytes,
                    savings_percent: 0.0,
                    status: "Success".into(),
                    output_path: Some(out_pdf.to_string_lossy().to_string()),
                });
                return Ok(());
            }
            Err(err) => {
                remove_pid_lock();
                let msg = format!("Failed to merge PDF:\n{:#}", err);
                eprintln!("{}", msg);
                notifier.error_progress(&mut progress_handle, "PDF Merge Failed", &msg);

                let _ = add_history_entry(HistoryEntry {
                    timestamp: history::current_timestamp(),
                    target: "merge-pdf".into(),
                    file_count: cli.files.len(),
                    original_size_bytes: orig_bytes,
                    output_size_bytes: 0,
                    savings_percent: 0.0,
                    status: "Failed".into(),
                    output_path: None,
                });
                std::process::exit(1);
            }
        }
    }

    // Hedef formatı belirle
    let target = if let Some(r) = &cli.resize {
        format!("resize_{}", r)
    } else if let Some(ts) = &cli.target_size {
        // Alt-dize eşlemesi YOK: "18mb" 18 MB olmalı, 8 MB değil.
        // Ayrıştırma ve birim dönüşümü TargetAction::parse içinde yapılır.
        format!("target_{}", ts.trim().to_lowercase())
    } else if cli.strip_metadata {
        "strip-metadata".to_string()
    } else if cli.extract_frames {
        "frames".to_string()
    } else {
        match &cli.target {
            Some(t) => t.trim_start_matches('.').to_lowercase(),
            None => {
                eprintln!("Error: Please specify a target format (--target / -t <format>)");
                std::process::exit(1);
            }
        }
    };

    let files: Vec<PathBuf> = cli.files;
    let total_count = files.len();
    let cores = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);
    let jobs = cli
        .jobs
        .filter(|count| *count > 0)
        .unwrap_or_else(|| cores.min(4))
        .min(total_count.max(1));
    converter::set_job_count(jobs);

    let opts = ConvertOptions {
        jobs,
        target: target.clone(),
        quality: cli.quality.or(Some(config.image.quality)),
        image_lossless: cli.lossless || config.image.lossless,
        video_lossless: cli.lossless || config.video.lossless,
        crf: cli.crf.or(Some(config.video.crf)),
        preset: cli.preset.or(Some(config.video.preset.clone())),
        bitrate: cli.bitrate.or(Some(config.audio.bitrate.clone())),
        dpi: cli.dpi.or(Some(config.document.pdf_dpi)),
        gif_duration: cli.gif_duration.unwrap_or(15.0),
        gif_fps: cli.gif_fps.unwrap_or(15),
        max_pages: cli.max_pages.unwrap_or(200),
        custom_output_dir,
    };

    if cli.dry_run {
        println!("[Dry Run] Planning conversion for {} file(s):", files.len());
        println!("  Target Action : {}", opts.target);
        println!("  Parallel Jobs : {}", opts.jobs);
        if let Some(dir) = &opts.custom_output_dir {
            println!("  Output Folder : {}", dir.display());
        }
        println!();

        let mut collisions = 0;
        for (i, file) in files.iter().enumerate() {
            match converter::plan_output_path(file, &opts, cli.force) {
                Ok((predicted, exists)) => {
                    let status = if exists {
                        collisions += 1;
                        if cli.force {
                            "(overwriting existing)"
                        } else {
                            "(collision resolved with new name)"
                        }
                    } else {
                        "(new file)"
                    };
                    println!("  {:2}. {} ➔ {} {}", i + 1, file.display(), predicted.display(), status);
                }
                Err(err) => {
                    println!("  {:2}. {} ➔ [Hata: {}]", i + 1, file.display(), err);
                }
            }
        }
        println!();
        println!(
            "[Dry Run] Completed: {} file(s) planned, {} collision(s), 0 files modified.",
            files.len(),
            collisions
        );
        return Ok(());
    }

    write_pid_lock();

    let orig_total_bytes: u64 = files.iter().map(|f| f.metadata().map(|m| m.len()).unwrap_or(0)).sum();

    if total_count == 1 {
        let in_file = &files[0];
        let file_name = in_file.file_name().and_then(|s| s.to_str()).unwrap_or("dosya");
        let target_display = if target == "compress" {
            "Sıkıştırma".to_string()
        } else if target == "discord_8mb" {
            "Discord (8 MB)".to_string()
        } else if target == "email_25mb" {
            "E-Posta (25 MB)".to_string()
        } else if target == "web_opt" {
            "Web Optimize".to_string()
        } else if target.starts_with("resize_") {
            format!("Boyutlandırma ({})", target.trim_start_matches("resize_"))
        } else if let Some(raw_size) = target.strip_prefix("target_") {
            format!("Hedef Boyut ({})", raw_size.to_uppercase())
        } else if target == "strip-metadata" {
            "Metadata Temizleme".to_string()
        } else {
            target.to_uppercase()
        };

        let mut progress_handle = notifier.start_progress(
            "Konvert: Converting...",
            &format!("{} ➔ {}", file_name, target_display),
            15,
        );

        let (tx, rx) = std::sync::mpsc::channel();
        let target_file = files[0].clone();
        let convert_opts = opts.clone();
        std::thread::spawn(move || {
            let res = converter::convert_single(&target_file, &convert_opts);
            let _ = tx.send(res);
        });

        let mut progress_val = 20;
        let progress_title = format!("Konvert: {}", target_display);
        let progress_body = format!("İşleniyor: {}", file_name);

        let convert_res = loop {
            match rx.recv_timeout(std::time::Duration::from_millis(300)) {
                Ok(res) => break res,
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    if progress_val < 90 {
                        progress_val += 5;
                        notifier.update_progress(
                            &mut progress_handle,
                            &progress_title,
                            &progress_body,
                            progress_val,
                        );
                    }
                }
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                    break Err(anyhow::anyhow!("Dönüştürme işlemi beklenmeyen şekilde sonlandı"));
                }
            }
        };

        match convert_res {
            Ok(output) => {
                remove_pid_lock();
                let out_bytes = history::path_size_bytes(&output);
                let savings_str = calculate_savings_str(orig_total_bytes, out_bytes);
                let out_name = output.file_name().and_then(|s| s.to_str()).unwrap_or("output");

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
                    target: target.clone(),
                    file_count: 1,
                    original_size_bytes: orig_total_bytes,
                    output_size_bytes: out_bytes,
                    savings_percent: savings_pct,
                    status: "Success".into(),
                    output_path: Some(output.to_string_lossy().to_string()),
                });
            }
            Err(err) => {
                remove_pid_lock();
                let msg = format!("Failed to convert {}:\n{:#}", files[0].display(), err);
                eprintln!("{}", msg);
                notifier.error_progress(
                    &mut progress_handle,
                    t("notify_failed"),
                    &format!("{}\n{:#}", file_name, err),
                );

                let _ = add_history_entry(HistoryEntry {
                    timestamp: history::current_timestamp(),
                    target: target.clone(),
                    file_count: 1,
                    original_size_bytes: orig_total_bytes,
                    output_size_bytes: 0,
                    savings_percent: 0.0,
                    status: "Failed".into(),
                    output_path: None,
                });
                std::process::exit(1);
            }
        }
    } else {
        let mut progress_handle = notifier.start_progress(
            "Konvert: Batch Conversion",
            &format!("[0/{}] Processing files...", total_count),
            0,
        );

        let batch_result = converter::convert_batch_with_progress(&files, &opts, |curr, total, file, percent| {
            let file_name = file.file_name().and_then(|s| s.to_str()).unwrap_or("file");
            let summary = format!("Konvert: [{}/{}] Converting ({}%)", curr, total, percent);
            let body = file_name.to_string();
            notifier.update_progress(&mut progress_handle, &summary, &body, percent);
            println!("[{}/{}] {} ({}%)", curr, total, file_name, percent);
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
            target: target.clone(),
            file_count: total_count,
            original_size_bytes: orig_total_bytes,
            output_size_bytes: out_total_bytes,
            savings_percent: savings_pct,
            status: if fail_count == 0 { "Success".into() } else { "Partial".into() },
            output_path: batch_result.succeeded.first().map(|p| p.to_string_lossy().to_string()),
        });

        if fail_count == 0 {
            notifier.finish_progress(
                &mut progress_handle,
                t("notify_done_batch"),
                &format!("{} files converted.\n{}", success_count, savings_str),
            );
        } else if success_count > 0 {
            notifier.finish_progress(
                &mut progress_handle,
                t("notify_partial_batch"),
                &format!("{} succeeded, {} failed.\n{}", success_count, fail_count, savings_str),
            );
        } else {
            notifier.error_progress(
                &mut progress_handle,
                t("notify_failed"),
                &format!("None of the {} files could be converted.", fail_count),
            );
            std::process::exit(1);
        }
    }

    Ok(())
}

fn calculate_savings_str(orig: u64, out: u64) -> String {
    if orig > out && orig > 0 {
        let saved = orig - out;
        let pct = (saved as f64 / orig as f64) * 100.0;
        format!("{} ➔ {} ({:.0}% saved)", format_bytes(orig), format_bytes(out), pct)
    } else {
        format!("{} ➔ {}", format_bytes(orig), format_bytes(out))
    }
}

fn calculate_savings_pct(orig: u64, out: u64) -> f64 {
    if orig > out && orig > 0 {
        ((orig - out) as f64 / orig as f64) * 100.0
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_savings_pct() {
        assert_eq!(calculate_savings_pct(100, 20), 80.0);
        assert_eq!(calculate_savings_pct(100, 100), 0.0);
        assert_eq!(calculate_savings_pct(100, 120), 0.0);
    }

    #[test]
    fn test_calculate_savings_str() {
        let s = calculate_savings_str(1000, 500);
        assert!(s.contains("50% saved"));
    }
}
