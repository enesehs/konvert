use notify_rust::{Hint, Notification, NotificationHandle};

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
            .icon("konvert")
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

    pub fn start_progress(&self, title: &str, body: &str, percent: i32) -> Option<NotificationHandle> {
        if !self.enabled {
            return None;
        }
        Notification::new()
            .appname("Konvert")
            .summary(title)
            .body(body)
            .icon("konvert")
            .hint(Hint::CustomInt("value".into(), percent.clamp(0, 100)))
            .show()
            .ok()
    }

    pub fn update_progress(
        &self,
        handle: &mut Option<NotificationHandle>,
        title: &str,
        body: &str,
        percent: i32,
    ) {
        if !self.enabled {
            return;
        }
        if let Some(h) = handle {
            h.summary(title);
            h.body(body);
            h.icon("konvert");
            h.hint(Hint::CustomInt("value".into(), percent.clamp(0, 100)));
            let _ = h.update();
        } else {
            *handle = self.start_progress(title, body, percent);
        }
    }

    pub fn finish_progress(
        &self,
        handle: &mut Option<NotificationHandle>,
        title: &str,
        body: &str,
    ) {
        if !self.enabled {
            return;
        }
        if let Some(h) = handle {
            h.summary(title);
            h.body(body);
            h.icon("konvert");
            h.hint(Hint::CustomInt("value".into(), 100));
            let _ = h.update();
        } else {
            self.success(title, body);
        }
    }

    pub fn error_progress(
        &self,
        handle: &mut Option<NotificationHandle>,
        title: &str,
        body: &str,
    ) {
        if !self.enabled {
            return;
        }
        if let Some(h) = handle {
            h.summary(title);
            h.body(body);
            h.icon("dialog-error-symbolic");
            let _ = h.update();
        } else {
            self.error(title, body);
        }
    }
}
