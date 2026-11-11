use anyhow::{bail, Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn is_supported_video_target(target: &str) -> bool {
    matches!(
        target.to_lowercase().as_str(),
        "mp4" | "mkv" | "webm" | "avi" | "mov" | "flv" | "gif" | "frames" | "png-frames" | "extract-audio" | "extract_audio" | "compress" | "discord_8mb" | "target_8mb" | "email_25mb" | "target_25mb"
    )
}

pub fn convert_video(
    input_path: &Path,
    target_ext: &str,
    crf: Option<u8>,
    preset: Option<&str>,
    lossless: bool,
    custom_output_dir: Option<&Path>,
) -> Result<PathBuf> {
    let target = target_ext.to_lowercase();

    // 1. Videodan ses ayıklama
    if target == "extract-audio" || target == "extract_audio" {
        return crate::converter::audio::convert_audio(input_path, "mp3", Some("320k"), custom_output_dir);
    }

    // 2. Kareleri PNG klasörüne çıkarma
    if target == "frames" || target == "png-frames" {
        return extract_frames_to_folder(input_path, custom_output_dir);
    }

    // 3. Akıllı sınırlamalı yüksek kaliteli GIF
    if target == "gif" {
        let output_path = crate::converter::image::determine_output_path(
            input_path,
            "gif",
            custom_output_dir,
        );

        // Maksimum 15 saniye sınırı koy (disk patlamasını ve donmayı önler)
        let status = Command::new("ffmpeg")
            .arg("-y")
            .arg("-threads")
            .arg("0")
            .arg("-t")
            .arg("15") // İlk 15 saniye sınırı
            .arg("-i")
            .arg(input_path)
            .arg("-vf")
            .arg("fps=15,scale=flags=lanczos,split[s0][s1];[s0]palettegen[p];[s1][p]paletteuse")
            .arg(&output_path)
            .status()
            .with_context(|| "ffmpeg çalıştırılamadı (GIF)")?;

        if !status.success() {
            bail!("GIF dönüştürme işlemi başarısız oldu");
        }
        return Ok(output_path);
    }

    // 4. Video -> Video dönüştürme
    let output_path = crate::converter::image::determine_output_path(
        input_path,
        &target,
        custom_output_dir,
    );

    let mut cmd = Command::new("ffmpeg");
    cmd.arg("-y")
        .arg("-threads")
        .arg("0")
        .arg("-i")
        .arg(input_path);

    if lossless {
        match target.as_str() {
            "webm" => {
                cmd.arg("-c:v").arg("libvpx-vp9").arg("-lossless").arg("1");
            }
            _ => {
                cmd.arg("-c:v").arg("libx264").arg("-crf").arg("0");
            }
        }
    } else {
        let crf_val = crf.unwrap_or(23);
        cmd.arg("-crf").arg(crf_val.to_string());
        if let Some(p) = preset {
            cmd.arg("-preset").arg(p);
        }
    }

    cmd.arg(&output_path);

    let status = cmd.status().with_context(|| "ffmpeg çalıştırılamadı")?;
    if !status.success() {
        bail!("Video dönüştürme işlemi başarısız oldu: {}", output_path.display());
    }

    Ok(output_path)
}

pub fn extract_frames_to_folder(
    input_path: &Path,
    custom_output_dir: Option<&Path>,
) -> Result<PathBuf> {
    let file_stem = input_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("video");

    let base_dir = match custom_output_dir {
        Some(d) => d.to_path_buf(),
        None => input_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf(),
    };

    let frames_dir = base_dir.join(format!("{}_frames", file_stem));
    fs::create_dir_all(&frames_dir)?;

    let output_pattern = frames_dir.join("frame_%04d.png");

    let status = Command::new("ffmpeg")
        .arg("-y")
        .arg("-threads")
        .arg("0")
        .arg("-i")
        .arg(input_path)
        .arg(&output_pattern)
        .status()
        .with_context(|| "ffmpeg çalıştırılamadı (kare çıkarma)")?;

    if !status.success() {
        bail!("Video kareleri PNG olarak çıkarılamadı");
    }

    Ok(frames_dir)
}

/// Akıllı video sıkıştırma (CRF 28, slow önayarı ve 128k AAC ses ile en küçük dosya boyutu)
pub fn compress_video(input_path: &Path, custom_output_dir: Option<&Path>) -> Result<PathBuf> {
    let file_stem = input_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("video");

    let target_dir = match custom_output_dir {
        Some(dir) => dir.to_path_buf(),
        None => input_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf(),
    };

    let output_path = target_dir.join(format!("{}_compressed.mp4", file_stem));
    let status = Command::new("ffmpeg")
        .arg("-y")
        .arg("-threads")
        .arg("0")
        .arg("-i")
        .arg(input_path)
        .arg("-c:v")
        .arg("libx264")
        .arg("-crf")
        .arg("28")
        .arg("-preset")
        .arg("slow")
        .arg("-c:a")
        .arg("aac")
        .arg("-b:a")
        .arg("128k")
        .arg(&output_path)
        .status()
        .with_context(|| "ffmpeg çalıştırılamadı")?;

    if !status.success() {
        bail!("Video sıkıştırma başarısız oldu");
    }

    Ok(output_path)
}

/// ffprobe ile video süresini (saniye cinsinden) sorgula
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
        bail!("Video süresi okunamadı: {}", input_path.display());
    }

    let s = String::from_utf8_lossy(&output.stdout);
    let duration: f64 = s.trim().parse().with_context(|| "Video süresi çözümlenemedi")?;
    if duration <= 0.0 {
        bail!("Geçersiz video süresi: {}", duration);
    }
    Ok(duration)
}

