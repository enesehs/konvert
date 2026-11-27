use anyhow::Context;
use std::collections::HashSet;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Output, Stdio};
use std::sync::{Arc, Condvar, Mutex, OnceLock};
use std::time::{Duration, Instant};

/// Kullanıcı iptali nedeniyle dönüşümün erken sonlandırıldığını belirten özel hata türü.
#[derive(Debug, Clone, Copy)]
pub struct JobCancelledError;

impl fmt::Display for JobCancelledError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", crate::i18n::t("err_job_cancelled"))
    }
}

impl std::error::Error for JobCancelledError {}

pub fn cancelled_error() -> anyhow::Error {
    anyhow::Error::new(JobCancelledError)
}

pub fn is_cancelled_error(err: &anyhow::Error) -> bool {
    err.chain().any(|c| c.is::<JobCancelledError>()) || is_job_cancelled()
}

type PauseUiCallback = Arc<dyn Fn(bool) + Send + Sync>;

#[derive(Default)]
struct JobState {
    paused: bool,
    cancelled: bool,
    paused_at: Option<Instant>,
    paused_duration: Duration,
    children: HashSet<u32>,
    ui_pause_callback: Option<PauseUiCallback>,
}

struct JobControlInner {
    state: Mutex<JobState>,
    condvar: Condvar,
    write_pause_marker: bool,
}

#[derive(Clone)]
pub struct JobControl(Arc<JobControlInner>);

impl Default for JobControl {
    fn default() -> Self {
        Self::new()
    }
}

pub struct TrackedChildGuard<'a> {
    ctrl: &'a JobControl,
    pid: u32,
}

impl Drop for TrackedChildGuard<'_> {
    fn drop(&mut self) {
        if self.pid > 0 {
            let mut state = self.ctrl.lock_state();
            state.children.remove(&self.pid);
        }
    }
}

impl JobControl {
    /// Birim testleri için yan etkisiz (disk işaretçisiz) yalıtılmış iş denetleyici oluşturur.
    pub fn new() -> Self {
        Self::with_marker(false)
    }

    fn with_marker(write_pause_marker: bool) -> Self {
        Self(Arc::new(JobControlInner {
            state: Mutex::new(JobState::default()),
            condvar: Condvar::new(),
            write_pause_marker,
        }))
    }

