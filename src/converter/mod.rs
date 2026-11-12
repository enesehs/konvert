pub mod audio;
pub mod document;
pub mod image;
pub mod video;

use anyhow::{bail, Context, Result};
use rayon::prelude::*;
use std::collections::HashSet;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResizeSpec {
    Percent(u32),
    Fit1080p,
    Fit720p,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TargetAction {
    Convert(String),
    Compress,
    TargetSize(u64),
    WebOpt,
    StripMetadata,
    Resize(ResizeSpec),
    ExtractAudio,
    Frames,
    Gif,
    AudioToVideo,
}

/// "18mb", "1.5gb", "800kb", "500" (MB) gibi insan-okur boyutları bayta çevirir.
/// Ondalık değerler ve büyük/küçük harf kabul edilir. Geçersiz veya sıfır değerde `None`.
pub fn parse_size_bytes(raw: &str) -> Option<u64> {
    let s = raw.trim().to_lowercase();
    if s.is_empty() {
        return None;
    }

    let (num_part, multiplier) = if let Some(v) = s.strip_suffix("gib") {
        (v, 1024u64.pow(3))
    } else if let Some(v) = s.strip_suffix("gb") {
        (v, 1024u64.pow(3))
    } else if let Some(v) = s.strip_suffix('g') {
        (v, 1024u64.pow(3))
    } else if let Some(v) = s.strip_suffix("mib") {
        (v, 1024u64.pow(2))
    } else if let Some(v) = s.strip_suffix("mb") {
        (v, 1024u64.pow(2))
    } else if let Some(v) = s.strip_suffix('m') {
        (v, 1024u64.pow(2))
    } else if let Some(v) = s.strip_suffix("kib") {
        (v, 1024u64)
    } else if let Some(v) = s.strip_suffix("kb") {
        (v, 1024u64)
    } else if let Some(v) = s.strip_suffix('k') {
        (v, 1024u64)
    } else if let Some(v) = s.strip_suffix('b') {
        (v, 1u64)
    } else {
        // Birim yoksa MB kabul et (geriye dönük uyumluluk: "8" => 8 MB).
        (s.as_str(), 1024u64.pow(2))
    };

    let value: f64 = num_part.trim().parse().ok()?;
    if !value.is_finite() || value <= 0.0 {
        return None;
    }
    let bytes = (value * multiplier as f64).round();
    if bytes < 1.0 || bytes > u64::MAX as f64 {
        return None;
    }
    Some(bytes as u64)
}

impl TargetAction {
    pub fn parse(target: &str) -> Result<Self> {
        let t = target.trim().to_lowercase();
        match t.as_str() {
            "compress" => Ok(TargetAction::Compress),
            "web_opt" | "web-opt" => Ok(TargetAction::WebOpt),
            "strip-metadata" | "strip_metadata" => Ok(TargetAction::StripMetadata),
            "discord_8mb" | "target_8mb" => Ok(TargetAction::TargetSize(8 * 1024 * 1024)),
            "email_25mb" | "target_25mb" => Ok(TargetAction::TargetSize(25 * 1024 * 1024)),
            "audio-to-video" | "audio_to_video" => Ok(TargetAction::AudioToVideo),
            "extract-audio" | "extract_audio" => Ok(TargetAction::ExtractAudio),
            "frames" | "png-frames" => Ok(TargetAction::Frames),
            "gif" => Ok(TargetAction::Gif),
            _ if t.starts_with("resize_") => {
                let spec = t.trim_start_matches("resize_");
                match spec {
                    "1080p" | "1080" => Ok(TargetAction::Resize(ResizeSpec::Fit1080p)),
                    "720p" | "720" => Ok(TargetAction::Resize(ResizeSpec::Fit720p)),
                    _ => {
                        // Herhangi bir yüzde: 30, 30%, 150 ...
                        let percent = spec.trim_end_matches('%');
                        match percent.parse::<u32>() {
                            Ok(pct) if (1..=400).contains(&pct) => {
                                Ok(TargetAction::Resize(ResizeSpec::Percent(pct)))
                            }
                            _ => bail!(
                                "Geçersiz boyutlandırma parametresi: {} (örnek: 50%, 1080p, 720p)",
                                spec
                            ),
                        }
                    }
                }
            }
            _ if t.starts_with("target_") || t.ends_with("mb") || t.ends_with("gb") => {
                let raw = t.strip_prefix("target_").unwrap_or(&t);
                match parse_size_bytes(raw) {
                    Some(bytes) => Ok(TargetAction::TargetSize(bytes)),
                    None => Ok(TargetAction::Convert(t)),
                }
            }
            _ => Ok(TargetAction::Convert(t)),
        }
    }
}

pub fn sanitize_cli_path(path: &Path) -> PathBuf {
    let s = path.as_os_str().to_string_lossy();
    if s.starts_with('-') {
        PathBuf::from(format!("./{}", s))
    } else {
        path.to_path_buf()
    }
}

pub fn get_output_dir(input_path: &Path, custom_output_dir: Option<&Path>) -> PathBuf {
    if let Some(dir) = custom_output_dir {
        return dir.to_path_buf();
    }
    match input_path.parent() {
        Some(p) if !p.as_os_str().is_empty() => p.to_path_buf(),
        _ => PathBuf::from("."),
    }
}

pub fn ensure_parent_dir(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent)
            .with_context(|| format!("Çıktı klasörü oluşturulamadı: {}", parent.display()))?;
    }
    Ok(())
}

