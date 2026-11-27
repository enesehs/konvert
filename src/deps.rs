//! Harici araç bağımlılıklarının varlık kontrolü.

use std::ffi::OsStr;
use std::path::PathBuf;

/// PATH içinde çalıştırılabilir bir komut var mı?
pub fn command_exists(name: &str) -> bool {
    let Some(path_var) = std::env::var_os("PATH") else {
        return false;
    };
    command_exists_in_paths(name, &path_var)
}

fn command_exists_in_paths(name: &str, path_var: &OsStr) -> bool {
    std::env::split_paths(path_var).any(|dir| {
        let candidate: PathBuf = dir.join(name);
        let Ok(meta) = std::fs::metadata(&candidate) else {
            return false;
        };
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            meta.is_file() && (meta.permissions().mode() & 0o111 != 0)
        }
        #[cfg(not(unix))]
        {
            meta.is_file()
        }
    })
}

/// `magick` yoksa `convert` (ImageMagick 6) da kabul edilir.
pub fn imagemagick_available() -> bool {
    command_exists("magick") || command_exists("convert")
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

    #[cfg(unix)]
    #[test]
    fn test_command_exists_requires_executable_bit() {
        use std::os::unix::fs::PermissionsExt;

        let dir = std::env::temp_dir().join(format!("konvert_deps_exec_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let non_exec = dir.join("not_exec_tool");
        std::fs::write(&non_exec, b"#!/bin/sh\n").unwrap();
        std::fs::set_permissions(&non_exec, std::fs::Permissions::from_mode(0o644)).unwrap();
        assert!(!command_exists_in_paths("not_exec_tool", dir.as_os_str()));

        let exec = dir.join("exec_tool");
        std::fs::write(&exec, b"#!/bin/sh\n").unwrap();
        std::fs::set_permissions(&exec, std::fs::Permissions::from_mode(0o755)).unwrap();
        assert!(command_exists_in_paths("exec_tool", dir.as_os_str()));

        let _ = std::fs::remove_dir_all(&dir);
    }
}
