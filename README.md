<div align="center">

  <a href="https://enesehs.dev/konvert">
    <img src="assets/hero.png" alt="Konvert Banner" width="100%" />
  </a>

  <br><br>

  **English** • [Türkçe](README_tr.md) • [Deutsch](README_de.md) • [Français](README_fr.md) • [Español](README_es.md) • [Italiano](README_it.md) • [Русский](README_ru.md) • [中文](README_zh.md) • [日本語](README_ja.md)

  <br><br>

  <p align="center">
    <b>Right-click, convert. Your files never leave your computer.</b>
  </p>

  <p align="center">
    <a href="https://github.com/enesehs/konvert/releases"><img src="https://img.shields.io/github/v/release/enesehs/konvert?style=flat-square&color=3daee9&label=release" alt="Release" /></a>
    <a href="LICENSE"><img src="https://img.shields.io/badge/license-GPL--3.0-2ecc71.svg?style=flat-square" alt="License" /></a>
    <a href="https://kde.org"><img src="https://img.shields.io/badge/platform-Linux%20%2F%20KDE-1793d1.svg?style=flat-square" alt="Platform" /></a>
    <a href="https://enesehs.dev/konvert"><img src="https://img.shields.io/badge/website-enesehs.dev-blueviolet?style=flat-square" alt="Website" /></a>
  </p>

  <p align="center">
    <a href="assets/screenshots/screenshot_context_menu.png">
      <img src="assets/screenshots/screenshot_context_menu.png" alt="Konvert Context Menu Demo" width="92%" />
    </a>
    <br>
    <sub><i>Select one or hundreds of files in Dolphin, Nemo, Thunar, or Nautilus. Real-time progress bar. Done.</i></sub>
  </p>

</div>

---

## Why Konvert?

Online converters expose your private files to third-party servers and waste bandwidth uploading gigabytes of data. Dedicated desktop apps like Handbrake or Converseen take seconds to launch for a routine format change. Raw `ffmpeg` commands require memorizing dozens of obscure CLI flags.

**Konvert** sits right in the sweet spot:
- **Instant:** Right-click inside your file manager (Dolphin, Nemo, Thunar, Nautilus) and convert in seconds.
- **100% Private & Offline:** Zero telemetry, no cloud accounts, and no network requests. Everything runs locally on your CPU.
- **Blazing Fast:** Written in Rust 2024 with a Rayon thread pool that saturates your CPU cores without locking your desktop.
- **Accurate Target Sizing:** Need an 8 MB Discord clip or a 25 MB email attachment? Konvert calculates the exact bitrate ladder with post-encode verification.

---

## Benchmarks

Tested on a realistic 4K PNG (**18.8 MB**, 3840×2160) on Linux x86_64:

| Target Format / Mode | Time | Output Size | Savings | Notes |
|---|---|---|---|---|
| **JPEG XL (`.jxl`)** | **517 ms** | **1.4 MB** | **92%** | Next-generation compression speed |
| **WebP (`--profile web`)** | **843 ms** | **1.7 MB** | **90%** | Web-optimized 80% quality preset |
| **JPEG (`-q 85`)** | **228 ms** | **3.8 MB** | **79%** | Instant everyday conversion |
| **Target Size (`--target-size 1mb`)** | **2.1 s** | **693 KB** | **96%** | Dynamic ladder search (guaranteed < 1 MB) |
| **Unix Pipe (`cat in \| konvert -`)** | **37 ms** | **30 KB** | **93%** | Zero-disk-wait stdin/stdout streaming |

---

## Quickstart

Install prebuilt binaries, desktop context menus, man pages, and shell completions with a single command:

```bash
curl -sSL https://enesehs.dev/konvert/install.sh | bash
```

<details>
<summary><b>Other installation options (deb, Arch/AUR, AppImage, Flatpak, Source)</b></summary>
<br>

### Debian / Ubuntu (.deb)
```bash
# Download latest release:
sudo dpkg -i konvert_0.3.0_amd64.deb
sudo apt-get install -f

# Or build locally:
./packaging/build-deb.sh
```

### Arch Linux / AUR (PKGBUILD)
```bash
cd packaging && makepkg -si
```

### Standalone AppImage
```bash
./packaging/build-appimage.sh
```

### Flatpak
```bash
flatpak-builder --user --install --force-clean build-dir packaging/io.github.enesehs.konvert.json
```

