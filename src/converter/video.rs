use anyhow::{Context, Result, bail};
use std::fs;
use std::io::BufRead;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HwAccel {
    Nvenc,
    Vaapi,
    #[default]
    None,
}

static DETECTED_HWACCEL: std::sync::OnceLock<HwAccel> = std::sync::OnceLock::new();
static DETECTED_AV1_NVENC: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
static DETECTED_AV1_VAAPI: std::sync::OnceLock<bool> = std::sync::OnceLock::new();

pub fn probe_nvenc() -> bool {
    let has_nvidia = Path::new("/dev/nvidia0").exists()
        || Path::new("/dev/nvidiactl").exists()
        || Path::new("/dev/nvhost-ctrl").exists();
    if !has_nvidia {
        return false;
    }
    let status = Command::new("ffmpeg")
        .arg("-nostdin")
        .arg("-loglevel")
        .arg("error")
        .arg("-f")
        .arg("lavfi")
        .arg("-i")
        .arg("color=c=black:s=256x256:d=0.04")
        .arg("-c:v")
        .arg("h264_nvenc")
        .arg("-f")
        .arg("null")
        .arg("-")
        .status();
    matches!(status, Ok(s) if s.success())
}

pub fn probe_av1_nvenc() -> bool {
    *DETECTED_AV1_NVENC.get_or_init(|| {
        if !probe_nvenc() {
            return false;
        }
        let status = Command::new("ffmpeg")
            .arg("-nostdin")
            .arg("-loglevel")
            .arg("error")
            .arg("-f")
            .arg("lavfi")
            .arg("-i")
            .arg("color=c=black:s=256x256:d=0.04")
            .arg("-c:v")
            .arg("av1_nvenc")
            .arg("-f")
            .arg("null")
            .arg("-")
            .status();
        matches!(status, Ok(s) if s.success())
    })
}

pub fn probe_vaapi() -> bool {
    let render_node = if Path::new("/dev/dri/renderD128").exists() {
        "/dev/dri/renderD128"
    } else if Path::new("/dev/dri/renderD129").exists() {
        "/dev/dri/renderD129"
    } else {
        return false;
    };
    let status = Command::new("ffmpeg")
        .arg("-nostdin")
        .arg("-loglevel")
        .arg("error")
        .arg("-vaapi_device")
        .arg(render_node)
        .arg("-f")
        .arg("lavfi")
        .arg("-i")
        .arg("color=c=black:s=256x256:d=0.04")
        .arg("-vf")
        .arg("format=nv12,hwupload")
        .arg("-c:v")
        .arg("h264_vaapi")
        .arg("-f")
        .arg("null")
        .arg("-")
        .status();
    matches!(status, Ok(s) if s.success())
}

pub fn probe_av1_vaapi() -> bool {
    *DETECTED_AV1_VAAPI.get_or_init(|| {
        let render_node = if Path::new("/dev/dri/renderD128").exists() {
            "/dev/dri/renderD128"
        } else if Path::new("/dev/dri/renderD129").exists() {
            "/dev/dri/renderD129"
        } else {
            return false;
        };
        let status = Command::new("ffmpeg")
            .arg("-nostdin")
            .arg("-loglevel")
            .arg("error")
            .arg("-vaapi_device")
            .arg(render_node)
            .arg("-f")
            .arg("lavfi")
            .arg("-i")
            .arg("color=c=black:s=256x256:d=0.04")
            .arg("-vf")
            .arg("format=nv12,hwupload")
            .arg("-c:v")
            .arg("av1_vaapi")
            .arg("-f")
            .arg("null")
            .arg("-")
            .status();
        matches!(status, Ok(s) if s.success())
    })
}

pub fn detect_hardware_acceleration() -> HwAccel {
    *DETECTED_HWACCEL.get_or_init(|| {
        if probe_nvenc() {
            crate::logger::info("Hardware acceleration detected: NVIDIA NVENC");
            HwAccel::Nvenc
        } else if probe_vaapi() {
            crate::logger::info("Hardware acceleration detected: VA-API");
            HwAccel::Vaapi
        } else {
            crate::logger::info("No hardware acceleration detected; using CPU software encoders");
            HwAccel::None
        }
    })
}

