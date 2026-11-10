use anyhow::Result;

pub fn install_all() -> Result<Vec<String>> {
    let mut installed = Vec::new();

    if let Ok(p) = super::dolphin::install_dolphin_service(None) {
        installed.push(format!("KDE Dolphin ({})", p.display()));
    }

    if let Ok(p) = super::nautilus::install_nautilus_scripts(None) {
        installed.push(format!("GNOME Nautilus ({})", p.display()));
    }

    if let Ok(p) = super::thunar::install_thunar_actions(None) {
        installed.push(format!("XFCE Thunar ({})", p.display()));
    }

    let _ = super::entry::install_desktop_entry(None);

    Ok(installed)
}

pub fn uninstall_all() -> Result<Vec<String>> {
    let mut removed = Vec::new();

    if let Ok(true) = super::dolphin::uninstall_dolphin_service() {
        removed.push("KDE Dolphin".to_string());
    }

    if let Ok(true) = super::nautilus::uninstall_nautilus_scripts() {
        removed.push("GNOME Nautilus".to_string());
    }

    if let Ok(true) = super::thunar::uninstall_thunar_actions() {
        removed.push("XFCE Thunar".to_string());
    }

    Ok(removed)
}
