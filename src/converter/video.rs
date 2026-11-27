use anyhow::{Context, Result, bail};
use std::fs;
use std::io::{BufRead, Read};
use std::path::{Path, PathBuf};
use std::process::{ChildStderr, Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HwAccel {
    Nvenc,
    Vaapi,
    #[default]
    None,
}

static DETECTED_HWACCEL: std::sync::OnceLock<HwAccel> = std::sync::OnceLock::new();
static DETECTED_NVENC: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
static DETECTED_VAAPI_NODE: std::sync::OnceLock<Option<PathBuf>> = std::sync::OnceLock::new();
static DETECTED_VP9_VAAPI: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
static DETECTED_AV1_NVENC: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
static DETECTED_AV1_VAAPI: std::sync::OnceLock<bool> = std::sync::OnceLock::new();

fn probe_nvenc_encoder(encoder: &str) -> bool {
    let has_nvidia = Path::new("/dev/nvidia0").exists()
        || Path::new("/dev/nvidiactl").exists()
        || Path::new("/dev/nvhost-ctrl").exists();
    if !has_nvidia {
        return false;
    }
    let status = Command::new("ffmpeg")
        .args([
            "-nostdin",
            "-loglevel",
            "error",
            "-f",
            "lavfi",
            "-i",
            "color=c=black:s=256x256:d=0.04",
            "-c:v",
            encoder,
            "-f",
            "null",
            "-",
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    matches!(status, Ok(s) if s.success())
}

fn collect_dri_render_nodes() -> Vec<PathBuf> {
    let mut nodes: Vec<PathBuf> = fs::read_dir("/dev/dri")
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|entry| {
            let name = entry.file_name();
            let s = name.to_string_lossy();
            if s.starts_with("renderD") {
                Some(entry.path())
            } else {
                None
            }
        })
        .collect();
    nodes.sort();
    nodes
}

fn probe_vaapi_encoder_on_node(node: &Path, encoder: &str, extra_args: &[&str]) -> bool {
    let mut cmd = Command::new("ffmpeg");
    cmd.args(["-nostdin", "-loglevel", "error", "-vaapi_device"])
        .arg(node)
        .args([
            "-f",
            "lavfi",
            "-i",
            "color=c=black:s=256x256:d=0.04",
            "-vf",
            "format=nv12,hwupload",
            "-c:v",
            encoder,
        ])
        .args(extra_args)
        .args(["-f", "null", "-"])
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    matches!(cmd.status(), Ok(s) if s.success())
}

pub fn probe_nvenc() -> bool {
    *DETECTED_NVENC.get_or_init(|| probe_nvenc_encoder("h264_nvenc"))
}

pub fn probe_av1_nvenc() -> bool {
    *DETECTED_AV1_NVENC.get_or_init(|| probe_nvenc() && probe_nvenc_encoder("av1_nvenc"))
}

pub fn probe_vaapi_device() -> Option<&'static Path> {
    DETECTED_VAAPI_NODE
        .get_or_init(|| {
            collect_dri_render_nodes()
                .into_iter()
                .find(|node| probe_vaapi_encoder_on_node(node, "h264_vaapi", &[]))
        })
        .as_deref()
}

pub fn probe_vaapi() -> bool {
    probe_vaapi_device().is_some()
}

pub fn probe_vp9_vaapi() -> bool {
    *DETECTED_VP9_VAAPI.get_or_init(|| {
        let Some(node) = probe_vaapi_device() else {
            return false;
        };
        probe_vaapi_encoder_on_node(
            node,
            "vp9_vaapi",
            &["-rc_mode", "CQP", "-global_quality", "128"],
        )
    })
}

pub fn probe_av1_vaapi() -> bool {
    *DETECTED_AV1_VAAPI.get_or_init(|| {
        let Some(node) = probe_vaapi_device() else {
            return false;
        };
        probe_vaapi_encoder_on_node(
            node,
            "av1_vaapi",
            &["-rc_mode", "CQP", "-global_quality", "128"],
        )
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

/// Kanonik 0-51 (x264) CRF ölçeğini 0-63 (libvpx-vp9 / libsvtav1) ölçeğine dönüştürür.
fn map_crf_to_63(crf: u8) -> u8 {
    ((crf.min(51) as f32 * 63.0) / 51.0)
        .round()
        .clamp(0.0, 63.0) as u8
}

/// Kanonik 0-51 (x264) CRF ölçeğini VA-API VP9/AV1 için 1-255 global_quality ölçeğine dönüştürür.
fn map_crf_to_vaapi_255(crf: u8) -> u8 {
    ((crf.min(51) as f32 / 51.0) * 255.0)
        .round()
        .clamp(1.0, 255.0) as u8
}

/// x264 önayar adlarını veya sayısal 0-13 değerlerini libsvtav1 tam sayı önayarına çevirir.
fn map_svtav1_preset(preset: Option<&str>) -> String {
    let Some(raw) = preset.map(str::trim).filter(|s| !s.is_empty()) else {
        return "6".to_string();
    };
    let lower = raw.to_ascii_lowercase();
    if let Ok(n) = lower.parse::<u8>()
        && n <= 13
    {
        return n.to_string();
    }
    match lower.as_str() {
        "ultrafast" => "12",
        "superfast" => "10",
        "veryfast" => "8",
        "faster" => "7",
        "fast" => "6",
        "medium" => "6",
        "slow" => "4",
        "slower" => "3",
        "veryslow" => "2",
        _ => "6",
    }
    .to_string()
}

fn active_vaapi_device_path() -> &'static Path {
    probe_vaapi_device().unwrap_or_else(|| Path::new("/dev/dri/renderD128"))
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

static TWO_PASS_COUNTER: AtomicUsize = AtomicUsize::new(0);

fn spawn_stderr_collector(mut stderr: ChildStderr) -> std::thread::JoinHandle<String> {
    std::thread::spawn(move || {
        const MAX_STDERR_BYTES: usize = 4096;
        let mut ring: Vec<u8> = Vec::with_capacity(MAX_STDERR_BYTES);
        let mut chunk = [0u8; 1024];
        while let Ok(n) = stderr.read(&mut chunk) {
            if n == 0 {
                break;
            }
            let overflow = (ring.len() + n).saturating_sub(MAX_STDERR_BYTES);
            if overflow > 0 {
                let drain_len = overflow.min(ring.len());
                ring.drain(..drain_len);
            }
            let take_from = n.saturating_sub(MAX_STDERR_BYTES);
            ring.extend_from_slice(&chunk[take_from..n]);
        }
        String::from_utf8_lossy(&ring).into_owned()
    })
}

fn summarize_ffmpeg_stderr(stderr_tail: &str) -> String {
    let lines: Vec<&str> = stderr_tail
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect();
    if lines.is_empty() {
        return String::new();
    }
    let start = lines.len().saturating_sub(4);
    let joined = lines[start..].join(" | ");
    let mut clipped: String = joined.chars().take(500).collect();
    if joined.chars().count() > 500 {
        clipped.push_str("...");
    }
    crate::logger::error(&format!("FFmpeg stderr: {}", clipped));
    clipped
}

pub fn run_ffmpeg_command<F>(
    mut cmd: Command,
    duration_secs: Option<f64>,
    on_progress: Option<&mut F>,
) -> Result<()>
where
    F: FnMut(i32, &str) + ?Sized,
{
    crate::process::wait_if_paused_or_cancelled()?;

    if let Some(cb) = on_progress {
        cmd.arg("-nostdin")
            .arg("-progress")
            .arg("pipe:1")
            .arg("-nostats");
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        crate::process::configure_tracked_command(&mut cmd);

        let mut child = cmd.spawn().with_context(|| "ffmpeg çalıştırılamadı")?;
        let _child_guard = crate::process::register_running_child(child.id());
        let stderr_handle = child.stderr.take().map(spawn_stderr_collector);

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
                        if us < 0 {
                            continue;
                        }
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
        let stderr_tail = stderr_handle
            .and_then(|h| h.join().ok())
            .unwrap_or_default();
        if crate::process::is_job_cancelled() {
            return Err(crate::process::cancelled_error());
        }
        if !status.success() {
            let summary = summarize_ffmpeg_stderr(&stderr_tail);
            if summary.is_empty() {
                bail!("{}", crate::i18n::t("err_ffmpeg_failed"));
            }
            bail!("{}: {}", crate::i18n::t("err_ffmpeg_failed"), summary);
        }
    } else {
        cmd.arg("-nostdin");
        cmd.stdout(Stdio::null());
        cmd.stderr(Stdio::piped());
        crate::process::configure_tracked_command(&mut cmd);

        let mut child = cmd.spawn().with_context(|| "ffmpeg çalıştırılamadı")?;
        let _child_guard = crate::process::register_running_child(child.id());
        let stderr_handle = child.stderr.take().map(spawn_stderr_collector);
        let status = child.wait().with_context(|| "ffmpeg işlemi beklenemedi")?;
        let stderr_tail = stderr_handle
            .and_then(|h| h.join().ok())
            .unwrap_or_default();
        if crate::process::is_job_cancelled() {
            return Err(crate::process::cancelled_error());
        }
        if !status.success() {
            let summary = summarize_ffmpeg_stderr(&stderr_tail);
            if summary.is_empty() {
                bail!("{}", crate::i18n::t("err_ffmpeg_failed"));
            }
            bail!("{}: {}", crate::i18n::t("err_ffmpeg_failed"), summary);
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

    let needs_vaapi = match target {
        "webm" => !lossless && hwaccel == HwAccel::Vaapi && probe_vp9_vaapi(),
        "av1" => !lossless && hwaccel == HwAccel::Vaapi && probe_av1_vaapi(),
        "avi" | "flv" => false,
        _ => hwaccel == HwAccel::Vaapi && probe_vaapi(),
    };
    if needs_vaapi {
        cmd.arg("-vaapi_device").arg(active_vaapi_device_path());
    }

    cmd.arg("-i")
        .arg(input_path)
        .arg("-threads")
        .arg(crate::converter::ffmpeg_threads().to_string());

    if lossless {
        match target {
            "webm" => {
                cmd.args([
                    "-c:v",
                    "libvpx-vp9",
                    "-lossless",
                    "1",
                    "-deadline",
                    "good",
                    "-cpu-used",
                    "2",
                    "-row-mt",
                    "1",
                    "-c:a",
                    "libopus",
                ]);
            }
            "av1" => {
                cmd.args([
                    "-c:v",
                    "libsvtav1",
                    "-svtav1-params",
                    "lossless=1",
                    "-preset",
                    "6",
                    "-movflags",
                    "+faststart",
                    "-c:a",
                    "aac",
                ]);
            }
            _ => match hwaccel {
                HwAccel::Nvenc => {
                    cmd.args([
                        "-c:v",
                        "h264_nvenc",
                        "-preset",
                        "p7",
                        "-tune",
                        "lossless",
                        "-c:a",
                        "aac",
                    ]);
                }
                _ => {
                    cmd.args(["-c:v", "libx264", "-crf", "0"]);
                }
            },
        }
    } else {
        match target {
            "av1" => {
                let raw_crf = crf.unwrap_or(28).min(51);
                match hwaccel {
                    HwAccel::Nvenc if probe_av1_nvenc() => {
                        cmd.args([
                            "-c:v",
                            "av1_nvenc",
                            "-preset",
                            "p4",
                            "-rc",
                            "vbr",
                            "-b:v",
                            "0",
                            "-cq:v",
                            &raw_crf.to_string(),
                            "-pix_fmt",
                            "yuv420p",
                            "-movflags",
                            "+faststart",
                            "-c:a",
                            "aac",
                        ]);
                    }
                    HwAccel::Vaapi if probe_av1_vaapi() => {
                        let q = map_crf_to_vaapi_255(raw_crf).to_string();
                        cmd.args([
                            "-vf",
                            "format=nv12,hwupload",
                            "-c:v",
                            "av1_vaapi",
                            "-rc_mode",
                            "CQP",
                            "-global_quality",
                            &q,
                            "-movflags",
                            "+faststart",
                            "-c:a",
                            "aac",
                        ]);
                    }
                    _ => {
                        let crf_63 = map_crf_to_63(raw_crf).to_string();
                        let svt_preset = map_svtav1_preset(preset);
                        cmd.args([
                            "-c:v",
                            "libsvtav1",
                            "-crf",
                            &crf_63,
                            "-preset",
                            &svt_preset,
                            "-pix_fmt",
                            "yuv420p",
                            "-movflags",
                            "+faststart",
                            "-c:a",
                            "aac",
                        ]);
                    }
                }
            }
            "webm" => {
                let raw_crf = crf.unwrap_or(26).min(51);
                match hwaccel {
                    HwAccel::Vaapi if probe_vp9_vaapi() => {
                        let q = map_crf_to_vaapi_255(raw_crf).to_string();
                        cmd.args([
                            "-vf",
                            "format=nv12,hwupload",
                            "-c:v",
                            "vp9_vaapi",
                            "-rc_mode",
                            "CQP",
                            "-global_quality",
                            &q,
                            "-c:a",
                            "libopus",
                            "-b:a",
                            "128k",
                        ]);
                    }
                    _ => {
                        let crf_63 = map_crf_to_63(raw_crf).to_string();
                        cmd.args([
                            "-c:v",
                            "libvpx-vp9",
                            "-b:v",
                            "0",
                            "-crf",
                            &crf_63,
                            "-deadline",
                            "good",
                            "-cpu-used",
                            "2",
                            "-row-mt",
                            "1",
                            "-pix_fmt",
                            "yuv420p",
                            "-c:a",
                            "libopus",
                            "-b:a",
                            "128k",
                        ]);
                    }
                }
            }
            "avi" => {
                cmd.args(["-c:v", "mpeg4", "-qscale:v", "3", "-c:a", "mp3"]);
            }
            "flv" => {
                cmd.args(["-c:v", "flv1", "-qscale:v", "3", "-c:a", "mp3"]);
            }
            _ => {
                let crf_val = crf.unwrap_or(23).min(51);
                match hwaccel {
                    HwAccel::Nvenc => {
                        cmd.args([
                            "-c:v",
                            "h264_nvenc",
                            "-preset",
                            "p4",
                            "-rc",
                            "vbr",
                            "-b:v",
                            "0",
                            "-cq:v",
                            &crf_val.to_string(),
                            "-pix_fmt",
                            "yuv420p",
                            "-c:a",
                            "aac",
                        ]);
                    }
                    HwAccel::Vaapi => {
                        cmd.args([
                            "-vf",
                            "format=nv12,hwupload",
                            "-c:v",
                            "h264_vaapi",
                            "-qp",
                            &crf_val.to_string(),
                            "-c:a",
                            "aac",
                        ]);
                    }
                    HwAccel::None => {
                        cmd.args([
                            "-c:v",
                            "libx264",
                            "-crf",
                            &crf_val.to_string(),
                            "-pix_fmt",
                            "yuv420p",
                            "-c:a",
                            "aac",
                        ]);
                        if let Some(p) = preset {
                            cmd.arg("-preset").arg(p);
                        }
                    }
                }
                if target == "mp4" || target == "mov" {
                    cmd.args(["-movflags", "+faststart"]);
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
    match res {
        Ok(()) => {}
        Err(err) if crate::process::is_cancelled_error(&err) => return Err(err),
        Err(err) => {
            if effective_hwaccel != HwAccel::None {
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
                return Err(err);
            }
        }
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
            "fps={},scale='min(640,iw)':'min(640,ih)':force_original_aspect_ratio=decrease:flags=lanczos,split[s0][s1];[s0]palettegen[p];[s1][p]paletteuse",
            fps.max(1)
        ))
        .arg(pending.path());

    if duration_secs > 0.0 {
        eprintln!(
            "Not: GIF ilk {:.0} saniye ile sınırlandı (tamamı için: --gif-duration 0).",
            duration_secs
        );
    }

    let actual_dur = get_video_duration(input_path).ok();
    let total_dur = if duration_secs > 0.0 {
        Some(actual_dur.map_or(duration_secs, |actual| actual.min(duration_secs)))
    } else {
        actual_dur
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
    fs::create_dir_all(&base_dir)?;
    let frames_dir =
        crate::converter::claim_output_path(base_dir.join(format!("{}_frames", file_stem)));

    static FRAMES_TEMP_COUNTER: std::sync::atomic::AtomicUsize =
        std::sync::atomic::AtomicUsize::new(0);
    let cnt = FRAMES_TEMP_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let staging_dir = base_dir.join(format!(
        ".konvert_tmp_frames_{}_{}",
        std::process::id(),
        cnt
    ));
    struct FramesDirGuard(PathBuf);
    impl Drop for FramesDirGuard {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    fs::create_dir_all(&staging_dir)?;
    let staging_guard = FramesDirGuard(staging_dir.clone());

    let output_pattern = staging_dir.join("frame_%04d.png");

    let mut cmd = Command::new("ffmpeg");
    cmd.arg("-y")
        .arg("-i")
        .arg(input_path)
        .arg("-threads")
        .arg(crate::converter::ffmpeg_threads().to_string())
        .arg("-vf")
        .arg("fps=1")
        .arg(&output_pattern);

    run_ffmpeg_command::<fn(i32, &str)>(cmd, None, None)
        .with_context(|| crate::i18n::t("err_extract_frames"))?;

    if frames_dir.exists() {
        let _ = fs::remove_dir_all(&frames_dir);
    }
    fs::rename(&staging_dir, &frames_dir)
        .with_context(|| format!("Kare klasörü taşınamadı: {}", frames_dir.display()))?;
    std::mem::forget(staging_guard);

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
        if hw == HwAccel::Vaapi {
            cmd.arg("-vaapi_device").arg(active_vaapi_device_path());
        }
        cmd.arg("-i")
            .arg(input_path)
            .arg("-threads")
            .arg(crate::converter::ffmpeg_threads().to_string());

        match hw {
            HwAccel::Nvenc => {
                cmd.args([
                    "-c:v",
                    "h264_nvenc",
                    "-preset",
                    "p4",
                    "-rc",
                    "vbr",
                    "-b:v",
                    "0",
                    "-cq:v",
                    "28",
                    "-pix_fmt",
                    "yuv420p",
                    "-movflags",
                    "+faststart",
                    "-c:a",
                    "aac",
                    "-b:a",
                    "128k",
                ]);
            }
            HwAccel::Vaapi => {
                cmd.args([
                    "-vf",
                    "format=nv12,hwupload",
                    "-c:v",
                    "h264_vaapi",
                    "-qp",
                    "28",
                    "-movflags",
                    "+faststart",
                    "-c:a",
                    "aac",
                    "-b:a",
                    "128k",
                ]);
            }
            HwAccel::None => {
                cmd.args([
                    "-c:v",
                    "libx264",
                    "-crf",
                    "28",
                    "-preset",
                    "slow",
                    "-pix_fmt",
                    "yuv420p",
                    "-movflags",
                    "+faststart",
                    "-c:a",
                    "aac",
                    "-b:a",
                    "128k",
                ]);
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
    match res {
        Ok(()) => {}
        Err(err) if crate::process::is_cancelled_error(&err) => return Err(err),
        Err(err) => {
            if effective_hwaccel != HwAccel::None {
                crate::logger::warn(
                    "Hardware accelerated compression failed, falling back to CPU encoder...",
                );
                run_ffmpeg_command(build_compress_cmd(HwAccel::None), duration, on_progress)?;
            } else {
                return Err(err);
            }
        }
    }

    pending.commit()
}

fn build_strip_metadata_command(input_path: &Path, output_path: &Path, ext: &str) -> Command {
    let mut cmd = Command::new("ffmpeg");
    cmd.args(["-nostdin", "-y", "-i"]).arg(input_path).args([
        "-threads",
        &crate::converter::ffmpeg_threads().to_string(),
        "-map",
        "0",
        "-map",
        "-0:d",
        "-map_metadata",
        "-1",
        "-map_metadata:s",
        "-1",
        "-map_chapters",
        "-1",
        "-fflags",
        "+bitexact",
        "-c",
        "copy",
    ]);
    if matches!(ext, "mp4" | "mov" | "m4v") {
        cmd.args(["-movflags", "+faststart"]);
    }
    cmd.arg(output_path);
    cmd
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

    let mut cmd = build_strip_metadata_command(input_path, pending.path(), &ext);
    let output = crate::process::run_tracked_output(&mut cmd)
        .with_context(|| "ffmpeg çalıştırılamadı (video metadata temizleme)")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let detail = stderr.trim();
        if detail.is_empty() {
            bail!("{}", crate::i18n::t("err_video_strip_metadata"));
        }
        bail!("{}: {}", crate::i18n::t("err_video_strip_metadata"), detail);
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
        bail!(
            "{}: {}",
            crate::i18n::t("err_video_duration"),
            input_path.display()
        );
    }

    let s = String::from_utf8_lossy(&output.stdout);
    let duration: f64 = s
        .trim()
        .parse()
        .with_context(|| "Video süresi çözümlenemedi")?;
    if !duration.is_finite() || duration <= 0.0 {
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
    if !duration.is_finite() || duration <= 0.0 {
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

fn target_size_scale_filter(video_kbps: u32) -> Option<&'static str> {
    if video_kbps < 400 {
        Some(
            "scale=w='min(iw,854)':h='min(ih,480)':force_original_aspect_ratio=decrease:force_divisible_by=2",
        )
    } else if video_kbps < 900 {
        Some(
            "scale=w='min(iw,1280)':h='min(ih,720)':force_original_aspect_ratio=decrease:force_divisible_by=2",
        )
    } else {
        None
    }
}

struct PassLogGuard {
    prefix: PathBuf,
}

impl Drop for PassLogGuard {
    fn drop(&mut self) {
        let base = self.prefix.display().to_string();
        for suffix in ["-0.log", "-0.log.mbtree", "-0.log.temp"] {
            let _ = fs::remove_file(format!("{}{}", base, suffix));
        }
    }
}

fn configure_cpu_target_size_base(input_path: &Path, video_kbps: u32) -> Command {
    let mut cmd = Command::new("ffmpeg");
    cmd.arg("-y")
        .arg("-i")
        .arg(input_path)
        .arg("-threads")
        .arg(crate::converter::ffmpeg_threads().to_string())
        .arg("-c:v")
        .arg("libx264")
        .arg("-b:v")
        .arg(format!("{}k", video_kbps))
        .arg("-maxrate")
        .arg(format!("{}k", (video_kbps as f64 * 1.25) as u32))
        .arg("-bufsize")
        .arg(format!("{}k", video_kbps * 2))
        .arg("-preset")
        .arg("medium")
        .arg("-pix_fmt")
        .arg("yuv420p");

    if let Some(scale) = target_size_scale_filter(video_kbps) {
        cmd.arg("-vf").arg(scale);
    }
    cmd
}

fn build_target_size_pass1_command(
    input_path: &Path,
    pass_log_prefix: &Path,
    video_kbps: u32,
) -> Command {
    let mut cmd = configure_cpu_target_size_base(input_path, video_kbps);
    cmd.args(["-pass", "1", "-passlogfile"])
        .arg(pass_log_prefix)
        .args(["-an", "-f", "null", "/dev/null"]);
    cmd
}

fn build_target_size_pass2_command(
    input_path: &Path,
    output_path: &Path,
    pass_log_prefix: &Path,
    video_kbps: u32,
    audio_kbps: u32,
) -> Command {
    let mut cmd = configure_cpu_target_size_base(input_path, video_kbps);
    cmd.args(["-pass", "2", "-passlogfile"])
        .arg(pass_log_prefix)
        .args(["-movflags", "+faststart", "-c:a", "aac", "-b:a"])
        .arg(format!("{}k", audio_kbps))
        .arg(output_path);
    cmd
}

fn build_target_size_command(
    input_path: &Path,
    output_path: &Path,
    video_kbps: u32,
    audio_kbps: u32,
    hwaccel: HwAccel,
) -> Command {
    let scale_filter = target_size_scale_filter(video_kbps);
    if hwaccel == HwAccel::None {
        let mut cmd = configure_cpu_target_size_base(input_path, video_kbps);
        cmd.args(["-movflags", "+faststart", "-c:a", "aac", "-b:a"])
            .arg(format!("{}k", audio_kbps))
            .arg(output_path);
        return cmd;
    }

    let mut cmd = Command::new("ffmpeg");
    cmd.arg("-y");

    if hwaccel == HwAccel::Vaapi {
        cmd.arg("-vaapi_device").arg(active_vaapi_device_path());
    }

    cmd.arg("-i")
        .arg(input_path)
        .arg("-threads")
        .arg(crate::converter::ffmpeg_threads().to_string());

    match hwaccel {
        HwAccel::Nvenc => {
            if let Some(scale) = scale_filter {
                cmd.arg("-vf").arg(scale);
            }
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
            let vf = if let Some(scale) = scale_filter {
                format!("{},format=nv12,hwupload", scale)
            } else {
                "format=nv12,hwupload".to_string()
            };
            cmd.arg("-vf")
                .arg(vf)
                .arg("-c:v")
                .arg("h264_vaapi")
                .arg("-b:v")
                .arg(format!("{}k", video_kbps))
                .arg("-maxrate")
                .arg(format!("{}k", (video_kbps as f64 * 1.25) as u32));
        }
        HwAccel::None => unreachable!(),
    }

    cmd.args(["-movflags", "+faststart"])
        .arg("-c:a")
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

    let mut effective_hwaccel = resolve_hwaccel(hwaccel_mode);

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

        if effective_hwaccel == HwAccel::None {
            let pass_id = TWO_PASS_COUNTER.fetch_add(1, Ordering::Relaxed);
            let pass_log_prefix = std::env::temp_dir().join(format!(
                "konvert_2pass_{}_{}_{}",
                std::process::id(),
                pass_id,
                attempt
            ));
            let _guard = PassLogGuard {
                prefix: pass_log_prefix.clone(),
            };
            let pass1_cmd =
                build_target_size_pass1_command(input_path, &pass_log_prefix, video_kbps);
            match run_ffmpeg_command::<fn(i32, &str)>(pass1_cmd, Some(duration), None) {
                Ok(()) => {
                    let pass2_cmd = build_target_size_pass2_command(
                        input_path,
                        pending.path(),
                        &pass_log_prefix,
                        video_kbps,
                        audio_kbps,
                    );
                    let prog_cb = on_progress.as_deref_mut();
                    run_ffmpeg_command(pass2_cmd, Some(duration), prog_cb)?;
                }
                Err(err) if crate::process::is_cancelled_error(&err) => return Err(err),
                Err(_) => {
                    crate::logger::warn(
                        "Two-pass analysis failed, falling back to single-pass CPU bitrate encoding...",
                    );
                    let single_cmd = build_target_size_command(
                        input_path,
                        pending.path(),
                        video_kbps,
                        audio_kbps,
                        HwAccel::None,
                    );
                    let prog_cb = on_progress.as_deref_mut();
                    run_ffmpeg_command(single_cmd, Some(duration), prog_cb)?;
                }
            }
        } else {
            let cmd = build_target_size_command(
                input_path,
                pending.path(),
                video_kbps,
                audio_kbps,
                effective_hwaccel,
            );
            let prog_cb = on_progress.as_deref_mut();
            let res = run_ffmpeg_command(cmd, Some(duration), prog_cb);
            if let Err(err) = res {
                if crate::process::is_cancelled_error(&err) {
                    return Err(err);
                }
                crate::logger::warn(
                    "Hardware acceleration failed for target size compression, falling back to CPU encoder...",
                );
                effective_hwaccel = HwAccel::None;
                let fallback_cmd = build_target_size_command(
                    input_path,
                    pending.path(),
                    video_kbps,
                    audio_kbps,
                    HwAccel::None,
                );
                let prog_cb2 = on_progress.as_deref_mut();
                run_ffmpeg_command(fallback_cmd, Some(duration), prog_cb2)?;
            }
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
    cmd.arg("-y")
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
        .arg("-pix_fmt")
        .arg("yuv420p")
        .arg("-c:a")
        .arg("aac")
        .arg("-b:a")
        .arg("128k")
        .arg("-movflags")
        .arg("+faststart")
        .arg(pending.path());

    run_ffmpeg_command::<fn(i32, &str)>(cmd, None, None)
        .with_context(|| crate::i18n::t("err_web_video_opt"))?;

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
        assert!(calculate_video_bitrates(8 * 1024 * 1024, f64::NAN).is_err());
        assert!(calculate_video_bitrates(8 * 1024 * 1024, f64::INFINITY).is_err());
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
        let temp_dir = std::env::temp_dir().join(format!(
            "konvert_vid_size_test_{}_{}",
            std::process::id(),
            chrono::Local::now().timestamp_nanos_opt().unwrap_or(0)
        ));
        std::fs::create_dir_all(&temp_dir).unwrap();
        let test_file = temp_dir.join("small.mp4");
        std::fs::write(&test_file, b"fake video bytes").unwrap();

        // 8MB hedef boyutu, dosya çok küçük -> hata vermeli
        let res = compress_video_to_size(&test_file, 8 * 1024 * 1024, None);
        assert!(res.is_err());
        let err_msg = res.unwrap_err().to_string();
        assert!(err_msg.contains("zaten hedef boyuttan") || err_msg.contains("already smaller"));

        // 0 hedef boyutu -> hata vermeli
        assert!(compress_video_to_size(&test_file, 0, None).is_err());

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_crf_and_svtav1_preset_mappers() {
        assert_eq!(map_crf_to_63(23), 28);
        assert_eq!(map_crf_to_63(51), 63);
        assert_eq!(map_crf_to_63(0), 0);

        assert_eq!(map_crf_to_vaapi_255(0), 1);
        assert_eq!(map_crf_to_vaapi_255(23), 115);
        assert_eq!(map_crf_to_vaapi_255(51), 255);

        assert_eq!(map_svtav1_preset(Some("medium")), "6");
        assert_eq!(map_svtav1_preset(Some("slow")), "4");
        assert_eq!(map_svtav1_preset(Some("ultrafast")), "12");
        assert_eq!(map_svtav1_preset(Some("9")), "9");
        assert_eq!(map_svtav1_preset(None), "6");
    }

    #[test]
    fn test_build_convert_command_webm_av1_mp4_args() {
        let webm_cmd = build_convert_command(
            Path::new("in.mp4"),
            Path::new("out.webm"),
            "webm",
            Some(23),
            Some("medium"),
            false,
            HwAccel::None,
        );
        let webm_args: Vec<String> = webm_cmd
            .get_args()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();
        assert!(webm_args.windows(2).any(|w| w == ["-b:v", "0"]));
        assert!(webm_args.windows(2).any(|w| w == ["-crf", "28"]));
        assert!(webm_args.windows(2).any(|w| w == ["-row-mt", "1"]));
        assert!(webm_args.windows(2).any(|w| w == ["-cpu-used", "2"]));
        assert!(webm_args.windows(2).any(|w| w == ["-deadline", "good"]));
        assert!(webm_args.windows(2).any(|w| w == ["-pix_fmt", "yuv420p"]));

        let av1_cmd = build_convert_command(
            Path::new("in.mp4"),
            Path::new("out.mp4"),
            "av1",
            Some(28),
            Some("medium"),
            false,
            HwAccel::None,
        );
        let av1_args: Vec<String> = av1_cmd
            .get_args()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();
        assert!(av1_args.windows(2).any(|w| w == ["-c:v", "libsvtav1"]));
        assert!(av1_args.windows(2).any(|w| w == ["-preset", "6"]));
        assert!(
            av1_args
                .windows(2)
                .any(|w| w == ["-movflags", "+faststart"])
        );

        let mp4_cmd = build_convert_command(
            Path::new("in.mkv"),
            Path::new("out.mp4"),
            "mp4",
            Some(23),
            Some("medium"),
            false,
            HwAccel::None,
        );
        let mp4_args: Vec<String> = mp4_cmd
            .get_args()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();
        assert!(
            mp4_args
                .windows(2)
                .any(|w| w == ["-movflags", "+faststart"])
        );
    }

    #[test]
    fn test_build_strip_metadata_command_preserves_streams_and_faststart() {
        let cmd = build_strip_metadata_command(Path::new("in.mp4"), Path::new("out.mp4"), "mp4");
        let args: Vec<String> = cmd
            .get_args()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();
        assert!(args.windows(2).any(|w| w == ["-map", "0"]));
        assert!(args.windows(2).any(|w| w == ["-map", "-0:d"]));
        assert!(args.windows(2).any(|w| w == ["-map_metadata", "-1"]));
        assert!(args.windows(2).any(|w| w == ["-map_metadata:s", "-1"]));
        assert!(args.windows(2).any(|w| w == ["-map_chapters", "-1"]));
        assert!(args.windows(2).any(|w| w == ["-fflags", "+bitexact"]));
        assert!(args.windows(2).any(|w| w == ["-c", "copy"]));
        assert!(args.windows(2).any(|w| w == ["-movflags", "+faststart"]));
    }

    #[test]
    fn test_build_target_size_commands_aspect_ratio_faststart_and_2pass() {
        let low_cpu = build_target_size_command(
            Path::new("portrait.mp4"),
            Path::new("out.mp4"),
            300,
            64,
            HwAccel::None,
        );
        let low_cpu_args: Vec<String> = low_cpu
            .get_args()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();
        assert!(
            low_cpu_args
                .windows(2)
                .any(|w| w == ["-pix_fmt", "yuv420p"])
        );
        assert!(
            low_cpu_args
                .windows(2)
                .any(|w| w == ["-movflags", "+faststart"])
        );
        assert!(low_cpu_args.iter().any(|a| {
            a.contains("min(iw,854)")
                && a.contains("force_original_aspect_ratio=decrease:force_divisible_by=2")
        }));

        let mid_vaapi = build_target_size_command(
            Path::new("portrait.mp4"),
            Path::new("out.mp4"),
            700,
            96,
            HwAccel::Vaapi,
        );
        let mid_vaapi_args: Vec<String> = mid_vaapi
            .get_args()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();
        assert!(mid_vaapi_args.iter().any(|a| {
            a.contains("min(iw,1280)")
                && a.contains("force_original_aspect_ratio=decrease:force_divisible_by=2,format=nv12,hwupload")
        }));
        assert!(
            mid_vaapi_args
                .windows(2)
                .any(|w| w == ["-movflags", "+faststart"])
        );

        let mid_nvenc = build_target_size_command(
            Path::new("portrait.mp4"),
            Path::new("out.mp4"),
            700,
            96,
            HwAccel::Nvenc,
        );
        let mid_nvenc_args: Vec<String> = mid_nvenc
            .get_args()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();
        assert!(mid_nvenc_args.iter().any(|a| {
            a.contains("min(iw,1280)")
                && a.contains("force_original_aspect_ratio=decrease:force_divisible_by=2")
        }));
        assert!(
            mid_nvenc_args
                .windows(2)
                .any(|w| w == ["-pix_fmt", "yuv420p"])
        );

        let p1 = build_target_size_pass1_command(
            Path::new("in.mp4"),
            Path::new("/tmp/test_passlog"),
            650,
        );
        let p1_args: Vec<String> = p1
            .get_args()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();
        assert!(p1_args.windows(2).any(|w| w == ["-pass", "1"]));
        assert!(
            p1_args
                .windows(2)
                .any(|w| w == ["-passlogfile", "/tmp/test_passlog"])
        );
        assert!(p1_args.windows(2).any(|w| w == ["-pix_fmt", "yuv420p"]));

        let p2 = build_target_size_pass2_command(
            Path::new("in.mp4"),
            Path::new("out.mp4"),
            Path::new("/tmp/test_passlog"),
            650,
            96,
        );
        let p2_args: Vec<String> = p2
            .get_args()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();
        assert!(p2_args.windows(2).any(|w| w == ["-pass", "2"]));
        assert!(p2_args.windows(2).any(|w| w == ["-movflags", "+faststart"]));

        let nv_mp4 = build_convert_command(
            Path::new("in.mkv"),
            Path::new("out.mp4"),
            "mp4",
            Some(23),
            Some("p4"),
            false,
            HwAccel::Nvenc,
        );
        let nv_args: Vec<String> = nv_mp4
            .get_args()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();
        assert!(nv_args.windows(2).any(|w| w == ["-rc", "vbr"]));
        assert!(nv_args.windows(2).any(|w| w == ["-b:v", "0"]));
        assert!(nv_args.windows(2).any(|w| w == ["-cq:v", "23"]));
    }
}
