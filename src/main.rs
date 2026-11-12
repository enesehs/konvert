mod cli;
mod config;
mod converter;
mod deps;
mod desktop;
mod history;
mod i18n;
mod logger;
mod notification;

use anyhow::{Context, bail};
use clap::Parser;
use cli::Cli;
use config::Config;
use converter::ConvertOptions;
use history::{HistoryEntry, add_history_entry, format_bytes};
use i18n::t;
use notification::Notifier;
use std::fs;
use std::path::PathBuf;

fn pid_file_path() -> Option<PathBuf> {
    directories::ProjectDirs::from("", "", "konvert").map(|p| p.cache_dir().join("running.pid"))
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

fn check_for_updates() -> anyhow::Result<()> {
    let current_version = env!("CARGO_PKG_VERSION");
    println!(
        "Checking for updates... (Current version: v{})",
        current_version
    );

    let output = std::process::Command::new("curl")
        .args([
            "-sSL",
            "--connect-timeout",
            "5",
            "-H",
            "User-Agent: konvert",
            "https://api.github.com/repos/enesehs/konvert/releases/latest",
        ])
        .output();

    match output {
        Ok(out) if out.status.success() => {
            let body = String::from_utf8_lossy(&out.stdout);
            if let Some(pos) = body.find("\"tag_name\":") {
                let after = &body[pos + 11..];
                if let Some(start_quote) = after.find('"') {
                    let tag_part = &after[start_quote + 1..];
                    if let Some(end_quote) = tag_part.find('"') {
                        let tag = &tag_part[..end_quote];
                        let remote_ver = tag.trim_start_matches('v');
                        if remote_ver != current_version {
                            println!(
                                "\n✨ A new version of Konvert is available: {} (current: v{})",
                                tag, current_version
                            );
                            println!("Download release or run installer:");
                            println!(
                                "  curl -fsSL https://enesehs.github.io/konvert/install.sh | bash"
                            );
                            println!("  https://github.com/enesehs/konvert/releases/tag/{}", tag);
                            return Ok(());
                        } else {
                            println!("Konvert is up to date (v{}).", current_version);
                            return Ok(());
                        }
                    }
                }
            }
            println!("Notice: Could not parse latest release information from GitHub.");
            Ok(())
        }
        _ => {
            println!("Could not connect to update server. Please check your internet connection.");
            Ok(())
        }
    }
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    if let Some(shell) = cli.completions {
        use clap::CommandFactory;
        let mut cmd = cli::Cli::command();
        clap_complete::generate(shell, &mut cmd, "konvert", &mut std::io::stdout());
        return Ok(());
    }

    logger::init(cli.verbose);
    let config = Config::load();
    let chosen_lang = cli.lang.as_deref().or(config.general.language.as_deref());
    i18n::init(chosen_lang);

    let notifier = Notifier::new(config.general.notification_enabled && !cli.no_notify);

    converter::set_force_overwrite(cli.force);
    converter::set_create_backup(cli.backup);

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
            let status = if found {
                t("status_ok")
            } else {
                t("status_missing")
            };
            let role = if required {
                t("status_required")
            } else {
                t("status_optional")
            };
            println!("{:<10} {:<8} ({}) - {}", tool, status, role, purpose);
            if !found && required {
                missing_required = true;
            }
        }
        if missing_required {
            bail!("{}", t("err_missing_deps"));
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
        return Ok(());
    }

    if cli.install_dolphin {
        let path = desktop::dolphin::install_dolphin_service(None)
            .map_err(|e| anyhow::anyhow!("Dolphin installation error: {:#}", e))?;
        println!("Dolphin service menu installed: {}", path.display());
        notifier.success(t("notify_integ_title"), "Dolphin context menu installed.");
        return Ok(());
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
        return Ok(());
    }

    if cli.install_nemo {
        let path = desktop::nemo::install_nemo_actions_with_options(
            cli.menu_exec.as_deref(),
            cli.menu_dir.clone(),
        )
        .map_err(|e| anyhow::anyhow!("Nemo installation error: {:#}", e))?;
        println!("Nemo actions installed: {}", path.display());
        notifier.success(t("notify_integ_title"), "Nemo context menu installed.");
        return Ok(());
    }

    if cli.uninstall_nemo {
        let ok = desktop::nemo::uninstall_nemo_actions_with_options(cli.menu_dir)
            .map_err(|e| anyhow::anyhow!("Nemo uninstall error: {:#}", e))?;
        if ok {
            println!("Nemo actions removed.");
            notifier.success(t("notify_integ_title"), "Nemo context menu removed.");
        } else {
            println!("Nemo actions were not found.");
        }
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
            bail!(
                "Hiçbir entegrasyon kaldırılamadı ({} hata)",
                report.failed.len()
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
        return Ok(());
    }

    if cli.check_update {
        check_for_updates()?;
        return Ok(());
    }

    if cli.list_profiles {
        println!("Available Conversion Profiles:");
        println!("\n  Built-in:");
        println!("    web        - quality: 80, crf: 28, preset: fast, bitrate: 128k, dpi: 150");
        println!(
            "    balanced   - quality: 85, crf: 23, preset: medium, bitrate: 192k, dpi: 300 (default)"
        );
        println!("    archive    - quality: 95, crf: 18, preset: slow, bitrate: 320k, dpi: 600");
        println!("    lossless   - image/video lossless: true, crf: 0, bitrate: 320k");
        println!("\n  Custom (from config.toml):");
        if config.profiles.is_empty() {
            println!("    (none saved yet - use --save-profile <name> to create one)");
        } else {
            for (name, p) in &config.profiles {
                println!(
                    "    {:<10} - quality: {:?}, crf: {:?}, preset: {:?}, bitrate: {:?}, dpi: {:?}",
                    name, p.quality, p.crf, p.preset, p.bitrate, p.dpi
                );
            }
        }
        return Ok(());
    }

    if let Some(name) = cli.save_profile {
        let mut cfg = config.clone();
        let profile = config::ProfileConfig {
            quality: cli.quality,
            crf: cli.crf,
            preset: cli.preset.clone(),
            bitrate: cli.bitrate.clone(),
            dpi: cli.dpi,
            lossless: if cli.lossless { Some(true) } else { None },
        };
        cfg.profiles.insert(name.clone(), profile);
        cfg.save()?;
        println!("Profile '{}' saved to config.", name);
        return Ok(());
    }

    if cli.files.is_empty() {
        eprintln!("Error: No files specified for conversion.");
        std::process::exit(1);
    }

    let custom_output_dir = cli
        .output_dir
        .clone()
        .or_else(|| config.general.output_directory.as_ref().map(PathBuf::from));

    // 1. Merge images into single PDF
    if cli.merge_pdf {
        let files = if cli.recursive {
            converter::expand_files_recursive(&cli.files, None)
        } else {
            cli.files.clone()
        };
        if files.is_empty() {
            eprintln!("Error: No input files found to merge into PDF.");
            std::process::exit(1);
        }

        if cli.dry_run {
            println!(
                "[Dry Run] Planning merge into single PDF ({} input files):",
                files.len()
            );
            let first_stem = files
                .first()
                .and_then(|f| f.file_stem())
                .and_then(|s| s.to_str())
                .unwrap_or("images");
            let target_dir = converter::get_output_dir(&files[0], custom_output_dir.as_deref());
            let out_pdf = target_dir.join(format!("{}_merged.pdf", first_stem));
            for (i, f) in files.iter().enumerate() {
                println!("  {:2}. {}", i + 1, f.display());
            }
            println!("  ➔ Output: {}", out_pdf.display());
            println!(
                "  ➔ Collision: {}",
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
        let mut progress_handle = notifier.start_progress(
            "Konvert: Merging PDF...",
            &format!("Scanning {} images...", files.len()),
            15,
        );

        let (tx, rx) = std::sync::mpsc::channel();
        let merge_files = files.clone();
        let merge_out_dir = custom_output_dir.clone();
        std::thread::spawn(move || {
            let res = converter::merge_images_to_pdf(&merge_files, merge_out_dir.as_deref());
            let _ = tx.send(res);
        });

        let start_time = std::time::Instant::now();
        let mut progress_val = 15;
        let progress_title = "Konvert: Merging PDF...".to_string();

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
                        85 + (((elapsed_secs - 30) as f32 / (elapsed_secs + 30) as f32) * 10.0)
                            as i32
                    }
                    .min(95);

                    if target_pct > progress_val {
                        progress_val = target_pct;
                    }
                    let progress_body = format!("{} images ({}s)", files.len(), elapsed_secs);
                    notifier.update_progress(
                        &mut progress_handle,
                        &progress_title,
                        &progress_body,
                        progress_val,
                    );
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
                    &format!(
                        "{}\n{} images ({} ➔ {})",
                        out_name,
                        cli.files.len(),
                        format_bytes(orig_bytes),
                        format_bytes(out_bytes)
                    ),
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

    let target_action = converter::TargetAction::parse(&target).ok();
    let is_stdin = cli.files.len() == 1 && cli.files[0].as_os_str() == "-";
    let files: Vec<PathBuf> = if is_stdin {
        vec![PathBuf::from("-")]
    } else if cli.recursive {
        converter::expand_files_recursive(&cli.files, target_action.as_ref())
    } else {
        cli.files
    };

    if files.is_empty() {
        eprintln!("Error: No input files found to convert.");
        std::process::exit(1);
    }

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

    let mut opts = ConvertOptions {
        jobs,
        target: target.clone(),
        quality: cli.quality.or(Some(config.image.quality)),
        image_lossless: cli.lossless || config.image.lossless,
        video_lossless: cli.lossless || config.video.lossless,
        crf: cli.crf.or(Some(config.video.crf)),
        preset: cli.preset.clone().or(Some(config.video.preset.clone())),
        bitrate: cli.bitrate.clone().or(Some(config.audio.bitrate.clone())),
        dpi: cli.dpi.or(Some(config.document.pdf_dpi)),
        gif_duration: cli.gif_duration.unwrap_or(15.0),
        gif_fps: cli.gif_fps.unwrap_or(15),
        max_pages: cli.max_pages.unwrap_or(200),
        custom_output_dir,
    };

    if let Some(profile_name) = cli.profile.as_deref() {
        if let Some(p) = config.get_profile(profile_name) {
            if cli.quality.is_none()
                && let Some(q) = p.quality
            {
                opts.quality = Some(q);
            }
            if cli.crf.is_none()
                && let Some(c) = p.crf
            {
                opts.crf = Some(c);
            }
            if cli.preset.is_none()
                && let Some(ref pr) = p.preset
            {
                opts.preset = Some(pr.clone());
            }
            if cli.bitrate.is_none()
                && let Some(ref b) = p.bitrate
            {
                opts.bitrate = Some(b.clone());
            }
            if cli.dpi.is_none()
                && let Some(d) = p.dpi
            {
                opts.dpi = Some(d);
            }
            if let Some(l) = p.lossless
                && l
            {
                opts.image_lossless = true;
                opts.video_lossless = true;
            }
        } else {
            eprintln!(
                "Warning: Unknown profile '{}'. Use --list-profiles to view available profiles.",
                profile_name
            );
        }
    }

    if is_stdin {
        use std::io::Read;
        let mut input_bytes = Vec::new();
        std::io::stdin()
            .lock()
            .read_to_end(&mut input_bytes)
            .context("Failed to read from stdin")?;

        if input_bytes.is_empty() {
            eprintln!("Error: stdin was empty.");
            std::process::exit(1);
        }

        let detected = converter::detect_extension_from_bytes(&input_bytes);
        if cli.dry_run {
            println!(
                "[Dry Run] Planning stdin conversion: detected format .{} ➔ target: {}",
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

        let convert_res = converter::convert_single(&temp_file, &opts);
        let _ = std::fs::remove_file(&temp_file);

        match convert_res {
            Ok(output_path) => {
                use std::io::IsTerminal;
                if !std::io::stdout().is_terminal() && cli.output_dir.is_none() {
                    let out_data = std::fs::read(&output_path)?;
                    let _ = std::fs::remove_file(&output_path);
                    let mut out = std::io::stdout().lock();
                    std::io::Write::write_all(&mut out, &out_data)?;
                    std::io::Write::flush(&mut out)?;
                } else {
                    println!("Converted stdin ➔ {}", output_path.display());
                }
                return Ok(());
            }
            Err(e) => {
                eprintln!("Error: Stdin conversion failed: {:#}", e);
                std::process::exit(1);
            }
        }
    }

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
                    println!(
                        "  {:2}. {} ➔ {} {}",
                        i + 1,
                        file.display(),
                        predicted.display(),
                        status
                    );
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

    let orig_total_bytes: u64 = files
        .iter()
        .map(|f| f.metadata().map(|m| m.len()).unwrap_or(0))
        .sum();

    if total_count == 1 {
        let in_file = &files[0];
        let file_name = in_file
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("dosya");
        let target_display = if target == "compress" {
            t("action_compress").to_string()
        } else if target == "discord_8mb" {
            "Discord (8 MB)".to_string()
        } else if target == "email_25mb" {
            "E-Posta (25 MB)".to_string()
        } else if target == "web_opt" {
            t("action_web_opt").to_string()
        } else if target.starts_with("resize_") {
            format!(
                "{} ({})",
                t("action_resize"),
                target.trim_start_matches("resize_")
            )
        } else if let Some(raw_size) = target.strip_prefix("target_") {
            format!("{} ({})", t("action_target_size"), raw_size.to_uppercase())
        } else if target == "strip-metadata" {
            t("action_strip_metadata").to_string()
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

        let start_time = std::time::Instant::now();
        let mut progress_val = 15;
        let progress_title = format!("Konvert: {}", target_display);

        let convert_res = loop {
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
                        85 + (((elapsed_secs - 30) as f32 / (elapsed_secs + 30) as f32) * 10.0)
                            as i32
                    }
                    .min(95);

                    if target_pct > progress_val {
                        progress_val = target_pct;
                    }
                    let progress_body = format!(
                        "{}: {} ({}s)",
                        t("progress_processing"),
                        file_name,
                        elapsed_secs
                    );
                    notifier.update_progress(
                        &mut progress_handle,
                        &progress_title,
                        &progress_body,
                        progress_val,
                    );
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

        let batch_start = std::time::Instant::now();
        let batch_result =
            converter::convert_batch_with_progress(&files, &opts, |curr, total, file, percent| {
                let file_name = file.file_name().and_then(|s| s.to_str()).unwrap_or("file");
                let elapsed = batch_start.elapsed().as_secs_f64();
                let eta_str = if curr > 0 && curr < total {
                    let avg = elapsed / curr as f64;
                    let rem_secs = (avg * (total - curr) as f64).round() as u64;
                    if rem_secs < 60 {
                        format!(" - ETA: {}s", rem_secs)
                    } else {
                        format!(" - ETA: {}m {}s", rem_secs / 60, rem_secs % 60)
                    }
                } else {
                    String::new()
                };
                let summary = format!("Konvert: [{}/{}] ({}%{})", curr, total, percent, eta_str);
                let body = file_name.to_string();
                notifier.update_progress(&mut progress_handle, &summary, &body, percent);
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
            target: target.clone(),
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
                &format!("{} files converted.\n{}", success_count, savings_str),
            );
        } else if success_count > 0 {
            notifier.finish_progress(
                &mut progress_handle,
                t("notify_partial_batch"),
                &format!(
                    "{} succeeded, {} failed.\n{}",
                    success_count, fail_count, savings_str
                ),
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
        format!(
            "{} ➔ {} ({:.0}% saved)",
            format_bytes(orig),
            format_bytes(out),
            pct
        )
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
