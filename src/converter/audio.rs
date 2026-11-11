use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn is_supported_audio_target(target: &str) -> bool {
    matches!(
        target.to_lowercase().as_str(),
        "mp3" | "wav" | "flac" | "ogg" | "opus" | "m4a" | "aac" | "audio-to-video"
    )
}

pub fn convert_audio(
    input_path: &Path,
    target_ext: &str,
    bitrate: Option<&str>,
    custom_output_dir: Option<&Path>,
) -> Result<PathBuf> {
    let target = target_ext.to_lowercase();

    // 1. Audio to Video with Waveform Visualizer
    if target == "audio-to-video" || target == "audio_to_video" {
        let output_path = crate::converter::image::determine_output_path(
            input_path,
            "mp4",
            custom_output_dir,
        );

        let status = Command::new("ffmpeg")
            .arg("-y")
            .arg("-threads")
            .arg("0")
            .arg("-i")
            .arg(input_path)
            .arg("-filter_complex")
            .arg("[0:a]showwaves=s=1280x720:mode=line:colors=0x3daee9[v]") // KDE Breeze blue color
            .arg("-map")
            .arg("[v]")
            .arg("-map")
            .arg("0:a")
            .arg("-c:v")
            .arg("libx264")
            .arg("-pix_fmt")
            .arg("yuv420p")
            .arg("-c:a")
            .arg("aac")
            .arg("-shortest")
            .arg(&output_path)
            .status()
            .with_context(|| "ffmpeg çalıştırılamadı (ses -> video)")?;

        if !status.success() {
            bail!("Sesten video oluşturma başarısız oldu");
        }

        return Ok(output_path);
    }

    // 2. Audio to Audio conversion
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

    let b_rate = bitrate.unwrap_or("320k");
    match target.as_str() {
        "mp3" => {
            cmd.arg("-b:a").arg(b_rate);
        }
        "ogg" | "opus" | "m4a" | "aac" => {
            cmd.arg("-b:a").arg(b_rate);
        }
        "flac" => {
            cmd.arg("-c:a").arg("flac").arg("-compression_level").arg("8");
        }
        "wav" => {
            cmd.arg("-c:a").arg("pcm_s16le");
        }
        _ => {}
    }

    cmd.arg(&output_path);

    let status = cmd.status().with_context(|| "ffmpeg çalıştırılamadı (ses dönüştürme)")?;
    if !status.success() {
        bail!("Ses dönüştürme işlemi başarısız oldu: {}", output_path.display());
    }

    Ok(output_path)
}
