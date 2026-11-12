<div align="center">

  <a href="https://enesehs.dev/konvert">
    <img src="assets/hero.png" alt="Konvert Banner" width="100%" />
  </a>

  <br><br>

  [English](README.md) • **Türkçe** • [Deutsch](README_de.md) • [Français](README_fr.md) • [Español](README_es.md) • [Italiano](README_it.md) • [Русский](README_ru.md) • [中文](README_zh.md) • [日本語](README_ja.md)

  <br>

  [![Web Sitesi](https://img.shields.io/badge/Web%20Sitesi-enesehs.dev%2Fkonvert-007ebf?style=for-the-badge&logo=google-chrome&logoColor=white)](https://enesehs.dev/konvert)
  [![CI](https://img.shields.io/github/actions/workflow/status/enesehs/konvert/build.yml?branch=main&style=for-the-badge&logo=githubactions&logoColor=white&label=CI)](https://github.com/enesehs/konvert/actions/workflows/build.yml)
  [![GitHub Release](https://img.shields.io/badge/Release-v0.3.0-3daee9?style=for-the-badge&logo=github)](https://github.com/enesehs/konvert/releases)
  [![Lisans](https://img.shields.io/badge/Lisans-GPL--3.0--or--later-2ecc71?style=for-the-badge)](https://www.gnu.org/licenses/gpl-3.0.html)

  <br>

  <p align="center">
    <b>Linux ve tüm büyük masaüstü dosya yöneticileri için ultra hızlı, hafif, %100 çevrimdışı yerel sağ tık dosya dönüştürücüsü ve optimize edicisi.</b>
  </p>

  <p align="center">
    🌐 <b>Etkileşimli Canlı Demo ve SSS:</b> <a href="https://enesehs.dev/konvert"><b>enesehs.dev/konvert</b></a>
  </p>

  <p align="center">
    <a href="#hızlı-başlangıç">Hızlı Başlangıç</a> •
    <a href="#neden-konvert">Neden Konvert?</a> •
    <a href="#performans-ve-kıyaslama">Performans</a> •
    <a href="#ekran-görüntüleri">Ekran Görüntüleri</a> •
    <a href="#özellikler">Özellikler</a> •
    <a href="#kurulum">Kurulum</a> •
    <a href="#cli-kullanımı">CLI Kullanımı</a> •
    <a href="#mimari">Mimari</a>
  </p>

  <br>

  <a href="https://enesehs.dev/konvert">
    <img src="assets/demo.gif" alt="Konvert Sağ Tık Dönüşüm Akışı" width="92%" />
  </a>

</div>

---

## Hızlı Başlangıç

Saniyeler içinde kullanmaya başlayın:

```bash
# 1. Evrensel kurulum betiği ile kurun (veya kaynaktan derleyin)
curl -sSL https://enesehs.dev/konvert/install.sh | bash

# 2. Herhangi bir dosyayı anında dönüştürün veya optimize edin (CLI veya sağ tık ile)
konvert -t webp gorsel.png

# 3. Yerel Qt6 ve Breeze ayarlar panelini açın
konvert -s
```

---

## Neden Konvert?

Gizli dosyalarınızı çevrimiçi dönüştürücü sitelerine yüklemekten, tarayıcı sekmeleriyle uğraşmaktan veya hantal Electron uygulamalarının açılmasını beklemekten sıkıldınız mı?

* 🛡️ **Sıfır Telemetri & %100 Gizlilik:** Her bir bayt yalnızca yerel donanımınızda işlenir; sıfır ağ isteği ve sıfır arka plan takibi.
* 🖱️ **Yerel Sağ Tık Menüleri:** **KDE Dolphin**, **Cinnamon Nemo**, **XFCE Thunar** ve **GNOME Nautilus** üzerinde doğrudan sağ tıklayarak dönüştürün.
* ⚡ **Rust 2024 & Rayon Paralelliği:** Sıfır GC duraklaması ve ~25 MB bellek kullanımıyla çok çekirdekli paralel toplu işleme.
* 🎯 **Matematiksel Hedef Boyutlandırma:** Medyayı kesin boyut sınırlarına (Discord 8 MB, E-posta 25 MB) otomatik ayar arayışıyla sığdırın.
* 🔔 **Canlı D-Bus İlerleme Bildirimleri:** Masaüstü bildirim merkezinizde dinamik animasyonlu ilerleme çubuğu ve anlık kalan süre (ETA) tahmini.
* 📊 **Görsel Geçmiş ve Tasarruf Takibi:** Toplam disk alanı tasarrufunu izleyin, 80×80 küçük resim önizlemelerine bakın ve dosyaları tek tıkla açın.

👉 *Kapsamlı etkileşimli karşılaştırma ve detaylar için: [enesehs.dev/konvert](https://enesehs.dev/konvert).*

---

## Performans ve Kıyaslama

AMD Ryzen 7 (8 çekirdek / 16 iş parçacığı) üzerinde **19.4 MB 4K PNG** (`3840×2160`) testi:

| İşlem | Hedef / Kip | Çıktı Boyutu | Disk Tasarrufu | Süre (Tek Dosya) | Süre (Rayon `-j 8` ×50 Toplu) |
|---|---|---|---|---|---|
| **JPEG XL (`.jxl`)** | `--lossless` | 1.5 MB | **-%92.2** | 517 ms | **3.8 sn** (~3.9× hızlanma) |
| **WebP (`.webp`)** | `-q 85` | 1.9 MB | **-%90.2** | 843 ms | **5.4 sn** (~4.1× hızlanma) |
| **JPEG (`.jpg`)** | `-q 90` | 4.1 MB | **-%78.8** | 228 ms | **1.6 sn** (~3.8× hızlanma) |
| **Unix Boru Hattı (`-`)** | Stdin ➔ Stdout | Akış | — | **37 ms** gecikme | — |

*Sıfır Electron/JVM çalışma zamanı yükü. Taban bellek kullanımı (RSS): ~24–28 MB.*

---

## Ekran Görüntüleri

<div align="center">
  <sub><b>Masaüstü Sağ Tık Bağlam Menüsü Entegrasyonu</b></sub><br>
  <a href="assets/screenshots/screenshot_context_menu.png">
    <img src="assets/screenshots/screenshot_context_menu.png" alt="Masaüstü Sağ Tık Bağlam Menüsü Entegrasyonu" width="90%" />
  </a>
</div>

<br>

<table align="center" width="100%">
  <tr>
    <td width="33.3%" align="center" valign="top">
      <sub><b>Qt6 & Breeze Ayarlar Arayüzü</b></sub><br>
      <a href="assets/screenshots/screenshot_convert.png">
        <img src="assets/screenshots/screenshot_convert.png" alt="Ayarlar Arayüzü" width="100%" />
      </a>
    </td>
    <td width="33.3%" align="center" valign="top">
      <sub><b>Dönüşüm Geçmişi ve Tasarruf Paneli</b></sub><br>
      <a href="assets/screenshots/screenshot_history.png">
        <img src="assets/screenshots/screenshot_history.png" alt="Geçmiş Paneli" width="100%" />
      </a>
    </td>
    <td width="33.3%" align="center" valign="top">
      <sub><b>KDE Geliştirici Hakkında İletişim Kutusu</b></sub><br>
      <a href="assets/screenshots/screenshot_about.png">
        <img src="assets/screenshots/screenshot_about.png" alt="Hakkında Penceresi" width="100%" />
      </a>
    </td>
  </tr>
</table>

---

## Özellikler

| Özellik | Açıklama |
|---|---|
| <img src="assets/icons/konvert-layers.svg" width="16" height="16" align="absmiddle" /> **Çoklu Masaüstü Desteği** | Dolphin, Nemo, Thunar ve Nautilus için MIME türü kirliliği yaratmayan Fotoğraf, Video, Müzik ve Belge bağlam eylemleri. |
| <img src="assets/icons/konvert-zap.svg" width="16" height="16" align="absmiddle" /> **Rust 2024 Rayon Motoru** | Yapılandırılabilir iş parçacığı bütçeleri (`--jobs`), sıfır GC duraklaması ve LTO optimizasyonlu ikili dosya ile paralel toplu işleme. |
| <img src="assets/icons/konvert-compress.svg" width="16" height="16" align="absmiddle" /> **Hazır ve Özel Profiller** | Yerleşik profiller (`web`, `balanced`, `archive`, `lossless`) ve `config.toml` dosyasına kaydedilen özel profil yönetimi (`--save-profile <isim>`). |
| <img src="assets/icons/konvert-terminal.svg" width="16" height="16" align="absmiddle" /> **Unix Boru Hattı Akışı (`-`)** | Kabuk boru hatları (pipeline) için otomatik sihirli bayt format tespiti ile doğrudan stdin/stdout akış desteği. |
| <img src="assets/icons/konvert-shield.svg" width="16" height="16" align="absmiddle" /> **Atomik Veri Güvenliği** | Geçici `.part` dosyaları ile atomik yazma, çakışma koruması (`_2`, `_3`) ve isteğe bağlı `.bak` yedekleme (`--backup`). |
| <img src="assets/icons/konvert-image.svg" width="16" height="16" align="absmiddle" /> **Yeni Nesil Görsel Formatları** | JPEG XL (`.jxl`), AVIF, QOI, WebP, HEIC, ICO, BMP, TIFF ve `potrace` ile raster görsellerden vektör SVG çıkarımı. |
| <img src="assets/icons/konvert-document.svg" width="16" height="16" align="absmiddle" /> **Belge ve PDF Araçları** | Tablo ve başlıkları koruyan doğrudan UDF (UYAP) ve DOCX -> PDF/TXT dönüşümü ve çoklu görselden PDF birleştirme. |
| <img src="assets/icons/konvert-globe.svg" width="16" height="16" align="absmiddle" /> **Çok Dilli (i18n) Destek** | Komut satırında 9 yerelleştirilmiş dil (`--lang <code>`: EN, TR, DE, FR, ES, IT, RU, ZH, JA) ve Qt6 arayüzünde 5 dil. |

---

## Kurulum

### Yöntem 1: Evrensel Tek Satır Kurucu (Önerilen)

En güncel derlenmiş ikili dosyayı, simgeleri, kabuk tamamlamalarını ve dosya yöneticisi bağlam menülerini kurar:

```bash
curl -sSL https://enesehs.dev/konvert/install.sh | bash
```

> [!TIP]
> **Çalıştırmadan önce denetleyin:** Kurulum betiğini [`docs/public/install.sh`](docs/public/install.sh) adresinden inceleyebilir veya sağlama toplamını doğrulayabilirsiniz:
> ```bash
> curl -sSL https://enesehs.dev/konvert/install.sh -o /tmp/install.sh
> echo "faccff6b8e0601a1dce2d3efb215a69cfe08f748a557b6c01a4375236d3c8c0c  /tmp/install.sh" | sha256sum -c
> bash /tmp/install.sh
> ```

### Yöntem 2: Debian / Ubuntu (.deb Paketi)

[GitHub Releases](https://github.com/enesehs/konvert/releases) sayfasından `.deb` paketini indirin veya yerel olarak derleyin:

```bash
# Paketi kurun:
sudo dpkg -i packaging/konvert_*.deb
sudo apt-get install -f

# Veya depodan derleyin:
./packaging/build-deb.sh
```

### Yöntem 3: Arch Linux / AUR (PKGBUILD)

```bash
cd packaging
makepkg -si
```

### Yöntem 4: Bağımsız AppImage

```bash
# AppImage paketini derleyin:
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

## CLI Kullanımı

Konvert, tam bayrak doğrulama ve korumaya sahip temiz ve öngörülebilir bir komut satırı arayüzü sunar:

```bash
# ── Hazır Profiller ve Boru Hattı ──────────────────────────────────────
konvert --profile web -t webp foto.png        # Web profilini uygula (%80 kalite, 150 DPI)
konvert --profile archive -t jpg foto.png      # Arşiv profili (%95 kalite, 600 DPI)
konvert --profile lossless -t webp foto.png   # Kayıpsız kodlama modu
konvert --list-profiles                        # Kullanılabilir hazır ve kullanıcı profillerini listele
cat foto.png | konvert -t webp - > cikti.webp # Stdin/stdout boru hattı üzerinden dönüştür

# ── Akıllı Hedef Boyut Sıkıştırma ──────────────────────────────────────
konvert -t discord_8mb video.mp4               # Videoyu matematiksel olarak 8 MB altına sıkıştır
konvert -t email_25mb video.mp4                # Videoyu 25 MB e-posta sınırına sıkıştır
konvert --target-size 15MB video.mp4           # Özel hedef boyut sınırı belirle

# ── Görsel Dönüştürme ve Optimizasyon ──────────────────────────────────
konvert -t webp -q 90 foto.png                 # Belirli kalite ayarı ile WebP dönüşümü
konvert -t jxl --lossless foto.png             # Kayıpsız JPEG XL dönüşümü
konvert -t avif foto.png                       # AVIF yeni nesil dönüşüm
konvert -t svg foto.png                        # Raster görseli SVG vektör konturlarına dönüştür
konvert -t resize_50 foto.png                  # Çözünürlüğü %50 küçült
konvert -t resize_1080p foto.jpg               # Full HD (1920x1080) sınırına sığdır
konvert -t strip-metadata foto.jpg             # EXIF, GPS ve sensör meta verilerini temizle

# ── Belge ve PDF Araçları ──────────────────────────────────────────────
konvert --merge-pdf s1.png s2.jpg s3.webp      # Birden çok görseli tek bir PDF olarak birleştir
konvert -t pdf belge.udf                       # UDF (UYAP) belgesini doğrudan PDF'e çevir
konvert -t txt belge.docx                      # DOCX belgesinden biçimlendirilmiş metin ve tablo çıkar

# ── Video ve Ses Araçları ──────────────────────────────────────────────
konvert -t gif video.mp4                        # Yüksek kaliteli iki geçişli paletli GIF üret
konvert -t extract-audio video.mp4              # Ses akışını MP3 olarak dışa aktar
konvert --extract-frames video.mp4              # Video karelerini klasöre PNG olarak çıkar
konvert -t audio-to-video sarki.mp3             # Sesten dalga formu görselleştirmeli MP4 video üret

# ── Toplu İşleme ve Güvenlik Kontrolleri ───────────────────────────────
konvert -t webp -j 4 *.png                     # 4 CPU çekirdeği ile paralel toplu dönüşüm
konvert -r -t webp ./fotograflar               # Klasörleri özyinelemeli tara ve dönüştür
konvert --dry-run -t webp *.jpg                 # Çıktı yollarını ve çakışma planını önizle
konvert --backup -f -t png foto.jpg            # Otomatik .bak yedeği ile güvenli üzerine yazma
konvert --check-update                          # GitHub güncellemelerini denetle
konvert --cancel                                # Arka planda süren işlemi güvenle iptal et
konvert -s                                      # Qt6 Ayarlar Arayüzünü Aç
```

---

## Desteklenen Formatlar

| Kategori | Girdi Formatları | Hedef Formatlar | Motor Yetenekleri |
|---|---|---|---|
| <img src="assets/icons/konvert-image.svg" width="16" height="16" align="absmiddle" /> **Fotoğraflar** | PNG, JPG, JPEG, WebP, AVIF, JXL, SVG, BMP, ICO, TIFF, QOI, HEIC | WebP, PNG, JPG, SVG, ICO, HEIC, BMP, AVIF, JXL, TIFF, QOI | EXIF Otomatik Yönlendirme, EXIF Temizleme, %50 ve 1080p Yeniden Boyutlandırma, Çoklu Görselden PDF, Potrace Çizimi |
| <img src="assets/icons/konvert-video.svg" width="16" height="16" align="absmiddle" /> **Videolar** | MP4, MKV, WebM, AVI, MOV, FLV, TS, WMV | MP4, MKV, WebM, GIF | Discord 8 MB, E-posta 25 MB, Özel Hedef Boyutlandırma, Kare Çıkarma, Ses Çıkarma, İki Geçişli Paletli GIF |
| <img src="assets/icons/konvert-audio.svg" width="16" height="16" align="absmiddle" /> **Müzik / Ses** | MP3, WAV, FLAC, OGG, Opus, AAC, M4A | MP3, WAV, FLAC, OGG, Opus, AAC, M4A | Yapılandırılabilir Bit Hızı (`128k` - `320k`), Dalga Formu Videoları (MP4) Üretimi |
| <img src="assets/icons/konvert-document.svg" width="16" height="16" align="absmiddle" /> **Belgeler** | UDF (UYAP), DOCX, PDF, TXT | PDF, PNG, JPG, TXT | Çoklu DPI Sayfa Çizimi (36-1200 DPI), Ghostscript Optimizasyonu, DOCX/UDF Tablo ve Metin Çıkarımı |

---

## Mimari

```mermaid
flowchart TD
    FM["Masaüstü Dosya Yöneticileri ve Kabuk Boru Hatları<br><b>(KDE Dolphin • Cinnamon Nemo • XFCE Thunar • GNOME Nautilus)</b>"] --> CORE["<b>Rust 2024 Çekirdek Motoru (konvert)</b><br>• Rayon Paralel Toplu İş Parçacığı Havuzu (--jobs)<br>• FFmpeg, ImageMagick ve Ghostscript İşlem Boruları<br>• D-Bus Dinamik Bildirim ve İlerleme Entegrasyonu<br>• Atomik G/Ç ve Güvenli Yedekleme (.part / .bak)"]
    FM -.->|"konvert -s"| GUI["<b>Qt6 & Breeze Ayarlar Arayüzü</b><br>• Kalite, Profil ve Format Yapılandırması<br>• Toplam Disk Tasarrufu ve Görsel Geçmiş<br>• Özel Profil Düzenleyici"]
    CORE <--> GUI
```

---

## Katkıda Bulunma

Katkılarınız memnuniyetle karşılanır! Katkıda bulunmak için:

1. Depoyu forklayın.
2. Özellik dalınızı oluşturun (`git checkout -b feature/harika-ozellik`).
3. Değişikliklerinizi işleyin (`git commit -m 'feat: harika ozellik ekle'`).
4. Test ve lint kontrollerini çalıştırın (`cargo test && cargo clippy --all-targets -- -D warnings`).
5. Dalınıza gönderin (`git push origin feature/harika-ozellik`).
6. Bir **Pull Request** açın.

---

## Lisans

Bu proje **GNU Genel Kamu Lisansı v3.0 veya üstü (GPL-3.0-or-later)** ile lisanslanmıştır. Detaylar için [LICENSE](LICENSE) dosyasına göz atabilirsiniz.

---

<div align="center">
  <b>Geliştirici:</b> <a href="https://github.com/enesehs">enesehs</a> &bull;
  <b>İletişim:</b> <a href="mailto:enesehs@protonmail.com">enesehs@protonmail.com</a><br>
  <sub>Linux ve Açık Kaynak Topluluğu için tutkuyla geliştirildi.</sub>
</div>