    fn lock_state(&self) -> std::sync::MutexGuard<'_, JobState> {
        self.0.state.lock().unwrap_or_else(|e| e.into_inner())
    }

    pub fn reset(&self) {
        let mut state = self.lock_state();
        state.paused = false;
        state.cancelled = false;
        state.paused_at = None;
        state.paused_duration = Duration::ZERO;
        state.children.clear();
        drop(state);
        if self.0.write_pause_marker {
            remove_current_pause_marker();
        }
    }

    pub fn is_paused(&self) -> bool {
        let state = self.lock_state();
        state.paused && !state.cancelled
    }

    pub fn is_cancelled(&self) -> bool {
        self.lock_state().cancelled
    }

    pub fn paused_duration(&self) -> Duration {
        let state = self.lock_state();
        state.paused_duration
            + state
                .paused_at
                .map(|started| started.elapsed())
                .unwrap_or_default()
    }

    pub fn set_ui_pause_callback(&self, cb: Option<PauseUiCallback>) {
        self.lock_state().ui_pause_callback = cb;
    }

    pub fn wait_if_paused_or_cancelled(&self) -> anyhow::Result<()> {
        let mut state = self.lock_state();
        while state.paused && !state.cancelled {
            state = self
                .0
                .condvar
                .wait(state)
                .unwrap_or_else(|e| e.into_inner());
        }
        if state.cancelled {
            return Err(cancelled_error());
        }
        Ok(())
    }

    pub fn register_child(&self, pid: u32) -> TrackedChildGuard<'_> {
        let (paused, cancelled) = {
            let mut state = self.lock_state();
            if pid > 0 {
                state.children.insert(pid);
            }
            (state.paused, state.cancelled)
        };
        if pid > 0 {
            if cancelled {
                signal_child_group(pid, libc::SIGCONT);
                signal_child_group(pid, libc::SIGTERM);
            } else if paused {
                signal_child_group(pid, libc::SIGSTOP);
            }
        }
        TrackedChildGuard { ctrl: self, pid }
    }

    pub fn request_pause(&self) -> bool {
        let (children, cb) = {
            let mut state = self.lock_state();
            if state.cancelled || state.paused {
                return false;
            }
            state.paused = true;
            state.paused_at = Some(Instant::now());
            (
                state.children.iter().copied().collect::<Vec<_>>(),
                state.ui_pause_callback.clone(),
            )
        };

        if self.0.write_pause_marker {
            write_current_pause_marker();
        }
        for pid in children {
            signal_child_group(pid, libc::SIGSTOP);
        }
        if let Some(callback) = cb {
            callback(true);
        }
        true
    }

    pub fn request_resume(&self) -> bool {
        let (children, cb) = {
            let mut state = self.lock_state();
            if !state.paused {
                return false;
            }
            state.paused = false;
            if let Some(started) = state.paused_at.take() {
                state.paused_duration += started.elapsed();
            }
            self.0.condvar.notify_all();
            (
                state.children.iter().copied().collect::<Vec<_>>(),
                state.ui_pause_callback.clone(),
            )
        };

        if self.0.write_pause_marker {
            remove_current_pause_marker();
        }
        for pid in children {
            signal_child_group(pid, libc::SIGCONT);
        }
        if let Some(callback) = cb {
            callback(false);
        }
        true
    }

    pub fn request_cancel(&self) -> bool {
        let (children, cb) = {
            let mut state = self.lock_state();
            if state.cancelled {
                return false;
            }
            state.cancelled = true;
            state.paused = false;
            if let Some(started) = state.paused_at.take() {
                state.paused_duration += started.elapsed();
            }
            self.0.condvar.notify_all();
            (
                state.children.iter().copied().collect::<Vec<_>>(),
                state.ui_pause_callback.clone(),
            )
        };

        if self.0.write_pause_marker {
            remove_current_pause_marker();
        }
        if let Some(callback) = cb {
            callback(false);
        }

        for &pid in &children {
            // SIGSTOP altındaki alt süreç SIGTERM'i hemen işleyemez; önce uyandır.
            signal_child_group(pid, libc::SIGCONT);
            signal_child_group(pid, libc::SIGTERM);
        }

        if !children.is_empty() {
            let inner = Arc::clone(&self.0);
            std::thread::spawn(move || {
                for _ in 0..15 {
                    std::thread::sleep(Duration::from_millis(100));
                    let still_running = {
                        let state = inner.state.lock().unwrap_or_else(|e| e.into_inner());
                        children
                            .iter()
                            .copied()
                            .any(|pid| state.children.contains(&pid) && process_alive(pid as i32))
                    };
                    if !still_running {
                        return;
                    }
                }
                let survivors: Vec<u32> = {
                    let state = inner.state.lock().unwrap_or_else(|e| e.into_inner());
                    children
                        .iter()
                        .copied()
                        .filter(|pid| state.children.contains(pid) && process_alive(*pid as i32))
                        .collect()
                };
                for pid in survivors {
                    signal_child_group(pid, libc::SIGKILL);
                }
            });
        }

        true
    }
}

pub fn global_job_control() -> &'static JobControl {
    static GLOBAL: OnceLock<JobControl> = OnceLock::new();
    GLOBAL.get_or_init(|| JobControl::with_marker(true))
}

pub fn is_job_paused() -> bool {
    global_job_control().is_paused()
}

pub fn is_job_cancelled() -> bool {
    global_job_control().is_cancelled()
}

pub fn job_paused_duration() -> Duration {
    global_job_control().paused_duration()
}

pub fn wait_if_paused_or_cancelled() -> anyhow::Result<()> {
    global_job_control().wait_if_paused_or_cancelled()
}

pub fn request_pause() -> bool {
    global_job_control().request_pause()
}

pub fn request_resume() -> bool {
    global_job_control().request_resume()
}

pub fn request_cancel() -> bool {
    global_job_control().request_cancel()
}

pub fn register_running_child<'a>(pid: u32) -> TrackedChildGuard<'a> {
    global_job_control().register_child(pid)
}

pub fn set_ui_pause_callback(cb: Option<PauseUiCallback>) {
    global_job_control().set_ui_pause_callback(cb);
}

