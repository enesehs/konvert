# Changelog

All notable changes to **Konvert** will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.4.0] - 2026-11-19

### Added
- **Single Source of Truth (SSoT) Architecture:** Single-writer configuration persistence via `konvert --save-config-json -` and `#[serde(flatten)] extra` preserving custom profile sections (`[profiles.*]`) across updates.
- **KDE Plasma Live D-Bus Progress & ETA Notifications:** In-place notification updates with dynamic `"x-kde-progress"` hints, real ETA reporting, and smooth visual feedback without flooding the notification center.
- **True Potrace SVG Vector Tracing:** Integrated `magick -> flatten pbm:- | potrace -s - -o output.svg` vector pipeline with graceful bitmap embed fallback when Potrace is absent.
- **System Diagnostic Doctor CLI (`konvert --doctor`):** Comprehensive system environment diagnostics verifying 10 external tool dependencies with one-click copyable package manager commands (Arch, Debian/Ubuntu, Fedora) and Qt6 GUI integration.
- **KDE Plasma 6 Accent Color Integration:** Dynamic symbolic icon (`konvert-symbolic.svg`) bound to `.ColorScheme-Highlight` and native theme CSS classes.
- **Security & Threat Model Documentation:** Formal threat model covering Ghostscript/ImageMagick delegate CVE risks, direct POSIX exec injection protection, and 50 MB DoS bounds.

### Changed
- **Modular CLI Architecture:** Refactored 1,200+ line monolithic `src/main.rs` into specialized submodules (`src/process.rs`, `src/gui.rs`, `src/update.rs`, `src/dispatcher.rs`).
- **Core Engine i18n Standardization:** Replaced 35+ hardcoded error messages across `src/converter/` with localized `crate::i18n::t(...)` keys.
- **Benchmark Baselines & Speedup Grounding:** Added bare CLI baseline single-file execution metrics (`cjxl`, `magick`, `ffmpeg`) proving Rayon batch throughput gains (~7-8x speedup).
- **Hype-Free Grounded Terminology:** Replaced "ultra-fast" with "fast, lightweight" and unified supported formats count to "35+ format".

### Fixed
- Fixed critical config data loss bug where Qt6 GUI `ConfigManager::save()` wiped out custom profile tables.
- Fixed 7 dead language selector links in documentation headers.
- Fixed `Cargo.toml` metadata to explicitly list all 4 supported desktop file managers.

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