pub fn resolve_hwaccel(mode_opt: Option<&str>) -> HwAccel {
    match mode_opt.map(|s| s.trim().to_lowercase()).as_deref() {
        Some("nvenc") => {
            if probe_nvenc() {
                HwAccel::Nvenc
            } else {
                crate::logger::warn(
                    "NVENC requested but not supported on this device, falling back to CPU",
                );
                HwAccel::None
            }
        }
        Some("vaapi") => {
            if probe_vaapi() {
                HwAccel::Vaapi
            } else {
                crate::logger::warn(
                    "VA-API requested but not supported on this device, falling back to CPU",
                );
                HwAccel::None
            }
        }
        Some("none") | Some("cpu") => HwAccel::None,
        Some("auto") | None | Some("") => detect_hardware_acceleration(),
        _ => detect_hardware_acceleration(),
    }
}

pub fn is_supported_video_target(target: &str) -> bool {
    matches!(
        target.to_lowercase().as_str(),
        "mp4" | "mkv" | "webm" | "avi" | "mov" | "flv" | "av1"
    )
}

pub type VideoProgressFn<'a> = &'a mut (dyn FnMut(i32, &str) + 'a);

pub fn run_ffmpeg_command<F>(
    mut cmd: Command,
    duration_secs: Option<f64>,
    on_progress: Option<&mut F>,
) -> Result<()>
where
    F: FnMut(i32, &str) + ?Sized,
{
    if let Some(cb) = on_progress {
        cmd.arg("-nostdin")
            .arg("-progress")
            .arg("pipe:1")
            .arg("-nostats");
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::null());

        let mut child = cmd.spawn().with_context(|| "ffmpeg çalıştırılamadı")?;

        if let Some(stdout) = child.stdout.take() {
            let reader = std::io::BufReader::new(stdout);
            let mut last_emit = std::time::Instant::now();
            let mut current_speed = 1.0f64;

            for line in reader.lines().map_while(Result::ok) {
                if let Some((k, v)) = line.split_once('=') {
                    let k = k.trim();
                    let v = v.trim();
                    if k == "speed"
                        && let Some(s) = v.strip_suffix('x')
                        && let Ok(sp) = s.parse::<f64>()
                        && sp > 0.05
                    {
                        current_speed = sp;
                    } else if (k == "out_time_us" || k == "out_time_ms")
                        && let Ok(us) = v.parse::<i64>()
                    {
                        let cur_secs = (us as f64) / 1_000_000.0;
                        if let Some(total_dur) = duration_secs
                            && total_dur > 0.0
                        {
                            let pct = ((cur_secs / total_dur) * 100.0).clamp(0.0, 99.0) as i32;
                            let remaining = ((total_dur - cur_secs) / current_speed).max(0.0);
                            if last_emit.elapsed().as_millis() >= 250 {
                                let rem_secs = remaining.round() as u64;
                                let eta_str = if rem_secs < 60 {
                                    format!(" (ETA: {}s)", rem_secs)
                                } else {
                                    format!(" (ETA: {}m {}s)", rem_secs / 60, rem_secs % 60)
                                };
                                let detail = format!("{}%{}", pct, eta_str);
                                cb(pct, &detail);
                                last_emit = std::time::Instant::now();
                            }
                        }
                    }
                }
            }
        }

        let status = child.wait().with_context(|| "ffmpeg işlemi beklenemedi")?;
        if !status.success() {
            bail!("{}", crate::i18n::t("err_ffmpeg_failed"));
        }
    } else {
        cmd.arg("-nostdin");
        let status = cmd.status().with_context(|| "ffmpeg çalıştırılamadı")?;
        if !status.success() {
            bail!("{}", crate::i18n::t("err_ffmpeg_failed"));
        }
    }
    Ok(())
}

