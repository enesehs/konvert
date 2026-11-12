use anyhow::Result;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fs;
use std::path::{Path, PathBuf};

#[cfg(unix)]
use std::os::unix::io::AsRawFd;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HistoryEntry {
    pub timestamp: String,
    pub target: String,
    pub file_count: usize,
    pub original_size_bytes: u64,
    pub output_size_bytes: u64,
    pub savings_percent: f64,
    pub status: String,
    #[serde(default)]
    pub output_path: Option<String>,
}

pub fn get_history_path() -> Option<PathBuf> {
    let proj_dirs = ProjectDirs::from("", "", "konvert")?;
    Some(proj_dirs.data_local_dir().join("history.json"))
}

pub fn load_history() -> Vec<HistoryEntry> {
    let Some(path) = get_history_path() else {
        return Vec::new();
    };
    if !path.exists() {
        return Vec::new();
    }
    let Ok(content) = fs::read_to_string(&path) else {
        return Vec::new();
    };
    match serde_json::from_str::<Vec<HistoryEntry>>(&content) {
        Ok(entries) => entries,
        Err(err) => {
            // Bozuk geçmişi sessizce silmek yerine yedekle; yeni kayıtlar temiz başlasın.
            let backup = path.with_extension("json.corrupt");
            eprintln!(
                "Uyarı: geçmiş dosyası okunamadı ({}); yedekleniyor: {}",
                err,
                backup.display()
            );
            let _ = fs::rename(&path, &backup);
            Vec::new()
        }
    }
}

/// Geçmiş dosyası için süreçler arası yazma kilidi (flock).
/// Kilit, döndürülen dosya kapanınca serbest kalır.
#[cfg(unix)]
fn lock_history(path: &Path) -> Option<fs::File> {
    let lock_path = path.with_extension("lock");
    let file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(false)
        .open(lock_path)
        .ok()?;
    unsafe {
        libc::flock(file.as_raw_fd(), libc::LOCK_EX);
    }
    Some(file)
}

pub fn add_history_entry(entry: HistoryEntry) -> Result<()> {
    let Some(path) = get_history_path() else {
        return Ok(());
    };
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Oku-değiştir-yaz yarışını önlemek için süreçler arası kilit.
    #[cfg(unix)]
    let _lock = lock_history(&path);

    let mut history: VecDeque<HistoryEntry> = load_history().into();
    history.push_front(entry); // O(1) front insertion

    // En fazla son 100 işlemi sakla
    if history.len() > 100 {
        history.truncate(100);
    }

    let json = serde_json::to_string(&history)?;
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, json)?;
    fs::rename(&tmp, &path)?;
    Ok(())
}

#[allow(dead_code)]
pub fn clear_history() -> Result<()> {
    if let Some(path) = get_history_path()
        && path.exists()
    {
        fs::remove_file(path)?;
    }
    Ok(())
}

pub fn current_timestamp() -> String {
    chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

/// Bir dosyanın veya dizinin toplam boyutu (dizinlerde alt dosyalar toplanır).
///
/// Dizin çıktılarında (kare çıkarma, PDF→PNG) `metadata().len()` anlamsız
/// (4-64 KB) değer döndürdüğü için geçmiş/istatistiklerde bu fonksiyon kullanılır.
pub fn path_size_bytes(path: &std::path::Path) -> u64 {
    match fs::metadata(path) {
        Ok(meta) if meta.is_file() => meta.len(),
        Ok(meta) if meta.is_dir() => fs::read_dir(path)
            .map(|entries| {
                entries
                    .flatten()
                    .map(|entry| path_size_bytes(&entry.path()))
                    .sum()
            })
            .unwrap_or(0),
        _ => 0,
    }
}

pub fn format_bytes(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;

    let b = bytes as f64;
    if b >= GB {
        format!("{:.1} GB", b / GB)
    } else if b >= MB {
        format!("{:.1} MB", b / MB)
    } else if b >= KB {
        format!("{:.1} KB", b / KB)
    } else {
        format!("{} B", bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(500), "500 B");
        assert_eq!(format_bytes(1024), "1.0 KB");
        assert_eq!(format_bytes(1024 * 1024 * 5), "5.0 MB");
        assert_eq!(format_bytes(1024 * 1024 * 1024 * 2), "2.0 GB");
    }

    #[test]
    fn test_path_size_bytes_sums_directory() {
        let dir = std::env::temp_dir().join(format!(
            "konvert_size_test_{}_{}",
            std::process::id(),
            chrono::Local::now().timestamp_nanos_opt().unwrap_or(0)
        ));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("a.bin"), vec![0u8; 100]).unwrap();
        std::fs::create_dir_all(dir.join("sub")).unwrap();
        std::fs::write(dir.join("sub/b.bin"), vec![0u8; 50]).unwrap();

        assert_eq!(path_size_bytes(&dir), 150);
        assert_eq!(path_size_bytes(&dir.join("a.bin")), 100);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_current_timestamp() {
        let ts = current_timestamp();
        assert!(ts.contains("-"));
        assert!(ts.contains(":"));
    }
}
