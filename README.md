<div align="center">

  <a href="https://enesehs.dev/konvert">
    <img src="assets/hero.png" alt="Konvert Banner" width="100%" />
  </a>

  <br><br>

  **English** • [Türkçe](README_tr.md)

  <br>

  [![Website](https://img.shields.io/badge/Website-enesehs.dev%2Fkonvert-007ebf?style=for-the-badge&logo=google-chrome&logoColor=white)](https://enesehs.dev/konvert)
  [![CI](https://img.shields.io/github/actions/workflow/status/enesehs/konvert/build.yml?branch=main&style=for-the-badge&logo=githubactions&logoColor=white&label=CI)](https://github.com/enesehs/konvert/actions/workflows/build.yml)
  [![GitHub Release](https://img.shields.io/badge/Release-v0.4.0-3daee9?style=for-the-badge&logo=github)](https://github.com/enesehs/konvert/releases)
  [![License](https://img.shields.io/badge/License-GPL--3.0--or--later-2ecc71?style=for-the-badge)](https://www.gnu.org/licenses/gpl-3.0.html)

  <br>

  <p align="center">
    <b>Fast, lightweight, 100% offline native right-click file converter & optimizer for Linux and all major desktop file managers.</b>
  </p>

  <p align="center">
    <img src="assets/icons/konvert-globe.svg" width="16" height="16" align="absmiddle" /> <b>Interactive Web Demo & FAQ:</b> <a href="https://enesehs.dev/konvert"><b>enesehs.dev/konvert</b></a>
  </p>

  <p align="center">
    <a href="#quick-start">Quick Start</a> •
    <a href="#why-konvert">Why Konvert?</a> •
    <a href="#benchmarks">Benchmarks</a> •
    <a href="#screenshots">Screenshots</a> •
    <a href="#features">Features</a> •
    <a href="#installation">Installation</a> •
    <a href="#cli-usage">CLI Usage</a> •
    <a href="#architecture">Architecture</a>
  </p>

  <br>

  <a href="https://enesehs.dev/konvert">
    <img src="assets/output.mp4" alt="Konvert Right-Click Conversion Workflow" width="90%" />
  </a>

</div>

---

## Quick Start

Get up and running in seconds:

```bash
# 1. Install via universal script (or build from source)
curl -sSL https://enesehs.dev/konvert/install.sh | bash

# 2. Convert or optimize any file instantly (CLI or right-click in file manager)
konvert -t webp photo.png

# 3. Open the native Qt6 & Breeze settings panel
konvert -s
```

---

## Why Konvert?

Tired of uploading private files to online converter sites, switching to browser tabs, or waiting for bloated Electron apps?

* <img src="assets/icons/konvert-shield.svg" width="16" height="16" align="absmiddle" /> **Zero Telemetry & 100% Privacy:** Every single byte stays on your local hardware; zero network requests or background telemetry.
* <img src="assets/icons/konvert-layers.svg" width="16" height="16" align="absmiddle" /> **Native Right-Click Menus:** Instant right-click integration for **KDE Dolphin**, **Cinnamon Nemo**, **XFCE Thunar**, and **GNOME Nautilus**.
* <img src="assets/icons/konvert-zap.svg" width="16" height="16" align="absmiddle" /> **Rust 2024 & Rayon Parallelism:** Multi-threaded parallel batch execution with zero GC pauses and ~25 MB peak memory footprint.
* <img src="assets/icons/konvert-compress.svg" width="16" height="16" align="absmiddle" /> **Mathematical Target Sizing:** Compress videos and images to strict limits (Discord 8 MB, Email 25 MB) with automated ladder search.
* <img src="assets/icons/konvert-monitor.svg" width="16" height="16" align="absmiddle" /> **Live In-Place Plasma Notifications:** Dynamic animated progress bars and live ETA updates in your desktop notification center.
* <img src="assets/icons/konvert-scaling.svg" width="16" height="16" align="absmiddle" /> **Audit History & Savings:** Track cumulative disk space savings, preview 80×80 thumbnails, and reveal files in one click.

<img src="assets/icons/konvert-globe.svg" width="16" height="16" align="absmiddle" /> *Explore full interactive comparison and feature deep-dives at [enesehs.dev/konvert](https://enesehs.dev/konvert).*

---

## Benchmarks

Tested on AMD Ryzen 7 (8 cores / 16 threads).

### Batch Processing & Speedup (19.4 MB 4K PNG)

| Operation | Mode | Output Size | Disk Saved | Bare Subprocess (Single) | Konvert (Single) | Konvert (Rayon `-j 8` ×50) |
|---|---|---|---|---|---|---|
| **JPEG XL (`.jxl`)** | `--lossless` | 1.5 MB | **-92.2%** | 515 ms (`cjxl`) | 517 ms | **3.8s** (~6.8× vs serial `cjxl`) |
| **WebP (`.webp`)** | `-q 85` | 1.9 MB | **-90.2%** | 840 ms (`magick`) | 843 ms | **5.4s** (~7.8× vs serial `magick`) |
| **JPEG (`.jpg`)** | `-q 90` | 4.1 MB | **-78.8%** | 225 ms (`magick`) | 228 ms | **1.6s** (~7.0× vs serial `magick`) |
| **Unix Pipe (`-`)** | Stdin -> Stdout | Stream | — | — | **37 ms** latency | — |

*Performance Model Breakdown: For single-file conversion, Konvert delegates directly to native underlying tools (`cjxl`, `magick`, `ffmpeg`) with minimal wrapper latency (~2–3 ms POSIX exec overhead). The primary performance advantage emerges in batch processing: instead of executing heavy shell scripts serially (e.g., 50 files taking ~26s in JXL or ~42s in WebP), Konvert dispatches jobs across a Rayon multi-threaded work-stealing pool (`-j 8`), delivering up to 7–8× total batch throughput speedup without spawning shell interpreters.*

*Zero Electron/JVM runtime overhead. Base resident memory (RSS): ~24–28 MB (Rust core process). Note: Peak memory usage during 4K video transcoding or heavy PDF processing is governed by FFmpeg and Ghostscript subprocesses.*

### Quality & Fidelity Assessment (PSNR, SSIM & SNR)

Evaluated with high-fidelity reference media via `benchmark_quality.py`:

#### Image Quality (1080p Reference Frame)

| Format / Profile | Output Size | Disk Saved | Latency | PSNR (dB) | SSIM (Structural Similarity) |
|---|---|---|---|---|---|
| **WebP (`-q 85`)** | 70.6 KB | **-97.3%** | 128.8 ms | 37.67 dB | **0.9985** |
| **JPEG XL (`.jxl`)** | 72.7 KB | **-97.3%** | 140.8 ms | 40.14 dB | **0.9984** |
| **AVIF (`-q 80`)** | 35.4 KB | **-98.7%** | 1106.6 ms | 42.31 dB | **0.9983** |
| **JPEG (`-q 90`)** | 212.2 KB | **-92.0%** | 54.9 ms | 42.67 dB | **0.9989** |
| **WebP (`--lossless`)** | 820.5 KB | **-69.1%** | 49.7 ms | 61.59 dB | **1.0000** |

#### Video, Audio & Document Fidelity

| Target / Preset | Reference | Output Size | Disk Saved | Quality Metric |
|---|---|---|---|---|
| **Discord 8MB Profile** | 1080p MP4 (20.6 MB) | 6.98 MB | **-66.1%** | **40.05 dB PSNR** / **0.9906 SSIM** |
| **MP3 (320 kbps)** | 48 kHz WAV (375 KB) | 158.5 KB | **-57.7%** | **74.18 dB SNR** |
| **MP3 (192 kbps)** | 48 kHz WAV (375 KB) | 95.1 KB | **-74.6%** | **30.46 dB SNR** |
| **FLAC Audio** | 48 kHz WAV (375 KB) | 97.2 KB | **-74.1%** | **Lossless (Exact Match)** |
| **DOCX -> TXT** | `source_doc.docx` | 682 B | — | **100% Word & Character Recall** |
| **UDF (UYAP) -> TXT** | `source_court.udf` | 519 B | — | **100% Word & Character Recall** |

*Reproduce measurements locally anytime with `python3 ~/konvert-test/benchmark_quality.py`.*

---

## Screenshots

<div align="center">
  <sub><b>Desktop Right-Click Context Menu Integration</b></sub><br>
  <a href="assets/screenshots/screenshot_context_menu.png">
    <img src="assets/screenshots/screenshot_context_menu.png" alt="Desktop Context Menu Integration" width="90%" />
  </a>
</div>

<br>

<table align="center" width="100%">
  <tr>
    <td width="33.3%" align="center" valign="top">
      <sub><b>Qt6 & Breeze Settings GUI</b></sub><br>
      <a href="assets/screenshots/screenshot_convert.png">
        <img src="assets/screenshots/screenshot_convert.png" alt="Settings UI" width="100%" />
      </a>
    </td>
    <td width="33.3%" align="center" valign="top">
      <sub><b>Conversion History & Disk Savings</b></sub><br>
      <a href="assets/screenshots/screenshot_history.png">
        <img src="assets/screenshots/screenshot_history.png" alt="History & Previews" width="100%" />
      </a>
    </td>
    <td width="33.3%" align="center" valign="top">
      <sub><b>KDE Developer About Dialog</b></sub><br>
      <a href="assets/screenshots/screenshot_about.png">
        <img src="assets/screenshots/screenshot_about.png" alt="About Dialog" width="100%" />
      </a>
    </td>
  </tr>
</table>

---

## Features

| Feature | Description |
|---|---|
| <img src="assets/icons/konvert-layers.svg" width="16" height="16" align="absmiddle" /> **Multi-Desktop Integration** | Separated context actions for Photos, Videos, Audio, and Documents without MIME-type pollution across Dolphin, Nemo, Thunar, and Nautilus. |
| <img src="assets/icons/konvert-zap.svg" width="16" height="16" align="absmiddle" /> **Rust 2024 Rayon Engine** | Multi-threaded parallel batch execution with configurable thread budgets (`--jobs`), zero GC pauses, and LTO-optimized release binaries. |
| <img src="assets/icons/konvert-compress.svg" width="16" height="16" align="absmiddle" /> **Preset Profiles** | Built-in presets (`web`, `balanced`, `archive`, `lossless`) plus custom profile creation (`--save-profile <name>`) saved in `config.toml`. |
| <img src="assets/icons/konvert-terminal.svg" width="16" height="16" align="absmiddle" /> **Unix Pipeline (`-`)** | Direct stdin/stdout streaming with automatic magic-byte format identification for shell pipelines. |
| <img src="assets/icons/konvert-shield.svg" width="16" height="16" align="absmiddle" /> **Atomic Data Safety** | All file writes use temporary `.part` files with atomic commits, collision avoidance (`_2`, `_3`), and optional `.bak` backups (`--backup`). |
| <img src="assets/icons/konvert-image.svg" width="16" height="16" align="absmiddle" /> **Next-Gen Image Formats** | Native support for JPEG XL (`.jxl`), AVIF, QOI, WebP, HEIC, ICO, BMP, TIFF, and raster-to-SVG embedding via ImageMagick (vector tracing with Potrace on roadmap). |
| <img src="assets/icons/konvert-document.svg" width="16" height="16" align="absmiddle" /> **Document & PDF Tools** | Direct UDF (UYAP) and DOCX to PDF/TXT conversions with structural table and header extraction, plus multi-image PDF merge. |
| <img src="assets/icons/konvert-globe.svg" width="16" height="16" align="absmiddle" /> **Multi-Language (i18n)** | UI and CLI interface localized in 9 languages (`--lang <code>`: EN, TR, DE, FR, ES, IT, RU, ZH, JA; core engine errors undergoing full i18n migration) and 5 in Qt6 Settings GUI. |

---

## Installation

### Method 1: Universal One-Line Installer (Recommended)

Installs the precompiled release binary, icons, shell completions, and file manager service menus:

```bash
curl -sSL https://enesehs.dev/konvert/install.sh | bash
```

> [!TIP]
> **Audit before running:** You can inspect the installer source at [`docs/public/install.sh`](docs/public/install.sh) or verify its checksum:
> ```bash
> curl -sSL https://enesehs.dev/konvert/install.sh -o /tmp/install.sh
> curl -sSL https://enesehs.dev/konvert/install.sh.sha256 -o /tmp/install.sh.sha256
> (cd /tmp && sha256sum -c install.sh.sha256)
> bash /tmp/install.sh
> ```

### Method 2: Debian / Ubuntu (.deb Package)

Download the `.deb` release package from [GitHub Releases](https://github.com/enesehs/konvert/releases) or build it locally:

```bash
# Install package:
sudo dpkg -i packaging/konvert_*.deb
sudo apt-get install -f

# Or build from repository:
./packaging/build-deb.sh
```

### Method 3: Arch Linux / AUR (PKGBUILD)

```bash
cd packaging
makepkg -si
```

### Method 4: Portable AppImage

```bash
# Build standalone AppImage bundle:
./packaging/build-appimage.sh
```

### Method 5: Flatpak (Flathub Manifest)

```bash
flatpak-builder --user --install --force-clean build-dir packaging/io.github.enesehs.konvert.json
```

### Method 6: Local Build from Source

```bash
git clone https://github.com/enesehs/konvert.git
cd konvert
./scripts/install.sh
```

---

## CLI Usage

Konvert provides a clean, predictable CLI interface with full flag validation:

```bash
# ── Preset Profiles & Pipeline ─────────────────────────────────────────
konvert --profile web -t webp photo.png       # Apply web profile (80% quality, 150 DPI)
konvert --profile archive -t jpg photo.png     # Apply archive profile (95% quality, 600 DPI)
konvert --profile lossless -t webp photo.png  # Lossless encoding mode
konvert --list-profiles                       # List available built-in and user profiles
cat photo.png | konvert -t webp - > out.webp  # Stream conversion via stdin/stdout pipe

# ── Smart Target Size Compression ──────────────────────────────────────
konvert -t discord_8mb video.mp4              # Compress video mathematically to fit 8 MB
konvert -t email_25mb video.mp4               # Compress video to fit 25 MB
konvert --target-size 15MB video.mp4          # Custom target size constraint

# ── Image Conversion & Optimization ────────────────────────────────────
konvert -t webp -q 90 photo.png                # WebP with explicit quality setting
konvert -t jxl --lossless photo.png            # Lossless JPEG XL conversion
konvert -t avif photo.png                      # AVIF next-generation conversion
konvert -t svg photo.png                       # Vectorize raster image to SVG contours
konvert -t resize_50 photo.png                 # Downscale resolution by 50%
konvert -t resize_1080p photo.jpg              # Fit within Full HD (1920x1080) bounds
konvert -t strip-metadata photo.jpg            # Strip EXIF, GPS, and sensor metadata

# ── Document & PDF Tools ───────────────────────────────────────────────
konvert --merge-pdf p1.png p2.jpg p3.webp     # Merge multiple images into a single PDF
konvert -t pdf document.udf                    # Convert UDF document directly to PDF
konvert -t txt document.docx                   # Extract formatted text and tables from DOCX

# ── Video & Audio Utilities ────────────────────────────────────────────
konvert -t gif video.mp4                       # High-quality two-pass palette GIF
konvert -t extract-audio video.mp4             # Extract audio stream as MP3
konvert --extract-frames video.mp4             # Extract video frames into folder as PNGs
konvert -t audio-to-video song.mp3             # Generate waveform MP4 video from audio

# ── Batch Processing & Safety Controls ─────────────────────────────────
konvert -t webp -j 4 *.png                    # Parallel batch conversion with 4 CPU threads
konvert -r -t webp ./photos                    # Recursively scan and convert directories
konvert --dry-run -t webp *.jpg                # Preview output paths and collision plan
konvert --backup -f -t png photo.jpg           # Overwrite safely with automatic .bak backup
konvert --check-update                         # Check GitHub releases for software updates
konvert --cancel                               # Safely cancel ongoing background conversion
konvert -s                                     # Open Qt6 Settings GUI
```

---

## Supported Formats

| Category | Input Formats | Target Formats | Engine Capabilities |
|---|---|---|---|
| <img src="assets/icons/konvert-image.svg" width="16" height="16" align="absmiddle" /> **Photos** | PNG, JPG, JPEG, WebP, AVIF, JXL, SVG, BMP, ICO, TIFF, QOI, HEIC | WebP, PNG, JPG, SVG, ICO, HEIC, BMP, AVIF, JXL, TIFF, QOI | EXIF Auto-Orientation, EXIF Stripping, 50% & 1080p Resizing, Multi-Image PDF Merge, SVG Bitmap Embedding |
| <img src="assets/icons/konvert-video.svg" width="16" height="16" align="absmiddle" /> **Videos** | MP4, MKV, WebM, AVI, MOV, FLV, TS, WMV | MP4, MKV, WebM, GIF | Discord 8 MB, Email 25 MB, Custom Target Sizing, Frame Extraction, Audio Extraction, Two-Pass Palette GIF |
| <img src="assets/icons/konvert-audio.svg" width="16" height="16" align="absmiddle" /> **Audio** | MP3, WAV, FLAC, OGG, Opus, AAC, M4A | MP3, WAV, FLAC, OGG, Opus, AAC, M4A | Configurable Bitrates (`128k` to `320k`), Waveform Video (MP4) Generation |
| <img src="assets/icons/konvert-document.svg" width="16" height="16" align="absmiddle" /> **Documents** | UDF (UYAP), DOCX, PDF, TXT | PDF, PNG, JPG, TXT | Multi-DPI Page Rendering (36-1200 DPI), Ghostscript Optimization, DOCX/UDF Table & Text Extraction |

---

## Architecture

```mermaid
flowchart TD
    FM["Desktop File Managers & Shell Pipelines<br><b>(KDE Dolphin • Cinnamon Nemo • XFCE Thunar • GNOME Nautilus)</b>"] --> CORE["<b>Rust 2024 Core Engine (konvert)</b><br>• Rayon Parallel Batch Thread Pool (--jobs)<br>• FFmpeg, ImageMagick & Ghostscript Pipelines<br>• D-Bus In-Place Progress Notifications<br>• Atomic I/O & Backup Safety (.part / .bak)"]
    FM -.->|"konvert -s"| GUI["<b>Qt6 & Breeze Settings GUI</b><br>• Quality, Profile & Format Config<br>• Cumulative History & Disk Savings<br>• Custom Profile Editor"]
    CORE <--> GUI
```

---

## Security & Threat Model

Konvert is engineered around a zero-telemetry, 100% offline security posture. Because conversions delegate to system-level external tools for specific formats, users and system administrators should be aware of the following security considerations:

* **External Engine Delegates:** Complex document and vector conversions delegate to system binaries (`ghostscript` for PostScript/PDF rasterization, `imagemagick` for format negotiation, and `libreoffice` for DOCX/UDF rendering). When processing untrusted files from external sources, system administrators should ensure security patches are applied and ImageMagick security policies (`/etc/ImageMagick-7/policy.xml`) restrict vulnerable legacy coders.
* **Input Sanitization & Injection Prevention:** Konvert does not execute commands through a shell interpreter (`sh -c`). All external processes are invoked via direct POSIX `exec` arrays with strictly validated file paths, preventing command injection vulnerabilities.
* **Denial of Service (DoS) Guards:** XML document extraction (UDF, DOCX) enforces a strict 50 MB decompression limit to prevent zip-bomb attacks.
* **System Diagnostics:** Run `konvert --doctor` at any time to verify the status, path, and version of all external delegate binaries.

---

## Contributing

Contributions are welcome and appreciated! To contribute:

1. Fork the repository.
2. Create your feature branch (`git checkout -b feature/amazing-feature`).
3. Commit your changes (`git commit -m 'feat: add amazing feature'`).
4. Ensure tests and lints pass (`cargo test && cargo clippy --all-targets -- -D warnings`).
5. Push to your branch (`git push origin feature/amazing-feature`).
6. Open a **Pull Request**.

---

## License

This project is licensed under the **GNU General Public License v3.0 or later (GPL-3.0-or-later)**. See the [LICENSE](LICENSE) file for details.

---

<div align="center">
  <b>Developer:</b> <a href="https://github.com/enesehs">enesehs</a> &bull;
  <b>Contact:</b> <a href="mailto:enesehs@protonmail.com">enesehs@protonmail.com</a><br>
  <sub>Crafted with dedication for Linux and the Open Source Community.</sub>
</div>