/// Alt süreci kendi süreç grubuna ayırır ve ana süreç çökerse sonlanmasını güvenceye alır.
#[cfg(unix)]
pub fn configure_tracked_command(cmd: &mut Command) {
    use std::os::unix::process::CommandExt;
    // SAFETY: pre_exec runs inside the forked child before execve. `setpgid(0, 0)` places the
    // child into its own process group so pause/cancel signals target only that child tree.
    // On Linux, `prctl(PR_SET_PDEATHSIG, SIGTERM)` asks the kernel to clean up if parent dies.
    unsafe {
        cmd.pre_exec(|| {
            let _ = libc::setpgid(0, 0);
            #[cfg(target_os = "linux")]
            {
                let _ = libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGTERM);
            }
            Ok(())
        });
    }
}

#[cfg(not(unix))]
pub fn configure_tracked_command(_cmd: &mut Command) {}

pub fn run_tracked_status_with(ctrl: &JobControl, cmd: &mut Command) -> anyhow::Result<ExitStatus> {
    ctrl.wait_if_paused_or_cancelled()?;
    configure_tracked_command(cmd);
    let mut child = cmd.spawn().with_context(|| "alt işlem başlatılamadı")?;
    let guard = ctrl.register_child(child.id());
    let status = child.wait().with_context(|| "alt işlem beklenemedi")?;
    drop(guard);
    if ctrl.is_cancelled() {
        return Err(cancelled_error());
    }
    Ok(status)
}

pub fn run_tracked_status(cmd: &mut Command) -> anyhow::Result<ExitStatus> {
    run_tracked_status_with(global_job_control(), cmd)
}

pub fn run_tracked_output_with(ctrl: &JobControl, cmd: &mut Command) -> anyhow::Result<Output> {
    ctrl.wait_if_paused_or_cancelled()?;
    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
    configure_tracked_command(cmd);
    let child = cmd.spawn().with_context(|| "alt işlem başlatılamadı")?;
    let guard = ctrl.register_child(child.id());
    let output = child
        .wait_with_output()
        .with_context(|| "alt işlem çıktısı okunamadı")?;
    drop(guard);
    if ctrl.is_cancelled() {
        return Err(cancelled_error());
    }
    Ok(output)
}

pub fn run_tracked_output(cmd: &mut Command) -> anyhow::Result<Output> {
    run_tracked_output_with(global_job_control(), cmd)
}

#[cfg(unix)]
fn signal_child_group(pid: u32, sig: libc::c_int) {
    let Ok(pid_i32) = i32::try_from(pid) else {
        return;
    };
    if pid_i32 <= 0 {
        return;
    }
    // SAFETY: `pid_i32` is a validated positive PID for a tracked child process group.
    unsafe {
        if libc::kill(-pid_i32, sig) != 0 {
            let _ = libc::kill(pid_i32, sig);
        }
    }
}

#[cfg(not(unix))]
fn signal_child_group(_pid: u32, _sig: i32) {}

/// Eski tekil PID kilit dosyası konumu (~/.cache/konvert/running.pid).
pub fn legacy_pid_file_path() -> Option<PathBuf> {
    directories::ProjectDirs::from("", "", "konvert").map(|p| p.cache_dir().join("running.pid"))
}

/// Aktif işlem bazlı PID kilit dosyalarının klasörü (~/.cache/konvert/pids).
pub fn pids_dir_path() -> Option<PathBuf> {
    directories::ProjectDirs::from("", "", "konvert").map(|p| p.cache_dir().join("pids"))
}

/// Bu sürecin PID kilit dosyasının konumunu döndürür (~/.cache/konvert/pids/<pid>.pid).
pub fn pid_file_path() -> Option<PathBuf> {
    pids_dir_path().map(|dir| dir.join(format!("{}.pid", std::process::id())))
}

/// Belirtilen PID için duraklatma işaretçi dosyasının konumunu döndürür (~/.cache/konvert/pids/<pid>.paused).
pub fn pause_file_path_for(pid: i32) -> Option<PathBuf> {
    if pid <= 0 {
        return None;
    }
    pids_dir_path().map(|dir| dir.join(format!("{}.paused", pid)))
}

/// Hedef PID şu anda duraklatılmış durumda mı?
pub fn is_pid_paused(pid: i32) -> bool {
    pause_file_path_for(pid).is_some_and(|p| p.exists())
}

fn write_current_pause_marker() {
    if let Some(p) = pause_file_path_for(std::process::id() as i32) {
        if let Some(parent) = p.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::write(&p, "paused");
    }
}