pub fn determine_output_path(
    input_path: &Path,
    target_ext: &str,
    custom_output_dir: Option<&Path>,
) -> PathBuf {
    let file_stem = input_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("converted");

    let target_dir = get_output_dir(input_path, custom_output_dir);
    let candidate = target_dir.join(format!("{}.{}", file_stem, target_ext));
    if candidate == input_path {
        target_dir.join(format!("{}_konvert.{}", file_stem, target_ext))
    } else {
        candidate
    }
}

pub fn determine_suffixed_output_path(
    input_path: &Path,
    suffix: &str,
    target_ext: &str,
    custom_output_dir: Option<&Path>,
) -> PathBuf {
    let file_stem = input_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("file");

    let target_dir = get_output_dir(input_path, custom_output_dir);
    let candidate_name = if file_stem.ends_with(&format!("_{}", suffix)) {
        format!("{}_new.{}", file_stem, target_ext)
    } else {
        format!("{}_{}.{}", file_stem, suffix, target_ext)
    };
    let candidate = target_dir.join(candidate_name);
    if candidate == input_path {
        target_dir.join(format!("{}_{}_konvert.{}", file_stem, suffix, target_ext))
    } else {
        candidate
    }
}

static FORCE_OVERWRITE: AtomicBool = AtomicBool::new(false);
static JOB_COUNT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(1);

/// `--force` verildiğinde mevcut çıktıların üzerine yazılır; aksi halde
/// `name_2`, `name_3` … gibi benzersiz adlar üretilir.
pub fn set_force_overwrite(enabled: bool) {
    FORCE_OVERWRITE.store(enabled, Ordering::Relaxed);
}

pub fn force_overwrite() -> bool {
    FORCE_OVERWRITE.load(Ordering::Relaxed)
}

/// Aynı anda çalışacak dönüşüm sayısını ayarlar (ffmpeg iş parçacığı bütçesi için).
pub fn set_job_count(jobs: usize) {
    JOB_COUNT.store(jobs.max(1), Ordering::Relaxed);
}

/// Tek bir ffmpeg sürecine ayrılacak iş parçacığı sayısı.
/// Paralel iş sayısı arttıkça her süreç daha az çekirdek kullanır; böylece
/// N paralel video kodlaması CPU'yu aşırı abone etmez.
pub fn ffmpeg_threads() -> usize {
    let jobs = JOB_COUNT.load(Ordering::Relaxed).max(1);
    let cores = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);
    ffmpeg_threads_for(cores, jobs)
}

/// İş parçacığı bütçesi hesabı (test edilebilir saf fonksiyon).
pub fn ffmpeg_threads_for(cores: usize, jobs: usize) -> usize {
    (cores / jobs.max(1)).max(1)
}

fn claimed_outputs() -> &'static Mutex<HashSet<PathBuf>> {
    static CLAIMED: OnceLock<Mutex<HashSet<PathBuf>>> = OnceLock::new();
    CLAIMED.get_or_init(|| Mutex::new(HashSet::new()))
}

