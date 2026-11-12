# Changelog

All notable changes to **Konvert** will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0] - 2026-11-12

### Added
- **Multi-Threaded Rayon Parallel Engine:** Capped parallel batch conversions with `--jobs <N>` and smart CPU core budget.
- **Conversion Presets & Profiles:** Added `--profile <web|balanced|archive|lossless>`, `--list-profiles`, `--save-profile <name>`, and customizable profiles in `config.toml`.
- **Unix stdin/stdout Pipeline (`-`):** Stream conversions directly via pipes with automatic magic-byte format detection (`cat in.png | konvert -t webp - > out.webp`).
- **Accurate Batch Progress & ETA:** Dynamic time estimation (`ETA: Xs` / `Xm Ys`) and smooth single-task progression.
- **AppImage Packaging & Update Checking:** Added `packaging/build-appimage.sh` for portable bundles and `--check-update` for online GitHub release checks.
- **Data Safety Guarantees:** Atomic `.part` file creation, collision prevention (`_2`, `_3`), and optional `.bak` backups before overwriting (`--backup`).
- **Expanded Format Ecosystem:** Native support for JPEG XL (`.jxl`), AVIF, QOI, HEIC, ICO, BMP, TIFF, and contour tracing to SVG via `potrace`.
- **Document Enhancements:** Direct UDF (UYAP) and DOCX to PDF/TXT with structural extraction for tables, headers, footers, and 50MB DoS memory guards.
- **Multi-Desktop Context Menus:** Native integration scripts for KDE Dolphin, Cinnamon Nemo, XFCE Thunar, and GNOME Nautilus with Lucide SVG action icons.
- **Multi-Language (i18n):** Full support for 9 languages in CLI (`--lang <code>`) and 5 in the Qt6 Settings GUI.
- **Shell Completions & Man Page:** Added `docs/konvert.1` and `--completions <bash|zsh|fish>`.
- **Multi-Architecture CI/CD:** GitHub Actions matrix with native `aarch64` and `x86_64` builds, Flatpak validation, and automated PKGBUILD checksum updating.

### Changed
- Refactored `TargetAction::parse` to eliminate substring matching and enforce strict unit parsing.
- Enhanced PDF compression using direct Ghostscript engine with automatic policy.xml diagnostic hints.
- Qt6 GUI modernized: Dedicated Settings tab, output folder selector, live process monitor, and `QFileSystemWatcher` multi-instance sync.
- Removed community dedication frame from About tab for a cleaner, unified layout.

### Fixed
- Fixed photo orientation bug where EXIF tags were lost during native decoding (`decode_oriented`).
- Fixed process cancellation (`--cancel`) to validate PID ownership and terminate child process groups cleanly.
- Fixed installer permission error caused by unbound `TARGET_HOME` variable.
- Fixed Thunar `uca.xml` parser bug that previously destroyed user custom actions upon removal.
- Fixed target-size logic that previously processed files already smaller than the requested limit.

---

## [0.2.0] - 2026-11-11

### Added
- Initial Qt6 C++ Breeze-themed Settings GUI.
- KIO ServiceMenu desktop integration for KDE Plasma 6.
- History recording with local disk savings tracking.
- Ghostscript PDF compression and multi-image PDF merge.
- Video Discord (8 MB) and Email (25 MB) target sizing presets.
