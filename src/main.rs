use std::fs;
use std::path::PathBuf;

use anyhow::{Result, bail};
use clap::Parser;

use konvert::cli::{self, Cli};
use konvert::config::{self, Config};
use konvert::converter::{self, ConvertOptions};
use konvert::desktop;
use konvert::dispatcher::{
    handle_desktop_integrations, run_batch_conversion, run_dry_run, run_merge_pdf,
    run_single_conversion, run_stdin_conversion,
};
use konvert::doctor;
use konvert::gui::launch_settings_gui;
use konvert::i18n::{self, t};
use konvert::logger;
use konvert::notification::Notifier;
use konvert::process::{
    list_active_konvert_pids, pause_file_path_for, pause_process, resume_process,
    terminate_process, write_pid_lock,
};
use konvert::update::check_for_updates;

fn main() -> Result<()> {
    let cli = Cli::parse();

    if let Some(shell) = cli.completions {
        use clap::CommandFactory;
        let mut cmd = cli::Cli::command();
        clap_complete::generate(shell, &mut cmd, "konvert", &mut std::io::stdout());
        return Ok(());
    }

    logger::init(cli.verbose);
    let is_readonly_invocation = cli.cancel
        || cli.pause
        || cli.resume
        || cli.doctor
        || cli.check_deps
        || cli.settings
        || cli.dump_config_json
        || cli.config_get.is_some()
        || cli.list_profiles
        || cli.check_update
        || cli.dry_run;
    let config = if is_readonly_invocation {
        Config::load_readonly()
    } else {
        Config::load()
    };
    let chosen_lang = cli.lang.as_deref().or(config.general.language.as_deref());
    i18n::init(chosen_lang);

    let notifier = Notifier::new(config.general.notification_enabled && !cli.no_notify);

    converter::set_force_overwrite(cli.force);
    converter::set_create_backup(cli.backup);

    if cli.pause {
        let active = list_active_konvert_pids();
        if active.is_empty() {
            println!("No active conversion process found.");
            return Ok(());
        }

        let mut paused_pids = Vec::new();
        for (pid, _) in active {
            if pause_process(pid) {
                if let Some(marker) = pause_file_path_for(pid) {
                    for _ in 0..10 {
                        if marker.exists() {
                            break;
                        }
                        std::thread::sleep(std::time::Duration::from_millis(15));
                    }
                }
                paused_pids.push(pid);
            }
        }

        if !paused_pids.is_empty() {
            let pid_list: Vec<String> = paused_pids.iter().map(ToString::to_string).collect();
            println!(
                "Active Konvert process paused (PID: {}).",
                pid_list.join(", ")
            );
        } else {
            println!("Failed to pause active conversion process.");
        }
        return Ok(());
    }

    if cli.resume {
        let active = list_active_konvert_pids();
        if active.is_empty() {
            println!("No active conversion process found.");
            return Ok(());
        }

        let mut resumed_pids = Vec::new();
        for (pid, _) in active {
            if resume_process(pid) {
                if let Some(marker) = pause_file_path_for(pid) {
                    for _ in 0..10 {
                        if !marker.exists() {
                            break;
                        }
                        std::thread::sleep(std::time::Duration::from_millis(15));
                    }
                }
                resumed_pids.push(pid);
            }
        }

        if !resumed_pids.is_empty() {
            let pid_list: Vec<String> = resumed_pids.iter().map(ToString::to_string).collect();
            println!(
                "Active Konvert process resumed (PID: {}).",
                pid_list.join(", ")
            );
        } else {
            println!("Failed to resume active conversion process.");
        }
        return Ok(());
    }

    if cli.cancel {
        let active = list_active_konvert_pids();
        if active.is_empty() {
            println!("No active conversion process found.");
            return Ok(());
        }

        let mut cancelled_pids = Vec::new();
        for (pid, lock_path) in active {
            let terminated = terminate_process(pid);
            let _ = fs::remove_file(&lock_path);
            if let Some(pause_path) = pause_file_path_for(pid) {
                let _ = fs::remove_file(pause_path);
            }
            if terminated {
                cancelled_pids.push(pid);
            }
        }
        converter::cleanup_recorded_temps();

        if !cancelled_pids.is_empty() {
            let pid_list: Vec<String> = cancelled_pids.iter().map(ToString::to_string).collect();
            println!(
                "Active Konvert process cancelled (PID: {}).",
                pid_list.join(", ")
            );
            notifier.success("Konvert", "Conversion process cancelled.");
        } else {
            println!("Conversion process already finished.");
        }
        return Ok(());
    }

    if cli.doctor || cli.check_deps {
        let report = doctor::diagnose();
        doctor::print_doctor_report(&report);
        if !report.all_required_installed {
            bail!("{}", t("err_missing_deps"));
        }
        return Ok(());
    }

    if cli.settings {
        launch_settings_gui()
            .map_err(|e| anyhow::anyhow!("Failed to launch settings app: {:#}", e))?;
        return Ok(());
    }

    if cli.dump_config_json {
        println!("{}", serde_json::to_string(&config)?);
        return Ok(());
    }

    if let Some(key) = cli.config_get {
        if let Some(val) = config.get_key_value(&key) {
            println!("{}", val);
            return Ok(());
        } else {
            bail!("Unknown config key: {}", key);
        }
    }

    if let Some(pair) = cli.config_set {
        let mut parts = pair.splitn(2, '=');
        let key = parts.next().unwrap_or("").trim();
        let val = parts.next().unwrap_or("").trim();
        if key.is_empty() {
            bail!("Invalid format for --config-set. Use KEY=VALUE");
        }
        let mut cfg = config.clone();
        cfg.set_key_value(key, val)?;
        cfg.save()?;
        println!("Configuration updated: {} = {}", key, val);
        if key == "context_menu.enabled_actions" {
            let menu_exec = cli.menu_exec.as_deref();
            let _ = desktop::detector::sync_installed_menus(
                &cfg.context_menu.enabled_actions,
                menu_exec,
                cli.menu_dir,
            );
        }
        return Ok(());
    }

    if let Some(json_payload) = cli.save_config_json {
        let content = if json_payload == "-" {
            use std::io::Read;
            let mut buf = String::new();
            std::io::stdin().read_to_string(&mut buf)?;
            buf
        } else {
            json_payload
        };

        let mut cfg = config.clone();
        cfg.update_from_json(&content)?;
        cfg.save()?;

        let menu_exec = cli.menu_exec.as_deref();
        let _ = desktop::detector::sync_installed_menus(
            &cfg.context_menu.enabled_actions,
            menu_exec,
            cli.menu_dir,
        );
        println!("Configuration saved and context menu synchronized.");
        return Ok(());
    }

    if handle_desktop_integrations(&cli, &config, &notifier)? {
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
        let profile = config::ProfileConfig {
            quality: cli.quality,
            crf: cli.crf,
            preset: cli.preset.clone(),
            bitrate: cli.bitrate.clone(),
            dpi: cli.dpi,
            lossless: if cli.lossless { Some(true) } else { None },
        };
        let validated_name = config::validate_custom_profile(&name, &profile)?;
        let mut cfg = config.clone();
        cfg.profiles.insert(validated_name.clone(), profile);
        cfg.save()?;
        println!("Profile '{}' saved to config.", validated_name);
        return Ok(());
    }

    if cli.files.is_empty() {
        // Masaüstü oturumunda argümansız çağrıldıysa Ayarlar arayüzünü aç
        let has_display = std::env::var("WAYLAND_DISPLAY")
            .map(|v| !v.trim().is_empty())
            .unwrap_or(false)
            || std::env::var("DISPLAY")
                .map(|v| !v.trim().is_empty())
                .unwrap_or(false);

        if has_display {
            launch_settings_gui()
                .map_err(|e| anyhow::anyhow!("Failed to launch settings app: {:#}", e))?;
            return Ok(());
        }

        bail!(
            "No files specified for conversion. Run 'konvert --help' for usage instructions or 'konvert -s' to open settings."
        );
    }

    let custom_output_dir = cli
        .output_dir
        .clone()
        .or_else(|| config.general.output_directory.as_ref().map(PathBuf::from));

    // 1. Çoklu görseli PDF olarak birleştir
    if cli.merge_pdf {
        let files = if cli.recursive {
            converter::expand_files_recursive(&cli.files, None)
        } else {
            cli.files.clone()
        };
        if files.is_empty() {
            bail!("No input files found to merge into PDF.");
        }

        return run_merge_pdf(&files, custom_output_dir.as_deref(), cli.dry_run, &notifier);
    }

    // Hedef formatı belirle
    let target = if let Some(r) = &cli.resize {
        format!("resize_{}", r)
    } else if let Some(ts) = &cli.target_size {
        // Alt-dize eşlemesi YOK: "18mb" 18 MB olmalı, 8 MB değil.
        format!("target_{}", ts.trim().to_lowercase())
    } else if cli.strip_metadata {
        "strip-metadata".to_string()
    } else if cli.extract_frames {
        "frames".to_string()
    } else {
        match &cli.target {
            Some(t) => t.trim_start_matches('.').to_lowercase(),
            None => {
                bail!("Please specify a target format (--target / -t <format>)");
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
        cli.files.clone()
    };

    if files.is_empty() {
        bail!("No input files found to convert.");
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
        hwaccel: cli.hwaccel.clone().or(Some(config.video.hwaccel.clone())),
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
            if !cli.lossless
                && let Some(l) = p.lossless
            {
                opts.image_lossless = l;
                opts.video_lossless = l;
            }
        } else {
            eprintln!(
                "Warning: Unknown profile '{}'. Use --list-profiles to view available profiles.",
                profile_name
            );
        }
    }

    if is_stdin {
        return run_stdin_conversion(&opts, cli.dry_run, cli.output_dir.as_deref());
    }

    if cli.dry_run {
        return run_dry_run(&files, &opts, cli.force);
    }

    write_pid_lock();

    let orig_total_bytes: u64 = files
        .iter()
        .map(|f| f.metadata().map(|m| m.len()).unwrap_or(0))
        .sum();

    let target_display = if target == "compress" {
        t("action_compress").to_string()
    } else if target == "discord_8mb" {
        "Discord (8 MB)".to_string()
    } else if target == "email_25mb" {
        t("action_email_25mb").to_string()
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

    if total_count == 1 {
        run_single_conversion(
            &files[0],
            &opts,
            orig_total_bytes,
            &target_display,
            &notifier,
        )
    } else {
        run_batch_conversion(&files, &opts, orig_total_bytes, &target_display, &notifier)
    }
}

#[cfg(test)]
mod tests {
    use konvert::dispatcher::{calculate_savings_pct, calculate_savings_str};

    #[test]
    fn test_calculate_savings_pct() {
        assert_eq!(calculate_savings_pct(100, 20), 80.0);
        assert_eq!(calculate_savings_pct(100, 100), 0.0);
        assert_eq!(calculate_savings_pct(100, 120), 0.0);
    }

    #[test]
    fn test_calculate_savings_str() {
        let s = calculate_savings_str(1000, 500);
        assert!(s.contains("-50%"));
    }
}
