use notify_rust::{Hint, Notification, NotificationHandle, Timeout};
use zbus::{Connection, MatchRule, MessageStream, OwnedMatchRule, OwnedObjectPath, block_on};

enum JobMsg {
    SetInfo(String),
    SetDescriptionField(u32, String, String),
    SetAmount {
        processed: u64,
        total: u64,
        unit: String,
    },
    SetPercent(u32),
    SetSuspended(bool),
    Terminate(String),
}

async fn next_dbus_message(stream: &mut MessageStream) -> Option<zbus::Result<zbus::Message>> {
    use std::pin::Pin;
    use zbus::export::futures_core::Stream;
    std::future::poll_fn(|cx| Pin::new(&mut *stream).poll_next(cx)).await
}

fn has_session_dbus() -> bool {
    std::env::var("DBUS_SESSION_BUS_ADDRESS")
        .map(|v| !v.trim().is_empty())
        .unwrap_or(false)
}

fn is_kde_jobview_environment() -> bool {
    if !has_session_dbus() {
        return false;
    }
    ["XDG_CURRENT_DESKTOP", "DESKTOP_SESSION"]
        .iter()
        .filter_map(|key| std::env::var(key).ok())
        .any(|val| {
            let lower = val.to_ascii_lowercase();
            lower.contains("kde") || lower.contains("plasma")
        })
}

pub struct KdeJobView {
    tx: std::sync::mpsc::Sender<JobMsg>,
    done_rx: std::sync::mpsc::Receiver<()>,
    worker: Option<std::thread::JoinHandle<()>>,
    terminated: bool,
}