fn remove_current_pause_marker() {
    if let Some(p) = pause_file_path_for(std::process::id() as i32) {
        let _ = fs::remove_file(p);
    }
}

#[cfg(unix)]
static SIGNAL_PID_PATH: OnceLock<std::ffi::CString> = OnceLock::new();
#[cfg(unix)]
static SIGNAL_PAUSE_PATH: OnceLock<std::ffi::CString> = OnceLock::new();
#[cfg(unix)]
static SIGNAL_WRITE_FD: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(-1);
#[cfg(unix)]
static SIGNAL_WATCHER_ONCE: OnceLock<()> = OnceLock::new();
#[cfg(unix)]
static PENDING_PAUSE_ACTION: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(0);
#[cfg(unix)]
static PENDING_SIG_CANCEL: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);
#[cfg(unix)]
static CANCEL_SIGNAL_ARMED: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

#[cfg(unix)]
extern "C" fn handle_job_signal(sig: libc::c_int) {
    use std::sync::atomic::Ordering;

    let mut force_default = false;
    match sig {
        libc::SIGUSR1 => {
            PENDING_PAUSE_ACTION.store(1, Ordering::Release);
        }
        libc::SIGUSR2 => {
            PENDING_PAUSE_ACTION.store(2, Ordering::Release);
        }
        libc::SIGINT | libc::SIGTERM => {
            if !CANCEL_SIGNAL_ARMED.swap(true, Ordering::AcqRel) {
                PENDING_SIG_CANCEL.store(true, Ordering::Release);
            } else {
                force_default = true;
            }
        }
        _ => {}
    }

    if force_default {
        if let Some(path) = SIGNAL_PID_PATH.get() {
            // SAFETY: `path` is an immutable valid CString initialized before handler registration.
            unsafe {
                libc::unlink(path.as_ptr());
            }
        }
        if let Some(pause_path) = SIGNAL_PAUSE_PATH.get() {
            // SAFETY: `pause_path` is an immutable valid CString initialized before handler registration.
            unsafe {
                libc::unlink(pause_path.as_ptr());
            }
        }
        // SAFETY: Restores default handler and re-raises on second Ctrl+C / SIGTERM.
        unsafe {
            libc::signal(sig, libc::SIG_DFL);
            libc::raise(sig);
        }
        return;
    }

    let write_fd = SIGNAL_WRITE_FD.load(Ordering::Acquire);
    if write_fd >= 0 {
        let byte: [u8; 1] = [1];
        // SAFETY: Non-blocking write of 1 byte to self-pipe is async-signal-safe per POSIX.
        unsafe {
            let _ = libc::write(write_fd, byte.as_ptr().cast::<libc::c_void>(), 1);
        }
    }
}

#[cfg(unix)]
fn start_signal_watcher_once() {
    use std::sync::atomic::Ordering;

    SIGNAL_WATCHER_ONCE.get_or_init(|| {
        let mut fds = [0 as libc::c_int; 2];
        // SAFETY: `fds` is a valid 2-int array passed to POSIX `pipe`.
        if unsafe { libc::pipe(fds.as_mut_ptr()) } != 0 {
            return;
        }
        let read_fd = fds[0];
        let write_fd = fds[1];

        // SAFETY: `read_fd` and `write_fd` are newly opened valid pipe descriptors.
        unsafe {
            libc::fcntl(read_fd, libc::F_SETFD, libc::FD_CLOEXEC);
            libc::fcntl(write_fd, libc::F_SETFD, libc::FD_CLOEXEC);
            let flags = libc::fcntl(write_fd, libc::F_GETFL);
            if flags >= 0 {
                libc::fcntl(write_fd, libc::F_SETFL, flags | libc::O_NONBLOCK);
            }
        }

        SIGNAL_WRITE_FD.store(write_fd, Ordering::Release);

        let _ = std::thread::Builder::new()
            .name("konvert-signal-watcher".to_string())
            .spawn(move || {
                let mut buf = [0u8; 16];
                loop {
                    // SAFETY: `read_fd` stays open for process lifetime and reads into local buffer.
                    let n = unsafe {
                        libc::read(read_fd, buf.as_mut_ptr().cast::<libc::c_void>(), buf.len())
                    };
                    if n < 0 {
                        let err = std::io::Error::last_os_error();
                        if err.raw_os_error() == Some(libc::EINTR) {
                            continue;
                        }
                        break;
                    }
                    if n == 0 {
                        break;
                    }

                    if PENDING_SIG_CANCEL.swap(false, Ordering::AcqRel) {
                        global_job_control().request_cancel();
                    }
                    match PENDING_PAUSE_ACTION.swap(0, Ordering::AcqRel) {
                        1 => {
                            global_job_control().request_pause();
                        }
                        2 => {
                            global_job_control().request_resume();
                        }
                        _ => {}
                    }
                }
            });
    });
}

