//! Sistem bağımlılık teşhisi ve ortam denetimi (`konvert --doctor`).

use std::process::Command;

#[derive(Debug, Clone, serde::Serialize)]
pub struct ToolCheck {
    pub name: &'static str,
    pub command: &'static str,
    pub required: bool,
    pub installed: bool,
    pub version: Option<String>,
    pub purpose: &'static str,
    pub arch_pkg: &'static str,
    pub deb_pkg: &'static str,
    pub fedora_pkg: &'static str,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct DoctorReport {
    pub os: String,
    pub desktop: String,
    pub config_path: Option<String>,
    pub tools: Vec<ToolCheck>,
    pub all_required_installed: bool,
    pub missing_required: Vec<&'static str>,
    pub missing_optional: Vec<&'static str>,
}

pub const DIAGNOSTIC_TOOLS: &[(&str, &str, bool, &str, &str, &str, &str)] = &[
    (
        "FFmpeg",
        "ffmpeg",
        true,
        "Video/ses kodlama ve transkodlama",
        "ffmpeg",
        "ffmpeg",
        "ffmpeg",
    ),
    (
        "FFprobe",
        "ffprobe",
        true,
        "Medya akış analizi ve süre tespiti",
        "ffmpeg",
        "ffmpeg",
        "ffmpeg",
    ),
    (
        "ImageMagick",
        "magick",
        false,
        "Raster görsel gömme ve PDF birleştirme",
        "imagemagick",
        "imagemagick",
        "ImageMagick",
    ),
    (
        "Ghostscript",
        "gs",
        false,
        "PDF boyut sıkıştırma ve çoklu PDF birleştirme",
        "ghostscript",
        "ghostscript",
        "ghostscript",
    ),
    (
        "Poppler Utils",
        "pdftoppm",
        false,
        "PDF sayfalarını yüksek çözünürlüklü PNG/JPG'ye çıkarma",
        "poppler",
        "poppler-utils",
        "poppler-utils",
    ),
    (
        "LibreOffice",
        "soffice",
        false,
        "DOCX ve UYAP (UDF) belgelerini PDF'e dönüştürme",
        "libreoffice-fresh",
        "libreoffice",
        "libreoffice",
    ),
    (
        "JPEG XL Encoder",
        "cjxl",
        false,
        "Görselleri JPEG XL (.jxl) formatına dönüştürme",
        "libjxl",
        "libjxl-tools",
        "libjxl",
    ),
    (
        "JPEG XL Decoder",
        "djxl",
        false,
        "JPEG XL (.jxl) dosyalarını açma ve işleme",
        "libjxl",
        "libjxl-tools",
        "libjxl",
    ),
    (
        "Oxipng",
        "oxipng",
        false,
        "Kayıpsız PNG optimizasyonu ve sıkıştırma",
        "oxipng",
        "oxipng",
        "oxipng",
    ),
    (
        "Potrace",
        "potrace",
        false,
        "Monokrom çizim ve logoları vektörize etme (SVG)",
        "potrace",
        "potrace",
        "potrace",
    ),
];

fn query_tool_version(cmd: &str) -> Option<String> {
    let out = match cmd {
        "ffmpeg" | "ffprobe" => Command::new(cmd).arg("-version").output().ok(),
        "magick" => Command::new("magick")
            .arg("-version")
            .output()
            .or_else(|_| Command::new("convert").arg("-version").output())
            .ok(),
        "gs" => Command::new("gs").arg("--version").output().ok(),
        "pdftoppm" => Command::new("pdftoppm").arg("-v").output().ok(),
        "soffice" => Command::new("soffice").arg("--version").output().ok(),
        "cjxl" => Command::new("cjxl").arg("--version").output().ok(),
        "djxl" => Command::new("djxl").arg("--version").output().ok(),
        "oxipng" => Command::new("oxipng").arg("--version").output().ok(),
        "potrace" => Command::new("potrace").arg("--version").output().ok(),
        _ => Command::new(cmd).arg("--version").output().ok(),
    };

    let output = out?;
    let raw = if !output.stdout.is_empty() {
        String::from_utf8_lossy(&output.stdout).to_string()
    } else {
        String::from_utf8_lossy(&output.stderr).to_string()
    };

    let first_line = raw.lines().next().unwrap_or("").trim();
    if first_line.is_empty() {
        None
    } else {
        Some(first_line.to_string())
    }
}