/// Aynı süreç içinde iki işin aynı çıktı yoluna yazmasını engeller.
///
/// Toplu dönüşümde aynı gövdeye sahip dosyalar (örn. `a/photo.png` ve
/// `b/photo.jpg`) aynı hedefi hesaplar; bu fonksiyon ikincisi için `_2`, `_3`
/// … eklenmiş benzersiz bir yol döndürür. Böylece paralel çalışan kodlayıcılar
/// birbirinin çıktısını ezmez.
pub fn claim_output_path(candidate: PathBuf) -> PathBuf {
    let mut claimed = match claimed_outputs().lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    let force = force_overwrite();
    let path = next_free_path(&candidate, |p| {
        claimed.contains(p) || (!force && p.exists())
    });
    claimed.insert(path.clone());
    path
}

/// `candidate` "alınmış" ise sonuna `_2`, `_3` … ekleyerek ilk boş yolu döndürür.
pub fn next_free_path(candidate: &Path, taken: impl Fn(&Path) -> bool) -> PathBuf {
    if !taken(candidate) {
        return candidate.to_path_buf();
    }

    let stem = candidate
        .file_stem()
        .map(|s| s.to_os_string())
        .unwrap_or_else(|| OsString::from("output"));
    let ext = candidate.extension().map(|e| e.to_os_string());
    let parent = candidate.parent().unwrap_or_else(|| Path::new(""));

    for i in 2..=9999u32 {
        let mut name = stem.clone();
        name.push(format!("_{}", i));
        if let Some(ext) = &ext {
            name.push(".");
            name.push(ext);
        }
        let path = parent.join(name);
        if !taken(&path) {
            return path;
        }
    }

    // Pratikte ulaşılmaz: 9998 çakışma varsa son adayı döndür.
    candidate.to_path_buf()
}

fn pending_temps_file() -> Option<PathBuf> {
    directories::ProjectDirs::from("", "", "konvert")
        .map(|dirs| dirs.cache_dir().join("pending-temps"))
}

fn temp_owner_pid(path: &Path) -> Option<u32> {
    let name = path.file_name()?.to_str()?;
    let rest = name.strip_prefix(".konvert-tmp-")?;
    rest.split('-').next()?.parse().ok()
}

#[cfg(unix)]
fn process_alive(pid: u32) -> bool {
    unsafe { libc::kill(pid as i32, 0) == 0 }
}

#[cfg(not(unix))]
fn process_alive(_pid: u32) -> bool {
    true
}

fn record_temp_file(temp: &Path) {
    let Some(file_path) = pending_temps_file() else {
        return;
    };
    if let Some(parent) = file_path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(mut file) = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&file_path)
    {
        use std::io::Write;
        let _ = writeln!(file, "{}", temp.display());
    }
}

/// Önceki çalışmalardan (çökme/iptal) kalan geçici çıktıları temizler.
///
/// Kayıt dosyasındaki her satır `.konvert-tmp-<pid>-...` biçimindedir; yalnızca
/// sahibi artık yaşamayan süreçlerin dosyaları silinir. Çalışan başka bir
/// Konvert sürecinin geçici dosyalarına dokunulmaz.
pub fn cleanup_recorded_temps() {
    let Some(file_path) = pending_temps_file() else {
        return;
    };
    let Ok(content) = fs::read_to_string(&file_path) else {
        return;
    };

    let mut keep = String::new();
    for line in content.lines() {
        let path = Path::new(line);
        let is_temp = path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with(".konvert-tmp-"));
        if !is_temp {
            continue;
        }

        match temp_owner_pid(path) {
            Some(pid) if process_alive(pid) => {
                keep.push_str(line);
                keep.push('\n');
            }
            Some(_) => {
                let _ = fs::remove_file(path);
            }
            None => {}
        }
    }

    let _ = fs::write(&file_path, keep);
}

