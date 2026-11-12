<div align="center">

  <a href="https://enesehs.dev/konvert">
    <img src="assets/hero.png" alt="Konvert Banner" width="100%" />
  </a>

  <br><br>

  [English](README.md) • **Türkçe** • [Deutsch](README_de.md) • [Français](README_fr.md) • [Español](README_es.md) • [Italiano](README_it.md) • [Русский](README_ru.md) • [中文](README_zh.md) • [日本語](README_ja.md)

  <br><br>

  <p align="center">
    <b>Sağ tıkla, çevir. Dosyaların bilgisayarından hiç çıkmasın.</b>
  </p>

  <p align="center">
    <a href="https://github.com/enesehs/konvert/releases"><img src="https://img.shields.io/github/v/release/enesehs/konvert?style=flat-square&color=3daee9&label=sürüm" alt="Sürüm" /></a>
    <a href="LICENSE"><img src="https://img.shields.io/badge/lisans-GPL--3.0-2ecc71.svg?style=flat-square" alt="Lisans" /></a>
    <a href="https://kde.org"><img src="https://img.shields.io/badge/platform-Linux%20%2F%20KDE-1793d1.svg?style=flat-square" alt="Platform" /></a>
    <a href="https://enesehs.dev/konvert"><img src="https://img.shields.io/badge/web%20sitesi-enesehs.dev-blueviolet?style=flat-square" alt="Web Sitesi" /></a>
  </p>

  <p align="center">
    <a href="assets/screenshots/screenshot_context_menu.png">
      <img src="assets/screenshots/screenshot_context_menu.png" alt="Konvert Bağlam Menüsü Gösterimi" width="92%" />
    </a>
    <br>
    <sub><i>Dolphin, Nemo, Thunar veya Nautilus üzerinde tek ya da yüzlerce dosya seçin. Gerçek zamanlı ilerleme çubuğu. Bitti.</i></sub>
  </p>

</div>

---

## Neden Konvert?

Çevrim içi dönüştürücüler kişisel ve gizli dosyalarınızı üçüncü taraf sunuculara maruz bırakır, gigabaytlarca dosya yüklerken kotanızı ve vaktinizi tüketir. Handbrake veya Converseen gibi büyük masaüstü uygulamalarının ise sıradan bir format değişimi için açılması saniyeler sürer. Doğrudan `ffmpeg` komutları ise onlarca karmaşık terminal parametresini ezberlemeyi zorunlu kılar.

**Konvert** tam bu noktada devreye girer:
- **Anında:** Dosya yöneticinizde (Dolphin, Nemo, Thunar, Nautilus) sağ tıklayın ve saniyeler içinde dönüştürün.
- **%100 Gizli & Çevrimdışı:** Sıfır telemetri, bulut hesabı yok, internet bağımlılığı yok. Her şey yerel işlemcinizde (CPU) koşar.
- **Ultra Hızlı:** Rayon iş parçacığı havuzuyla masaüstünüzü kilitlemeden tüm işlemci çekirdeklerinizi sonuna kadar kullanan Rust 2024 motoru.
- **Hassas Hedef Boyutlandırma:** 8 MB Discord klibi veya 25 MB e-posta eki mi lazım? Konvert dinamik bit hızı merdiveni ve kodlama sonrası doğrulama ile boyutu garantiler.

---

## Performans Testleri

Linux x86_64 üzerinde gerçekçi bir 4K PNG (**18.8 MB**, 3840×2160) ile test edilmiştir:

| Hedef Format / Mod | Süre | Çıktı Boyutu | Tasarruf | Notlar |
|---|---|---|---|---|
| **JPEG XL (`.jxl`)** | **517 ms** | **1.4 MB** | **%92** | Yeni nesil süper hızlı sıkıştırma |
| **WebP (`--profile web`)** | **843 ms** | **1.7 MB** | **%90** | Web için optimize edilmiş %80 kalite |
| **JPEG (`-q 85`)** | **228 ms** | **3.8 MB** | **%79** | Anında günlük hızlı dönüşüm |
| **Hedef Boyut (`--target-size 1mb`)** | **2.1 sn** | **693 KB** | **%96** | Dinamik merdiven arama (garantili < 1 MB) |
| **Unix Boru Hattı (`cat in \| konvert -`)** | **37 ms** | **30 KB** | **%93** | Disksiz stdin/stdout anlık veri akışı |

---

## Hızlı Kurulum

Önceden derlenmiş ikili dosyalar, masaüstü sağ tık menüleri, man sayfası ve kabuk tamamlama betiklerini tek satırda kurun:

```bash
curl -sSL https://enesehs.dev/konvert/install.sh | bash
```

<details>
<summary><b>Diğer kurulum seçenekleri (.deb, Arch/AUR, AppImage, Flatpak, Kaynaktan Derleme)</b></summary>
<br>

### Debian / Ubuntu (.deb)
```bash
# Son sürümü kurun:
sudo dpkg -i konvert_0.3.0_amd64.deb
sudo apt-get install -f

# Veya yerel olarak paketleyin:
./packaging/build-deb.sh
```

### Arch Linux / AUR (PKGBUILD)
```bash
cd packaging && makepkg -si
```

### Taşınabilir AppImage
```bash
./packaging/build-appimage.sh
```