impl KdeJobView {
    pub fn new(app_name: &str, icon: &str, info: &str) -> Option<Self> {
        if !is_kde_jobview_environment() {
            return None;
        }
        let (tx, rx) = std::sync::mpsc::channel();
        let (init_tx, init_rx) = std::sync::mpsc::channel();
        let (done_tx, done_rx) = std::sync::mpsc::channel();
        let app = app_name.to_string();
        let ic = icon.to_string();
        let initial_info = info.to_string();

        let worker = std::thread::spawn(move || {
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
                        &(app.as_str(), ic.as_str(), 3i32),
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

                if init_tx.send(true).is_err() {
                    let _ = conn
                        .call_method(
                            Some("org.kde.kuiserver"),
                            &job_path,
                            Some("org.kde.JobViewV2"),
                            "terminate",
                            &("",),
                        )
                        .await;
                    let _ = conn.close().await;
                    return;
                }

                // Plasma 6 JobViewV2 düğmelerinden gelen Duraklat / Devam Et / İptal sinyallerini dinle.
                let conn_sig = conn.clone();
                let job_path_sig = job_path.clone();
                let listener_handle = std::thread::spawn(move || {
                    let Ok(rule) = MatchRule::builder()
                        .msg_type(zbus::message::Type::Signal)
                        .interface("org.kde.JobViewV2")
                        .path(job_path_sig.as_str())
                        .build()
                    else {
                        return;
                    };
                    block_on(async move {
                        let Ok(mut stream) = MessageStream::for_match_rule(
                            OwnedMatchRule::from(rule),
                            &conn_sig,
                            None,
                        )
                        .await
                        else {
                            return;
                        };
                        while let Some(Ok(msg)) = next_dbus_message(&mut stream).await {
                            let header = msg.header();
                            match header.member().map(|m| m.as_str()) {
                                Some("suspendRequested") => {
                                    crate::process::request_pause();
                                }
                                Some("resumeRequested") => {
                                    crate::process::request_resume();
                                }
                                Some("cancelRequested") => {
                                    crate::process::request_cancel();
                                }
                                _ => {}
                            }
                        }
                    });
                });

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
                        JobMsg::SetDescriptionField(index, name, value) => {
                            let _ = conn
                                .call_method(
                                    Some("org.kde.kuiserver"),
                                    &job_path,
                                    Some("org.kde.JobViewV2"),
                                    "setDescriptionField",
                                    &(index, name.as_str(), value.as_str()),
                                )
                                .await;
                        }
                        JobMsg::SetAmount {
                            processed,
                            total,
                            unit,
                        } => {
                            if total > 0 {
                                let _ = conn
                                    .call_method(
                                        Some("org.kde.kuiserver"),
                                        &job_path,
                                        Some("org.kde.JobViewV2"),
                                        "setTotalAmount",
                                        &(total, unit.as_str()),
                                    )
                                    .await;
                            }
                            let _ = conn
                                .call_method(
                                    Some("org.kde.kuiserver"),
                                    &job_path,
                                    Some("org.kde.JobViewV2"),
                                    "setProcessedAmount",
                                    &(processed, unit.as_str()),
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
                        JobMsg::SetSuspended(paused) => {
                            let _ = conn
                                .call_method(
                                    Some("org.kde.kuiserver"),
                                    &job_path,
                                    Some("org.kde.JobViewV2"),
                                    "setSuspended",
                                    &(paused,),
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
                let _ = conn.close().await;
                let _ = listener_handle.join();
            });
            let _ = done_tx.send(());
        });

        match init_rx.recv_timeout(std::time::Duration::from_millis(300)) {
            Ok(true) => {
                let pause_tx = tx.clone();
                crate::process::set_ui_pause_callback(Some(std::sync::Arc::new(move |paused| {
                    let _ = pause_tx.send(JobMsg::SetSuspended(paused));
                })));
                Some(Self {
                    tx,
                    done_rx,
                    worker: Some(worker),
                    terminated: false,
                })
            }
            _ => None,
        }
    }

    pub fn set_info(&self, message: &str) {
        if self.terminated {
            return;
        }
        let _ = self.tx.send(JobMsg::SetInfo(message.to_string()));
    }

    pub fn set_description_field(&self, index: u32, name: &str, value: &str) {
        if self.terminated {
            return;
        }
        let _ = self.tx.send(JobMsg::SetDescriptionField(
            index,
            name.to_string(),
            value.to_string(),
        ));
    }

    pub fn set_amount(&self, processed: u64, total: u64, unit: &str) {
        if self.terminated {
            return;
        }
        let _ = self.tx.send(JobMsg::SetAmount {
            processed,
            total,
            unit: unit.to_string(),
        });
    }

    pub fn set_percent(&self, percent: i32) {
        if self.terminated {
            return;
        }
        let clamped = percent.clamp(0, 100) as u32;
        let _ = self.tx.send(JobMsg::SetPercent(clamped));
    }

    pub fn set_suspended(&self, paused: bool) {
        if self.terminated {
            return;
        }
        let _ = self.tx.send(JobMsg::SetSuspended(paused));
    }

    pub fn terminate(&mut self, error: &str) {
        if self.terminated {
            return;
        }
        self.terminated = true;
        crate::process::set_ui_pause_callback(None);
        if self.tx.send(JobMsg::Terminate(error.to_string())).is_ok() {
            let _ = self
                .done_rx
                .recv_timeout(std::time::Duration::from_millis(250));
        }
        if let Some(worker) = self.worker.take()
            && worker.is_finished()
        {
            let _ = worker.join();
        }
    }
}

impl Drop for KdeJobView {
    fn drop(&mut self) {
        self.terminate("");
    }
}

pub struct JobProgressUpdate<'a> {
    pub headline: &'a str,
    pub source_value: &'a str,
    pub target_value: &'a str,
    pub percent: i32,
    pub detail_text: Option<&'a str>,
    pub batch_progress: Option<(u64, u64)>,
}

pub fn format_job_fallback_body(
    source_label: &str,
    source_val: &str,
    target_label: &str,
    target_val: &str,
    detail: &str,
) -> String {
    format!(
        "{}: {}\n{}: {} ({})",
        source_label, source_val, target_label, target_val, detail
    )
}

fn resolve_job_metrics(percent: i32, detail_text: Option<&str>) -> (i32, String) {
    let clamped = percent.clamp(0, 100);
    let detail = detail_text
        .filter(|s| !s.is_empty())
        .map(ToString::to_string)
        .unwrap_or_else(|| format!("{}%", clamped));
    (clamped, detail)
}

fn apply_kde_job_update(
    job: &KdeJobView,
    update: &JobProgressUpdate<'_>,
    source_label: &str,
    target_label: &str,
    clamped: i32,
    detail: &str,
) {
    job.set_description_field(0, source_label, update.source_value);
    job.set_description_field(
        1,
        target_label,
        &format!("{} | {}", update.target_value, detail),
    );
    if let Some((processed, total)) = update.batch_progress {
        job.set_amount(processed, total, "files");
    }
    job.set_percent(clamped);
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
        if !self.enabled || !has_session_dbus() {
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
        if !self.enabled || !has_session_dbus() {
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
        if !self.enabled || !has_session_dbus() {
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

    pub fn start_job(&self, update: &JobProgressUpdate<'_>) -> Option<ProgressSession> {
        if !self.enabled || !has_session_dbus() {
            return None;
        }

        let (clamped, detail) = resolve_job_metrics(update.percent, update.detail_text);
        let source_label = crate::i18n::t("notify_field_source");
        let target_label = crate::i18n::t("notify_field_target");

        if let Some(kde_job) = KdeJobView::new("Konvert", "konvert-symbolic", update.headline) {
            apply_kde_job_update(
                &kde_job,
                update,
                source_label,
                target_label,
                clamped,
                &detail,
            );
            return Some(ProgressSession {
                kde_job: Some(kde_job),
                fallback: None,
            });
        }

        let summary = format!("Konvert - {}", update.headline);
        let body = format_job_fallback_body(
            source_label,
            update.source_value,
            target_label,
            update.target_value,
            &detail,
        );

        let handle = Notification::new()
            .appname("Konvert")
            .summary(&summary)
            .body(&body)
            .icon("konvert-symbolic")
            .timeout(Timeout::Never)
            .hint(Hint::Resident(true))
            .hint(Hint::CustomInt("value".into(), clamped))
            .hint(Hint::CustomInt("x-kde-progress".into(), clamped))
            .hint(Hint::Custom("x-kde-progress-text".into(), detail))
            .show()
            .ok();

        Some(ProgressSession {
            kde_job: None,
            fallback: handle,
        })
    }

    pub fn update_job(&self, handle: &mut Option<ProgressSession>, update: &JobProgressUpdate<'_>) {
        if !self.enabled {
            return;
        }
        if let Some(session) = handle {
            let (clamped, detail) = resolve_job_metrics(update.percent, update.detail_text);
            let source_label = crate::i18n::t("notify_field_source");
            let target_label = crate::i18n::t("notify_field_target");

            if let Some(ref mut job) = session.kde_job {
                job.set_info(update.headline);
                apply_kde_job_update(job, update, source_label, target_label, clamped, &detail);
            } else if let Some(ref mut h) = session.fallback {
                let summary = format!("Konvert - {}", update.headline);
                let body = format_job_fallback_body(
                    source_label,
                    update.source_value,
                    target_label,
                    update.target_value,
                    &detail,
                );
                h.summary(&summary);
                h.body(&body);
                h.icon("konvert-symbolic");
                h.timeout(Timeout::Never);
                h.hint(Hint::CustomInt("value".into(), clamped));
                h.hint(Hint::CustomInt("x-kde-progress".into(), clamped));
                h.hint(Hint::Custom("x-kde-progress-text".into(), detail));
                let _ = h.update();
            }
        } else {
            *handle = self.start_job(update);
        }
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
                return;
            }
            if let Some(h) = session.fallback.take() {
                h.close();
            }
        }
        self.error(title, body);
    }

    pub fn set_job_paused(&self, handle: &mut Option<ProgressSession>, paused: bool) {
        if !self.enabled {
            return;
        }
        if let Some(session) = handle
            && let Some(ref job) = session.kde_job
        {
            job.set_suspended(paused);
        }
    }

    pub fn cancel_progress(&self, handle: &mut Option<ProgressSession>, title: &str, body: &str) {
        if !self.enabled {
            return;
        }
        if let Some(mut session) = handle.take() {
            if let Some(ref mut job) = session.kde_job {
                job.terminate("");
                return;
            }
            if let Some(h) = session.fallback.take() {
                h.close();
            }
        }
        self.success(title, body);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_job_fallback_body() {
        let body = format_job_fallback_body(
            "Source",
            "clip.mp4 (21.0 MB)",
            "Target",
            "WEBM",
            "45% | 2.1x",
        );
        assert_eq!(
            body,
            "Source: clip.mp4 (21.0 MB)\nTarget: WEBM (45% | 2.1x)"
        );

        let paused_special = format_job_fallback_body(
            "Source",
            "clip <1> & \"test\".mp4 (10.0 MB)",
            "Target",
            "MP4 (Paused)",
            "50% | Paused",
        );
        assert_eq!(
            paused_special,
            "Source: clip <1> & \"test\".mp4 (10.0 MB)\nTarget: MP4 (Paused) (50% | Paused)"
        );
    }

    #[test]
    fn test_disabled_notifier_start_job_returns_none() {
        let notifier = Notifier::new(false);
        let session = notifier.start_job(&JobProgressUpdate {
            headline: "Converting: WEBM",
            source_value: "clip.mp4 (21.0 MB)",
            target_value: "WEBM",
            percent: 25,
            detail_text: Some("25%"),
            batch_progress: None,
        });
        assert!(session.is_none());
    }
}
