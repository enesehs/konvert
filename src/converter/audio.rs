use anyhow::{Context, Result, bail};
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn is_supported_audio_target(target: &str) -> bool {
    matches!(
        target.to_lowercase().as_str(),
        "mp3" | "wav" | "flac" | "ogg" | "opus" | "m4a" | "aac"
    )
}

pub fn convert_audio_to_video(
    input_path: &Path,
    custom_output_dir: Option<&Path>,
) -> Result<PathBuf> {
    let output_path = crate::converter::claim_output_path(crate::converter::determine_output_path(
        input_path,
        "mp4",
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
        .arg("-filter_complex")
        .arg("[0:a]showwaves=s=1280x720:mode=line:colors=0x3daee9[v]")
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
        .arg(pending.path())
        .status()
        .with_context(|| "ffmpeg çalıştırılamadı (ses -> video)")?;

    if !status.success() {
        bail!("Sesten video oluşturma başarısız oldu");
    }

    pending.commit()
}

pub fn convert_audio(
    input_path: &Path,
    target_ext: &str,
    bitrate: Option<&str>,
    custom_output_dir: Option<&Path>,
) -> Result<PathBuf> {
    let target = target_ext.to_lowercase();
    let output_path = crate::converter::claim_output_path(crate::converter::determine_output_path(
        input_path,
        &target,
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
        .arg("-vn"); // Gömülü kapak resimlerini yoksay (ses codec hatalarını önler)

    let default_bitrate = match target.as_str() {
        "opus" | "ogg" | "aac" | "m4a" => "160k",
        _ => "256k",
    };
    let b_rate = bitrate.unwrap_or(default_bitrate);

    match target.as_str() {
        "mp3" | "ogg" | "opus" | "m4a" | "aac" => {
            cmd.arg("-b:a").arg(b_rate);
        }
        "flac" => {
            cmd.arg("-c:a")
                .arg("flac")
                .arg("-compression_level")
                .arg("8");
        }
        "wav" => {
            cmd.arg("-c:a").arg("pcm_s16le");
        }
        _ => {}
    }

    cmd.arg(pending.path());

    let status = cmd
        .status()
        .with_context(|| "ffmpeg çalıştırılamadı (ses dönüştürme)")?;
    if !status.success() {
        bail!(
            "Ses dönüştürme işlemi başarısız oldu: {}",
            output_path.display()
        );
    }

    pending.commit()
}

pub fn strip_audio_metadata(
    input_path: &Path,
    custom_output_dir: Option<&Path>,
) -> Result<PathBuf> {
    let ext = input_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("mp3")
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
        .arg("-vn")
        .arg("-map_metadata")
        .arg("-1")
        .arg("-c")
        .arg("copy")
        .arg(pending.path())
        .status()
        .with_context(|| "ffmpeg çalıştırılamadı (ses metadata temizleme)")?;

    if !status.success() {
        bail!("Ses metadata temizleme başarısız oldu");
    }

    pending.commit()
}