#[cfg(unix)]
fn install_signal_handlers(pid_path: &Path) {
    use std::os::unix::ffi::OsStrExt;

    if let Ok(c_path) = std::ffi::CString::new(pid_path.as_os_str().as_bytes()) {
        let _ = SIGNAL_PID_PATH.set(c_path);
    }
    if let Some(pause_path) = pause_file_path_for(std::process::id() as i32)
        && let Ok(c_pause) = std::ffi::CString::new(pause_path.as_os_str().as_bytes())
    {
        let _ = SIGNAL_PAUSE_PATH.set(c_pause);
    }

    start_signal_watcher_once();

    // SAFETY: Standard sigaction registration with valid handler pointer and zeroed mask.
    unsafe {
        let mut sa: libc::sigaction = std::mem::zeroed();
        sa.sa_sigaction = handle_job_signal as *const () as usize;
        sa.sa_flags = libc::SA_RESTART;
        libc::sigemptyset(&mut sa.sa_mask);
        libc::sigaction(libc::SIGINT, &sa, std::ptr::null_mut());
        libc::sigaction(libc::SIGTERM, &sa, std::ptr::null_mut());
        libc::sigaction(libc::SIGUSR1, &sa, std::ptr::null_mut());
        libc::sigaction(libc::SIGUSR2, &sa, std::ptr::null_mut());
    }
}

#[cfg(not(unix))]
fn install_signal_handlers(_pid_path: &Path) {}

/// İşlem bir terminale bağlı değilse (sağ tık menüsü, .desktop, arka plan servisi)
/// kendi süreç grubunun lideri olur. Böylece `--cancel` çağrıldığında ffmpeg,
/// gs veya magick gibi alt süreçler de tek bir sinyalle temiz bir şekilde sonlandırılabilir.
#[cfg(unix)]
pub fn detach_process_group() {
    // SAFETY: libc::isatty checks whether standard descriptors are attached to a terminal.
    // libc::setsid creates a new session and process group only when detached from all TTYs.
    unsafe {
        let attached_to_tty = libc::isatty(libc::STDIN_FILENO) == 1
            || libc::isatty(libc::STDOUT_FILENO) == 1
            || libc::isatty(libc::STDERR_FILENO) == 1;
        if !attached_to_tty {
            let _ = libc::setsid();
        }
    }
}

#[cfg(not(unix))]
pub fn detach_process_group() {}

/// Aktif işlem için PID başına kilit dosyasını yazar, sinyal işleyiciyi kurar ve süreç grubunu ayrıştırır.
pub fn write_pid_lock() {
    global_job_control().reset();
    #[cfg(unix)]
    {
        use std::sync::atomic::Ordering;
        PENDING_PAUSE_ACTION.store(0, Ordering::Release);
        PENDING_SIG_CANCEL.store(false, Ordering::Release);
        CANCEL_SIGNAL_ARMED.store(false, Ordering::Release);
    }
    detach_process_group();
    crate::converter::cleanup_recorded_temps();
    if let Some(p) = pid_file_path() {
        if let Some(parent) = p.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::write(&p, std::process::id().to_string());
        install_signal_handlers(&p);
    }
}

/// İşlem başarıyla bittiğinde veya iptal edildiğinde geçerli sürecin PID kilit dosyasını temizler.
pub fn remove_pid_lock() {
    if let Some(p) = pid_file_path() {
        let _ = fs::remove_file(p);
    }
    remove_current_pause_marker();
    if let Some(legacy) = legacy_pid_file_path()
        && let Ok(content) = fs::read_to_string(&legacy)
        && content.trim().parse::<u32>().ok() == Some(std::process::id())
    {
        let _ = fs::remove_file(legacy);
    }
}