/// Hedef boyuta göre akıllı video sıkıştırması (Discord 8MB, E-Posta 25MB vb.)
pub fn compress_video_to_size(
    input_path: &Path,
    target_bytes: u64,
    custom_output_dir: Option<&Path>,
) -> Result<PathBuf> {
    let duration = get_video_duration(input_path).unwrap_or(30.0);
    let target_mb = (target_bytes as f64) / (1024.0 * 1024.0);

    // %5 kapsayıcı ve paketleme payı bırak
    let available_bits = (target_bytes as f64) * 8.0 * 0.95;
    let total_kbps = ((available_bits / duration) / 1000.0) as u32;

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

    let file_stem = input_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("video");

    let target_dir = match custom_output_dir {
        Some(dir) => dir.to_path_buf(),
        None => input_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf(),
    };

    let suffix = if target_mb <= 8.5 {
        "discord_8mb"
    } else if target_mb <= 25.5 {
        "email_25mb"
    } else {
        "target_size"
    };

    let output_path = target_dir.join(format!("{}_{}.mp4", file_stem, suffix));

    let mut cmd = Command::new("ffmpeg");
    cmd.arg("-y")
        .arg("-threads")
        .arg("0")
        .arg("-i")
        .arg(input_path)
        .arg("-c:v")
        .arg("libx264")
        .arg("-b:v")
        .arg(format!("{}k", video_kbps))
        .arg("-maxrate")
        .arg(format!("{}k", (video_kbps as f64 * 1.25) as u32))
        .arg("-bufsize")
        .arg(format!("{}k", video_kbps * 2))
        .arg("-c:a")
        .arg("aac")
        .arg("-b:a")
        .arg(format!("{}k", audio_kbps))
        .arg("-preset")
        .arg("medium");

    // Düşük bit hızlarında çözünürlüğü makul seviyeye indir (aşırı pikselleşmeyi önler)
    if video_kbps < 400 {
        cmd.arg("-vf").arg("scale=-2:'min(ih,480)'");
    } else if video_kbps < 900 {
        cmd.arg("-vf").arg("scale=-2:'min(ih,720)'");
    }

    cmd.arg(&output_path);

    let status = cmd.status().with_context(|| "ffmpeg çalıştırılamadı (hedef boyut)")?;
    if !status.success() {
        bail!("Hedef boyut video sıkıştırma işlemi başarısız oldu");
    }

    Ok(output_path)
}
