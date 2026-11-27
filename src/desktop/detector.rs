use anyhow::Result;
use std::path::PathBuf;

/// Masaüstü entegrasyon kurulum/kaldırma sonucu.
///
/// Tek tek başarısızlıklar yutulmaz; komut satırından çıkış koduna ve
/// kullanıcıya görünen rapora yansır.
pub struct IntegrationReport {
    pub affected: Vec<String>,
    pub failed: Vec<(String, String)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileManager {
    Dolphin,
    Nautilus,
    Nemo,
    Thunar,
}

pub fn detect_installed_file_managers() -> Vec<FileManager> {
    let mut found = Vec::new();
    if crate::deps::command_exists("dolphin") {
        found.push(FileManager::Dolphin);
    }
    if crate::deps::command_exists("nautilus") {
        found.push(FileManager::Nautilus);
    }
    if crate::deps::command_exists("nemo") {
        found.push(FileManager::Nemo);
    }
    if crate::deps::command_exists("thunar") {
        found.push(FileManager::Thunar);
    }
    found
}

pub fn sync_installed_menus(
    enabled_actions: &[String],
    menu_exec: Option<&str>,
    menu_dir: Option<PathBuf>,
) -> Result<Vec<PathBuf>> {
    if let Some(dir) = menu_dir {
        let p = super::dolphin::install_dolphin_service_with_options(
            enabled_actions,
            menu_exec,
            Some(dir),
        )?;
        return Ok(vec![p]);
    }

    let mut targets = detect_installed_file_managers();
    if super::dolphin::is_dolphin_installed() && !targets.contains(&FileManager::Dolphin) {
        targets.push(FileManager::Dolphin);
    }
    if super::nautilus::is_nautilus_installed() && !targets.contains(&FileManager::Nautilus) {
        targets.push(FileManager::Nautilus);
    }
    if super::nemo::is_nemo_installed() && !targets.contains(&FileManager::Nemo) {
        targets.push(FileManager::Nemo);
    }
    if super::thunar::is_thunar_installed() && !targets.contains(&FileManager::Thunar) {
        targets.push(FileManager::Thunar);
    }
    if targets.is_empty() {
        targets.push(FileManager::Dolphin);
    }

    let mut synced = Vec::new();
    let mut first_err: Option<anyhow::Error> = None;

    for fm in targets {
        let res = match fm {
            FileManager::Dolphin => super::dolphin::install_dolphin_service_with_options(
                enabled_actions,
                menu_exec,
                None,
            ),
            FileManager::Nautilus => {
                super::nautilus::install_nautilus_scripts_with_actions(enabled_actions, menu_exec)
            }
            FileManager::Nemo => {
                super::nemo::install_nemo_actions_with_actions(enabled_actions, menu_exec, None)
            }
            FileManager::Thunar => {
                super::thunar::install_thunar_actions_with_actions(enabled_actions, menu_exec)
            }
        };
        match res {
            Ok(p) => synced.push(p),
            Err(e) if first_err.is_none() => first_err = Some(e),
            Err(_) => {}
        }
    }

    if synced.is_empty()
        && let Some(err) = first_err
    {
        return Err(err);
    }
    Ok(synced)
}

pub fn install_all() -> IntegrationReport {
    let cfg = crate::config::Config::load();
    install_all_with_config(&cfg.context_menu.enabled_actions)
}

pub fn install_all_with_config(enabled_actions: &[String]) -> IntegrationReport {
    let mut report = IntegrationReport {
        affected: Vec::new(),
        failed: Vec::new(),
    };

    let mut managers = detect_installed_file_managers();
    if managers.is_empty() {
        managers.push(FileManager::Dolphin);
    }

    for fm in managers {
        match fm {
            FileManager::Dolphin => {
                match super::dolphin::install_dolphin_service_with_options(
                    enabled_actions,
                    None,
                    None,
                ) {
                    Ok(p) => report
                        .affected
                        .push(format!("KDE Dolphin ({})", p.display())),
                    Err(e) => report
                        .failed
                        .push(("KDE Dolphin".to_string(), format!("{:#}", e))),
                }
            }
            FileManager::Nautilus => {
                match super::nautilus::install_nautilus_scripts_with_actions(enabled_actions, None)
                {
                    Ok(p) => report
                        .affected
                        .push(format!("GNOME Nautilus ({})", p.display())),
                    Err(e) => report
                        .failed
                        .push(("GNOME Nautilus".to_string(), format!("{:#}", e))),
                }
            }
            FileManager::Nemo => {
                match super::nemo::install_nemo_actions_with_actions(enabled_actions, None, None) {
                    Ok(p) => report
                        .affected
                        .push(format!("Cinnamon Nemo ({})", p.display())),
                    Err(e) => report
                        .failed
                        .push(("Cinnamon Nemo".to_string(), format!("{:#}", e))),
                }
            }
            FileManager::Thunar => {
                match super::thunar::install_thunar_actions_with_actions(enabled_actions, None) {
                    Ok(p) => report
                        .affected
                        .push(format!("XFCE Thunar ({})", p.display())),
                    Err(e) => report
                        .failed
                        .push(("XFCE Thunar".to_string(), format!("{:#}", e))),
                }
            }
        }
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

    match super::entry::uninstall_desktop_entry() {
        Ok(true) => report.affected.push("Masaüstü kısayolu".to_string()),
        Ok(false) => {}
        Err(e) => report
            .failed
            .push(("Masaüstü kısayolu".to_string(), format!("{:#}", e))),
    }

    report
}