/// Çalışmakta olan tüm Konvert arka plan süreçlerini tarar ve bayat kilit dosyalarını temizler.
pub fn list_active_konvert_pids() -> Vec<(i32, PathBuf)> {
    let self_pid = std::process::id() as i32;
    let mut active = Vec::new();
    let mut seen = std::collections::HashSet::new();

    let mut candidates = Vec::new();
    if let Some(pids_dir) = pids_dir_path()
        && let Ok(entries) = fs::read_dir(pids_dir)
    {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("pid") {
                candidates.push(path);
            }
        }
    }
    if let Some(legacy) = legacy_pid_file_path()
        && legacy.exists()
    {
        candidates.push(legacy);
    }

    for path in candidates {
        let Ok(content) = fs::read_to_string(&path) else {
            let _ = fs::remove_file(&path);
            continue;
        };
        let Ok(pid) = content.trim().parse::<i32>() else {
            let _ = fs::remove_file(&path);
            continue;
        };
        if pid == self_pid {
            continue;
        }
        if pid <= 0 || !process_is_konvert(pid) {
            let _ = fs::remove_file(&path);
            if let Some(pause_path) = pause_file_path_for(pid) {
                let _ = fs::remove_file(pause_path);
            }
            continue;
        }
        if seen.insert(pid) {
            active.push((pid, path));
        }
    }
    active
}

/// Çalışmakta olan bir Konvert sürecini duraklatmak için SIGUSR1 gönderir (yalnızca ana sürece).
pub fn pause_process(pid: i32) -> bool {
    if pid <= 0 || !process_alive(pid) || !process_is_konvert(pid) {
        return false;
    }
    #[cfg(unix)]
    // SAFETY: `pid` is validated positive, alive, and verified to be a konvert process.
    // Sending SIGUSR1 only to `pid` (not `-pid`) avoids killing child processes on USR1.
    unsafe {
        libc::kill(pid, libc::SIGUSR1) == 0
    }
    #[cfg(not(unix))]
    {
        false
    }
}

/// Duraklatılmış bir Konvert sürecini devam ettirmek için SIGUSR2 gönderir.
pub fn resume_process(pid: i32) -> bool {
    if pid <= 0 || !process_alive(pid) || !process_is_konvert(pid) {
        return false;
    }
    #[cfg(unix)]
    // SAFETY: `pid` is validated positive, alive, and verified to be a konvert process.
    unsafe {
        libc::kill(pid, libc::SIGUSR2) == 0
    }
    #[cfg(not(unix))]
    {
        false
    }
}

/// PID dosyasındaki sürecin gerçekten Konvert olup olmadığını doğrular.
/// Bayat kayıt (PID yeniden kullanımı) durumunda ilgisiz sistem süreçlerinin öldürülmesini önler.
pub fn process_is_konvert(pid: i32) -> bool {
    if pid <= 0 {
        return false;
    }
    let comm = fs::read_to_string(format!("/proc/{}/comm", pid)).unwrap_or_default();
    if comm.trim() == "konvert" {
        return true;
    }
    match fs::read_link(format!("/proc/{}/exe", pid)) {
        Ok(exe) => exe
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name == "konvert" || name == "konvert (deleted)"),
        Err(_) => false,
    }
}

/// Sürecin hayatta olup olmadığını kontrol eder (kill(pid, 0)).
pub fn process_alive(pid: i32) -> bool {
    if pid <= 0 {
        return false;
    }
    #[cfg(unix)]
    // SAFETY: libc::kill with signal 0 performs error checking only without sending
    // an actual signal. It safely verifies if the target pid (> 0) exists and is accessible.
    unsafe {
        libc::kill(pid, 0) == 0
    }

    #[cfg(not(unix))]
    {
        false
    }
}

