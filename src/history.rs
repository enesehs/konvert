use anyhow::Result;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fs;
use std::path::PathBuf;

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
    if let Some(path) = get_history_path()
        && path.exists()
        && let Ok(content) = fs::read_to_string(&path)
        && let Ok(entries) = serde_json::from_str::<Vec<HistoryEntry>>(&content)
    {
        return entries;
    }
    Vec::new()
}

pub fn add_history_entry(entry: HistoryEntry) -> Result<()> {
    let mut history: VecDeque<HistoryEntry> = load_history().into();
    history.push_front(entry); // O(1) front insertion

    // En fazla son 100 işlemi sakla
    if history.len() > 100 {
        history.truncate(100);
    }

    if let Some(path) = get_history_path() {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string(&history)?;
        fs::write(path, json)?;
    }
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
    fn test_current_timestamp() {
        let ts = current_timestamp();
        assert!(ts.contains("-"));
        assert!(ts.contains(":"));
    }
}