/// Yazma sırasında hedef dosyayı doğrudan açmak yerine geçici bir dosyaya
/// yazan ve başarıda `rename` ile hedefe taşıyan yardımcı.
///
/// Geçici ad, uzantıyı korur (`.konvert-tmp-<pid>-<sayaç>-<ad>.<ext>`), böylece
/// ffmpeg/ImageMagick gibi araçlar çıktı biçimini uzantıdan çıkarabilir.
/// Hata veya `drop` durumunda geçici dosya silinir; mevcut hedef dosya bozulmaz.
pub struct PendingOutput {
    temp: PathBuf,
    final_path: PathBuf,
    committed: bool,
}

impl PendingOutput {
    pub fn new(final_path: &Path) -> Self {
        static TEMP_COUNTER: std::sync::atomic::AtomicUsize =
            std::sync::atomic::AtomicUsize::new(0);
        let counter = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        let parent = final_path.parent().unwrap_or_else(|| Path::new(""));
        let file_name = final_path
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| "output".to_string());
        let temp = parent.join(format!(
            ".konvert-tmp-{}-{}-{}",
            std::process::id(),
            counter,
            file_name
        ));
        record_temp_file(&temp);
        Self {
            temp,
            final_path: final_path.to_path_buf(),
            committed: false,
        }
    }

    pub fn path(&self) -> &Path {
        &self.temp
    }

    pub fn commit(mut self) -> Result<PathBuf> {
        fs::rename(&self.temp, &self.final_path).with_context(|| {
            format!(
                "Geçici çıktı hedefe taşınamadı: {} -> {}",
                self.temp.display(),
                self.final_path.display()
            )
        })?;
        self.committed = true;
        Ok(self.final_path.clone())
    }
}

impl Drop for PendingOutput {
    fn drop(&mut self) {
        if !self.committed {
            let _ = fs::remove_file(&self.temp);
        }
    }
}

/// `PendingOutput` kullanarak yazma yapar; closure tamponlu yazıcıyı alır.
pub fn write_file_atomic<F>(output_path: &Path, write: F) -> Result<PathBuf>
where
    F: FnOnce(&mut std::io::BufWriter<std::fs::File>) -> Result<()>,
{
    use std::io::Write;

    let pending = PendingOutput::new(output_path);
    let file = std::fs::File::create(pending.path()).with_context(|| {
        format!("Dosya oluşturulamadı: {}", pending.path().display())
    })?;
    let mut writer = std::io::BufWriter::with_capacity(64 * 1024, file);
    write(&mut writer)?;
    writer.flush().context("Tampon diske yazılamadı")?;
    pending.commit()
}

pub fn run_imagemagick<I, S>(args: I) -> Result<()>
where
    I: IntoIterator<Item = S> + Clone,
    S: AsRef<std::ffi::OsStr>,
{
    let status = Command::new("magick")
        .args(args.clone())
        .status()
        .or_else(|_| Command::new("convert").args(args).status())
        .with_context(|| "ImageMagick (magick veya convert) çalıştırılamadı")?;

    if !status.success() {
        bail!("ImageMagick komutu başarısız oldu (çıkış kodu 0 değil)");
    }
    Ok(())
}

pub struct BatchResult {
    pub succeeded: Vec<PathBuf>,
    pub failed: Vec<(PathBuf, String)>,
}

#[derive(Clone)]
pub struct ConvertOptions {
    pub target: String,
    pub quality: Option<u8>,
    /// Görsel/işleme yollarında lossless mod (config.image.lossless veya `-l`).
    pub image_lossless: bool,
    /// Video kodlamada lossless mod (config.video.lossless veya `-l`).
    pub video_lossless: bool,
    /// Aynı anda işlenecek dosya sayısı (batch).
    pub jobs: usize,
    /// GIF süresi (saniye). 0 = videonun tamamı.
    pub gif_duration: f64,
    /// GIF kare hızı.
    pub gif_fps: u32,
    /// PDF→görsel dönüşümünde en fazla sayfa (0 = sınırsız).
    pub max_pages: u32,
    pub crf: Option<u8>,
    pub preset: Option<String>,
    pub bitrate: Option<String>,
    pub dpi: Option<u32>,
    pub custom_output_dir: Option<PathBuf>,
}

