use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn is_supported_media_target(target: &str) -> bool {
    matches!(
        target.to_lowercase().as_str(),
        "mp3" | "wav" | "flac" | "ogg" | "m4a" | "opus" | "mp4" | "webm" | "gif" | "mkv"
    )
}

pub fn convert_media(
    input_path: &Path,
    target_ext: &str,
    custom_output_dir: Option<&Path>,
) -> Result<PathBuf> {
    let output_path = crate::converter::image::determine_output_path(
        input_path,
        target_ext,
        custom_output_dir,
    );

    let mut cmd = Command::new("ffmpeg");
    cmd.arg("-y") // Overwrite output files without asking
        .arg("-i")
        .arg(input_path);

    match target_ext.to_lowercase().as_str() {
        "gif" => {
            // High quality gif palette generation
            cmd.arg("-vf").arg("fps=15,scale=flags=lanczos,split[s0][s1];[s0]palettegen[p];[s1][p]paletteuse");
        }
        "mp3" => {
            cmd.arg("-b:a").arg("320k");
        }
        _ => {}
    }

    cmd.arg(&output_path);

    let status = cmd
        .status()
        .with_context(|| "ffmpeg komutu çalıştırılamadı. Lütfen sisteminizde ffmpeg kurulu olduğundan emin olun.")?;

    if !status.success() {
        bail!("ffmpeg dönüştürme işlemi hata ile sonuçlandı");
    }

    Ok(output_path)
}