/// Önce SIGCONT + SIGTERM (varsa tüm süreç grubuna), ardından 2 saniye içinde yanıt gelmezse SIGKILL gönderir.
/// Süreç gerçekten sonlandırıldıysa `true` döner.
pub fn terminate_process(pid: i32) -> bool {
    if pid <= 0 {
        return false;
    }
    #[cfg(unix)]
    // SAFETY: getpgid and kill are standard POSIX syscalls with validated positive pid.
    // Negative target (-pid) targets the entire process group if the process is a group leader,
    // ensuring spawned child processes (e.g. ffmpeg, gs) are terminated as well.
    unsafe {
        if !process_alive(pid) {
            return false;
        }

        if let Some(pause_path) = pause_file_path_for(pid) {
            let _ = fs::remove_file(pause_path);
        }

        // Süreç kendi grubunun lideriyse grubu, değilse sadece kendisini hedefle.
        let is_group_leader = libc::getpgid(pid) == pid;
        let target = if is_group_leader { -pid } else { pid };

        // Duraklatılmış alt süreçler SIGCONT olmadan SIGTERM'i işleyemez.
        let _ = libc::kill(target, libc::SIGCONT);
        let mut sent = libc::kill(target, libc::SIGTERM);
        if sent != 0 && is_group_leader {
            let _ = libc::kill(pid, libc::SIGCONT);
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
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_pids_are_rejected_safely() {
        for bad_pid in [0, -1, -100, i32::MIN] {
            assert!(!process_is_konvert(bad_pid));
            assert!(!process_alive(bad_pid));
            assert!(!pause_process(bad_pid));
            assert!(!resume_process(bad_pid));
            assert!(!terminate_process(bad_pid));
        }
    }

    #[test]
    fn test_job_control_pause_resume_tracks_paused_duration() {
        let ctrl = JobControl::new();
        assert!(!ctrl.is_paused());
        assert!(!ctrl.is_cancelled());
        assert_eq!(ctrl.paused_duration(), Duration::ZERO);

        assert!(ctrl.request_pause());
        assert!(!ctrl.request_pause());
        assert!(ctrl.is_paused());

        std::thread::sleep(Duration::from_millis(35));
        assert!(ctrl.paused_duration() >= Duration::from_millis(20));

        assert!(ctrl.request_resume());
        assert!(!ctrl.request_resume());
        assert!(!ctrl.is_paused());
        let recorded = ctrl.paused_duration();
        assert!(recorded >= Duration::from_millis(20));

        std::thread::sleep(Duration::from_millis(15));
        assert_eq!(ctrl.paused_duration(), recorded);
    }

    #[test]
    fn test_job_control_cancel_wakes_paused_waiter() {
        let ctrl = JobControl::new();
        assert!(ctrl.request_pause());

        let waiter_ctrl = ctrl.clone();
        let (started_tx, started_rx) = std::sync::mpsc::channel();
        let handle = std::thread::spawn(move || {
            let _ = started_tx.send(());
            waiter_ctrl.wait_if_paused_or_cancelled()
        });

        started_rx.recv().unwrap();
        std::thread::sleep(Duration::from_millis(20));
        assert!(ctrl.request_cancel());
        assert!(!ctrl.request_pause());
        assert!(!ctrl.request_resume());

        let res = handle.join().unwrap();
        let err = res.expect_err("cancelled waiter must error");
        assert!(err.chain().any(|c| c.is::<JobCancelledError>()));
    }

    #[cfg(unix)]
    #[test]
    fn test_tracked_child_stops_resumes_and_cancels() {
        let ctrl = JobControl::new();
        let run_ctrl = ctrl.clone();
        let (done_tx, done_rx) = std::sync::mpsc::channel();

        let worker = std::thread::spawn(move || {
            let mut cmd = Command::new("sleep");
            cmd.arg("10");
            let res = run_tracked_status_with(&run_ctrl, &mut cmd);
            let _ = done_tx.send(res);
        });

        // Alt süreç kaydolana kadar kısa süre bekle.
        let mut child_pid = None;
        for _ in 0..40 {
            if let Some(&pid) = ctrl.lock_state().children.iter().next() {
                child_pid = Some(pid);
                break;
            }
            std::thread::sleep(Duration::from_millis(25));
        }
        let pid = child_pid.expect("tracked sleep child should register");

        assert!(ctrl.request_pause());
        let mut stopped = false;
        for _ in 0..30 {
            if let Ok(stat) = fs::read_to_string(format!("/proc/{}/stat", pid))
                && stat.contains(" T ")
            {
                stopped = true;
                break;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        assert!(
            stopped,
            "tracked child process should enter stopped state T"
        );

        assert!(ctrl.request_resume());
        std::thread::sleep(Duration::from_millis(20));
        assert!(ctrl.request_cancel());

        let result = done_rx
            .recv_timeout(Duration::from_secs(3))
            .expect("cancelled tracked child should finish promptly");
        let err = result.expect_err("cancelled tracked child must return JobCancelledError");
        assert!(err.chain().any(|c| c.is::<JobCancelledError>()));
        worker.join().unwrap();
    }
}
