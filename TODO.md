# Konvert — Denetim Sonrası TODO ve Yol Haritası

> **Sürüm:** v0.2.0 · **Denetim tarihi:** 2026-11-11
> **Kapsam:** Rust çekirdeği (`src/`), Qt6 GUI (`gui/`), paketleme (`packaging/`), CI (`.github/`), doküman (README + docs sitesi)
> **Amaç:** v0.2.0 kod denetiminde bulunan tüm hataları, riskleri ve eksikleri kanıtlarıyla listelemek; her biri için uygulanabilir çözüm, kabul kriteri ve test tanımlamak.
> **Not:** Bu dosya, depodaki eski "Core Optimization Roadmap" dosyasının yerine geçer. Tamamlanan performans işleri §11'de arşivlendi.
> **İlerleme (2026-11-12):** 67 maddeden 52'si tam, 5'i kısmi (~%85). P0 %100, P1 ~%78, P2 11/13, P3 14/15, P4 8,5/12, P5 7/12. Birim test sayısı 28 → 57. Sıradaki: P4-2/6, P2-9/10, P1-7 (i18n), P3-14, P5-7..12.

---

## 0. Nasıl Okunur

| Alan | Anlam |
|---|---|
| **Öncelik** | P0 = veri kaybı/yanlış sonuç/güvenlik · P1 = yayın öncesi olmalı · P2 = önemli iyileştirme · P3 = GUI/UX · P4 = paketleme/CI/doküman · P5 = yeni özellik |
| **Efor** | S ≤ 2 saat · M ≈ 1 gün · L ≥ 2–3 gün |
| **Durum** | `[ ]` açık · `[~]` devam ediyor · `[x]` kapalı |
| **Kanıt** | `dosya:satır` referansı ve/veya çalıştırılmış komut + gözlenen çıktı |

**Kural:** Bir madde ancak "Kabul kriteri" ve "Test" satırları karşılandığında `[x]` yapılır. Test yoksa madde kapanmaz.

---

## 1. Yönetici Özeti

v0.2.0 derleniyor, `cargo clippy` uyarısız geçiyor, 28 birim testi başarılı. Mimari modüler ve okunaklı; shell injection yok, `anyhow` ile bağlamlı hatalar var, LTO/strip ile 7,9 MB binary üretiliyor.

Ancak uçtan uca denemelerde **6 kritik hata** doğrulandı: toplu dönüşümde çıktı kaybı, Thunar ayar dosyasının bozulması, PDF'te hedef boyutun hiç uygulanmaması, `--target-size` ayrıştırma hatası, EXIF yön kaybı ve doğrulanmamış PID öldüren `--cancel`.

| Öncelik | Adet | Tema |
|---|---:|---|
| **P0** | 6 | Veri kaybı, yanlış çıktı, güvenlik |
| **P1** | 9 | Ayar/i18n tutarlılığı, eşzamanlılık, kurulum |
| **P2** | 13 | Motor doğruluğu ve dayanıklılık |
| **P3** | 15 | Qt GUI hataları ve UX |
| **P4** | 12 | Paketleme, CI, doküman doğruluğu |
| **P5** | 12 | Eksik özellikler |

**İlk üç iş:** P0-1 (çıktı çakışması) → P0-2 (Thunar kaldırma) → P0-3/P0-4 (hedef boyut sözleşmesi). Bu dördü birlikte, "kullanıcı verisi bozulmaz" garantisinin minimumudur.

---

## P0 — Kritik

