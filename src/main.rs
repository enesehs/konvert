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
use konvert::process::{pid_file_path, process_is_konvert, terminate_process, write_pid_lock};
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

    if cli.doctor || cli.check_deps {
        let report = doctor::diagnose();
        doctor::print_doctor_report(&report);
        if !report.all_required_installed {
            bail!("{}", t("err_missing_deps"));
        }
        return Ok(());
    }

    if cli.settings {
        if let Err(e) = launch_settings_gui() {
            eprintln!("Failed to launch settings app: {:#}", e);
            std::process::exit(1);
        }
        return Ok(());
    }

    if let Some(key) = cli.config_get {
        if let Some(val) = config.get_key_value(&key) {
            println!("{}", val);
            return Ok(());
        } else {
            eprintln!("Unknown config key: {}", key);
            std::process::exit(1);
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
            let _ = desktop::dolphin::install_dolphin_service_with_options(
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
        let _ = desktop::dolphin::install_dolphin_service_with_options(
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
        // Masaüstü oturumunda argümansız çağrıldıysa Ayarlar arayüzünü aç
        let has_display = std::env::var("WAYLAND_DISPLAY")
            .map(|v| !v.trim().is_empty())
            .unwrap_or(false)
            || std::env::var("DISPLAY")
                .map(|v| !v.trim().is_empty())
                .unwrap_or(false);

        if has_display {
            if let Err(e) = launch_settings_gui() {
                eprintln!("Failed to launch settings app: {:#}", e);
                std::process::exit(1);
            }
            return Ok(());
        }

        eprintln!("Error: No files specified for conversion.");
        eprintln!("Run 'konvert --help' for usage instructions or 'konvert -s' to open settings.");
        std::process::exit(1);
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
            eprintln!("Error: No input files found to merge into PDF.");
            std::process::exit(1);
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
        cli.files.clone()
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