### Flatpak
```bash
flatpak-builder --user --install --force-clean build-dir packaging/io.github.enesehs.konvert.json
```

### Kaynaktan Derleme
```bash
git clone https://github.com/enesehs/konvert.git
cd konvert && ./scripts/install.sh
```
</details>

---

## CLI ve Profiller

Konvert doğrudan terminalden veya kabuk (shell) betiklerinden çağrılabilir:

```bash
# ── Profiller & Boru Hatları ──────────────────────────────────────────
konvert --profile web -t webp photo.png       # %80 kalite, 150 DPI web modu
konvert --profile archive -t jpg photo.png     # %95 kalite, 600 DPI arşiv modu
cat photo.png | konvert -t webp - > out.webp  # stdin/stdout boru hattı üzerinden aktar

# ── Akıllı Hedef Boyutlandırma ─────────────────────────────────────────
konvert -t discord_8mb video.mp4              # 8 MB Discord yükleme limitine sığdır
konvert -t email_25mb video.mp4               # 25 MB e-posta ek limitine sığdır
konvert --target-size 15MB video.mp4          # İstenilen özel boyuta sığdır

# ── Toplu İşlem & Güvenlik Kontrolleri ─────────────────────────────────
konvert -t webp -j 4 *.png                    # 4 CPU iş parçacığıyla paralel toplu dönüşüm
konvert -r -t webp ./fotograflar               # Klasörü alt dizinleriyle özyinelemeli tara
konvert --dry-run -t webp *.jpg                # Dosyaları değiştirmeden çıktıları önizle
konvert --backup -f -t png photo.jpg           # Üzerine yazmadan önce otomatik .bak yedeği al
konvert --cancel                               # Çalışan arka plan işlemini iptal et
konvert -s                                     # Qt6 Ayarlar Arayüzünü aç
```

---

## Arayüz Vitrini

<table align="center" width="100%">
  <tr>
    <td width="33.3%" align="center" valign="top">
      <sub><b>Qt6 & Breeze Ayarlar Paneli</b></sub><br>
      <a href="assets/screenshots/screenshot_convert.png">
        <img src="assets/screenshots/screenshot_convert.png" alt="Ayarlar Arayüzü" width="100%" />
      </a>
    </td>
    <td width="33.3%" align="center" valign="top">
      <sub><b>Disk Tasarrufu ve Geçmiş</b></sub><br>
      <a href="assets/screenshots/screenshot_history.png">
        <img src="assets/screenshots/screenshot_history.png" alt="Geçmiş Paneli" width="100%" />
      </a>
    </td>
    <td width="33.3%" align="center" valign="top">
      <sub><b>KDE Breeze Hakkında Paneli</b></sub><br>
      <a href="assets/screenshots/screenshot_about.png">
        <img src="assets/screenshots/screenshot_about.png" alt="Hakkında Paneli" width="100%" />
      </a>
    </td>
  </tr>
</table>

---

## Desteklenen Formatlar

<details open>
<summary><b>Tüm ortam ve belge format tablosunu görüntülemek için tıklayın</b></summary>
<br>

| Kategori | Girdi Formatları | Hedef Formatlar | Yetenekler |
|---|---|---|---|
| **Fotoğraflar** | PNG, JPG, JPEG, WebP, AVIF, JXL, SVG, BMP, ICO, TIFF, QOI, HEIC | WebP, PNG, JPG, SVG, ICO, HEIC, BMP, AVIF, JXL, TIFF, QOI | Otomatik EXIF yönü, EXIF temizleme, %50 ve 1080p ölçekleme, çoklu görselden PDF birleştirme, potrace vektör çizimi |
| **Videolar** | MP4, MKV, WebM, AVI, MOV, FLV, TS, WMV | MP4, MKV, WebM, GIF | Discord 8 MB, E-posta 25 MB, kare çıkarma, ses ayıklama, iki aşamalı palet GIF |
| **Ses** | MP3, WAV, FLAC, OGG, Opus, AAC, M4A | MP3, WAV, FLAC, OGG, Opus, AAC, M4A | Ayarlanabilir bit hızı (`128k` - `320k`), dalga formu video üretimi |
| **Belgeler** | UDF (UYAP), DOCX, PDF, TXT | PDF, PNG, JPG, TXT | Çoklu DPI işleme (36-1200 DPI), Ghostscript motoru, DOCX/UDF tablo ve metin çıkarma |

</details>

---

## Katkıda Bulunma

Katkılarınızı memnuniyetle kabul ediyoruz!

1. Depoyu forklayın.
2. Özellik dalınızı oluşturun (`git checkout -b feature/harika-ozellik`).
3. Testlerin geçtiğini doğrulayın (`cargo test && cargo clippy --all-targets -- -D warnings`).
4. Değişikliklerinizi commit'leyin ve bir **Pull Request** açın.

---

<div align="center">

  **Diller:**
  [English](README.md) • **Türkçe** • [Deutsch](README_de.md) • [Français](README_fr.md) • [Español](README_es.md) • [Italiano](README_it.md) • [Русский](README_ru.md) • [中文](README_zh.md) • [日本語](README_ja.md)

  <br><br>

  <sub><a href="LICENSE">GPL-3.0-or-later</a> ile lisanslanmıştır &bull; Geliştirici: <a href="https://github.com/enesehs">enesehs</a></sub>

</div>