fn build_convert_command(
    input_path: &Path,
    output_path: &Path,
    target: &str,
    crf: Option<u8>,
    preset: Option<&str>,
    lossless: bool,
    hwaccel: HwAccel,
) -> Command {
    let mut cmd = Command::new("ffmpeg");
    cmd.arg("-y");

    let vaapi_node = if Path::new("/dev/dri/renderD128").exists() {
        "/dev/dri/renderD128"
    } else {
        "/dev/dri/renderD129"
    };

    if hwaccel == HwAccel::Vaapi {
        cmd.arg("-vaapi_device").arg(vaapi_node);
    }

    cmd.arg("-i")
        .arg(input_path)
        .arg("-threads")
        .arg(crate::converter::ffmpeg_threads().to_string());

    if lossless {
        match target {
            "webm" => {
                cmd.arg("-c:v").arg("libvpx-vp9").arg("-lossless").arg("1");
            }
            "av1" => {
                cmd.arg("-c:v")
                    .arg("libsvtav1")
                    .arg("-crf")
                    .arg("0")
                    .arg("-c:a")
                    .arg("flac");
            }
            _ => match hwaccel {
                HwAccel::Nvenc => {
                    cmd.arg("-c:v")
                        .arg("h264_nvenc")
                        .arg("-preset")
                        .arg("p7")
                        .arg("-tune")
                        .arg("lossless")
                        .arg("-c:a")
                        .arg("aac");
                }
                _ => {
                    cmd.arg("-c:v").arg("libx264").arg("-crf").arg("0");
                }
            },
        }
    } else {
        match target {
            "av1" => {
                let crf_val = crf.unwrap_or(28);
                match hwaccel {
                    HwAccel::Nvenc if probe_av1_nvenc() => {
                        cmd.arg("-c:v")
                            .arg("av1_nvenc")
                            .arg("-preset")
                            .arg("p4")
                            .arg("-cq:v")
                            .arg(crf_val.to_string())
                            .arg("-pix_fmt")
                            .arg("yuv420p")
                            .arg("-c:a")
                            .arg("aac");
                    }
                    HwAccel::Vaapi if probe_av1_vaapi() => {
                        cmd.arg("-vf")
                            .arg("format=nv12,hwupload")
                            .arg("-c:v")
                            .arg("av1_vaapi")
                            .arg("-qp")
                            .arg(crf_val.to_string())
                            .arg("-c:a")
                            .arg("aac");
                    }
                    _ => {
                        cmd.arg("-c:v")
                            .arg("libsvtav1")
                            .arg("-crf")
                            .arg(crf_val.to_string())
                            .arg("-preset")
                            .arg(preset.unwrap_or("6"))
                            .arg("-pix_fmt")
                            .arg("yuv420p")
                            .arg("-c:a")
                            .arg("aac");
                    }
                }
            }
            "webm" => {
                let crf_val = crf.unwrap_or(30);
                match hwaccel {
                    HwAccel::Vaapi => {
                        cmd.arg("-vf")
                            .arg("format=nv12,hwupload")
                            .arg("-c:v")
                            .arg("vp9_vaapi")
                            .arg("-qp")
                            .arg(crf_val.to_string())
                            .arg("-c:a")
                            .arg("libopus");
                    }
                    _ => {
                        cmd.arg("-c:v")
                            .arg("libvpx-vp9")
                            .arg("-b:v")
                            .arg("0")
                            .arg("-crf")
                            .arg(crf_val.to_string())
                            .arg("-c:a")
                            .arg("libopus");
                    }
                }
            }
            "avi" => {
                cmd.arg("-c:v")
                    .arg("mpeg4")
                    .arg("-qscale:v")
                    .arg("3")
                    .arg("-c:a")
                    .arg("mp3");
            }
            "flv" => {
                cmd.arg("-c:v")
                    .arg("flv1")
                    .arg("-qscale:v")
                    .arg("3")
                    .arg("-c:a")
                    .arg("mp3");
            }
            _ => {
                let crf_val = crf.unwrap_or(23);
                match hwaccel {
                    HwAccel::Nvenc => {
                        cmd.arg("-c:v")
                            .arg("h264_nvenc")
                            .arg("-preset")
                            .arg("p4")
                            .arg("-cq:v")
                            .arg(crf_val.to_string())
                            .arg("-pix_fmt")
                            .arg("yuv420p")
                            .arg("-c:a")
                            .arg("aac");
                    }
                    HwAccel::Vaapi => {
                        cmd.arg("-vf")
                            .arg("format=nv12,hwupload")
                            .arg("-c:v")
                            .arg("h264_vaapi")
                            .arg("-qp")
                            .arg(crf_val.to_string())
                            .arg("-c:a")
                            .arg("aac");
                    }
                    HwAccel::None => {
                        cmd.arg("-c:v")
                            .arg("libx264")
                            .arg("-crf")
                            .arg(crf_val.to_string())
                            .arg("-pix_fmt")
                            .arg("yuv420p")
                            .arg("-c:a")
                            .arg("aac");
                        if let Some(p) = preset {
                            cmd.arg("-preset").arg(p);
                        }
                    }
                }
            }
        }
    }

    cmd.arg(output_path);
    cmd
}

