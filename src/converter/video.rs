use anyhow::{Context, Result, bail};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn is_supported_video_target(target: &str) -> bool {
    matches!(
        target.to_lowercase().as_str(),
        "mp4" | "mkv" | "webm" | "avi" | "mov" | "flv"
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
        .arg(crate::converter::ffmpeg_threads().to_string());

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
        match target.as_str() {
            "webm" => {
                let crf_val = crf.unwrap_or(30);
                cmd.arg("-c:v")
                    .arg("libvpx-vp9")
                    .arg("-b:v")
                    .arg("0")
                    .arg("-crf")
                    .arg(crf_val.to_string())
                    .arg("-c:a")
                    .arg("libopus");
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
                cmd.arg("-c:v")
                    .arg("libx264")
                    .arg("-crf")
                    .arg(crf_val.to_string())
                    .arg("-c:a")
                    .arg("aac");
                if let Some(p) = preset {
                    cmd.arg("-preset").arg(p);
                }
            }
        }
    }

    cmd.arg(pending.path());

    let status = cmd.status().with_context(|| "ffmpeg çalıştırılamadı")?;
    if !status.success() {
        bail!(
            "Video dönüştürme işlemi başarısız oldu: {}",
            output_path.display()
        );
    }

    pending.commit()
}

pub fn convert_to_gif(
    input_path: &Path,
    custom_output_dir: Option<&Path>,
    duration_secs: f64,
    fps: u32,
) -> Result<PathBuf> {
    let output_path = crate::converter::claim_output_path(crate::converter::determine_output_path(
        input_path,
        "gif",
        custom_output_dir,
    ));
    crate::converter::ensure_parent_dir(&output_path)?;
    let pending = crate::converter::PendingOutput::new(&output_path);

    let mut cmd = Command::new("ffmpeg");
    cmd.arg("-nostdin").arg("-y");
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

    let status = cmd
        .status()
        .with_context(|| "ffmpeg çalıştırılamadı (GIF)")?;

    if !status.success() {
        bail!("GIF dönüştürme işlemi başarısız oldu");
    }

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
        bail!("Video kareleri PNG olarak çıkarılamadı");
    }

    Ok(frames_dir)
}

pub fn extract_audio(input_path: &Path, custom_output_dir: Option<&Path>) -> Result<PathBuf> {
    crate::converter::audio::convert_audio(input_path, "mp3", Some("256k"), custom_output_dir)
}

pub fn compress_video(input_path: &Path, custom_output_dir: Option<&Path>) -> Result<PathBuf> {
    let output_path =
        crate::converter::claim_output_path(crate::converter::determine_suffixed_output_path(
            input_path,
            "compressed",
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
        .arg(pending.path())
        .status()
        .with_context(|| "ffmpeg çalıştırılamadı")?;

    if !status.success() {
        bail!("Video sıkıştırma başarısız oldu");
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
        bail!("Video metadata temizleme başarısız oldu");
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
        bail!("Video süresi okunamadı: {}", input_path.display());
    }

    let s = String::from_utf8_lossy(&output.stdout);
    let duration: f64 = s
        .trim()
        .parse()
        .with_context(|| "Video süresi çözümlenemedi")?;
    if duration <= 0.0 {
        bail!("Geçersiz video süresi: {}", duration);
    }
    Ok(duration)
}

pub fn calculate_video_bitrates(
    target_bytes: u64,
    duration: f64,
) -> Result<(u32, u32, &'static str)> {
    if target_bytes == 0 {
        bail!("Hedef dosya boyutu 0 olamaz");
    }
    if duration <= 0.0 {
        bail!("Geçersiz video süresi: {}", duration);
    }

    let target_mb = (target_bytes as f64) / (1024.0 * 1024.0);
    let available_bits = (target_bytes as f64) * 8.0 * 0.95;
    let total_kbps = ((available_bits / duration) / 1000.0) as u32;

    // Minimum 114 kbps (50 video + 64 audio) gerekir, altı hedef boyutu garanti edemez
    if total_kbps < 114 {
        bail!(
            "Video süresi ({:.0} sn) bu hedef boyut ({:.1} MB) için çok uzun; hedefe ulaşmak mümkün değil",
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
) -> Command {
    let mut cmd = Command::new("ffmpeg");
    cmd.arg("-nostdin")
        .arg("-y")
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
        .arg("-c:a")
        .arg("aac")
        .arg("-b:a")
        .arg(format!("{}k", audio_kbps))
        .arg("-preset")
        .arg("medium");

    // yuv420p için hem genişlik hem yükseklik çift sayı olmak zorundadır
    if video_kbps < 400 {
        cmd.arg("-vf")
            .arg("scale='trunc(min(iw,854)/2)*2:trunc(min(ih,480)/2)*2'");
    } else if video_kbps < 900 {
        cmd.arg("-vf")
            .arg("scale='trunc(min(iw,1280)/2)*2:trunc(min(ih,720)/2)*2'");
    }

    cmd.arg(output_path);
    cmd
}

pub fn compress_video_to_size(
    input_path: &Path,
    target_bytes: u64,
    custom_output_dir: Option<&Path>,
) -> Result<PathBuf> {
    if target_bytes == 0 {
        bail!("Hedef dosya boyutu 0 olamaz");
    }

    let current_bytes = fs::metadata(input_path).map(|m| m.len()).unwrap_or(0);
    if current_bytes == 0 {
        bail!("Dosya boş veya boyutu okunamadı: {}", input_path.display());
    }

    if current_bytes <= target_bytes {
        bail!(
            "Video zaten hedef boyuttan ({}) daha küçük ({}). Hedef boyut sıkıştırması için dosyanın {}'dan büyük olması gerekir.",
            crate::history::format_bytes(target_bytes),
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

        let mut cmd = build_target_size_command(input_path, pending.path(), video_kbps, audio_kbps);
        let status = cmd
            .status()
            .with_context(|| "ffmpeg çalıştırılamadı (hedef boyut)")?;
        if !status.success() {
            bail!("Hedef boyut video sıkıştırma işlemi başarısız oldu");
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
            "Video hedef boyuta ({}) sıkıştırılamadı. Ulaşılan en küçük boyut: {}",
            crate::history::format_bytes(target_bytes),
            crate::history::format_bytes(size)
        );
    }
    bail!("Video hedef boyuta sıkıştırılamadı: hiçbir kodlama denemesi sonuç üretmedi")
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
        bail!("Web video optimizasyonu başarısız oldu");
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
        assert!(!is_supported_video_target("compress"));
        assert!(!is_supported_video_target("discord_8mb"));
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
        assert!(err_msg.contains("zaten hedef boyuttan"));

        // 0 hedef boyutu -> hata vermeli
        assert!(compress_video_to_size(&test_file, 0, None).is_err());

        let _ = std::fs::remove_file(&test_file);
    }
}
