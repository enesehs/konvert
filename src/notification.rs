use notify_rust::{Hint, Notification, NotificationHandle, Timeout};
use zbus::{Connection, OwnedObjectPath, block_on};

enum JobMsg {
    SetInfo(String),
    SetPercent(u32),
    Terminate(String),
}

pub struct KdeJobView {
    tx: std::sync::mpsc::Sender<JobMsg>,
    terminated: bool,
}

impl KdeJobView {
    pub fn new(app_name: &str, icon: &str, info: &str) -> Option<Self> {
        let (tx, rx) = std::sync::mpsc::channel();
        let (init_tx, init_rx) = std::sync::mpsc::channel();
        let app = app_name.to_string();
        let ic = icon.to_string();
        let initial_info = info.to_string();

        std::thread::spawn(move || {
            block_on(async {
                let Ok(conn) = Connection::session().await else {
                    let _ = init_tx.send(false);
                    return;
                };

                let Ok(reply) = conn
                    .call_method(
                        Some("org.kde.kuiserver"),
                        "/JobViewServer",
                        Some("org.kde.JobViewServer"),
                        "requestView",
                        &(app.as_str(), ic.as_str(), 0i32),
                    )
                    .await
                else {
                    let _ = init_tx.send(false);
                    return;
                };

                let Ok(job_path): Result<OwnedObjectPath, _> = reply.body().deserialize() else {
                    let _ = init_tx.send(false);
                    return;
                };

                let _ = init_tx.send(true);

                let _ = conn
                    .call_method(
                        Some("org.kde.kuiserver"),
                        &job_path,
                        Some("org.kde.JobViewV2"),
                        "setInfoMessage",
                        &(initial_info.as_str(),),
                    )
                    .await;
                let _ = conn
                    .call_method(
                        Some("org.kde.kuiserver"),
                        &job_path,
                        Some("org.kde.JobViewV2"),
                        "setPercent",
                        &(0u32,),
                    )
                    .await;

                while let Ok(msg) = rx.recv() {
                    match msg {
                        JobMsg::SetInfo(text) => {
                            let _ = conn
                                .call_method(
                                    Some("org.kde.kuiserver"),
                                    &job_path,
                                    Some("org.kde.JobViewV2"),
                                    "setInfoMessage",
                                    &(text.as_str(),),
                                )
                                .await;
                        }
                        JobMsg::SetPercent(pct) => {
                            let _ = conn
                                .call_method(
                                    Some("org.kde.kuiserver"),
                                    &job_path,
                                    Some("org.kde.JobViewV2"),
                                    "setPercent",
                                    &(pct,),
                                )
                                .await;
                        }
                        JobMsg::Terminate(err) => {
                            let _ = conn
                                .call_method(
                                    Some("org.kde.kuiserver"),
                                    &job_path,
                                    Some("org.kde.JobViewV2"),
                                    "terminate",
                                    &(err.as_str(),),
                                )
                                .await;
                            break;
                        }
                    }
                }
            });
        });

        match init_rx.recv_timeout(std::time::Duration::from_millis(300)) {
            Ok(true) => Some(Self {
                tx,
                terminated: false,
            }),
            _ => None,
        }
    }

    pub fn set_info(&self, message: &str) {
        if self.terminated {
            return;
        }
        let _ = self.tx.send(JobMsg::SetInfo(message.to_string()));
    }

    pub fn set_percent(&self, percent: i32) {
        if self.terminated {
            return;
        }
        let clamped = percent.clamp(0, 100) as u32;
        let _ = self.tx.send(JobMsg::SetPercent(clamped));
    }

    pub fn terminate(&mut self, error: &str) {
        if self.terminated {
            return;
        }
        self.terminated = true;
        let _ = self.tx.send(JobMsg::Terminate(error.to_string()));
    }
}

impl Drop for KdeJobView {
    fn drop(&mut self) {
        self.terminate("");
    }
}

