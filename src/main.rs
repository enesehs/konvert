mod cli;
mod config;
mod converter;
mod desktop;
mod history;
mod i18n;
mod notification;

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

fn write_pid_lock() {
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

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let config = Config::load();

    let notifier = Notifier::new(config.general.notification_enabled && !cli.no_notify);

    if cli.cancel {
        if let Some(p) = pid_file_path()
            && p.exists()
            && let Ok(content) = fs::read_to_string(&p)
            && let Ok(pid) = content.trim().parse::<i32>()
        {
            #[cfg(unix)]
            unsafe {
                libc::kill(pid, libc::SIGTERM);
            }
            let _ = fs::remove_file(&p);
            println!("Active Konvert process cancelled (PID: {}).", pid);
            notifier.success("Konvert", "Conversion process cancelled.");
            return Ok(());
        }
        println!("No active conversion process found.");
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
        match desktop::dolphin::install_dolphin_service_with_options(
            &crate::config::Config::load().context_menu.enabled_actions,
            menu_exec,
            cli.menu_dir,
        ) {
            Ok(path) => {
                println!("Context menu synchronized with config: {}", path.display());
            }
            Err(e) => {
                eprintln!("Failed to sync context menu: {}", e);
            }
        }
        return Ok(());
    }

    if cli.install_all {
        match desktop::detector::install_all() {
            Ok(installed) => {
                for item in &installed {
                    println!("Installed: {}", item);
                }
                notifier.success(
                    t("notify_integ_title"),
                    &format!("Integrations installed for {} desktop managers.", installed.len()),
                );
            }
            Err(e) => {
                eprintln!("Installation error: {}", e);
                notifier.error(t("notify_failed"), &format!("Installation failed: {}", e));
            }
        }
        return Ok(());
    }

    if cli.install_dolphin {
        match desktop::dolphin::install_dolphin_service(None) {
            Ok(path) => {
                println!("Dolphin service menu installed: {}", path.display());
                notifier.success(t("notify_integ_title"), "Dolphin context menu installed.");
            }
            Err(e) => {
                eprintln!("Dolphin installation error: {}", e);
                notifier.error(t("notify_failed"), &format!("Installation failed: {}", e));
            }
        }
        return Ok(());
    }

    if cli.uninstall_all {
        match desktop::detector::uninstall_all() {
            Ok(removed) => {
                for item in &removed {
                    println!("Removed: {}", item);
                }
                notifier.success(
                    t("notify_integ_title"),
                    &format!("Integrations removed for {} desktop managers.", removed.len()),
                );
            }
            Err(e) => {
                eprintln!("Removal error: {}", e);
                notifier.error(t("notify_failed"), &format!("Removal failed: {}", e));
            }
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
                eprintln!("Dolphin removal error: {}", e);
                notifier.error(t("notify_failed"), &format!("Removal failed: {}", e));
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
        write_pid_lock();
        let orig_bytes: u64 = cli.files.iter().map(|f| f.metadata().map(|m| m.len()).unwrap_or(0)).sum();
        let mut progress_handle = notifier.start_progress(
            "Konvert: Merging PDF...",
            &format!("Scanning {} images...", cli.files.len()),
            20,
        );

        match converter::merge_images_to_pdf(&cli.files, custom_output_dir.as_deref()) {
            Ok(out_pdf) => {
                remove_pid_lock();
                let out_bytes = out_pdf.metadata().map(|m| m.len()).unwrap_or(0);
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
        let clean = ts.to_lowercase();
        if clean.contains("8") {
            "discord_8mb".to_string()
        } else if clean.contains("25") {
            "email_25mb".to_string()
        } else {
            format!("target_{}", clean)
        }
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

    let opts = ConvertOptions {
        target: target.clone(),
        quality: cli.quality.or(Some(config.image.quality)),
        lossless: cli.lossless || config.image.lossless,
        crf: cli.crf.or(Some(config.video.crf)),
        preset: cli.preset.or(Some(config.video.preset.clone())),
        bitrate: cli.bitrate.or(Some(config.audio.bitrate.clone())),
        dpi: cli.dpi.or(Some(config.document.pdf_dpi)),
        custom_output_dir,
    };

    let files: Vec<PathBuf> = cli.files;
    let total_count = files.len();
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
        } else if target == "strip-metadata" {
            "Metadata Temizleme".to_string()
        } else {
            target.to_uppercase()
        };

        let mut progress_handle = notifier.start_progress(
            "Konvert: Converting...",
            &format!("{} ➔ {}", file_name, target_display),
            20,
        );

        match converter::convert_single(&files[0], &opts) {
            Ok(output) => {
                remove_pid_lock();
                let out_bytes = output.metadata().map(|m| m.len()).unwrap_or(0);
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

        let out_total_bytes: u64 = batch_result.succeeded.iter().map(|f| f.metadata().map(|m| m.len()).unwrap_or(0)).sum();
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