### Build from Source
```bash
git clone https://github.com/enesehs/konvert.git
cd konvert && ./scripts/install.sh
```
</details>

---

## CLI & Presets

Konvert can be called directly from your terminal or shell scripts:

```bash
# ── Presets & Pipelines ───────────────────────────────────────────────
konvert --profile web -t webp photo.png       # 80% quality, 150 DPI
konvert --profile archive -t jpg photo.png     # 95% quality, 600 DPI
cat photo.png | konvert -t webp - > out.webp  # Stream via stdin/stdout pipe

# ── Smart Target Size ─────────────────────────────────────────────────
konvert -t discord_8mb video.mp4              # Fits the 8 MB Discord upload limit
konvert -t email_25mb video.mp4               # Fits the 25 MB email attachment limit
konvert --target-size 15MB video.mp4          # Arbitrary target size

# ── Batch & Safety Controls ───────────────────────────────────────────
konvert -t webp -j 4 *.png                    # Multi-threaded batch with 4 CPU workers
konvert -r -t webp ./photos                    # Recursively convert entire directories
konvert --dry-run -t webp *.jpg                # Preview output paths without modifying disk
konvert --backup -f -t png photo.jpg           # Safe overwrite with automatic .bak backup
konvert --cancel                               # Stop ongoing background conversion
konvert -s                                     # Open Qt6 Settings GUI
```

---

## Interface Showcase

<table align="center" width="100%">
  <tr>
    <td width="33.3%" align="center" valign="top">
      <sub><b>Qt6 Settings Panel</b></sub><br>
      <a href="assets/screenshots/screenshot_convert.png">
        <img src="assets/screenshots/screenshot_convert.png" alt="Settings UI" width="100%" />
      </a>
    </td>
    <td width="33.3%" align="center" valign="top">
      <sub><b>Disk Savings & History</b></sub><br>
      <a href="assets/screenshots/screenshot_history.png">
        <img src="assets/screenshots/screenshot_history.png" alt="History & Previews" width="100%" />
      </a>
    </td>
    <td width="33.3%" align="center" valign="top">
      <sub><b>KDE Breeze About</b></sub><br>
      <a href="assets/screenshots/screenshot_about.png">
        <img src="assets/screenshots/screenshot_about.png" alt="About Dialog" width="100%" />
      </a>
    </td>
  </tr>
</table>

---

## Supported Formats

<details open>
<summary><b>Click to expand full media & document format table</b></summary>
<br>

| Category | Input Formats | Target Formats | Capabilities |
|---|---|---|---|
| **Photos** | PNG, JPG, JPEG, WebP, AVIF, JXL, SVG, BMP, ICO, TIFF, QOI, HEIC | WebP, PNG, JPG, SVG, ICO, HEIC, BMP, AVIF, JXL, TIFF, QOI | EXIF auto-orientation, EXIF stripping, 50% & 1080p scaling, multi-image PDF merge, potrace tracing |
| **Videos** | MP4, MKV, WebM, AVI, MOV, FLV, TS, WMV | MP4, MKV, WebM, GIF | Discord 8 MB, Email 25 MB, frame extraction, audio extraction, two-pass palette GIF |
| **Audio** | MP3, WAV, FLAC, OGG, Opus, AAC, M4A | MP3, WAV, FLAC, OGG, Opus, AAC, M4A | Bitrate tuning (`128k` to `320k`), waveform video generation |
| **Documents** | UDF (UYAP), DOCX, PDF, TXT | PDF, PNG, JPG, TXT | Multi-DPI rendering (36-1200 DPI), Ghostscript engine, DOCX/UDF table & text extraction |

</details>

---

## Contributing

Contributions are welcome!

1. Fork the repository.
2. Create your branch (`git checkout -b feature/awesome-feature`).
3. Verify test suite passes (`cargo test && cargo clippy --all-targets -- -D warnings`).
4. Commit your changes and open a **Pull Request**.

---

<div align="center">

  **Translations:**
  **English** • [Türkçe](README_tr.md) • [Deutsch](README_de.md) • [Français](README_fr.md) • [Español](README_es.md) • [Italiano](README_it.md) • [Русский](README_ru.md) • [中文](README_zh.md) • [日本語](README_ja.md)

  <br><br>

  <sub>Licensed under <a href="LICENSE">GPL-3.0-or-later</a> &bull; Developed by <a href="https://github.com/enesehs">enesehs</a></sub>

</div>
