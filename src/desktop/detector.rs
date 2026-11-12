
/// Masaüstü entegrasyon kurulum/kaldırma sonucu.
///
/// Tek tek başarısızlıklar yutulmaz; komut satırından çıkış koduna ve
/// kullanıcıya görünen rapora yansır.
pub struct IntegrationReport {
    pub affected: Vec<String>,
    pub failed: Vec<(String, String)>,
}

pub fn install_all() -> IntegrationReport {
    let mut report = IntegrationReport {
        affected: Vec::new(),
        failed: Vec::new(),
    };

    match super::dolphin::install_dolphin_service(None) {
        Ok(p) => report
            .affected
            .push(format!("KDE Dolphin ({})", p.display())),
        Err(e) => report
            .failed
            .push(("KDE Dolphin".to_string(), format!("{:#}", e))),
    }

    match super::nautilus::install_nautilus_scripts(None) {
        Ok(p) => report
            .affected
            .push(format!("GNOME Nautilus ({})", p.display())),
        Err(e) => report
            .failed
            .push(("GNOME Nautilus".to_string(), format!("{:#}", e))),
    }

    match super::nemo::install_nemo_actions(None) {
        Ok(p) => report
            .affected
            .push(format!("Cinnamon Nemo ({})", p.display())),
        Err(e) => report
            .failed
            .push(("Cinnamon Nemo".to_string(), format!("{:#}", e))),
    }

    match super::thunar::install_thunar_actions(None) {
        Ok(p) => report
            .affected
            .push(format!("XFCE Thunar ({})", p.display())),
        Err(e) => report
            .failed
            .push(("XFCE Thunar".to_string(), format!("{:#}", e))),
    }

    match super::entry::install_desktop_entry(None) {
        Ok(p) => report
            .affected
            .push(format!("Masaüstü kısayolu ({})", p.display())),
        Err(e) => report
            .failed
            .push(("Masaüstü kısayolu".to_string(), format!("{:#}", e))),
    }

    report
}

pub fn uninstall_all() -> IntegrationReport {
    let mut report = IntegrationReport {
        affected: Vec::new(),
        failed: Vec::new(),
    };

    match super::dolphin::uninstall_dolphin_service() {
        Ok(true) => report.affected.push("KDE Dolphin".to_string()),
        Ok(false) => {}
        Err(e) => report
            .failed
            .push(("KDE Dolphin".to_string(), format!("{:#}", e))),
    }

    match super::nautilus::uninstall_nautilus_scripts() {
        Ok(true) => report.affected.push("GNOME Nautilus".to_string()),
        Ok(false) => {}
        Err(e) => report
            .failed
            .push(("GNOME Nautilus".to_string(), format!("{:#}", e))),
    }

    match super::nemo::uninstall_nemo_actions() {
        Ok(true) => report.affected.push("Cinnamon Nemo".to_string()),
        Ok(false) => {}
        Err(e) => report
            .failed
            .push(("Cinnamon Nemo".to_string(), format!("{:#}", e))),
    }

    match super::thunar::uninstall_thunar_actions() {
        Ok(true) => report.affected.push("XFCE Thunar".to_string()),
        Ok(false) => {}
        Err(e) => report
            .failed
            .push(("XFCE Thunar".to_string(), format!("{:#}", e))),
    }

    report
}