pub struct ProgressSession {
    kde_job: Option<KdeJobView>,
    fallback: Option<NotificationHandle>,
}

pub struct Notifier {
    enabled: bool,
}

impl Notifier {
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }

    pub fn success(&self, title: &str, body: &str) {
        if !self.enabled {
            return;
        }
        let _ = Notification::new()
            .appname("Konvert")
            .summary(title)
            .body(body)
            .icon("konvert-symbolic")
            .show();
    }

    pub fn error(&self, title: &str, body: &str) {
        if !self.enabled {
            return;
        }
        let _ = Notification::new()
            .appname("Konvert")
            .summary(title)
            .body(body)
            .icon("dialog-error-symbolic")
            .show();
    }

    pub fn start_progress(
        &self,
        title: &str,
        body: &str,
        percent: i32,
        progress_text: Option<&str>,
    ) -> Option<ProgressSession> {
        if !self.enabled {
            return None;
        }

        // Try KDE Plasma Native kuiserver JobView first
        if let Some(kde_job) = KdeJobView::new("Konvert", "konvert-symbolic", body) {
            kde_job.set_percent(percent);
            return Some(ProgressSession {
                kde_job: Some(kde_job),
                fallback: None,
            });
        }

        // Fallback to standard freedesktop notification (GNOME, XFCE, Sway, etc.)
        let clamped = percent.clamp(0, 100);
        let prog_str = progress_text
            .map(ToString::to_string)
            .unwrap_or_else(|| format!("{}%", clamped));

        let handle = Notification::new()
            .appname("Konvert")
            .summary(title)
            .body(body)
            .icon("konvert-symbolic")
            .timeout(Timeout::Never)
            .hint(Hint::Resident(true))
            .hint(Hint::CustomInt("value".into(), clamped))
            .hint(Hint::CustomInt("x-kde-progress".into(), clamped))
            .hint(Hint::Custom("x-kde-progress-text".into(), prog_str))
            .show()
            .ok();

        Some(ProgressSession {
            kde_job: None,
            fallback: handle,
        })
    }

    pub fn update_progress(
        &self,
        handle: &mut Option<ProgressSession>,
        title: &str,
        body: &str,
        percent: i32,
        progress_text: Option<&str>,
    ) {
        if !self.enabled {
            return;
        }
        if let Some(session) = handle {
            if let Some(ref mut job) = session.kde_job {
                job.set_info(body);
                job.set_percent(percent);
            } else if let Some(ref mut h) = session.fallback {
                let clamped = percent.clamp(0, 100);
                let prog_str = progress_text
                    .map(ToString::to_string)
                    .unwrap_or_else(|| format!("{}%", clamped));

                h.summary(title);
                h.body(body);
                h.icon("konvert-symbolic");
                h.timeout(Timeout::Never);
                h.hint(Hint::CustomInt("value".into(), clamped));
                h.hint(Hint::CustomInt("x-kde-progress".into(), clamped));
                h.hint(Hint::Custom("x-kde-progress-text".into(), prog_str));
                let _ = h.update();
            }
        } else {
            *handle = self.start_progress(title, body, percent, progress_text);
        }
    }

    pub fn finish_progress(&self, handle: &mut Option<ProgressSession>, title: &str, body: &str) {
        if !self.enabled {
            return;
        }
        if let Some(mut session) = handle.take() {
            if let Some(ref mut job) = session.kde_job {
                job.terminate("");
            } else if let Some(h) = session.fallback.take() {
                h.close();
            }
        }
        self.success(title, body);
    }

    pub fn error_progress(&self, handle: &mut Option<ProgressSession>, title: &str, body: &str) {
        if !self.enabled {
            return;
        }
        if let Some(mut session) = handle.take() {
            if let Some(ref mut job) = session.kde_job {
                job.terminate(body);
            } else if let Some(h) = session.fallback.take() {
                h.close();
            }
        }
        self.error(title, body);
    }
}