#[allow(dead_code)]
pub fn convert_video(
    input_path: &Path,
    target_ext: &str,
    crf: Option<u8>,
    preset: Option<&str>,
    lossless: bool,
    custom_output_dir: Option<&Path>,
) -> Result<PathBuf> {
    convert_video_with_progress(
        input_path,
        target_ext,
        crf,
        preset,
        lossless,
        None,
        custom_output_dir,
        None,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn convert_video_with_progress(
    input_path: &Path,
    target_ext: &str,
    crf: Option<u8>,
    preset: Option<&str>,
    lossless: bool,
    hwaccel_mode: Option<&str>,
    custom_output_dir: Option<&Path>,
    mut on_progress: Option<VideoProgressFn<'_>>,
) -> Result<PathBuf> {
    let target = target_ext.to_lowercase();
    let output_path = if target == "av1" {
        crate::converter::claim_output_path(crate::converter::determine_suffixed_output_path(
            input_path,
            "av1",
            "mp4",
            custom_output_dir,
        ))
    } else {
        crate::converter::claim_output_path(crate::converter::determine_output_path(
            input_path,
            &target,
            custom_output_dir,
        ))
    };
    crate::converter::ensure_parent_dir(&output_path)?;
    let pending = crate::converter::PendingOutput::new(&output_path);

    let duration = if on_progress.is_some() {
        get_video_duration(input_path).ok()
    } else {
        None
    };

    let effective_hwaccel = resolve_hwaccel(hwaccel_mode);
    let cmd = build_convert_command(
        input_path,
        pending.path(),
        &target,
        crf,
        preset,
        lossless,
        effective_hwaccel,
    );

    let res = run_ffmpeg_command(cmd, duration, on_progress.as_deref_mut());
    if res.is_err() && effective_hwaccel != HwAccel::None {
        crate::logger::warn(
            "Hardware acceleration conversion failed, falling back to CPU encoder...",
        );
        let fallback_cmd = build_convert_command(
            input_path,
            pending.path(),
            &target,
            crf,
            preset,
            lossless,
            HwAccel::None,
        );
        run_ffmpeg_command(fallback_cmd, duration, on_progress)?;
    } else {
        res?;
    }

    pending.commit()
}

#[allow(dead_code)]
pub fn convert_to_gif(
    input_path: &Path,
    custom_output_dir: Option<&Path>,
    duration_secs: f64,
    fps: u32,
) -> Result<PathBuf> {
    convert_to_gif_with_progress(input_path, custom_output_dir, duration_secs, fps, None)
}

pub fn convert_to_gif_with_progress(
    input_path: &Path,
    custom_output_dir: Option<&Path>,
    duration_secs: f64,
    fps: u32,
    on_progress: Option<VideoProgressFn<'_>>,
) -> Result<PathBuf> {
    let output_path = crate::converter::claim_output_path(crate::converter::determine_output_path(
        input_path,
        "gif",
        custom_output_dir,
    ));
    crate::converter::ensure_parent_dir(&output_path)?;
    let pending = crate::converter::PendingOutput::new(&output_path);

    let mut cmd = Command::new("ffmpeg");
    cmd.arg("-y");
    if duration_secs > 0.0 {
        // `--gif-duration 0` verilirse video tamamı dönüştürülür.
        cmd.arg("-t").arg(duration_secs.to_string());
    }
    cmd.arg("-i")
        .arg(input_path)
        .arg("-threads")
        .arg(crate::converter::ffmpeg_threads().to_string())
        .arg("-vf")
        .arg(format!(
            "fps={},scale='min(640,iw)':-1:flags=lanczos,split[s0][s1];[s0]palettegen[p];[s1][p]paletteuse",
            fps.max(1)
        ))
        .arg(pending.path());

    if duration_secs > 0.0 {
        eprintln!(
            "Not: GIF ilk {:.0} saniye ile sınırlandı (tamamı için: --gif-duration 0).",
            duration_secs
        );
    }

    let total_dur = if duration_secs > 0.0 {
        Some(duration_secs)
    } else {
        get_video_duration(input_path).ok()
    };

    run_ffmpeg_command(cmd, total_dur, on_progress)?;

    pending.commit()
}

pub fn extract_frames(input_path: &Path, custom_output_dir: Option<&Path>) -> Result<PathBuf> {
    let file_stem = input_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("video");

    let base_dir = crate::converter::get_output_dir(input_path, custom_output_dir);
    let frames_dir =
        crate::converter::claim_output_path(base_dir.join(format!("{}_frames", file_stem)));
    fs::create_dir_all(&frames_dir)?;

    let output_pattern = frames_dir.join("frame_%04d.png");

    let status = Command::new("ffmpeg")
        .arg("-nostdin")
        .arg("-y")
        .arg("-i")
        .arg(input_path)
        .arg("-threads")
        .arg(crate::converter::ffmpeg_threads().to_string())
        .arg("-vf")
        .arg("fps=1")
        .arg(&output_pattern)
        .status()
        .with_context(|| "ffmpeg çalıştırılamadı (kare çıkarma)")?;

    if !status.success() {
        bail!("{}", crate::i18n::t("err_extract_frames"));
    }

    Ok(frames_dir)
}

pub fn extract_audio(input_path: &Path, custom_output_dir: Option<&Path>) -> Result<PathBuf> {
    crate::converter::audio::convert_audio(input_path, "mp3", Some("256k"), custom_output_dir)
}

#[allow(dead_code)]
pub fn compress_video(input_path: &Path, custom_output_dir: Option<&Path>) -> Result<PathBuf> {
    compress_video_with_progress(input_path, custom_output_dir, None, None)
}

pub fn compress_video_with_progress(
    input_path: &Path,
    custom_output_dir: Option<&Path>,
    hwaccel_mode: Option<&str>,
    mut on_progress: Option<VideoProgressFn<'_>>,
) -> Result<PathBuf> {
    let output_path =
        crate::converter::claim_output_path(crate::converter::determine_suffixed_output_path(
            input_path,
            "compressed",
            "mp4",
            custom_output_dir,
        ));
    crate::converter::ensure_parent_dir(&output_path)?;
    let pending = crate::converter::PendingOutput::new(&output_path);

    let duration = if on_progress.is_some() {
        get_video_duration(input_path).ok()
    } else {
        None
    };

    let effective_hwaccel = resolve_hwaccel(hwaccel_mode);

    let build_compress_cmd = |hw: HwAccel| {
        let mut cmd = Command::new("ffmpeg");
        cmd.arg("-y");
        let vaapi_node = if Path::new("/dev/dri/renderD128").exists() {
            "/dev/dri/renderD128"
        } else {
            "/dev/dri/renderD129"
        };
        if hw == HwAccel::Vaapi {
            cmd.arg("-vaapi_device").arg(vaapi_node);
        }
        cmd.arg("-i")
            .arg(input_path)
            .arg("-threads")
            .arg(crate::converter::ffmpeg_threads().to_string());

        match hw {
            HwAccel::Nvenc => {
                cmd.arg("-c:v")
                    .arg("h264_nvenc")
                    .arg("-preset")
                    .arg("p4")
                    .arg("-cq:v")
                    .arg("28")
                    .arg("-pix_fmt")
                    .arg("yuv420p")
                    .arg("-c:a")
                    .arg("aac")
                    .arg("-b:a")
                    .arg("128k");
            }
            HwAccel::Vaapi => {
                cmd.arg("-vf")
                    .arg("format=nv12,hwupload")
                    .arg("-c:v")
                    .arg("h264_vaapi")
                    .arg("-qp")
                    .arg("28")
                    .arg("-c:a")
                    .arg("aac")
                    .arg("-b:a")
                    .arg("128k");
            }
            HwAccel::None => {
                cmd.arg("-c:v")
                    .arg("libx264")
                    .arg("-crf")
                    .arg("28")
                    .arg("-preset")
                    .arg("slow")
                    .arg("-pix_fmt")
                    .arg("yuv420p")
                    .arg("-c:a")
                    .arg("aac")
                    .arg("-b:a")
                    .arg("128k");
            }
        }
        cmd.arg(pending.path());
        cmd
    };

    let res = run_ffmpeg_command(
        build_compress_cmd(effective_hwaccel),
        duration,
        on_progress.as_deref_mut(),
    );
    if res.is_err() && effective_hwaccel != HwAccel::None {
        crate::logger::warn(
            "Hardware accelerated compression failed, falling back to CPU encoder...",
        );
        run_ffmpeg_command(build_compress_cmd(HwAccel::None), duration, on_progress)?;
    } else {
        res?;
    }

    pending.commit()
}

pub fn strip_video_metadata(
    input_path: &Path,
    custom_output_dir: Option<&Path>,
) -> Result<PathBuf> {
    let ext = input_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("mp4")
        .to_lowercase();
    let output_path =
        crate::converter::claim_output_path(crate::converter::determine_suffixed_output_path(
            input_path,
            "clean",
            &ext,
            custom_output_dir,
        ));
    crate::converter::ensure_parent_dir(&output_path)?;
    let pending = crate::converter::PendingOutput::new(&output_path);

    let status = Command::new("ffmpeg")
        .arg("-nostdin")
        .arg("-y")
        .arg("-i")
        .arg(input_path)
        .arg("-threads")
        .arg(crate::converter::ffmpeg_threads().to_string())
        .arg("-map_metadata")
        .arg("-1")
        .arg("-c")
        .arg("copy")
        .arg(pending.path())
        .status()
        .with_context(|| "ffmpeg çalıştırılamadı (video metadata temizleme)")?;

    if !status.success() {
        bail!("{}", crate::i18n::t("err_video_strip_metadata"));
    }

    pending.commit()
}

pub fn get_video_duration(input_path: &Path) -> Result<f64> {
    let output = Command::new("ffprobe")
        .arg("-v")
        .arg("error")
        .arg("-show_entries")
        .arg("format=duration")
        .arg("-of")
        .arg("default=noprint_wrappers=1:nokey=1")
        .arg(input_path)
        .output()
        .with_context(|| "ffprobe çalıştırılamadı")?;

    if !output.status.success() {
        bail!("{}: {}", crate::i18n::t("err_video_duration"), input_path.display());
    }

    let s = String::from_utf8_lossy(&output.stdout);
    let duration: f64 = s
        .trim()
        .parse()
        .with_context(|| "Video süresi çözümlenemedi")?;
    if duration <= 0.0 {
        bail!("{}: {}", crate::i18n::t("err_invalid_duration"), duration);
    }
    Ok(duration)
}

pub fn calculate_video_bitrates(
    target_bytes: u64,
    duration: f64,
) -> Result<(u32, u32, &'static str)> {
    if target_bytes == 0 {
        bail!("{}", crate::i18n::t("err_target_size_zero"));
    }
    if duration <= 0.0 {
        bail!("{}: {}", crate::i18n::t("err_invalid_duration"), duration);
    }

    let target_mb = (target_bytes as f64) / (1024.0 * 1024.0);
    let available_bits = (target_bytes as f64) * 8.0 * 0.95;
    let total_kbps = ((available_bits / duration) / 1000.0) as u32;

    // Minimum 114 kbps (50 video + 64 audio) gerekir, altı hedef boyutu garanti edemez
    if total_kbps < 114 {
        bail!(
            "{}: {:.0}s -> {:.1} MB",
            crate::i18n::t("err_target_size_not_reached"),
            duration,
            target_mb
        );
    }

    let audio_kbps: u32 = if total_kbps < 300 {
        64
    } else if total_kbps < 800 {
        96
    } else {
        128
    };

    let video_kbps = if total_kbps > audio_kbps + 50 {
        total_kbps - audio_kbps
    } else {
        50
    };

    // Sonek yalnızca tam preset hedeflerinde sabit; ara boyutlar `target_size`.
    let suffix = if target_bytes == 8 * 1024 * 1024 {
        "discord_8mb"
    } else if target_bytes == 25 * 1024 * 1024 {
        "email_25mb"
    } else {
        "target_size"
    };

    Ok((video_kbps, audio_kbps, suffix))
}

fn build_target_size_command(
    input_path: &Path,
    output_path: &Path,
    video_kbps: u32,
    audio_kbps: u32,
    hwaccel: HwAccel,
) -> Command {
    let mut cmd = Command::new("ffmpeg");
    cmd.arg("-y");

    let vaapi_node = if Path::new("/dev/dri/renderD128").exists() {
        "/dev/dri/renderD128"
    } else {
        "/dev/dri/renderD129"
    };

    if hwaccel == HwAccel::Vaapi {
        cmd.arg("-vaapi_device").arg(vaapi_node);
    }

    cmd.arg("-i")
        .arg(input_path)
        .arg("-threads")
        .arg(crate::converter::ffmpeg_threads().to_string());

    match hwaccel {
        HwAccel::Nvenc => {
            cmd.arg("-c:v")
                .arg("h264_nvenc")
                .arg("-preset")
                .arg("p4")
                .arg("-b:v")
                .arg(format!("{}k", video_kbps))
                .arg("-maxrate")
                .arg(format!("{}k", (video_kbps as f64 * 1.25) as u32))
                .arg("-bufsize")
                .arg(format!("{}k", video_kbps * 2))
                .arg("-pix_fmt")
                .arg("yuv420p");
        }
        HwAccel::Vaapi => {
            cmd.arg("-vf")
                .arg("format=nv12,hwupload")
                .arg("-c:v")
                .arg("h264_vaapi")
                .arg("-b:v")
                .arg(format!("{}k", video_kbps))
                .arg("-maxrate")
                .arg(format!("{}k", (video_kbps as f64 * 1.25) as u32));
        }
        HwAccel::None => {
            cmd.arg("-c:v")
                .arg("libx264")
                .arg("-b:v")
                .arg(format!("{}k", video_kbps))
                .arg("-maxrate")
                .arg(format!("{}k", (video_kbps as f64 * 1.25) as u32))
                .arg("-bufsize")
                .arg(format!("{}k", video_kbps * 2))
                .arg("-preset")
                .arg("medium");

            if video_kbps < 400 {
                cmd.arg("-vf")
                    .arg("scale='trunc(min(iw,854)/2)*2:trunc(min(ih,480)/2)*2'");
            } else if video_kbps < 900 {
                cmd.arg("-vf")
                    .arg("scale='trunc(min(iw,1280)/2)*2:trunc(min(ih,720)/2)*2'");
            }
        }
    }

    cmd.arg("-c:a")
        .arg("aac")
        .arg("-b:a")
        .arg(format!("{}k", audio_kbps))
        .arg(output_path);
    cmd
}

#[allow(dead_code)]
pub fn compress_video_to_size(
    input_path: &Path,
    target_bytes: u64,
    custom_output_dir: Option<&Path>,
) -> Result<PathBuf> {
    compress_video_to_size_with_progress(input_path, target_bytes, custom_output_dir, None, None)
}

pub fn compress_video_to_size_with_progress(
    input_path: &Path,
    target_bytes: u64,
    custom_output_dir: Option<&Path>,
    hwaccel_mode: Option<&str>,
    mut on_progress: Option<VideoProgressFn<'_>>,
) -> Result<PathBuf> {
    if target_bytes == 0 {
        bail!("{}", crate::i18n::t("err_target_size_zero"));
    }

    let current_bytes = fs::metadata(input_path).map(|m| m.len()).unwrap_or(0);
    if current_bytes == 0 {
        bail!("{}: {}", crate::i18n::t("err_file_empty_or_unreadable"), input_path.display());
    }

    if current_bytes <= target_bytes {
        bail!(
            "{} ({} <= {}).",
            crate::i18n::t("err_file_already_smaller"),
            crate::history::format_bytes(current_bytes),
            crate::history::format_bytes(target_bytes)
        );
    }

    let duration = get_video_duration(input_path)?;
    let (initial_video_kbps, audio_kbps, suffix) =
        calculate_video_bitrates(target_bytes, duration)?;

    let output_path =
        crate::converter::claim_output_path(crate::converter::determine_suffixed_output_path(
            input_path,
            suffix,
            "mp4",
            custom_output_dir,
        ));
    crate::converter::ensure_parent_dir(&output_path)?;
    let pending = crate::converter::PendingOutput::new(&output_path);

    let effective_hwaccel = resolve_hwaccel(hwaccel_mode);

    // Tek geçiş hedefi aşabilir: çıktıyı ölç, aşarsa bit hızını düşürüp
    // en fazla 3 kez dene (hedef boyut sözleşmesi).
    let mut video_kbps = initial_video_kbps;
    let mut best_size: Option<u64> = None;

    for attempt in 0..3 {
        if attempt > 0 {
            video_kbps = (video_kbps as f64 * 0.8).round() as u32;
            if video_kbps < 50 {
                break;
            }
        }

        let cmd = build_target_size_command(
            input_path,
            pending.path(),
            video_kbps,
            audio_kbps,
            effective_hwaccel,
        );
        let prog_cb = on_progress.as_deref_mut();
        let res = run_ffmpeg_command(cmd, Some(duration), prog_cb);
        if res.is_err() && effective_hwaccel != HwAccel::None {
            crate::logger::warn(
                "Hardware acceleration failed for target size compression, falling back to CPU encoder...",
            );
            let fallback_cmd = build_target_size_command(
                input_path,
                pending.path(),
                video_kbps,
                audio_kbps,
                HwAccel::None,
            );
            let prog_cb2 = on_progress.as_deref_mut();
            run_ffmpeg_command(fallback_cmd, Some(duration), prog_cb2)?;
        } else {
            res?;
        }

        let Ok(meta) = fs::metadata(pending.path()) else {
            continue;
        };
        let size = meta.len();
        if size <= target_bytes {
            return pending.commit();
        }
        best_size = Some(best_size.map_or(size, |best| best.min(size)));
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
    bail!("{}", crate::i18n::t("err_video_target_size_failed"))
}

pub fn web_optimize_video(input_path: &Path, custom_output_dir: Option<&Path>) -> Result<PathBuf> {
    let output_path =
        crate::converter::claim_output_path(crate::converter::determine_suffixed_output_path(
            input_path,
            "web",
            "mp4",
            custom_output_dir,
        ));
    crate::converter::ensure_parent_dir(&output_path)?;
    let pending = crate::converter::PendingOutput::new(&output_path);

    let mut cmd = Command::new("ffmpeg");
    cmd.arg("-nostdin")
        .arg("-y")
        .arg("-i")
        .arg(input_path)
        .arg("-threads")
        .arg(crate::converter::ffmpeg_threads().to_string())
        .arg("-c:v")
        .arg("libx264")
        .arg("-crf")
        .arg("23")
        .arg("-preset")
        .arg("medium")
        .arg("-c:a")
        .arg("aac")
        .arg("-b:a")
        .arg("128k")
        .arg("-movflags")
        .arg("+faststart")
        .arg(pending.path());

    let status = cmd
        .status()
        .with_context(|| "ffmpeg çalıştırılamadı (web video optimizasyonu)")?;
    if !status.success() {
        bail!("{}", crate::i18n::t("err_web_video_opt"));
    }

    pending.commit()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_video_bitrates_8mb() {
        let (video_k, audio_k, suffix) = calculate_video_bitrates(8 * 1024 * 1024, 60.0).unwrap();
        assert_eq!(suffix, "discord_8mb");
        assert!(video_k > 0);
        assert!(audio_k >= 64);
    }

    #[test]
    fn test_calculate_video_bitrates_25mb() {
        let (video_k, audio_k, suffix) = calculate_video_bitrates(25 * 1024 * 1024, 120.0).unwrap();
        assert_eq!(suffix, "email_25mb");
        assert!(video_k > 0);
        assert!(audio_k >= 96);
    }

    #[test]
    fn test_calculate_video_bitrates_invalid() {
        assert!(calculate_video_bitrates(0, 60.0).is_err());
        assert!(calculate_video_bitrates(8 * 1024 * 1024, 0.0).is_err());
        assert!(calculate_video_bitrates(8 * 1024 * 1024, -10.0).is_err());
        // Çok uzun video, çok küçük hedef -> imkansız bitrate hatası
        assert!(calculate_video_bitrates(1024 * 1024, 3600.0).is_err());
    }

    #[test]
    fn test_is_supported_video_target() {
        assert!(is_supported_video_target("mp4"));
        assert!(is_supported_video_target("mkv"));
        assert!(is_supported_video_target("webm"));
        assert!(is_supported_video_target("av1"));
        assert!(!is_supported_video_target("compress"));
        assert!(!is_supported_video_target("discord_8mb"));
    }

    #[test]
    fn test_hwaccel_resolve() {
        assert_eq!(resolve_hwaccel(Some("none")), HwAccel::None);
        assert_eq!(resolve_hwaccel(Some("cpu")), HwAccel::None);
    }

    #[test]
    fn test_compress_video_to_size_size_guards() {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("konvert_test_small.mp4");
        std::fs::write(&test_file, b"fake video bytes").unwrap();

        // 8MB hedef boyutu, dosya çok küçük -> hata vermeli
        let res = compress_video_to_size(&test_file, 8 * 1024 * 1024, None);
        assert!(res.is_err());
        let err_msg = res.unwrap_err().to_string();
        assert!(err_msg.contains("zaten hedef boyuttan") || err_msg.contains("already smaller"));

        // 0 hedef boyutu -> hata vermeli
        assert!(compress_video_to_size(&test_file, 0, None).is_err());

        let _ = std::fs::remove_file(&test_file);
    }
}
