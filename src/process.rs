use std::fs;
use std::path::PathBuf;

/// PID kilit dosyasının konumunu döndürür (~/.cache/konvert/running.pid).
pub fn pid_file_path() -> Option<PathBuf> {
    directories::ProjectDirs::from("", "", "konvert").map(|p| p.cache_dir().join("running.pid"))
}

/// İşlem bir terminale bağlı değilse (sağ tık menüsü, .desktop, arka plan servisi)
/// kendi süreç grubunun lideri olur. Böylece `--cancel` çağrıldığında ffmpeg,
/// gs veya magick gibi alt süreçler de tek bir sinyalle temiz bir şekilde sonlandırılabilir.
#[cfg(unix)]
pub fn detach_process_group() {
    // SAFETY: libc::isatty checks if stdin is attached to a terminal using a valid descriptor.
    // libc::setsid creates a new session and process group if the calling process is not already
    // a process group leader. It does not dereference pointers and ignores failure cleanly.
    unsafe {
        if libc::isatty(libc::STDIN_FILENO) == 0 {
            let _ = libc::setsid();
        }
    }
}

#[cfg(not(unix))]
pub fn detach_process_group() {}

/// Aktif işlem için PID dosyasını yazar ve süreç grubu ayrıştırmasını tetikler.
pub fn write_pid_lock() {
    detach_process_group();
    crate::converter::cleanup_recorded_temps();
    if let Some(p) = pid_file_path() {
        if let Some(parent) = p.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::write(p, std::process::id().to_string());
    }
}

/// İşlem başarıyla bittiğinde veya iptal edildiğinde PID kilit dosyasını temizler.
pub fn remove_pid_lock() {
    if let Some(p) = pid_file_path() {
        let _ = fs::remove_file(p);
    }
}

/// PID dosyasındaki sürecin gerçekten Konvert olup olmadığını doğrular.
/// Bayat kayıt (PID yeniden kullanımı) durumunda ilgisiz sistem süreçlerinin öldürülmesini önler.
pub fn process_is_konvert(pid: i32) -> bool {
    let comm = fs::read_to_string(format!("/proc/{}/comm", pid)).unwrap_or_default();
    if comm.trim() == "konvert" {
        return true;
    }
    match fs::read_link(format!("/proc/{}/exe", pid)) {
        Ok(exe) => exe
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("konvert")),
        Err(_) => false,
    }
}

/// Sürecin hayatta olup olmadığını kontrol eder (kill(pid, 0)).
pub fn process_alive(pid: i32) -> bool {
    #[cfg(unix)]
    // SAFETY: libc::kill with signal 0 performs error checking only without sending
    // an actual signal. It safely verifies if the target pid exists and is accessible.
    unsafe {
        libc::kill(pid, 0) == 0
    }

    #[cfg(not(unix))]
    {
        let _ = pid;
        false
    }
}

/// Önce SIGTERM (varsa tüm süreç grubuna), ardından 2 saniye içinde yanıt gelmezse SIGKILL gönderir.
/// Süreç gerçekten sonlandırıldıysa `true` döner.
pub fn terminate_process(pid: i32) -> bool {
    #[cfg(unix)]
    // SAFETY: getpgid and kill are standard POSIX syscalls with scalar parameters.
    // Negative pid (-pid) targets the entire process group if the process is a group leader,
    // ensuring spawned child processes (e.g. ffmpeg, gs) are terminated as well.
    unsafe {
        if !process_alive(pid) {
            return false;
        }

        // Süreç kendi grubunun lideriyse grubu, değilse sadece kendisini hedefle.
        let is_group_leader = libc::getpgid(pid) == pid;
        let target = if is_group_leader { -pid } else { pid };

        let mut sent = libc::kill(target, libc::SIGTERM);
        if sent != 0 && is_group_leader {
            sent = libc::kill(pid, libc::SIGTERM);
        }
        if sent != 0 {
            return false;
        }

        for _ in 0..20 {
            if !process_alive(pid) {
                return true;
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }

        let _ = libc::kill(target, libc::SIGKILL);
        std::thread::sleep(std::time::Duration::from_millis(100));
        true
    }

    #[cfg(not(unix))]
    {
        let _ = pid;
        false
    }
}
