use anyhow::{Context, Result};
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

    let mut cmd = Command::new("ffmpeg");
    cmd.arg("-y")
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
        .arg("-b:a")
        .arg("192k")
        .arg("-movflags")
        .arg("+faststart")
        .arg("-shortest")
        .arg(pending.path());

    crate::converter::video::run_ffmpeg_command::<fn(i32, &str)>(cmd, None, None)
        .with_context(|| crate::i18n::t("err_audio_to_video"))?;

    pending.commit()
}

fn parse_audio_bitrate_kbps(raw: &str, default_kbps: u32) -> u32 {
    let trimmed = raw.trim().to_ascii_lowercase();
    if let Some(m_str) = trimmed
        .strip_suffix("mbps")
        .or_else(|| trimmed.strip_suffix('m'))
    {
        if let Ok(mbps) = m_str.trim().parse::<f64>()
            && mbps.is_finite()
            && mbps > 0.0
        {
            return (mbps * 1000.0).round() as u32;
        }
        return default_kbps;
    }
    let num_str = trimmed
        .strip_suffix("kbps")
        .or_else(|| trimmed.strip_suffix('k'))
        .unwrap_or(&trimmed)
        .trim();
    match num_str.parse::<u32>() {
        Ok(0) => default_kbps,
        Ok(val) if val >= 8000 => (val / 1000).max(1),
        Ok(val) => val,
        Err(_) => default_kbps,
    }
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
    cmd.arg("-y")
        .arg("-i")
        .arg(input_path)
        .arg("-threads")
        .arg(crate::converter::ffmpeg_threads().to_string())
        .arg("-vn"); // Gömülü kapak resimlerini yoksay (ses codec hatalarını önler)

    let default_kbps = match target.as_str() {
        "opus" | "ogg" | "aac" | "m4a" => 160,
        _ => 256,
    };
    let parsed_kbps = bitrate
        .map(|b| parse_audio_bitrate_kbps(b, default_kbps))
        .unwrap_or(default_kbps);

    let clamped_kbps = match target.as_str() {
        "ogg" => parsed_kbps.clamp(32, 192),
        "opus" => parsed_kbps.clamp(6, 510),
        "mp3" => parsed_kbps.clamp(8, 320),
        "m4a" | "aac" => parsed_kbps.clamp(16, 320),
        _ => parsed_kbps,
    };
    let safe_bitrate = format!("{}k", clamped_kbps);

    match target.as_str() {
        "mp3" => {
            cmd.args(["-c:a", "libmp3lame", "-b:a", &safe_bitrate]);
        }
        "ogg" => {
            cmd.args(["-c:a", "libvorbis", "-b:a", &safe_bitrate]);
        }
        "opus" => {
            cmd.args(["-c:a", "libopus", "-b:a", &safe_bitrate]);
        }
        "m4a" => {
            cmd.args([
                "-c:a",
                "aac",
                "-b:a",
                &safe_bitrate,
                "-movflags",
                "+faststart",
            ]);
        }
        "aac" => {
            cmd.args(["-c:a", "aac", "-b:a", &safe_bitrate]);
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

    crate::converter::video::run_ffmpeg_command::<fn(i32, &str)>(cmd, None, None).with_context(
        || {
            format!(
                "{}: {}",
                crate::i18n::t("err_ffmpeg_failed"),
                output_path.display()
            )
        },
    )?;

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

    let mut cmd = Command::new("ffmpeg");
    cmd.arg("-y")
        .arg("-i")
        .arg(input_path)
        .arg("-threads")
        .arg(crate::converter::ffmpeg_threads().to_string())
        .arg("-vn")
        .arg("-map_metadata")
        .arg("-1")
        .arg("-map_metadata:s")
        .arg("-1")
        .arg("-fflags")
        .arg("+bitexact")
        .arg("-c")
        .arg("copy");
    if ext == "m4a" {
        cmd.args(["-movflags", "+faststart"]);
    }
    cmd.arg(pending.path());

    crate::converter::video::run_ffmpeg_command::<fn(i32, &str)>(cmd, None, None)
        .with_context(|| crate::i18n::t("err_audio_strip_metadata"))?;

    pending.commit()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_audio_bitrate_kbps() {
        assert_eq!(parse_audio_bitrate_kbps("192k", 160), 192);
        assert_eq!(parse_audio_bitrate_kbps("320K", 160), 320);
        assert_eq!(parse_audio_bitrate_kbps("128kbps", 160), 128);
        assert_eq!(parse_audio_bitrate_kbps("192000", 160), 192);
        assert_eq!(parse_audio_bitrate_kbps("1m", 160), 1000);
        assert_eq!(parse_audio_bitrate_kbps("invalid", 160), 160);
        assert_eq!(parse_audio_bitrate_kbps("0k", 160), 160);
    }
}
