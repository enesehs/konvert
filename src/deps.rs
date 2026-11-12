//! Harici araç bağımlılıklarının varlık kontrolü.

use std::path::PathBuf;

pub struct Dependency {
    pub tool: &'static str,
    pub required: bool,
    pub purpose: &'static str,
}

pub const DEPENDENCIES: &[Dependency] = &[
    Dependency {
        tool: "ffmpeg",
        required: true,
        purpose: "video/ses dönüştürme",
    },
    Dependency {
        tool: "ffprobe",
        required: true,
        purpose: "video süresi/boyut hesabı",
    },
    Dependency {
        tool: "magick",
        required: false,
        purpose: "SVG/HEIC/PDF birleştirme (convert de olur)",
    },
    Dependency {
        tool: "gs",
        required: false,
        purpose: "PDF sıkıştırma",
    },
    Dependency {
        tool: "pdftoppm",
        required: false,
        purpose: "PDF → PNG/JPG",
    },
    Dependency {
        tool: "soffice",
        required: false,
        purpose: "DOCX/UDF → PDF",
    },
    Dependency {
        tool: "cjxl",
        required: false,
        purpose: "JXL'ye dönüştürme",
    },
    Dependency {
        tool: "djxl",
        required: false,
        purpose: "JXL okuma",
    },
];

/// PATH içinde çalıştırılabilir bir komut var mı?
pub fn command_exists(name: &str) -> bool {
    let Some(path_var) = std::env::var_os("PATH") else {
        return false;
    };
    std::env::split_paths(&path_var).any(|dir| {
        let candidate: PathBuf = dir.join(name);
        std::fs::metadata(&candidate)
            .map(|meta| meta.is_file())
            .unwrap_or(false)
    })
}

/// `magick` yoksa `convert` (ImageMagick 6) da kabul edilir.
pub fn imagemagick_available() -> bool {
    command_exists("magick") || command_exists("convert")
}

pub fn check() -> Vec<(String, bool, bool, String)> {
    DEPENDENCIES
        .iter()
        .map(|dep| {
            let found = if dep.tool == "magick" {
                imagemagick_available()
            } else {
                command_exists(dep.tool)
            };
            (
                dep.tool.to_string(),
                found,
                dep.required,
                dep.purpose.to_string(),
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_known_command_exists() {
        // `sh` neredeyse her Linux sisteminde PATH'te bulunur.
        assert!(command_exists("sh"));
        assert!(!command_exists("konvert-olmayan-bir-arac"));
    }

    #[test]
    fn test_check_covers_all_dependencies() {
        let report = check();
        assert_eq!(report.len(), DEPENDENCIES.len());
    }
}
