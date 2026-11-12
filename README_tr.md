<div align="center">

  <a href="https://enesehs.dev/konvert">
    <img src="assets/hero.png" alt="Konvert Banner" width="100%" />
  </a>

  <br><br>

  [English](README.md) • **Türkçe**

  <br>

  [![Web Sitesi](https://img.shields.io/badge/Web%20Sitesi-enesehs.dev%2Fkonvert-007ebf?style=for-the-badge&logo=google-chrome&logoColor=white)](https://enesehs.dev/konvert)
  [![GitHub Release](https://img.shields.io/badge/Release-v0.3.0-3daee9?style=for-the-badge&logo=github)](https://github.com/enesehs/konvert/releases)
  [![Lisans](https://img.shields.io/badge/Lisans-GPL--3.0--or--later-2ecc71?style=for-the-badge)](https://www.gnu.org/licenses/gpl-3.0.html)
  [![Rust](https://img.shields.io/badge/Rust-2024%20Edition-e05d44?style=for-the-badge&logo=rust)](https://www.rust-lang.org/)
  [![Qt 6](https://img.shields.io/badge/Qt-6%20Widgets%20(Breeze)-41CD52?style=for-the-badge&logo=qt)](https://www.qt.io/)
  [![Platform](https://img.shields.io/badge/Platform-Linux%20%2F%20KDE%20Plasma%206-1793d1?style=for-the-badge&logo=kde)](https://kde.org/)
  [![Gizlilik](https://img.shields.io/badge/Gizlilik-%25100%20%C3%87evrimd%C4%B1%C5%9F%C4%B1%20%26%20Telemetrisiz-7158e2?style=for-the-badge)](https://github.com/enesehs/konvert)

  <br>

  <p align="center">
    <b>Linux ve tüm büyük masaüstü dosya yöneticileri için ultra hızlı, hafif, %100 çevrimdışı yerel sağ tık dosya dönüştürücüsü ve optimize edicisi.</b>
  </p>

  <p align="center">
    <a href="#neden-konvert">Neden Konvert?</a> •
    <a href="#ekran-görüntüleri">Ekran Görüntüleri</a> •
    <a href="#özellikler">Özellikler</a> •
    <a href="#kurulum">Kurulum</a> •
    <a href="#cli-kullanımı-ve-profiller">CLI Kullanımı</a> •
    <a href="#desteklenen-formatlar">Desteklenen Formatlar</a> •
    <a href="#mimari">Mimari</a>
  </p>

</div>

---

## <img src="assets/icons/konvert-shield.svg" width="22" height="22" align="absmiddle" /> Neden Konvert?

İnternetteki güvensiz web sitelerine gizli dosyalarınızı yüklemekten, sıradan bir dönüştürme için tarayıcı açmaktan veya hantal Electron uygulamalarının dakikalarca açılmasını beklemekten yoruldunuz mu?

**Konvert**, modern Linux masaüstleri için sıfırdan geliştirilmiş olup ultra hızlı **Rust 2024** motorunu yerel **Qt6 & KDE Breeze** ayarlar arayüzü ile birleştirir:

* <img src="assets/icons/konvert-shield.svg" width="16" height="16" align="absmiddle" /> **Sıfır Telemetri & %100 Gizlilik:** Dosyalarınız cihazınızdan asla ayrılmaz. Bulut yüklemesi, ağ bağımlılığı ve arka plan izleyicisi yoktur.
* <img src="assets/icons/konvert-layers.svg" width="16" height="16" align="absmiddle" /> **Kusursuz Bağlam Menüsü:** **KDE Dolphin**, **Cinnamon Nemo**, **XFCE Thunar** ve **GNOME Nautilus** üzerinde doğrudan sağ tıklayarak dönüştürün.
* <img src="assets/icons/konvert-monitor.svg" width="16" height="16" align="absmiddle" /> **Canlı D-Bus İlerlemesi ve Dinamik ETA:** Yerel masaüstü bildirim merkezinde animasyonlu ilerleme çubuğu ve gerçek kalan süre (`ETA: Xs`) tahmini.
* <img src="assets/icons/konvert-compress.svg" width="16" height="16" align="absmiddle" /> **Hassas Hedef Boyutlandırma:** Discord (8 MB), E-posta (25 MB) ve özel limitler için dinamik ayar merdiveni ve boyut koruma mekanizması.
* <img src="assets/icons/konvert-zap.svg" width="16" height="16" align="absmiddle" /> **Rayon Çok Çekirdekli Paralellik:** Akıllı işlemci bütçelemesi (`--jobs`) ile yüzlerce dosyayı aynı anda paralel işleyin.
* <img src="assets/icons/konvert-image.svg" width="16" height="16" align="absmiddle" /> **İşlem Günlüğü ve Önizlemeler:** Yerel disk alanı tasarrufunu izleyin, 80×80 küçük resim önizlemelerine bakın ve çıktıları tek tıkla dosya yöneticisinde gösterin.

---

## <img src="assets/icons/konvert-monitor.svg" width="22" height="22" align="absmiddle" /> Ekran Görüntüleri

<div align="center">

### Sağ Tık Dosya Yöneticisi Entegrasyonu
*Herhangi bir görsel veya medyaya sağ tıklayın; program açmadan anında dönüştürün*

<img src="assets/screenshots/screenshot_context_menu.png" alt="Dolphin Sağ Tık Bağlam Menüsü" width="92%" />

<br><br>

### Modern Qt6 & KDE Breeze Ayarlar Arayüzü
*Bağlam menülerini kategori bazında yönetin, dönüştürme kalitelerini ve kodlama profillerini ayarlayın*

<img src="assets/screenshots/screenshot_convert.png" alt="Konvert Ayarlar Arayüzü" width="90%" />

<br><br>

### Dönüşüm Geçmişi ve Tasarruf Paneli
*Toplam disk alanı tasarrufu takipçisi, küçük resim önizlemeleri ve hızlı dosya yönetimi*

<img src="assets/screenshots/screenshot_history.png" alt="Konvert Geçmiş Paneli" width="90%" />

<br><br>

### Özgün KDE Geliştirici Hakkında İletişim Kutusu
*Standart KDE sekmeleri, sistem meta veri denetimi ve etkileşimli dil seçimi*

<img src="assets/screenshots/screenshot_about.png" alt="Konvert Hakkında İletişim Kutusu" width="90%" />

</div>

---

## <img src="assets/icons/konvert-zap.svg" width="22" height="22" align="absmiddle" /> Özellikler

| Özellik | Açıklama |
|---|---|
| <img src="assets/icons/konvert-layers.svg" width="16" height="16" align="absmiddle" /> **Çoklu Masaüstü Desteği** | Dolphin, Nemo, Thunar ve Nautilus için MIME türü kirliliği yaratmayan temiz sağ tık eylemleri. |
| <img src="assets/icons/konvert-zap.svg" width="16" height="16" align="absmiddle" /> **Rust 2024 Rayon Motoru** | Yapılandırılabilir iş parçacığı bütçesi (`--jobs`), sıfır GC duraklaması ve LTO ile optimize edilmiş binary. |
| <img src="assets/icons/konvert-compress.svg" width="16" height="16" align="absmiddle" /> **Hazır Profiller** | Dahili profiller (`web`, `balanced`, `archive`, `lossless`) ve `config.toml` dosyasına kaydedilebilen özel profiller (`--save-profile`). |
| <img src="assets/icons/konvert-terminal.svg" width="16" height="16" align="absmiddle" /> **Unix Boru Hattı (`-`)** | Shell scriptleri ve boru hatları için sihirli bayt (magic bytes) format algılamalı stdin/stdout desteği. |
| <img src="assets/icons/konvert-shield.svg" width="16" height="16" align="absmiddle" /> **Atomik Veri Güvenliği** | Geçici `.part` dosyaları, çakışma durumunda otomatik `_2`, `_3` adlandırması ve üzerine yazmadan önce `.bak` yedeği (`--backup`). |
| <img src="assets/icons/konvert-image.svg" width="16" height="16" align="absmiddle" /> **Yeni Nesil Görsel Formatları** | JPEG XL (`.jxl`), AVIF, QOI, WebP, HEIC, ICO, BMP, TIFF desteği ve `potrace` ile SVG vektörleştirme. |
| <img src="assets/icons/konvert-document.svg" width="16" height="16" align="absmiddle" /> **Belge ve PDF Araçları** | UDF (UYAP) ve DOCX belgelerini tablo ve üstbilgi yapısını koruyarak PDF/TXT formatına çevirme; çoklu görseli tek PDF yapma. |
| <img src="assets/icons/konvert-globe.svg" width="16" height="16" align="absmiddle" /> **Çok Dilli (i18n) Destek** | CLI'da 9 dil (`--lang <code>`: TR, EN, DE, FR, ES, IT, RU, ZH, JA) ve Qt6 GUI'de 5 dil desteği. |

---

## <img src="assets/icons/konvert-compress.svg" width="22" height="22" align="absmiddle" /> Kurulum

### Yöntem 1: Evrensel Tek Satırlık Kurucu (Önerilen)

En güncel binary, ikonlar, man sayfası, kabuk tamamlama ve dosya yöneticisi menülerini kurar:

```bash
# enesehs.dev üzerinden:
curl -sSL https://enesehs.dev/konvert/install.sh | bash

# Veya GitHub üzerinden:
curl -sSL https://raw.githubusercontent.com/enesehs/konvert/main/docs/public/install.sh | bash
```

### Yöntem 2: Debian / Ubuntu (.deb Paketi)

[GitHub Releases](https://github.com/enesehs/konvert/releases) sayfasından `.deb` paketini indirin veya yerelde derleyin:

```bash
# Paketi kurun:
sudo dpkg -i packaging/konvert_*.deb
sudo apt-get install -f

# Veya yerel olarak paketleyin:
./packaging/build-deb.sh
```

### Yöntem 3: Arch Linux / AUR (PKGBUILD)

```bash
cd packaging
makepkg -si
```

### Yöntem 4: Taşınabilir AppImage

```bash
# Bağımsız AppImage paketi üretin:
./packaging/build-appimage.sh
```

### Yöntem 5: Flatpak (Flathub Manifesti)

```bash
flatpak-builder --user --install --force-clean build-dir packaging/io.github.enesehs.konvert.json
```

### Yöntem 6: Kaynaktan Yerel Derleme

```bash
git clone https://github.com/enesehs/konvert.git
cd konvert
./scripts/install.sh
```

---

## <img src="assets/icons/konvert-terminal.svg" width="22" height="22" align="absmiddle" /> CLI Kullanımı ve Profiller

```bash
# ── Hazır Profiller ve Boru Hattı ──────────────────────────────────────
konvert --profile web -t webp photo.png       # Web profili (%80 kalite, 150 DPI)
konvert --profile archive -t jpg photo.png     # Arşiv profili (%95 kalite, 600 DPI)
konvert --profile lossless -t webp photo.png  # Kayıpsız kodlama modu
konvert --list-profiles                       # Mevcut profilleri listele
cat photo.png | konvert -t webp - > out.webp  # stdin/stdout üzerinden akış borulama

# ── Akıllı Hedef Boyutlandırma ─────────────────────────────────────────
konvert -t discord_8mb video.mp4              # 8 MB Discord sınırına sığdır
konvert -t email_25mb video.mp4               # 25 MB e-posta sınırına sığdır
konvert --target-size 15MB video.mp4          # Özel boyut hedefi (15 MB)

# ── Görsel Dönüştürme ve Optimizasyon ──────────────────────────────────
konvert -t webp -q 90 photo.png                # Belirli kaliteyle WebP dönüşümü
konvert -t jxl --lossless photo.png            # Kayıpsız JPEG XL dönüşümü
konvert -t avif photo.png                      # Yeni nesil AVIF dönüşümü
konvert -t svg photo.png                       # Vektörleştirerek SVG konturuna dönüştür
konvert -t resize_50 photo.png                 # Çözünürlüğü %50 küçült
konvert -t resize_1080p photo.jpg              # Full HD (1920x1080) sınırına sığdır
konvert -t strip-metadata photo.jpg            # EXIF, GPS ve gizlilik verilerini sil

# ── Belge ve PDF Araçları ──────────────────────────────────────────────
konvert --merge-pdf p1.png p2.jpg p3.webp     # Çoklu görseli tek PDF yap
konvert -t pdf belge.udf                       # UDF belgesini doğrudan PDF yap
konvert -t txt belge.docx                      # DOCX tablosu ve metnini dışa aktar

# ── Video ve Ses Araçları ──────────────────────────────────────────────
konvert -t gif video.mp4                       # Yüksek kaliteli iki aşamalı palet GIF
konvert -t extract-audio video.mp4             # Sesi MP3 olarak ayıkla
konvert --extract-frames video.mp4             # Kareleri klasöre PNG olarak çıkar
konvert -t audio-to-video ses.mp3              # Ses dosyasından dalga formu MP4 video üret

# ── Toplu İşlem ve Güvenlik Kontrolleri ─────────────────────────────────
konvert -t webp -j 4 *.png                    # 4 CPU çekirdeği ile paralel toplu dönüşüm
konvert -r -t webp ./fotograflar               # Alt klasörleri özyinelemeli tara ve dönüştür
konvert --dry-run -t webp *.jpg                # Dosyaları yazmadan çıktı planını önizle
konvert --backup -f -t png photo.jpg           # Üzerine yazmadan önce otomatik .bak yedeği al
konvert --check-update                         # Çevrimiçi GitHub güncellemelerini denetle
konvert --cancel                               # Çalışan arka plan işlemini güvenle iptal et
konvert -s                                     # Qt6 Ayarlar Arayüzünü Aç
```

---

## <img src="assets/icons/konvert-folder.svg" width="22" height="22" align="absmiddle" /> Desteklenen Formatlar

| Kategori | Girdi Formatları | Hedef Formatlar | Özel Yetenekler |
|---|---|---|---|
| <img src="assets/icons/konvert-image.svg" width="16" height="16" align="absmiddle" /> **Görseller** | PNG, JPG, JPEG, WebP, AVIF, JXL, SVG, BMP, ICO, TIFF, QOI, HEIC | WebP, PNG, JPG, SVG, ICO, HEIC, BMP, AVIF, JXL, TIFF, QOI | Otomatik EXIF Yönü Düzeltme, Metadata Temizleme, %50 ve 1080p Ölçekleme, Çoklu Görselden PDF, Potrace Çizim |
| <img src="assets/icons/konvert-video.svg" width="16" height="16" align="absmiddle" /> **Videolar** | MP4, MKV, WebM, AVI, MOV, FLV, TS, WMV | MP4, MKV, WebM, GIF | Discord 8 MB, E-posta 25 MB, Özel Hedef Boyut, Kare Ayıklama, Ses Ayıklama, İki Aşamalı Palet GIF |
| <img src="assets/icons/konvert-audio.svg" width="16" height="16" align="absmiddle" /> **Ses** | MP3, WAV, FLAC, OGG, Opus, AAC, M4A | MP3, WAV, FLAC, OGG, Opus, AAC, M4A | Ayarlanabilir Bit Hızları (`128k` - `320k`), Dalga Formu MP4 Video Üretimi |
| <img src="assets/icons/konvert-document.svg" width="16" height="16" align="absmiddle" /> **Belgeler** | UDF (UYAP), DOCX, PDF, TXT | PDF, PNG, JPG, TXT | Çoklu Çözünürlük (36-1200 DPI), Ghostscript Optimizasyonu, DOCX/UDF Tablo ve Metin Çıkarma |

---

## <img src="assets/icons/konvert-layers.svg" width="22" height="22" align="absmiddle" /> Mimari

```
┌─────────────────────────────────────────────────────────────────────────┐
│            Linux Masaüstü Dosya Yöneticileri ve Kabuk Boru Hatları      │
│     (KDE Dolphin • Cinnamon Nemo • XFCE Thunar • GNOME Nautilus)        │
└────────────────────────────────────┬────────────────────────────────────┘
                                     │
                    ┌────────────────┴────────────────┐
                    ▼                                 ▼
      ┌───────────────────────────┐     ┌───────────────────────────┐
      │     Rust 2024 Çekirdeği   │     │   Qt6 C++ Masaüstü Ayar   │
      │                           │     │                           │
      │ • Rayon İş Havuzu (-j)    │     │ • KDE Breeze Tasarım Dili │
      │ • FFmpeg & ImageMagick    │     │ • Canlı Süreç Monitörü    │
      │ • Ghostscript & Poppler   │     │ • Çoklu Örnek Eşzamanlama │
      │ • Atomik I/O (.part/.bak) │     │ • Çıktı Klasörü Seçici    │
      │ • D-Bus Yerinde İlerleme  │     │ • Görsel Geçmiş & Tasarruf│
      └───────────────────────────┘     └───────────────────────────┘
```

---

## <img src="assets/icons/konvert-git.svg" width="22" height="22" align="absmiddle" /> Katkıda Bulunma

Katkılarınızı memnuniyetle kabul ediyoruz! Katkıda bulunmak için:

1. Depoyu forklayın.
2. Özellik dalınızı oluşturun (`git checkout -b feature/harika-ozellik`).
3. Değişikliklerinizi commit'leyin (`git commit -m 'feat: harika ozellik ekle'`).
4. Testleri ve lintleri doğrulayın (`cargo test && cargo clippy --all-targets -- -D warnings`).
5. Dalınıza gönderin (`git push origin feature/harika-ozellik`).
6. Bir **Pull Request** açın.

---

## <img src="assets/icons/konvert-license.svg" width="22" height="22" align="absmiddle" /> Lisans

Bu proje **GNU General Public License v3.0 veya üzeri (GPL-3.0-or-later)** ile lisanslanmıştır. Detaylar için [LICENSE](LICENSE) dosyasına bakabilirsiniz.

---

<div align="center">
  <b>Geliştirici:</b> <a href="https://github.com/enesehs">enesehs</a> &bull;
  <b>İletişim:</b> <a href="mailto:enesehs@protonmail.com">enesehs@protonmail.com</a><br>
  <sub>Linux ve Açık Kaynak Topluluğu için özenle geliştirildi.</sub>
</div>