pub fn convert_single(input_path: &Path, opts: &ConvertOptions) -> Result<PathBuf> {
    let sanitized_input = sanitize_cli_path(input_path);
    let input_path = &sanitized_input;

    if !input_path.exists() {
        bail!("Girdi dosyası mevcut değil: {}", input_path.display());
    }

    let action = TargetAction::parse(&opts.target)?;
    let ext = input_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    let out_dir = opts.custom_output_dir.as_deref();

    match action {
        TargetAction::StripMetadata => {
            if is_image_ext(&ext) {
                image::strip_metadata(input_path, out_dir)
            } else if is_video_ext(&ext) {
                video::strip_video_metadata(input_path, out_dir)
            } else if is_audio_ext(&ext) {
                audio::strip_audio_metadata(input_path, out_dir)
            } else {
                bail!("Metadata temizleme sadece görsel, video ve ses dosyaları için desteklenir: .{}", ext)
            }
        }
        TargetAction::Resize(spec) => {
            if is_image_ext(&ext) {
                image::resize_image(input_path, &spec, out_dir)
            } else {
                bail!("Yeniden boyutlandırma sadece görseller için geçerlidir (.{})", ext)
            }
        }
        TargetAction::Compress => {
            if is_video_ext(&ext) {
                video::compress_video(input_path, out_dir)
            } else if is_image_ext(&ext) {
                image::compress_image(input_path, opts.quality, out_dir)
            } else if ext == "pdf" {
                document::compress_pdf(input_path, out_dir)
            } else {
                bail!("Bu format için sıkıştırma desteklenmiyor: .{}", ext)
            }
        }
        TargetAction::TargetSize(target_bytes) => {
            if is_video_ext(&ext) {
                video::compress_video_to_size(input_path, target_bytes, out_dir)
            } else if is_image_ext(&ext) {
                image::compress_image_to_size(input_path, target_bytes, out_dir)
            } else if ext == "pdf" {
                document::compress_pdf_to_size(input_path, target_bytes, out_dir)
            } else {
                bail!("Bu format için hedef boyut sıkıştırması desteklenmiyor: .{}", ext)
            }
        }
        TargetAction::WebOpt => {
            if is_video_ext(&ext) {
                video::web_optimize_video(input_path, out_dir)
            } else if is_image_ext(&ext) {
                image::web_optimize_image(input_path, out_dir)
            } else {
                bail!("Web optimizasyonu sadece görsel ve videolar için geçerlidir (.{})", ext)
            }
        }
        TargetAction::ExtractAudio => {
            if is_video_ext(&ext) {
                video::extract_audio(input_path, out_dir)
            } else {
                bail!("Videodan ses ayıklama sadece video dosyaları için geçerlidir (.{})", ext)
            }
        }
        TargetAction::Frames => {
            if is_video_ext(&ext) {
                video::extract_frames(input_path, out_dir)
            } else {
                bail!("Kare çıkarma sadece video dosyaları için geçerlidir (.{})", ext)
            }
        }
        TargetAction::Gif => {
            if is_video_ext(&ext) {
                video::convert_to_gif(input_path, out_dir, opts.gif_duration, opts.gif_fps)
            } else {
                bail!("GIF dönüşümü sadece video dosyaları için geçerlidir (.{})", ext)
            }
        }
        TargetAction::AudioToVideo => {
            if is_audio_ext(&ext) {
                audio::convert_audio_to_video(input_path, out_dir)
            } else {
                bail!("Ses dalgası videosu oluşturmak için bir ses dosyası gereklidir (.{})", ext)
            }
        }
        TargetAction::Convert(target_fmt) => {
            if ext == "pdf" && (target_fmt == "png" || target_fmt == "jpg" || target_fmt == "jpeg") {
                document::convert_pdf_to_images(
                    input_path,
                    &target_fmt,
                    opts.dpi.unwrap_or(300),
                    opts.max_pages,
                    out_dir,
                )
            } else if ext == "udf" {
                document::convert_udf(input_path, &target_fmt, out_dir)
            } else if ext == "docx" {
                document::convert_docx(input_path, &target_fmt, out_dir)
            } else if is_image_ext(&ext) && image::is_supported_image_target(&target_fmt) {
                image::convert_image(input_path, &target_fmt, opts.quality, opts.image_lossless, out_dir)
            } else if is_video_ext(&ext) && video::is_supported_video_target(&target_fmt) {
                video::convert_video(
                    input_path,
                    &target_fmt,
                    opts.crf,
                    opts.preset.as_deref(),
                    opts.video_lossless,
                    out_dir,
                )
            } else if is_video_ext(&ext) && audio::is_supported_audio_target(&target_fmt) {
                // Videodan doğrudan ses çıkarma (ör. `-t mp3 video.mp4`).
                audio::convert_audio(input_path, &target_fmt, opts.bitrate.as_deref(), out_dir)
            } else if is_audio_ext(&ext) && audio::is_supported_audio_target(&target_fmt) {
                audio::convert_audio(input_path, &target_fmt, opts.bitrate.as_deref(), out_dir)
            } else {
                bail!("Desteklenmeyen dönüştürme: .{} -> .{}", ext, target_fmt)
            }
        }
    }
}

