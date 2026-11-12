use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};

static VERBOSE: AtomicBool = AtomicBool::new(false);

pub fn init(verbose: bool) {
    let env_verbose = std::env::var("RUST_LOG")
        .map(|v| {
            let v = v.to_lowercase();
            v == "1" || v == "true" || v == "debug" || v == "trace"
        })
        .unwrap_or(false);

    VERBOSE.store(verbose || env_verbose, Ordering::Relaxed);
}

pub fn is_verbose() -> bool {
    VERBOSE.load(Ordering::Relaxed)
}

pub fn log_file_path() -> Option<PathBuf> {
    directories::ProjectDirs::from("", "", "konvert")
        .map(|p| p.cache_dir().join("konvert.log"))
}

#[allow(dead_code)]
pub fn log(level: &str, message: &str) {
    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
    let line = format!("[{}] [{}] {}", timestamp, level, message);

    if is_verbose() || level == "ERROR" {
        eprintln!("{}", line);
    }

    if let Some(path) = log_file_path() {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&path) {
            let _ = writeln!(file, "{}", line);
        }
    }
}

#[allow(dead_code)]
pub fn debug(message: &str) {
    if is_verbose() {
        log("DEBUG", message);
    }
}

#[allow(dead_code)]
pub fn info(message: &str) {
    log("INFO", message);
}

#[allow(dead_code)]
pub fn error(message: &str) {
    log("ERROR", message);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logger_verbose_flag() {
        init(true);
        assert!(is_verbose());
        init(false);
        // Unless RUST_LOG was set in environment
        if std::env::var("RUST_LOG").is_err() {
            assert!(!is_verbose());
        }
    }

    #[test]
    fn test_logger_file_path() {
        let p = log_file_path();
        assert!(p.is_some());
        assert!(p.unwrap().ends_with("konvert.log"));
    }
}