### P0-1 · Toplu dönüşümde çıktı çakışması ve sessiz kayıp
- **Durum:** `[x]` · **Efor:** M · **Alan:** converter / batch · **Tamamlandı:** 2026-11-11 · süreç-içi benzersiz çıktı tahsisi + 3 test
- **Sorun:** Çıktı yolu yalnızca dosya gövdesinden üretiliyor ([src/converter/mod.rs:102-119](src/converter/mod.rs#L102-L119)). Aynı adlı dosyalar (farklı klasörlerden veya farklı uzantılardan) aynı hedefe yazıyor; batch `par_iter` ile paralel olduğu için iki kodlayıcı aynı dosyaya eşzamanlı yazabiliyor ([src/converter/mod.rs:316-333](src/converter/mod.rs#L316-L333)).
- **Kanıt:**
  ```bash
  konvert -t webp -o out photo.png photo.jpg sub/photo.png
  # Batch conversion finished: 3 succeeded, 0 failed.
  # out/ içinde tek dosya: photo.webp (900x700) — diğer ikisi kayıp
  ```
- **Etki:** Sessiz veri kaybı; "3 succeeded" raporu yanlış. `photo.png` + `photo.jpg` gibi çok yaygın bir senaryoda bile tetikleniyor.
- **Çözüm:**
  1. Dönüşümden önce **plan fazı** ekle: tüm girdiler için hedef yolu hesapla, çakışanları sırayla `_2`, `_3` … ile ayır (klasör adını da hesaba kat: `sub/photo.webp` → `photo_sub.webp` veya `out/sub/photo.webp`).
  2. Paralel çalışmada yarışı önlemek için yolu `OpenOptions::new().write(true).create_new(true)` ile **atomik rezerve** et; `AlreadyExists` gelirse sonraki ada geç.
  3. `--force` verilmedikçe mevcut dosyanın üzerine yazma (bkz. P1-2).
- **Kabul kriteri:**
  - [ ] Aynı gövdeye sahip N dosya → N ayrı çıktı, N "success".
  - [ ] Aynı anda iki thread aynı yola yazamaz.
  - [ ] Çıktı adları deterministik ve kullanıcıya raporda gösterilir.
- **Test:** `tests/batch_collision.rs` — 3 aynı adlı fixture, çıktı sayısı == 3, her dosyanın boyutu > 0.

---

### P0-2 · `--uninstall-all` Thunar `uca.xml` dosyasını bozuyor
- **Durum:** `[x]` · **Efor:** S–M · **Alan:** desktop/thunar · **Tamamlandı:** 2026-11-11 · XML-farkında kaldırma + atomik yazma + 4 test
- **Sorun:** Kaldırma işlemi satır bazlı bir durum makinesiyle yapılıyor ([src/desktop/thunar.rs:129-174](src/desktop/thunar.rs#L129-L174)); `<action>` dışındaki satırlar (`<?xml …?>`, `<actions>`, `</actions>`) hiçbir zaman tampona alınmıyor ve dosyaya geri yazılmıyor.
- **Kanıt:** Kullanıcı action'ı eklenmiş `uca.xml` üzerinde `konvert --uninstall-all` sonrası dosya:
  ```xml
    <action> ... kullanıcının action'ı ... </action>
  ```
  Kök eleman ve XML bildirimi yok → Thunar tüm özel action'ları kaybediyor; komut yine "Removed: XFCE Thunar" diyor.
- **Etki:** Sessiz kullanıcı verisi kaybı; başka bir dosya yöneticisi eklentisiyle birlikte kullanılıyorsa tüm menüler gider.
- **Çözüm:**
  1. Kaldırmayı **XML-farkında** yap: `<action>` … `</action>` bloklarını tara, yalnızca `unique-id` değeri `konvert-` ile başlayan blokları at; blok dışı her baytı olduğu gibi koru.
  2. Daha sağlamı: `quick-xml` (veya `roxmltree`) ile ayrıştır, düğüm sil, yeniden serileştir; ayrıştırma başarısızsa **dosyaya dokunma** ve hata döndür.
  3. Yazmayı atomik yap: `uca.xml.tmp` → `fs::rename`.
  4. Kaldırma sonrası doğrulama: dosya hâlâ `xmllint --noout` ile geçerli olmalı.
- **Kabul kriteri:**
  - [ ] Konvert action'ları silinir, kullanıcı action'ları birebir korunur.
  - [ ] Geçersiz/bozuk girdi dosyasında veri değişmez, hata döner.
  - [ ] Boş `<actions></actions>` kalıyorsa dosya makul biçimde bırakılır (silinmez).
- **Test:** `tests/thunar_uca.rs` — fixture: 2 kullanıcı + 5 konvert action; kaldırma sonrası 2 kullanıcı action'ı ve kök eleman korunmalı.

---

### P0-3 · PDF'te `--target-size` hiç uygulanmıyor
- **Durum:** `[x]` · **Efor:** M · **Alan:** converter/document · **Tamamlandı:** 2026-11-11 · ayar merdiveni + boyut doğrulama
- **Sorun:** Hedef < 10 MB ise `/screen`, değilse `/ebook` seçilip ghostscript **bir kez** çalıştırılıyor; çıktı boyutu hiç ölçülmüyor ([src/converter/document.rs:404-428](src/converter/document.rs#L404-L428)).
- **Kanıt:**
  ```bash
  konvert --target-size 1mb big.pdf
  # Converted: ./big_compressed.pdf
  # 6.3 MB ➔ 2.1 MB (67% saved)   ← hedef 1 MB, sonuç 2.1 MB, "başarılı"
  ```
- **Etki:** Discord/e-posta sınırı vaadi PDF'te geçersiz; kullanıcı dosyayı yükleyemeyeceğini ancak dışarıda öğrenir.
- **Çözüm:**
  1. Ortak yardımcı yaz: `fn ensure_under_target(path: &Path, target: u64) -> Result<u64>` (ölç, gerekiyorsa hata).
  2. PDF için **ayar merdiveni** uygula (görseldeki `compress_image_to_size` deseninin aynısı):
     `(/screen)` → `(/ebook)` → `(/screen + -dColorImageResolution=96 -dDownsampleColorImages=true)` → `(/ebook + 72 dpi)`.
     Her adımda üret → boyut ölç → `≤ target` ise dön.
  3. Hiçbiri tutmazsa: en küçük üretilen adayı da sil, en iyi boyutu mesajda bildir ve `Err` döndür (mevcut görsel davranışıyla tutarlı).
  4. Video için de encode sonrası boyut doğrulaması ekle (bkz. P2-12).
- **Kabul kriteri:**
  - [ ] `--target-size` verildiğinde dönen dosya `≤ target` veya açık hata.
  - [ ] Hata mesajı "ulaşılan en küçük boyut" bilgisini içerir.
  - [ ] Başarılı işlemde geçmişe doğru `output_size_bytes` yazılır.
- **Test:** `tests/target_size.rs` — 6 MB PDF + 1 MB hedef → ya `≤1MB` ya hata; asla "başarılı + büyük dosya".

---

### P0-4 · `--target-size` ayrıştırması sayı yerine alt-dize arıyor
- **Durum:** `[x]` · **Efor:** S · **Alan:** cli/main · **Tamamlandı:** 2026-11-11 · `parse_size_bytes` + alt-dize eşlemesi kaldırıldı
- **Sorun:** [src/main.rs:276-299](src/main.rs#L276-L299) içinde `if clean.contains("8") { "discord_8mb" }` ve `if clean.contains("25") { "email_25mb" }` var. "18mb", "128mb", "800mb", "8k" gibi girdiler 8 MB'a; "25" içeren her şey 25 MB'a yönleniyor.
- **Kanıt:**
  ```bash
  konvert --target-size 18mb big.jpg   # 13.4 MB girdi
  # 13.4 MB ➔ 7.3 MB (45% saved)      ← 18 MB istendi, 8 MB uygulandı
  # Ayrıca "dosya hedeften küçük" hatası da verilmedi.
  ```
- **Etki:** Kullanıcının istediğinden farklı sonuç; üstelik sessiz.
- **Çözüm:**
  1. Ayrıştırmayı tek yere taşı: `TargetAction::parse` `"<sayı>[.<ondalık>](b|kb|mb|gb|mib)"` biçimini okusun; `contains` tamamen kalksın.
  2. `main.rs` yalnızca `target = format!("target_size:{}", n)` gibi normalize edilmiş değeri taşısın; eşleme tabloları tek kaynakta olsun.
  3. Geçersiz değerde net hata: "Geçersiz hedef boyut: `18xb` (örnek: 8mb, 25mb, 1.5gb)".
  4. Birim farkını belgele: MB = 1024×1024 (mevcut davranış) veya SI; ikisini karıştırma.
- **Kabul kriteri:**
  - [ ] `18mb` → 18×1024×1024 bayt.
  - [ ] `2.5mb` kabul edilir (veya net hata).
  - [ ] `128kb`, `1gb` doğru ölçeklenir.
  - [ ] Aynı davranış CLI ve GUI'de.
- **Test:** Tablo tabanlı birim testi: girdi → beklenen bayt; `contains` regresyonu için "18mb ≠ 8mb" testi.

---

### P0-5 · EXIF yönü kayboluyor (fotoğraflar yan yatıyor)
- **Durum:** `[x]` · **Efor:** M · **Alan:** converter/image · **Tamamlandı:** 2026-11-11 · `decode_with_orientation` + fixture testi
- **Sorun:** Yerel yol `image::open` kullanıyor ([src/converter/image.rs:96-99](src/converter/image.rs#L96-L99)); `image` crate yön etiketini otomatik uygulamıyor. Yalnızca WebP ve bazı ImageMagick yolları `-auto-orient` içeriyor ([src/converter/image.rs:161-173](src/converter/image.rs#L161-L173)).
- **Kanıt:** PIL ile üretilmiş 200×100, `Orientation=6` JPEG:
  ```
  konvert -t png  → 200x100   (yanlış)
  konvert -t webp → 100x200   (doğru)
  ```
- **Etki:** Telefon fotoğraflarının en sık dönüşümünde (JPG→PNG/AVIF/ICO/TIFF/BMP/QOI/JXL, `resize`, metadata temizleme yedeği) görsel 90° yanlış.
- **Çözüm:**
  1. Tek bir `decode_oriented(path) -> Result<DynamicImage>` fonksiyonu yaz; tüm yerel yollar bunu kullansın.
  2. EXIF yönünü dosyadan oku (`kamadak-exif` veya `image` 0.25'in decoder `orientation()` API'si — sürümde yoksa `kamadak-exif` güvenli seçim) ve `DynamicImage::apply_orientation(...)` uygula (0.25'te mevcut; yoksa `imageops::rotate90/rotate180/rotate270/flip*` ile elle).
  3. `magick -auto-orient` yalnızca **fallback** olsun; hızlı yol native kalsın.
  4. `strip_metadata` çıktısında yön piksele gömülüp etiket silinmeli (mevcut IM yolu doğru yapıyor; native yedek yapmıyor).
- **Kabul kriteri:**
  - [ ] 8 yön değerinin tamamı için doğru piksel boyutu.
  - [ ] JPG→PNG/AVIF/ICO/TIFF/BMP/QOI/JXL ve `resize` aynı sonucu verir.
  - [ ] Çıktıda orientation etiketi kalmaz (strip hedefinde).
- **Test:** `tests/exif_orientation.rs` — 8 fixture (orientation 1–8), beklenen `(w,h)` tablosu.

---

### P0-6 · `--cancel`: doğrulanmamış PID, yanlış başarı raporu, yarım dosya
- **Durum:** `[x]` · **Efor:** M · **Alan:** cli/main/converter · **Tamamlandı:** 2026-11-11 · PID doğrulama, süreç grubu ile sonlandırma, kill sonucu, bayat kilit ve geçici dosya temizliği
- **Sorun:** `~/.cache/konvert/running.pid` içindeki PID koşulsuz `SIGTERM` alıyor; hedefin gerçekten Konvert olup olmadığı kontrol edilmiyor ([src/main.rs:45-59](src/main.rs#L45-L59)). `kill()` dönüş değeri yok sayılıyor. `panic = "abort"` ve SIGKILL/OOM durumunda PID dosyası kalıyor. İptalde yarım çıktı temizlenmiyor.
- **Kanıt:**
  ```bash
  sleep 60 & echo $! > ~/.cache/konvert/running.pid
  konvert --cancel
  # Active Konvert process cancelled (PID: 4).   → alakasız süreç öldü
  echo 999999 > ~/.cache/konvert/running.pid
  konvert --cancel
  # Active Konvert process cancelled (PID: 999999).  → olmayan PID için "başarılı"
  ```
- **Etki:** PID yeniden kullanımında ilgisiz kullanıcı süreci öldürülebilir; yarım/bozuk çıktı dosyaları kalır.
- **Çözüm:**
  1. **Doğrulama:** `/proc/<pid>/exe` (veya `cmdline`) hedefinin Konvert binary'si olduğunu kontrol et; değilse PID dosyasını sil ve "aktif iş yok" de.
  2. **Süreç grubu:** Alt süreçleri `CommandExt::process_group(0)` ile başlat, iptalde `kill(-pgid, SIGTERM)` gönder; böylece ffmpeg/magick çocukları da durur.
  3. **Sonuç kontrolü:** `kill()` 0 dönerse "cancelled", `ESRCH` ise "iş yok / bayat kayıt temizlendi".
  4. **Temizlik:** Tüm çıktılar önce `<hedef>.part` dosyasına yazılsın, başarıda `rename`; iptal/hata sinyalinde `.part` silinsin.
  5. **Sinyal yakalama:** SIGINT/SIGTERM için handler kur; PID dosyası + `.part` temizlensin (aksi halde `panic = "abort"` altında temizlik garantisi yok).
- **Kabul kriteri:**
  - [ ] Yabancı PID'e `--cancel` zarar vermez.
  - [ ] Olmayan PID'de "cancelled" denmez.
  - [ ] İptal sonrası `.part` ve yarım çıktı kalmaz.
  - [ ] İptal, alt ffmpeg sürecini de sonlandırır.
- **Test:** `tests/cancel.rs` — sahte PID dosyası + `sleep` süreci; `--cancel` sonrası süreç yaşıyor olmalı.

---

## P1 — Yüksek

| ID | Başlık | Alan | Efor |
|---|---|---|---|
| P1-1 | Video "lossless" anahtarı ölü, görsel lossless videoyu etkiliyor | config/main | S |
| P1-2 | Var olan çıktı sessizce eziliyor; yazma atomik değil | converter | M |
| P1-3 | Ayar şeması GUI ↔ CLI arasında kayıp yaşıyor | config/GUI | M |
| P1-4 | Toplu işte sınırsız paralellik, `--jobs` yok | batch | M |
| P1-5 | Kurulum/senkron hataları çıkış kodu 0 | desktop/main | S |
| P1-6 | Release tarball içindeki `install.sh` kaynaktan derliyor | packaging | S |
| P1-7 | i18n yarım: 87 sabit Türkçe metin, Türkçe dosya adları | i18n | L |
| P1-8 | README/website gerçekle uyuşmuyor | docs | M |
| P1-9 | Aynı çıktıya yazan ikili action'lar + çoklu action kaydı | desktop/GUI | M |

### P1-1 · Video "lossless" anahtarı ölü
- **Durum:** `[x]` · **Efor:** S · **Tamamlandı:** 2026-11-11
- **Yapılan:** `ConvertOptions` iki bayrağa ayrıldı (`image_lossless`, `video_lossless`); `main` bunları `config.image.lossless` ve `config.video.lossless`ten besliyor. GUI'deki iki onay kutusu artık kendi medya türünü etkiliyor.
- **Sorun:** GUI `[video] lossless` yazıyor ama motor okumuyor; `opts.lossless = cli.lossless || config.image.lossless` ([src/main.rs:301-310](src/main.rs#L301-L310)) olduğu için video da görsel ayarından besleniyor. "Kayıpsız Video" hiçbir şey yapmıyor; "Görselde kayıpsız" tüm videoları CRF 0'a çekiyor.
- **Çözüm:** `ConvertOptions`'a medya türüne göre ayrım ekle (`image_lossless`, `video_lossless`) veya `convert_single` içinde `is_video_ext` dalında `config.video.lossless` kullan. GUI'deki iki onay kutusunu ilgili alanlara bağla.
- **Kabul:** Video kutusu yalnız video dönüşümünü, fotoğraf kutusu yalnız görselleri etkiler. **Test:** config → ffmpeg argümanı eşleme testi (fake `ffmpeg` shim).

### P1-2 · Sessiz üzerine yazma ve atomik olmayan yazma
- **Durum:** `[x]` · **Efor:** M · **Tamamlandı:** 2026-11-11 · `--force`/`_2` politikası, tüm dosya yazıcılarında geçici ad + `rename`, iptalde geçici dosya temizliği (2 test). Dizin çıktıları (kareler/PDF sayfaları) benzersiz adlandırma ile korunuyor.
- **Sorun:** Yazımlar atomik değil: ffmpeg/gs/magick doğrudan hedef dosyaya yazıyor; hata veya iptalde yarım dosya kalıyor (P0-6 testinde 24 MB bozuk `.mkv`). Üzerine yazma politikası bu turda düzeltildi; kalan kısım atomiklik ve yarım dosya temizliği.
- **Not:** P0-6 kapsamında iptal artık süreç grubunu sonlandırıyor; ancak ffmpeg/gs yarım dosyayı bırakıyor. `.part` + `rename` ve `--force`/`--no-clobber` bu maddede ele alınacak.
- **Çözüm (kalan):** Tüm yazıcılar için `PendingOutput` yardımcısı: geçici ada yaz (uzantı korunur: `.konvert-tmp-<pid>-<ad>.<ext>`), başarıda `rename`, `Drop` ile hata/iptalde sil. Harici araçlar (ffmpeg/gs/magick/cjxl) ve yerel `image` yazıcıları aynı yardımcıyı kullanmalı.
- **Kabul:** Mevcut dosya `--force` olmadan değişmez; kesinti sonrası ilk dosya bozulmaz. **Test:** çakışma + SIGTERM senaryosu.

### P1-3 · Ayar şeması kaybı
- **Durum:** `[x]` · **Efor:** M · **Tamamlandı:** 2026-11-11 · `language` + `output_directory` şemada, i18n config'ten besleniyor, bozuk TOML `.bak`'a alınıyor, atomik kayıt, boş `LC_ALL`/`LANGUAGE` sırası düzeltildi. **Kalan (P3-10):** GUI'de çıktı klasörü seçici ve bilinmeyen anahtarların birebir korunması.
- **Sorun:** GUI `output_directory` alanını ne okuyor ne yazıyor ([gui/src/configmanager.cpp:114-160](gui/src/configmanager.cpp#L114-L160)) → her kayıtta kullanıcının çıktı klasörü silinir. GUI `language` yazıyor, Rust bilmiyor ([src/i18n.rs:19-49](src/i18n.rs#L19-L49)). `Config::load()` TOML'u ayrıştıramazsa varsayılanı yazıp tüm ayarları sıfırlıyor ([src/config.rs:121-143](src/config.rs#L121-L143)). Boş `LC_ALL` tanımlıysa `LANG` yok sayılıyor.
- **Çözüm:**
  1. Şemaya `language` ve `output_directory` ekle; GUI bu alanları okusun/yazsın.
  2. Tüm alanlara `#[serde(default)]`; bilinmeyen anahtarları korumak için `#[serde(flatten)] extra: BTreeMap<String, toml::Value>`.
  3. `schema_version` alanı + göç mantığı.
  4. Ayrıştırma hatasında: dosyayı `config.toml.bak-<ts>` olarak sakla, varsayılanı yaz, kullanıcıya bildir (stderr + GUI).
  5. Atomik yazma: tmp + rename (GUI'de `QSaveFile`).
  6. Dil tespiti: `LANGUAGE` → `LC_ALL` (boş değilse) → `LC_MESSAGES` → `LANG`; boş değerleri atla.
- **Kabul:** GUI'de çıktı klasörü seçimi kalıcı; dil CLI bildirimlerini de etkiler; bozuk TOML ayarları silmez.

### P1-4 · Sınırsız paralellik
- **Durum:** `[x]` · **Efor:** M · **Tamamlandı:** 2026-11-11
- **Yapılan:** `--jobs N` eklendi (varsayılan `min(çekirdek,4)`); batch artık `N`'lik parçalar hâlinde paralel çalışıyor, aynı anda en fazla `N` dönüşüm oluyor. ffmpeg `-threads` girdiden sonra (encoder seçeneği olarak) veriliyor ve `çekirdek/N` bütçesine göre hesaplanıyor; eski sabit `-threads 2` kaldırıldı. Bonus: `-t mp3 video.mp4` gibi video→ses hedefleri artık çalışıyor.
- **Sorun:** Rayon varsayılan thread sayısı kadar ffmpeg/ImageMagick aynı anda başlıyor; `--jobs` yok. `-threads 2` ffmpeg'de girdi decode'unu sınırlıyor, encode'u değil — TODO'daki "concurrency limits" maddesi kodda karşılıksız.
- **Çözüm:** `--jobs N` (varsayılan `min(available_parallelism, 4)`). Ağır işler (video/PDF) için semafor; `ThreadPoolBuilder::num_threads(jobs)`. ffmpeg'e `-threads <cores/jobs>` ver; `-threads 2`'yi kaldır. GUI'de "eşzamanlı iş" ayarı.
- **Kabul:** 10 video seçildiğinde eşzamanlı ffmpeg ≤ jobs; CPU %100 doygunlukta kalmaz.

### P1-5 · Sessiz kurulum hataları ve 0 çıkış kodu
- **Durum:** `[x]` · **Efor:** S · **Tamamlandı:** 2026-11-11
- **Yapılan:** `detector` artık `IntegrationReport { affected, failed }` döndürüyor; her başarısızlık adıyla raporlanıyor. Hiçbiri kurulamazsa exit 1; `--sync-menu`, `--install-dolphin` ve `--uninstall-dolphin` hataları da `Err` dönüyor.
- **Sorun:** `install_all` alt kurulum hatalarını yutuyor ([src/desktop/detector.rs:3-21](src/desktop/detector.rs#L3-L21)); `--install-all` başarısızken bile exit 0 ve hiç çıktı yok (test edildi: `HOME=/proc/sys konvert --install-all` → çıktı yok, exit 0). `--sync-menu` de hatada `Ok(())` dönüyor.
- **Çözüm:** `InstallReport { ok: Vec<String>, failed: Vec<(String,String)> }`; en az bir başarı varsa 0, hiç yoksa 1. Her hata stderr'e yazılsın. `main` içinde `bail!`/`Err` dön.
- **Kabul:** Yazılamayan HOME'da exit ≠ 0 ve anlaşılır hata.

### P1-6 · Release tarball'daki `install.sh` çalışmıyor
- **Durum:** `[x]` · **Efor:** S · **Tamamlandı:** 2026-11-11
- **Yapılan:** `scripts/install.sh` iki yerleşimi algılıyor: kaynak ağacı (derleme) ve tarball (hazır `konvert` + `konvert-settings`). Tarball'da derleme atlanıyor; `cmake`/`ninja` yoksa GUI uyarıyla atlanıyor, derleme hatası yutulmuyor; `.desktop` izni 644. Tarball simülasyonuyla uçtan uca doğrulandı (derleme olmadan kurulum tamam).
- **Sorun:** CI tarball'a `scripts/install.sh` koyuyor ([.github/workflows/build.yml:59-80](.github/workflows/build.yml#L59-L80)); script `cargo build` ve `cmake` çağırıyor, tarball'da `Cargo.toml` yok → kurulum patlıyor. Tarball'da hazır binary var ama script onları kullanmıyor.
- **Çözüm:** Tarball'a `install-binary.sh` koy (hazır binary'leri kopyalar) veya `scripts/install.sh`'a "prebuilt layout" tespiti ekle: `./konvert` yanında çalışıyorsa derlemeyi atla.
- **Kabul:** `tar.gz` indir → `./install.sh` → Rust/Qt olmadan kurulum tamamlanır.

### P1-7 · i18n yarım
- **Durum:** `[ ]` · **Efor:** L
- **Sorun:** 87 kullanıcıya görünen sabit Türkçe metin (`bail!` mesajları, ilerleme başlıkları, "PDF Birleştiriliyor", üretilen adlar `_birlestirildi.pdf`, `_sayfalar/sayfa-1.png`). Üç ayrı çeviri sistemi var: Rust `i18n.rs`, GUI `translator.h`, desktop `Name[xx]`. Core 9 dil, GUI 5 dil, GUI action etiketleri yalnız TR/EN.
- **Çözüm:**
  1. Tek kaynak: `fluent` (`.ftl`) veya gettext kataloğu; Rust + Qt aynı katalogdan beslensin (Qt `QTranslator`).
  2. Tüm `bail!`/`eprintln!` metinleri anahtara çevrilsin; parametreli mesajlar (`{path}`, `{size}`) için format desteği.
  3. Dosya adı sonekleri dile göre: `_merged.pdf`, `_pages/`, `page-001.png` (Türkçe özel adlar kaldırılsın).
  4. `--lang <code>` ve config `language`; CLI ile GUI aynı dili kullansın.
  5. `LANGUAGE`/boş `LC_ALL` düzeltmesi (P1-3/6).
- **Kabul:** `LANG=en_US.UTF-8 konvert -t webp yok.jpg` İngilizce hata verir; çıktı adları Türkçe değil. **Test:** `--lang` ile snapshot testi.

### P1-8 · Doküman ve website iddiaları
- **Durum:** `[~]` · **Efor:** M · **Kısmi (2026-11-11):**
  - `TS` girdisi artık gerçekten destekleniyor (`is_video_ext`, test edildi); README/README_tr'deki `TXT` girdisi iddiası kaldırıldı.
  - Dil sayısı düzeltildi (CLI 9 / GUI 5); "strict size limit" ifadesi gerçeğe çekildi.
  - Website: `/usr/local/bin` → `~/.local/bin`, uydurma `--preset <web|balanced|lossless>` → gerçek bayraklar.
  - **Nemo/Cinnamon desteği eklendi** ([nemo.rs](src/desktop/nemo.rs), 34 action, `--install-all`/`--uninstall-all` entegre) → website iddiası artık doğru.
  - **Kalan:** "50+ format" ve "interactive demo" iddiaları, diğer dil dosyalarındaki (de/fr/es/it/ru/zh/ja) metinlerin gözden geçirilmesi, doküman-format otomatik testi.
- **Sorun ve kanıt:**
  - README "TS" girdisi → `is_video_ext`'te yok; test: `Desteklenmeyen dönüştürme: .ts -> .mp4`.
  - README "TXT" girdisi → `.txt` için handler yok.
  - Website "Dolphin, Nautilus, Nemo, Thunar" → Nemo desteği kodda yok.
  - Website "binary'yi /usr/local/bin'e kurar" → `~/.local/bin`.
  - Website "`--preset <web|balanced|lossless>`" → böyle bir preset yok (x264 preset'i).
  - "50+ format" ve "interactive demo" gerçeği yansıtmıyor; README'de 5 dil yazıyor, core 9 dil destekliyor.
- **Çözüm:** README/README_tr/website metinlerini kodla eşle; desteklenen format çiftlerini **test edilebilir tablo** haline getir (P5/CI). Website'deki "Nemo" ve `/usr/local/bin` ifadelerini düzelt; `--preset` örneğini gerçek seçeneklerle değiştir.
- **Kabul:** Dokümanda adı geçen her format çifti için bir test var; yanlış iddia yok.

### P1-9 · Çakışan action çıktıları ve çoklu kayıt
- **Durum:** `[~]` · **Efor:** M · **Kısmi (2026-11-11):**
  - Çıktı çakışmaları giderildi: kayıpsız JXL artık `_jxl_lossless.jxl`, 8/25 MB action'ları `_8mb`/`_25mb` sonekini kullanıyor.
  - Rust tarafında tek kaynak: `ALL_ACTIONS` + `QUICK_ACTION_IDS` ([dolphin.rs](src/desktop/dolphin.rs)); `Config` varsayılanları, Thunar, Nautilus ve Nemo menüleri buradan üretiliyor (komut/isim/ikon kopyası yok). Yinelenen id ve kayıt testleri eklendi.
  - **Kalan:** Qt GUI (`mainwindow.cpp`) hâlâ kendi action listesini taşıyor; GUI'nin de aynı kaynaktan (ör. derleme sırasında üretilen JSON) beslenmesi gerekiyor.
- **Sorun:** `to_jxl` ve `to_jxl_lossless` ikisi de `{stem}.jxl` üretiyor ([src/converter/image.rs:110-115](src/converter/image.rs#L110-L115), [src/desktop/dolphin.rs:130-153](src/desktop/dolphin.rs#L130-L153)); 8MB/25MB foto ikisi de `_target.*`. Action tanımları Rust, C++ ve Nautilus/Thunar script'lerinde **üç kez** kopyalanmış → kaçınılmaz sapma.
- **Çözüm:** Action başına benzersiz sonek (`_jxl_lossless`, `_8mb`, `_25mb`). Tek kaynak: action listesini bir TOML/JSON'dan üret (Rust `include_str!`, GUI `QResource`, script üretici). Çakışan çiftleri GUI'de uyar.
- **Kabul:** Aynı anda çalıştırılan iki action birbirinin çıktısını ezmez; yeni action tek yerden eklenir.

---

## P2 — Motor Doğruluğu ve Dayanıklılık

| ID | Sorun | Çözüm özeti | Efor |
|---|---|---|---|
| P2-1 ✅ | ~~GIF 20 sn videoyu sessizce 15 sn'ye kırpıyor~~ | `--gif-duration` (0 = tamamı) + `--gif-fps` eklendi; kırpma artık stderr'de bildiriliyor (2026-11-11) | S |
| P2-2 ✅ | ~~Dizin çıktılarında anlamsız istatistik~~ | `history::path_size_bytes` (özyinelemeli toplam) + test; tüm istatistik/geçmiş yolları kullanıyor (2026-11-11) | S |
| P2-3 ✅ | ~~PDF→PNG sayfa sayısı sınırsız, adlar Türkçe~~ | `--max-pages` (varsayılan 200, 0 = sınırsız); çıktı `_pages/page-N.png`; dizin çakışması `_2` (2026-11-11) | S |
| P2-4 ✅ | ~~`--resize` yalnız 25/50/75/1080p/720p~~ | 1-400 arası herhangi bir yüzde (`--resize 30%`, `--resize 150`); 1080p/720p korunuyor (2026-11-11) | S |
| P2-5 ✅ | ~~JXL resize geçici dosyası `resize_temp_{pid}.png` — paralel batch'te yarış~~ | Atomik sayaç + `konvert_resize_<pid>_<n>.png`; hata yolunda sil (2026-11-11) | S |
| P2-6 ✅ | ~~Sayısal doğrulama yok~~ | clap aralık/whitelist doğrulayıcıları: quality 1-100, CRF 0-51, DPI 36-1200, jobs 1-64, bitrate ve preset (2026-11-11) | S |
| P2-7 ✅ | ~~Bağımlılık ön kontrolü yok~~ | `--check-deps` eklendi ([deps.rs](src/deps.rs)): PATH taraması, zorunlu/opsiyonel ayrımı, eksikse exit 1 (2026-11-11). GUI sağlık paneli P3-9'da | M |
| P2-8 ✅ | ~~PDF sıkıştırma/birleştirme yalnız ImageMagick/gs'ye bağlı; IM policy.xml engeli~~ | PDF girdilerinde Ghostscript doğrudan kullanılıyor; `run_imagemagick` içinde policy.xml hatası tespit edilip çözüm yönergesiyle raporlanıyor (2026-11-12) | M |
| P2-9 | `to_svg` aslında potrace ile vektörleştirme (çıktıda "Created by potrace") | Özelliği "SVG olarak izle/vektörleştir" diye adlandır; `--trace-colors`; kalite beklentisini dokümante et | S |
| P2-10 | DOCX/UDF metin çıkarımı eksik: tablolar, üstbilgi/altbilgi, dipnot; 50 MB'da sessiz kesme | `quick-xml` ile yapısal ayrıştırma; desteklenmeyen öğeleri raporla; kesme yerine açık hata | M |
| P2-11 ✅ | ~~`--merge-pdf` yüzlerce dosyada ARG_MAX/IM belleği riski~~ | 50'den fazla dosyada 50'şerli parçalama ve Ghostscript/IM ile birleşik montaj eklendi (2026-11-12) | S |
| P2-12 ✅ | ~~Video hedef boyut sonrası doğrulama yok~~ | Encode sonrası boyut ölçülüyor; aşarsa bit hızı ×0.8 ile en fazla 3 deneme, sonra en küçük boyutla hata (2026-11-11) | M |
| P2-13 ✅ | ~~Geçmiş dosyası yarışı ve bozulma~~ | `flock` ile süreçler arası kilit, `tmp` + `rename` atomik yazma, bozuk JSON `.corrupt` olarak yedekleniyor (2026-11-11). GUI tarafı hâlâ dosyayı doğrudan siliyor (P3-4) | M |

---

## P3 — Qt GUI

| ID | Sorun | Çözüm | Efor |
|---|---|---|---|
| P3-1 ✅ | ~~Slider her tik'te config yazıp süreç başlatıyor~~ | 400 ms tek-atımlık `QTimer` debounce; pencere kapanışında bekleyen değişiklik flush ediliyor (2026-11-11) | M |
| P3-2 ✅ | ~~Başarılı satırlar kırmızı~~ | Durum `toLower()` ile normalize ediliyor: success/partial/failed + TR karşılıkları; partial turuncu (2026-11-11) | S |
| P3-3 ✅ | ~~Geçmiş sekmesi yenilenmiyor~~ | `currentChanged` bağlandı; sekmeye geçişte `loadHistoryData()` (2026-11-11) | S |
| P3-4 ✅ | ~~Temizle onaysız/hatasız~~ | `QMessageBox` onayı + silme hatasında uyarı (2026-11-11) | S |
| P3-5 ✅ | ~~Toplam tasarruf şişiyor~~ | İmzalı delta toplanıyor; büyüyen işlemler düşülüyor (2026-11-11) | S |
| P3-6 ✅ | ~~Önizleme tam boy çözülüyor~~ | `QImageReader` + `setScaledSize(72,72)` ve `setAutoTransform`; UI thread'i küçük görsel okuyor (2026-11-11) | S |
| P3-7 ✅ | ~~Combo'lar config değerini yutuyor~~ | Preset ve bitrate değerleri listede yoksa eklenip seçiliyor (2026-11-11) | S |
| P3-8 ✅ | ~~`QFile` include yok~~ | `<QFile>` ve `<QMessageBox>` eklendi (2026-11-11) | S |
| P3-9 ✅ | ~~Senkron `konvert`'i PATH'ten çağırıyor~~ | Binary arama iyileştirildi; masaüstü entegrasyonu kur/kaldır UI'si ve bağımlılık kontrol paneli eklendi (2026-11-12) | M |
| P3-10 ✅ | ~~Genel ayarlar "Hakkında" sekmesinde gömülü~~ | Ayrı "Ayarlar" sekmesi eklendi; çıktı klasörü seçici (gözat/sıfırla), dil, bildirim ve süreç izleme taşındı (2026-11-12) | S |
| P3-11 ✅ | ~~Çalışan işi izleme/iptal yok~~ | `running.pid` ve süreç durumunu izleyen monitör eklendi; çalışan görevi `--cancel` ile sonlandırma butonu bağlandı (2026-11-12) | M |
| P3-12 ✅ | ~~Kaydetme hataları sessiz~~ | `save()` dönüşü kontrol edilip `QMessageBox` ile bildiriliyor (2026-11-11); `QSaveFile` hâlâ yapılabilir | S |
| P3-13 ✅ | ~~Aynı anda iki GUI config'i eziyor~~ | `QLockFile` ile tek örnek kilidi ve `QFileSystemWatcher` ile harici dosya değişikliklerinde otomatik UI senkronu eklendi (2026-11-12) | M |
| P3-14 | Erişilebilirlik ve çeviri: 106 satır içi `lang == "tr" ?` ternary; `lbl_video_crf` yalnız TR/EN | Tüm metinleri `translator.h` anahtarlarına taşı; `setBuddy`, sekme sırası, kısayollar, `F1` | M |
| P3-15 ✅ | ~~`--screenshot-all` dağıtımda~~ | Yalnız `KONVERT_DEBUG` (Debug) derlemesinde etkin (2026-11-11) | S |

**GUI için genel kabul:** Her ayar kaydedildiğinde geri okunabilir; her hata kullanıcıya görünür; hiçbir UI eylemi süreç fırtınası üretmez.

---

## P4 — Paketleme, CI ve Doküman Altyapısı

| ID | Sorun | Çözüm | Efor |
|---|---|---|---|
| P4-1 ✅ | ~~PKGBUILD bağımlılıkları eksik~~ | `imagemagick` depends'e, `ghostscript`/`libreoffice` optdepends'e eklendi (2026-11-11) | S |
| P4-2 `[~]` | ~~sha256 SKIP, lisans kurulmuyor~~ | Lisans `/usr/share/licenses/konvert/` altına kuruluyor; **kalan:** gerçek `sha256sums` (release tarball gerektirir) (2026-11-11) | S |
| P4-3 ✅ | ~~deb bağımlılıkları eksik~~ | `ghostscript` + `libreoffice` Recommends'e eklendi (2026-11-11) | S |
| P4-4 | aarch64 vaat ediliyor ama CI yalnız x86_64 üretiyor (installer aarch64 URL'i 404) | `ubuntu-24.04-arm` runner veya `cross` ile matris; tarball adları `konvert-linux-<arch>` | M |
| P4-5 ✅ | ~~Metainfo'da screenshot yok~~ | 3 ekran görüntüsü (bağlam menüsü, geçmiş, ayarlar) eklendi (2026-11-11) | S |
| P4-6 `[~]` | ~~Flatpak `branch: main`~~ | `tag: v0.2.0` ile sabitlendi; **kalan:** CI'da Flatpak derleme işi + sandbox notu (2026-11-11) | M |
| P4-7 ✅ | ~~Repo'da izlenen eski `.deb` (`packaging/konvert_0.1-beta_amd64.deb`)~~ | `git rm` ile depodan kaldırıldı; release artefaktları git dışında tutuluyor (2026-11-12) | S |
| P4-8 ✅ | ~~docs build hatası yutuluyor~~ | `next build && (cp … || true)` ile yalnız kopyalama hataları yutuluyor (2026-11-11) | S |
| P4-9 | Uzak installer checksum doğrulamıyor (sha256 yayınlanıyor ama kullanılmıyor) ([docs/public/install.sh:80-105](docs/public/install.sh#L80-L105)) | İndirme sonrası `sha256sum -c`; imza doğrulama (minisign/cosign) opsiyonu | M |
| P4-10 | İki ayrı installer (`scripts/install.sh` + `docs/public/install.sh`) sapıyor | Tek kaynak + build sırasında kopyalama; ortak `lib.sh` | M |
| P4-11 ✅ | ~~`scripts/install.sh` GUI derleme hatasını yutuyor~~ | `cmake`/`ninja` hatası artık kurulumu durduruyor; yoksa GUI uyarıyla atlanıyor (2026-11-11) | S |
| P4-12 ✅ | ~~CI'da lint yok~~ | `cargo clippy --all-targets -- -D warnings` ve `cargo fmt --check` CI adımları eklendi; tüm repo rustfmt standartlarına getirildi (2026-11-12) | M |

**Ek küçükler:** `entry.rs` ikonu `preferences-system-symbolic` yerine `konvert` olmalı; `.desktop` dosyaları 755 değil 644; `Exec` yolu boşluk içeriyorsa tırnaklanmalı ([dolphin.rs:572](src/desktop/dolphin.rs#L572)); `--install-all` masaüstü girdisi ile paketlenen `.desktop` tek kaynak olmalı.

---

## P5 — Eksik Özellikler

| ID | Özellik | Neden | Efor |
|---|---|---|---|
| P5-1 ✅ | ~~Tek görsel → PDF~~ | `image_to_pdf` eklendi (`-t pdf photo.png` → `photo.pdf`); ImageMagick `-auto-orient` + `PendingOutput` korumalı (2026-11-12) | S |
| P5-2 ✅ | ~~Klasör / recursive toplu dönüşüm (`-r`)~~ | `-r`/`--recursive` eklendi; alt klasörleri hedefe göre akıllı filtreleyerek toplu tarama ve işleme desteği (2026-11-12) | M |
| P5-3 ✅ | ~~`--dry-run`~~ | `--dry-run` eklendi; `plan_output_path` ile çıktı hedefleri, çakışma ve üzerine yazma durumları simüle ediliyor (2026-11-12) | S |
| P5-4 ✅ | ~~`--jobs`~~ | P1-4 kapsamında eklendi; clap range doğrulayıcısı ve thread budget ile tamamlandı (2026-11-11) | S |
| P5-5 ✅ | ~~Man page + shell completion~~ | `docs/konvert.1` yazıldı; `clap_complete` ile `--completions <bash|zsh|fish>` eklendi; .deb, PKGBUILD ve install.sh ile otomatik kurulum bağlandı (2026-11-12) | S |
| P5-6 ✅ | ~~Log dosyası / `--verbose` / `RUST_LOG`~~ | `src/logger.rs` eklendi; `-v`/`--verbose`, `RUST_LOG` desteği ve `~/.cache/konvert/konvert.log` teşhis günlüğü (2026-11-12) | S |
| P5-7 | Preset profilleri: kaydet/adlandır/sil/dışa-içe aktar | Güçlü kullanıcı; GUI'de karşılığı yok | M |
| P5-8 | Geri alma: çöp kutusuna taşıma / `.bak` | Yanlış dönüşümde kurtarma | M |
| P5-9 | stdin/stdout (`-` desteği) | Boru hattı kullanımı | M |
| P5-10 | Gerçek ilerleme yüzdesi + ETA | Şu an 300 ms'de +5 sahte artış; video/PDF'te uzun süre %90'da bekliyor | M |
| P5-11 | ~~Nemo/Cinnamon~~ ✅ Nemo eklendi (2026-11-11) · kalan: sistem geneli kurulum (`--prefix`) | Paket yöneticileri için | M |
| P5-12 | AppImage + güncelleme kontrolü | Dağıtım kolaylığı | L |

---

## 7. Test Stratejisi (bu maddelerin ön koşulu)

1. **Birim testleri:** Mevcut 28 test korunur (yardımcı fonksiyonlar).
2. **Entegrasyon testleri** (`tests/`): gerçek binary'yi `assert_cmd` ile çalıştır; küçük fixture'lar (10–100 KB) kullan. Kapsanacaklar: P0-1…P0-6, P1-1, P1-2, P1-5.
3. **Sahte araçlar:** `tests/shims/ffmpeg`, `shims/magick`, `shims/gs` — argümanları dosyaya yazıp 0 döner. Harici komut sözleşmelerini (P1-1, P2-12, P2-6) hızlı ve deterministik test eder.
4. **Snapshot/golden:** Thunar `uca.xml` kaldırma, `--help` çıktısı, i18n mesajları.
5. **Özellik (property) testleri:** `TargetAction::parse`, `determine_output_path` (çakışma/benzersizlik), `format_bytes`.
6. **CI matrisi:** x86_64 + aarch64; `cargo fmt --check` + `cargo clippy -- -D warnings`; `cargo test`; GUI build; `.deb`/tarball üretimi.
7. **Doküman testi:** README'deki format tablosundan üretilen test — her çift gerçekten destekleniyor mu?
8. **Elle kabul kontrolü:** §10'daki repro seti her sürümde çalıştırılır.

---

## 8. Yol Haritası

| Sprint | İçerik | Çıktı |
|---|---|---|
| **S1 — Veri güvenliği** | P0-1, P0-2, P0-3, P0-4, P0-5, P0-6 + entegrasyon testleri | "Kullanıcı verisi bozulmaz" garantisi — **tamamlandı** |
| **S2 — Tutarlılık** | P1-1…P1-6, P1-9, P2-12 | Ayar/i18n/paralellik/kurulum düzeltmeleri |
| **S3 — Deneyim** | P1-7, P1-8, P2-1…P2-11, P3-1…P3-15 | i18n, GUI, motor dayanıklılığı |
| **S4 — Dağıtım** | P4-1…P4-12, P5-1…P5-6 | Paketler, CI, ilk yeni özellikler |

Sıralama kuralı: aynı dosyaya dokunan maddeler birlikte yapılır (`converter/mod.rs` → P0-1, P1-2, P1-4; `config` → P1-3, P3-12; `i18n` → P1-7, P3-14).

---

## 9. Definition of Done

- [ ] Kod + test aynı PR'da; testler CI'da yeşil.
- [ ] `cargo fmt --check` ve `cargo clippy -- -D warnings` temiz.
- [ ] Kullanıcıya görünen tüm yeni metinler i18n anahtarıyla eklendi.
- [ ] Hata yolları çıkış koduna yansıyor (0 = başarı).
- [ ] Dosya yazımları atomik; hata/iptalde artık dosya kalmıyor.
- [ ] README/`--help`/website, davranışla aynı.
- [ ] Yeni davranış için elle repro komutu §10'a eklendi.

---

## 10. Ek — Doğrulama Repro Seti

Her sürümde çalıştırılacak, beklenen sonuçları yorumda:

```bash
# P0-1: çakışma → 3 ayrı çıktı olmalı
konvert -t webp -o /tmp/out a/photo.png b/photo.jpg
ls /tmp/out            # en az 2 farklı dosya

# P0-2: Thunar kaldırma → kullanıcı action'ı ve <actions> korunmalı
HOME=/tmp/kh konvert --install-all
# uca.xml'e bir kullanıcı action'ı ekle
HOME=/tmp/kh konvert --uninstall-all
xmllint --noout /tmp/kh/.config/Thunar/uca.xml && grep -c "user-terminal" /tmp/kh/.config/Thunar/uca.xml   # 1

# P0-3: PDF hedef boyut → ≤1MB veya hata
konvert --target-size 1mb big.pdf && stat -c%s big_compressed.pdf    # ≤ 1048576

# P0-4: 18mb → 18MB (8MB değil)
konvert --target-size 18mb big.jpg && stat -c%s big_target.jpg       # > 8*1024*1024

# P0-5: EXIF orientation=6 → döndürülmüş boyut
konvert -t png oriented.jpg && identify -format "%wx%h" oriented.png  # 100x200

# P0-6: yabancı PID korunmalı
sleep 60 & echo $! > ~/.cache/konvert/running.pid
konvert --cancel; kill -0 $! && echo "OK: süreç yaşıyor"

# P1-5: kurulum hatası → exit ≠ 0
HOME=/proc/sys konvert --install-all; echo $?        # 1
```

---

## 11. Arşiv — Tamamlanan Optimizasyonlar (v0.2.0)

Önceki roadmap'in tüm maddeleri `[x]`: Rayon paralel batch, LTO/strip profili, `BufWriter` I/O, in-memory JXL piping, native metadata strip, `OnceLock` dil önbelleği, `VecDeque` geçmiş, `-threads 0` (belgelendiği gibi uygulanmadı → P1-4'te düzeltilecek), `--install-all` bağlantısı.

**Kalan performans notları:** `-threads 2` encode'u sınırlamıyor (P1-4); `compress_image_to_size` 16 kez ImageMagick başlatıyor (P0-3'teki ayar merdiveni deseniyle azaltılabilir); `merge_images_to_pdf` tek süreçte yüzlerce argüman (P2-11).

---

## 12. İyi Olanlar — Regresyon Yaratmadan Korunacaklar

- Komutlar `Command::arg` ile çalıştırılıyor; shell interpolation yok.
- `-` ile başlayan dosya adlarına karşı `sanitize_cli_path` koruması.
- ZIP/XML okumalarında 50 MB sınırı; `zip` crate ile path traversal yok.
- `anyhow` bağlamları kullanıcıya anlamlı hata zinciri veriyor.
- KIO ServiceMenus kategorilere ayrılmış; MIME kirliliği yok.
- `clippy` temiz, LTO/strip ile küçük binary.
- Action ikonları (`assets/icons`) kodla tutarlı.