pub fn merge_images_to_pdf(images: &[PathBuf], custom_output_dir: Option<&Path>) -> Result<PathBuf> {
    image::merge_images_to_pdf(images, custom_output_dir)
}

pub fn convert_batch_with_progress<F>(
    files: &[PathBuf],
    opts: &ConvertOptions,
    progress_callback: F,
) -> BatchResult
where
    F: FnMut(usize, usize, &Path, i32) + Send,
{
    let total = files.len();
    if total == 0 {
        return BatchResult {
            succeeded: Vec::new(),
            failed: Vec::new(),
        };
    }

    let progress_state = Mutex::new((0usize, progress_callback));
    let jobs = opts.jobs.max(1);

    let mut results: Vec<Result<PathBuf, (PathBuf, String)>> = Vec::with_capacity(total);
    for chunk in files.chunks(jobs) {
        let mut chunk_results: Vec<Result<PathBuf, (PathBuf, String)>> = chunk
            .par_iter()
            .map(|file| {
                let res = convert_single(file, opts);

                if let Ok(mut lock) = progress_state.lock() {
                    lock.0 += 1;
                    let curr = lock.0;
                    let percent = ((curr as f32 / total as f32) * 100.0) as i32;
                    (lock.1)(curr, total, file, percent);
                }

                match res {
                    Ok(p) => Ok(p),
                    Err(e) => Err((file.clone(), format!("{:#}", e))),
                }
            })
            .collect();
        results.append(&mut chunk_results);
    }

    let mut succeeded = Vec::with_capacity(total);
    let mut failed = Vec::new();

    for r in results {
        match r {
            Ok(p) => succeeded.push(p),
            Err(f) => failed.push(f),
        }
    }

    BatchResult { succeeded, failed }
}

pub fn is_image_ext(ext: &str) -> bool {
    matches!(
        ext.to_lowercase().as_str(),
        "png" | "jpg" | "jpeg" | "webp" | "avif" | "jxl" | "bmp" | "ico" | "tiff" | "tif" | "qoi" | "svg" | "heic" | "heif"
    )
}

pub fn is_video_ext(ext: &str) -> bool {
    matches!(
        ext.to_lowercase().as_str(),
        "mp4" | "mkv" | "webm" | "avi" | "mov" | "flv" | "wmv" | "m4v" | "ts"
    )
}

