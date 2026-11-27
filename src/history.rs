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
    #[cfg(unix)]
    let _lock = lock_history(&path);
    load_history_unlocked(&path)
}

fn load_history_unlocked(path: &Path) -> Vec<HistoryEntry> {
    if !path.exists() {
        return Vec::new();
    }
    let Ok(content) = fs::read_to_string(path) else {
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
            let _ = fs::rename(path, &backup);
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
    // SAFETY: as_raw_fd() returns a valid file descriptor owned by 'file'.
    // flock is an advisory kernel lock retried on EINTR and released when 'file' drops.
    unsafe {
        while libc::flock(file.as_raw_fd(), libc::LOCK_EX) != 0 {
            let err = std::io::Error::last_os_error();
            if err.raw_os_error() != Some(libc::EINTR) {
                break;
            }
        }
    }
    Some(file)
}

pub fn add_history_entry(entry: HistoryEntry) -> Result<()> {
    let Some(path) = get_history_path() else {
        return Ok(());
    };
    add_history_entry_at(&path, entry)
}

fn add_history_entry_at(path: &Path, entry: HistoryEntry) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Oku-değiştir-yaz yarışını önlemek için süreçler arası kilit.
    #[cfg(unix)]
    let _lock = lock_history(path);

    let mut history: VecDeque<HistoryEntry> = load_history_unlocked(path).into();
    history.push_front(entry); // O(1) front insertion

    // En fazla son 100 işlemi sakla
    if history.len() > 100 {
        history.truncate(100);
    }

    let json = serde_json::to_string(&history)?;
    let tmp = path.with_extension(format!("json.{}.tmp", std::process::id()));
    fs::write(&tmp, json)?;
    fs::rename(&tmp, path)?;
    Ok(())
}

#[allow(dead_code)]
pub fn clear_history() -> Result<()> {
    if let Some(path) = get_history_path() {
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        #[cfg(unix)]
        let _lock = lock_history(&path);
        if path.exists() {
            fs::remove_file(path)?;
        }
    }
    Ok(())
}

pub fn current_timestamp() -> String {
    chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

const MAX_PATH_SIZE_DEPTH: usize = 64;

/// Bir dosyanın veya dizinin toplam boyutu (dizinlerde alt dosyalar toplanır; symlink döngüleri atlanır).
///
/// Dizin çıktılarında (kare çıkarma, PDF→PNG) `metadata().len()` anlamsız
/// (4-64 KB) değer döndürdüğü için geçmiş/istatistiklerde bu fonksiyon kullanılır.
pub fn path_size_bytes(path: &std::path::Path) -> u64 {
    path_size_bytes_bounded(path, 0)
}

fn path_size_bytes_bounded(path: &std::path::Path, depth: usize) -> u64 {
    if depth >= MAX_PATH_SIZE_DEPTH {
        return 0;
    }
    match fs::symlink_metadata(path) {
        Ok(meta) if meta.file_type().is_symlink() => 0,
        Ok(meta) if meta.is_file() => meta.len(),
        Ok(meta) if meta.is_dir() => fs::read_dir(path)
            .map(|entries| {
                entries
                    .flatten()
                    .map(|entry| path_size_bytes_bounded(&entry.path(), depth + 1))
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

    #[cfg(unix)]
    #[test]
    fn test_path_size_bytes_ignores_symlink_loops() {
        let dir = std::env::temp_dir().join(format!(
            "konvert_symlink_size_test_{}_{}",
            std::process::id(),
            chrono::Local::now().timestamp_nanos_opt().unwrap_or(0)
        ));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("real.bin"), vec![1u8; 42]).unwrap();
        std::os::unix::fs::symlink(&dir, dir.join("loop")).unwrap();

        assert_eq!(path_size_bytes(&dir), 42);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_add_history_entry_at_truncates_to_100_newest_entries() {
        let dir = std::env::temp_dir().join(format!(
            "konvert_history_limit_test_{}_{}",
            std::process::id(),
            chrono::Local::now().timestamp_nanos_opt().unwrap_or(0)
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let history_path = dir.join("history.json");

        for i in 0..105 {
            add_history_entry_at(
                &history_path,
                HistoryEntry {
                    timestamp: format!("2026-11-27 10:{:02}:00", i % 60),
                    target: format!("target_{i}"),
                    file_count: 1,
                    original_size_bytes: 1000,
                    output_size_bytes: 500,
                    savings_percent: 50.0,
                    status: "Success".to_string(),
                    output_path: None,
                },
            )
            .unwrap();
        }

        let loaded = load_history_unlocked(&history_path);
        assert_eq!(loaded.len(), 100);
        assert_eq!(loaded.first().unwrap().target, "target_104");
        assert_eq!(loaded.last().unwrap().target, "target_5");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_load_history_unlocked_backs_up_corrupt_json() {
        let dir = std::env::temp_dir().join(format!(
            "konvert_history_corrupt_test_{}_{}",
            std::process::id(),
            chrono::Local::now().timestamp_nanos_opt().unwrap_or(0)
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let history_path = dir.join("history.json");
        std::fs::write(&history_path, "{ not valid json").unwrap();

        let loaded = load_history_unlocked(&history_path);
        assert!(loaded.is_empty());

        let backup = history_path.with_extension("json.corrupt");
        assert!(backup.exists(), "corrupt history JSON must be backed up");
        assert_eq!(
            std::fs::read_to_string(&backup).unwrap(),
            "{ not valid json"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }
}
