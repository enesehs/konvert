<div align="center">

  <a href="https://enesehs.dev/konvert">
    <img src="assets/hero.png" alt="Konvert Banner" width="100%" />
  </a>

  <br>
  <br>

  **English** • [Türkçe](README_tr.md)

  <br>
  <br>

  [![Website](https://img.shields.io/badge/Website-enesehs.dev%2Fkonvert-007ebf?style=for-the-badge&logo=google-chrome&logoColor=white)](https://enesehs.dev/konvert)
  [![GitHub Release](https://img.shields.io/badge/Release-v0.1--beta-3daee9?style=for-the-badge&logo=github)](https://github.com/enesehs/konvert/releases)
  [![License](https://img.shields.io/badge/License-GPL--3.0--or--later-2ecc71?style=for-the-badge)](https://www.gnu.org/licenses/gpl-3.0.html)
  [![Rust](https://img.shields.io/badge/Rust-2024%20Edition-e05d44?style=for-the-badge&logo=rust)](https://www.rust-lang.org/)
  [![Qt 6](https://img.shields.io/badge/Qt-6%20Widgets%20(Breeze)-41CD52?style=for-the-badge&logo=qt)](https://www.qt.io/)
  [![Platform](https://img.shields.io/badge/Platform-Linux%20%2F%20KDE%20Plasma%206-1793d1?style=for-the-badge&logo=kde)](https://kde.org/)
  [![Privacy](https://img.shields.io/badge/Privacy-100%25%20Offline%20%26%20Zero%20Telemetry-7158e2?style=for-the-badge)](https://github.com/enesehs/konvert)

  <p align="center">
    <b>Ultra-fast, lightweight, 100% offline and native right-click file converter & optimizer for Linux and KDE Plasma 6.</b>
  </p>

  <p align="center">
    🚀 <b>Live Interactive Demo & Web: <a href="https://enesehs.dev/konvert">enesehs.dev/konvert</a></b>
  </p>

  <p align="center">
    <a href="#-why-konvert">Why Konvert?</a> •
    <a href="#-screenshots">Screenshots</a> •
    <a href="#-features">Features</a> •
    <a href="#-installation">Installation</a> •
    <a href="#-cli-usage--smart-compression">CLI Usage</a> •
    <a href="#-supported-formats">Supported Formats</a> •
    <a href="#-architecture">Architecture</a>
  </p>

</div>

---

## 🌟 Why Konvert?

Tired of uploading confidential files to random online converter websites, opening browser tabs for routine conversions, or waiting for bloated desktop applications to start?

**Konvert** is built from the ground up natively for modern Linux desktops and specifically **KDE Plasma 6 (Dolphin)**:
* **Zero Telemetry & 100% Privacy:** Your files never leave your computer. No network connection required; everything is processed locally across multiple CPU cores.
* **Seamless Dolphin Context Menu (KIO ServiceMenus):** Select one or hundreds of files in Dolphin, right-click, and convert in seconds without opening any heavy app.
* **Live In-Place Plasma Notifications:** Track single or batch conversion progress with an animated progress bar inside the native KDE notification center.
* **Target Size Profiles:** Compress videos and images to strict file size limits (Discord 8 MB, Email 25 MB) with intelligent mathematical bitrate calculation.
* **History with Thumbnail Previews:** Review conversion savings, inspect 80x80 thumbnail previews, open output files, or reveal them in Dolphin with one click.

---

## 📸 Screenshots

<div align="center">

### Dolphin Right-Click Context Menu Integration
*Right-click any image, video, audio, or document; convert instantly from the Konvert submenu*

<img src="assets/screenshots/screenshot_context_menu.png" alt="Dolphin Context Menu Integration" width="92%" />

<br><br>

### Modern Qt6 & KDE Breeze Settings GUI
*Manage context menus by category, configure conversion qualities, and tune encoding presets*

<img src="assets/screenshots/screenshot_convert.png" alt="Konvert Settings UI" width="90%" />

<br><br>

### Conversion History & File Preview Panel
*Total disk savings tracker, thumbnail previews, and quick file management actions*

<img src="assets/screenshots/screenshot_history.png" alt="Konvert History & Previews" width="90%" />

<br><br>

### Authentic KDE Developer "About" Dialog
*Dedicated to the open source community, featuring 4 standard KDE tabs and interactive language selection*

<img src="assets/screenshots/screenshot_about.png" alt="Konvert About Dialog" width="90%" />

</div>

---

## ✨ Features

| Feature | Description |
|---|---|
| **Dolphin Right-Click Integration** | Separated KIO ServiceMenus for Photos, Videos, Music, and Documents without MIME-type pollution. |
| **Rust 2024 Core Engine** | Multi-threaded parallel batch conversions using `Rayon`; zero GC pauses and ultra-low memory usage. |
| **Smart Target Sizing** | Automatic bitrate math for `discord_8mb`, `email_25mb`, and `web_opt` profiles. |
| **In-Place D-Bus Progress Notifications** | Updates progress dynamically in a single notification bubble without notification spam. |
| **Advanced Document & Media Utilities** | Direct UDF (UYAP) to PDF conversion, multi-image merge to PDF, EXIF stripping, and audio extraction. |
| **Multi-Language (i18n) Support** | Auto-detects desktop locale; supports English, Turkish, German, French, and Spanish. |

---

## 🚀 Installation

### Method 1: Universal One-Line Installer (Recommended)

Install the latest precompiled release, icons, and Dolphin service menus with a single command:

```bash
# Via enesehs.dev:
curl -sSL https://enesehs.dev/konvert/install.sh | bash

# Or via GitHub:
curl -sSL https://raw.githubusercontent.com/enesehs/konvert/main/docs/public/install.sh | bash
```

### Method 2: Local Build from Source

```bash
git clone https://github.com/enesehs/konvert.git
cd konvert
./scripts/install.sh
```

### Method 3: Debian / Ubuntu (.deb Package)

Download the latest `.deb` package from [GitHub Releases](https://github.com/enesehs/konvert/releases) or build it locally:

```bash
# Install package:
sudo dpkg -i packaging/konvert_*.deb
sudo apt-get install -f

# Or build the package locally:
./packaging/build-deb.sh
```

### Method 4: Arch Linux / AUR

```bash
cd packaging
makepkg -si
```

### Method 5: Flatpak (Flathub Manifest)

Konvert provides a ready-to-build manifest at `packaging/io.github.enesehs.konvert.json`:

```bash
flatpak-builder --user --install --force-clean build-dir packaging/io.github.enesehs.konvert.json
```

---

## 💻 CLI Usage & Smart Compression

Konvert can be called directly from the command line as well as Dolphin:

```bash
# 🎯 Smart Target Size Compression
konvert -t discord_8mb video.mp4     # Compress video mathematically to fit the 8 MB Discord limit
konvert -t email_25mb video.mp4       # Compress video to fit the 25 MB email attachment limit
konvert --target-size 15MB video.mp4  # Specify custom target size (15 MB)

# 🌐 One-Click Web Optimization
konvert -t web_opt photo.png          # 82% quality WebP + stripped EXIF metadata

# 🖼️ Image Conversion & Quality Control
konvert -t webp -q 90 photo.png       # WebP conversion with explicit quality setting
konvert -t jxl --lossless photo.png   # Next-generation lossless JPEG XL conversion
konvert -t resize_50 photo.png        # Downscale resolution by 50%
konvert -t resize_1080p photo.jpg     # Fit within Full HD (1920x1080) bounds
konvert -t strip-metadata photo.jpg   # Strip EXIF, GPS, and camera sensor metadata

# 📑 Document & PDF Tools
konvert --merge-pdf p1.png p2.jpg p3.webp  # Merge multiple images into a single PDF
konvert -t pdf document.udf           # Convert UDF document directly to PDF
konvert -t txt document.udf           # Extract text content from UDF file

# 🎬 Video & Audio Tools
konvert -t gif video.mp4              # High-quality two-pass palette GIF generation
konvert -t extract-audio video.mp4    # Extract audio stream as MP3
konvert --extract-frames video.mp4    # Extract video frames into a folder as PNGs
konvert -t audio-to-video song.mp3    # Generate waveform MP4 video from audio

# ⚡ Batch Processing & Cancellation
konvert -t webp *.png                 # Convert all PNGs in parallel using all CPU cores
konvert --cancel                      # Safely cancel ongoing background conversion
konvert -s                            # Open Qt6 Settings GUI
```

---

## 📊 Supported Formats

| Category | Input Formats | Target Formats | Special Features |
|---|---|---|---|
| **Photos** | PNG, JPG, JPEG, WebP, AVIF, JXL, SVG, BMP, ICO, TIFF, QOI | WebP, PNG, JPG, AVIF, JXL, BMP, ICO, TIFF, QOI | EXIF Stripping, 50% & 1080p Resizing, Multi-Image PDF Merge |
| **Videos** | MP4, MKV, WebM, AVI, MOV, FLV, TS, WMV | MP4, MKV, WebM, GIF | Discord 8 MB, Email 25 MB, Frame Extraction, Audio Extraction |
| **Music** | MP3, WAV, FLAC, OGG, Opus, AAC, M4A | MP3, WAV, FLAC, OGG, Opus, AAC, M4A | Waveform Video (MP4) Generation |
| **Documents** | UDF (UYAP), DOCX, PDF, TXT | PDF, PNG, JPG, TXT | Multi-DPI Page Rendering, UDF to PDF/Text |

---

## 🏗️ Architecture

Konvert uses a modular, multi-tier architecture built for native desktop performance:

```
┌─────────────────────────────────────────────────────────────┐
│                    KDE Plasma 6 / Dolphin                   │
│   (KIO ServiceMenus: Photos, Videos, Music, Documents)      │
└──────────────────────────────┬──────────────────────────────┘
                               │
                ┌──────────────┴──────────────┐
                ▼                             ▼
  ┌───────────────────────────┐ ┌───────────────────────────┐
  │   Rust Core CLI (konvert) │ │   Qt6 C++ (konvert-settings)│
  │                           │ │                           │
  │ • Rayon Multi-Threading   │ │ • KDE Breeze Design Language│
  │ • FFmpeg & ImageMagick    │ │ • Authentic KAboutData      │
  │ • Poppler & Ghostscript   │ │ • Multi-Language Support    │
  │ • D-Bus In-Place Progress │ │ • Visual History & Previews │
  └───────────────────────────┘ └───────────────────────────┘
```

---

## 🤝 Contributing

Contributions are welcome and appreciated! To contribute:

1. Fork the repository.
2. Create your feature branch (`git checkout -b feature/amazing-feature`).
3. Commit your changes (`git commit -m 'feat: add amazing feature'`).
4. Push to your branch (`git push origin feature/amazing-feature`).
5. Open a **Pull Request**.

---

## 📄 License

This project is licensed under the **GNU General Public License v3.0 or later (GPL-3.0-or-later)**. See the [LICENSE](LICENSE) file for details.

---

<div align="center">
  <b>Developer:</b> <a href="https://github.com/enesehs">enesehs</a> &bull;
  <b>Contact:</b> <a href="mailto:enesehs@protonmail.com">enesehs@protonmail.com</a><br>
  <sub>Crafted with dedication for Linux and the Open Source Community.</sub>
</div>