pub fn is_audio_ext(ext: &str) -> bool {
    matches!(
        ext.to_lowercase().as_str(),
        "mp3" | "wav" | "flac" | "ogg" | "m4a" | "aac" | "opus" | "wma" | "aiff"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_image_ext() {
        assert!(is_image_ext("png"));
        assert!(is_image_ext("PNG"));
        assert!(is_image_ext("jpg"));
        assert!(is_image_ext("JPG"));
        assert!(is_image_ext("webp"));
        assert!(is_image_ext("avif"));
        assert!(is_image_ext("jxl"));
        assert!(is_image_ext("heic"));
        assert!(is_image_ext("heif"));
        assert!(is_image_ext("bmp"));
        assert!(is_image_ext("ico"));
        assert!(is_image_ext("svg"));
        assert!(!is_image_ext("mp4"));
        assert!(!is_image_ext("pdf"));
    }

    #[test]
    fn test_is_video_ext() {
        assert!(is_video_ext("mp4"));
        assert!(is_video_ext("MP4"));
        assert!(is_video_ext("mkv"));
        assert!(is_video_ext("webm"));
        assert!(is_video_ext("ts"));
        assert!(!is_video_ext("png"));
        assert!(!is_video_ext("mp3"));
    }

    #[test]
    fn test_is_audio_ext() {
        assert!(is_audio_ext("mp3"));
        assert!(is_audio_ext("MP3"));
        assert!(is_audio_ext("flac"));
        assert!(is_audio_ext("wav"));
        assert!(!is_audio_ext("mp4"));
        assert!(!is_audio_ext("png"));
    }

    #[test]
    fn test_get_output_dir_empty_parent() {
        let path = Path::new("test.docx");
        let dir = get_output_dir(path, None);
        assert_eq!(dir, PathBuf::from("."));
        assert!(!dir.as_os_str().is_empty());
    }

    #[test]
    fn test_determine_suffixed_output_path() {
        let path = Path::new("my_photo.jpg");
        let out = determine_suffixed_output_path(path, "clean", "jpg", None);
        assert_eq!(out, PathBuf::from("./my_photo_clean.jpg"));

        let already_clean = Path::new("my_photo_clean.jpg");
        let out2 = determine_suffixed_output_path(already_clean, "clean", "jpg", None);
        assert_eq!(out2, PathBuf::from("./my_photo_clean_new.jpg"));
    }

    #[test]
    fn test_sanitize_cli_path() {
        let normal = Path::new("file.jpg");
        assert_eq!(sanitize_cli_path(normal), PathBuf::from("file.jpg"));

        let dash = Path::new("-dash_file.mp4");
        assert_eq!(sanitize_cli_path(dash), PathBuf::from("./-dash_file.mp4"));
    }

    #[test]
    fn test_target_action_parse() {
        assert_eq!(TargetAction::parse("compress").unwrap(), TargetAction::Compress);
        assert_eq!(TargetAction::parse("web_opt").unwrap(), TargetAction::WebOpt);
        assert_eq!(TargetAction::parse("web-opt").unwrap(), TargetAction::WebOpt);
        assert_eq!(TargetAction::parse("discord_8mb").unwrap(), TargetAction::TargetSize(8 * 1024 * 1024));
        assert_eq!(TargetAction::parse("email_25mb").unwrap(), TargetAction::TargetSize(25 * 1024 * 1024));
        assert_eq!(TargetAction::parse("50mb").unwrap(), TargetAction::TargetSize(50 * 1024 * 1024));
        assert_eq!(TargetAction::parse("target_100mb").unwrap(), TargetAction::TargetSize(100 * 1024 * 1024));
        assert_eq!(TargetAction::parse("strip-metadata").unwrap(), TargetAction::StripMetadata);
        assert_eq!(TargetAction::parse("resize_50%").unwrap(), TargetAction::Resize(ResizeSpec::Percent(50)));
        assert_eq!(TargetAction::parse("resize_30%").unwrap(), TargetAction::Resize(ResizeSpec::Percent(30)));
        assert_eq!(TargetAction::parse("resize_150").unwrap(), TargetAction::Resize(ResizeSpec::Percent(150)));
        assert!(TargetAction::parse("resize_0").is_err());
        assert!(TargetAction::parse("resize_401").is_err());
        assert_eq!(TargetAction::parse("resize_1080p").unwrap(), TargetAction::Resize(ResizeSpec::Fit1080p));
        assert!(TargetAction::parse("resize_invalid").is_err());
        assert_eq!(TargetAction::parse("webp").unwrap(), TargetAction::Convert("webp".to_string()));
    }

    #[test]
    fn test_target_size_is_not_substring_matched() {
        // Regresyon: "18mb" eskiden içinde "8" geçtiği için 8 MB'a düşüyordu.
        assert_eq!(
            TargetAction::parse("target_18mb").unwrap(),
            TargetAction::TargetSize(18 * 1024 * 1024)
        );
        assert_eq!(
            TargetAction::parse("128mb").unwrap(),
            TargetAction::TargetSize(128 * 1024 * 1024)
        );
        assert_eq!(
            TargetAction::parse("target_8mb").unwrap(),
            TargetAction::TargetSize(8 * 1024 * 1024)
        );
    }

    #[test]
    fn test_ffmpeg_thread_budget() {
        assert_eq!(ffmpeg_threads_for(16, 1), 16);
        assert_eq!(ffmpeg_threads_for(16, 4), 4);
        assert_eq!(ffmpeg_threads_for(4, 8), 1);
        assert_eq!(ffmpeg_threads_for(0, 2), 1);
        assert_eq!(ffmpeg_threads_for(8, 0), 8);
    }

    #[test]
    fn test_parse_size_bytes() {
        assert_eq!(parse_size_bytes("8mb"), Some(8 * 1024 * 1024));
        assert_eq!(parse_size_bytes("18 MB"), Some(18 * 1024 * 1024));
        assert_eq!(parse_size_bytes("1.5gb"), Some(1_610_612_736));
        assert_eq!(parse_size_bytes("800kb"), Some(800 * 1024));
        assert_eq!(parse_size_bytes("500"), Some(500 * 1024 * 1024));
        assert_eq!(parse_size_bytes("1024b"), Some(1024));
        assert_eq!(parse_size_bytes(""), None);
        assert_eq!(parse_size_bytes("0mb"), None);
        assert_eq!(parse_size_bytes("abc"), None);
        assert_eq!(parse_size_bytes("-5mb"), None);
    }

    #[test]
    fn test_next_free_path() {
        use std::collections::HashSet;

        let mut taken: HashSet<PathBuf> = HashSet::new();
        let candidate = Path::new("/tmp/photo.webp");

        assert_eq!(next_free_path(candidate, |p| taken.contains(p)), candidate);
        taken.insert(candidate.to_path_buf());
        assert_eq!(
            next_free_path(candidate, |p| taken.contains(p)),
            PathBuf::from("/tmp/photo_2.webp")
        );
        taken.insert(PathBuf::from("/tmp/photo_2.webp"));
        assert_eq!(
            next_free_path(candidate, |p| taken.contains(p)),
            PathBuf::from("/tmp/photo_3.webp")
        );

        // Uzantısız (dizin) çıktılar
        let dir = Path::new("/tmp/photo_frames");
        taken.clear();
        taken.insert(dir.to_path_buf());
        assert_eq!(
            next_free_path(dir, |p| taken.contains(p)),
            PathBuf::from("/tmp/photo_frames_2")
        );
    }

    #[test]
    fn test_claim_output_path_avoids_existing_files_unless_forced() {
        let dir = std::env::temp_dir().join(format!(
            "konvert_overwrite_test_{}_{}",
            std::process::id(),
            chrono::Local::now().timestamp_nanos_opt().unwrap_or(0)
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let candidate = dir.join("existing.webp");
        std::fs::write(&candidate, b"old").unwrap();

        set_force_overwrite(false);
        let claimed = claim_output_path(candidate.clone());
        assert_ne!(claimed, candidate, "mevcut dosya ezilmemeli");
        assert!(claimed.to_string_lossy().contains("_2"));

        set_force_overwrite(true);
        let forced = claim_output_path(candidate.clone());
        assert_eq!(forced, candidate, "--force ile üzerine yazılmalı");
        set_force_overwrite(false);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_claim_output_path_is_unique_within_process() {
        let base = std::env::temp_dir().join(format!(
            "konvert_claim_test_{}_{}.webp",
            std::process::id(),
            chrono::Local::now().timestamp_nanos_opt().unwrap_or(0)
        ));
        let first = claim_output_path(base.clone());
        let second = claim_output_path(base.clone());
        assert_ne!(first, second);
        assert!(second.to_string_lossy().contains("_2"));
    }

    #[test]
    fn test_batch_empty() {
        let opts = ConvertOptions {
            target: "webp".to_string(),
            quality: None,
            image_lossless: false,
            video_lossless: false,
            jobs: 1,
            gif_duration: 15.0,
            gif_fps: 15,
            max_pages: 200,
            crf: None,
            preset: None,
            bitrate: None,
            dpi: None,
            custom_output_dir: None,
        };
        let result = convert_batch_with_progress(&[], &opts, |_, _, _, _| {});
        assert!(result.succeeded.is_empty());
        assert!(result.failed.is_empty());
    }
}