pub fn diagnose() -> DoctorReport {
    let desktop = std::env::var("XDG_CURRENT_DESKTOP")
        .unwrap_or_else(|_| "Bilinmeyen Masaüstü Ortamı".to_string());
    let os = std::fs::read_to_string("/etc/os-release")
        .ok()
        .and_then(|content| {
            for line in content.lines() {
                if let Some(val) = line.strip_prefix("PRETTY_NAME=") {
                    return Some(val.trim_matches('"').to_string());
                }
            }
            None
        })
        .unwrap_or_else(|| "Linux".to_string());

    let config_path = crate::config::Config::config_path().map(|p| p.display().to_string());

    let mut tools = Vec::new();
    let mut missing_required = Vec::new();
    let mut missing_optional = Vec::new();

    for &(display_name, cmd, req, purpose, arch, deb, fedora) in DIAGNOSTIC_TOOLS {
        let is_installed = if cmd == "magick" {
            crate::deps::imagemagick_available()
        } else {
            crate::deps::command_exists(cmd)
        };

        let version = if is_installed {
            query_tool_version(cmd)
        } else {
            None
        };

        if !is_installed {
            if req {
                missing_required.push(cmd);
            } else {
                missing_optional.push(cmd);
            }
        }

        tools.push(ToolCheck {
            name: display_name,
            command: cmd,
            required: req,
            installed: is_installed,
            version,
            purpose,
            arch_pkg: arch,
            deb_pkg: deb,
            fedora_pkg: fedora,
        });
    }

    let all_required_installed = missing_required.is_empty();

    DoctorReport {
        os,
        desktop,
        config_path,
        tools,
        all_required_installed,
        missing_required,
        missing_optional,
    }
}

pub fn print_doctor_report(report: &DoctorReport) {
    println!("=== Konvert Sistem Teşhisi (--doctor) ===");
    println!("İşletim Sistemi:    {}", report.os);
    println!("Masaüstü Ortamı:    {}", report.desktop);
    if let Some(cfg) = &report.config_path {
        println!("Ayar Dosyası:       {}", cfg);
    }
    println!();

    println!(
        "{:<20} {:<12} {:<10} AÇIKLAMA / SÜRÜM",
        "BİLEŞEN", "DURUM", "TÜR"
    );
    println!("{:-<20} {:-<12} {:-<10} {:-<40}", "", "", "", "");

    for tool in &report.tools {
        let status_str = if tool.installed {
            "[PASS]"
        } else if tool.required {
            "[FAIL]"
        } else {
            "[OPTIONAL]"
        };

        let role_str = if tool.required {
            "Zorunlu"
        } else {
            "İsteğe Bağlı"
        };

        let detail_str = if let Some(v) = &tool.version {
            v.as_str()
        } else {
            tool.purpose
        };

        println!(
            "{:<20} {:<12} {:<10} {}",
            tool.name, status_str, role_str, detail_str
        );
    }

    println!();

    if report.all_required_installed && report.missing_optional.is_empty() {
        println!(
            "[TAMAM] Tüm çekirdek ve isteğe bağlı araçlar kurulu. Konvert tüm özellikleriyle hazır."
        );
    } else if report.all_required_installed {
        println!("[UYARI] Temel dönüşümler çalışabilir, ancak bazı isteğe bağlı bileşenler eksik.");
    } else {
        println!("[HATA] Zorunlu bağımlılıklar eksik! Video ve ses dönüşümleri çalışmayacaktır.");
    }

    let all_missing: Vec<&ToolCheck> = report.tools.iter().filter(|t| !t.installed).collect();
    if !all_missing.is_empty() {
        println!();
        println!("Eksik paketleri kurmak için komutlar:");

        let arch_pkgs: Vec<&str> = all_missing.iter().map(|t| t.arch_pkg).collect();
        let deb_pkgs: Vec<&str> = all_missing.iter().map(|t| t.deb_pkg).collect();
        let fedora_pkgs: Vec<&str> = all_missing.iter().map(|t| t.fedora_pkg).collect();

        println!("  * Arch Linux:    sudo pacman -S {}", arch_pkgs.join(" "));
        println!("  * Debian/Ubuntu: sudo apt install {}", deb_pkgs.join(" "));
        println!(
            "  * Fedora:        sudo dnf install {}",
            fedora_pkgs.join(" ")
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnose_produces_report() {
        let report = diagnose();
        assert!(!report.tools.is_empty());
        assert_eq!(report.tools.len(), DIAGNOSTIC_TOOLS.len());
    }

    #[test]
    fn test_query_tool_version_nonexistent() {
        assert!(query_tool_version("non_existent_tool_12345").is_none());
    }
}
