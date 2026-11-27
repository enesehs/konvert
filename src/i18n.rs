use std::sync::OnceLock;

static CURRENT_LANG: OnceLock<Language> = OnceLock::new();

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    En,
    Tr,
    De,
    Fr,
    Es,
    It,
    Ru,
    Zh,
    Ja,
}

impl Language {
    pub fn current() -> Self {
        *CURRENT_LANG.get_or_init(Self::detect_from_env)
    }

    /// Ayar dosyasındaki dili (varsa) uygular; yoksa sistem yerelini kullanır.
    /// İlk çağrıdan sonra yapılan çağrılar yok sayılır (OnceLock).
    pub fn init(language_code: Option<&str>) {
        let lang = language_code
            .and_then(Self::from_code)
            .unwrap_or_else(Self::detect_from_env);
        let _ = CURRENT_LANG.set(lang);
    }

    fn from_code(code: &str) -> Option<Self> {
        let code = code.trim().to_lowercase();
        let code = code.split([':', '.', '@']).next().unwrap_or("");
        let code = code.split(['_', '-']).next().unwrap_or("");
        match code {
            "tr" => Some(Language::Tr),
            "de" => Some(Language::De),
            "fr" => Some(Language::Fr),
            "es" => Some(Language::Es),
            "it" => Some(Language::It),
            "ru" => Some(Language::Ru),
            "zh" => Some(Language::Zh),
            "ja" => Some(Language::Ja),
            "en" => Some(Language::En),
            _ => None,
        }
    }

    fn is_c_posix_locale(val: &str) -> bool {
        let upper = val.trim().to_ascii_uppercase();
        upper == "C" || upper == "POSIX" || upper.starts_with("C.") || upper.starts_with("POSIX.")
    }

    fn detect_from_env() -> Self {
        Self::detect_from_env_with(|k| std::env::var(k).ok())
    }

    fn detect_from_env_with<F>(get: F) -> Self
    where
        F: Fn(&str) -> Option<String>,
    {
        // GNU gettext sırası: LC_ALL / LC_MESSAGES / LANG 'C' veya 'POSIX' ise LANGUAGE yok sayılır.
        let base_locale = ["LC_ALL", "LC_MESSAGES", "LANG"].iter().find_map(|key| {
            get(key)
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty())
        });
        if base_locale.as_deref().is_some_and(Self::is_c_posix_locale) {
            return Language::En;
        }

        if let Some(language_list) = get("LANGUAGE") {
            for candidate in language_list.split(':') {
                let trimmed = candidate.trim();
                if trimmed.is_empty() {
                    continue;
                }
                if Self::is_c_posix_locale(trimmed) {
                    return Language::En;
                }
                if let Some(lang) = Self::from_code(trimmed) {
                    return lang;
                }
            }
        }

        base_locale
            .as_deref()
            .and_then(Self::from_code)
            .unwrap_or(Language::En)
    }
}

/// Ayar dosyasındaki dili uygular (varsa). `t()` çağrılarından önce çağrılmalı.
pub fn init(language_code: Option<&str>) {
    Language::init(language_code);
}

pub fn t(key: &str) -> &'static str {
    t_lang(key, Language::current())
}

pub fn t_lang(key: &str, lang: Language) -> &'static str {
    match (key, lang) {
        // CLI & Common Status / Actions
        ("status_ok", Language::Zh) => "正常",
        ("status_ok", _) => "OK",

        ("status_missing", Language::Tr) => "EKSİK",
        ("status_missing", Language::De) => "FEHLT",
        ("status_missing", Language::Fr) => "MANQUANT",
        ("status_missing", Language::Es) => "FALTANTE",
        ("status_missing", Language::It) => "MANCANTE",
        ("status_missing", Language::Ru) => "ОТСУТСТВУЕТ",
        ("status_missing", Language::Zh) => "缺失",
        ("status_missing", Language::Ja) => "未検出",
        ("status_missing", _) => "MISSING",

        ("status_required", Language::Tr) => "Zorunlu",
        ("status_required", Language::De) => "Erforderl.",
        ("status_required", Language::Fr) => "Requis",
        ("status_required", Language::Es) => "Requerido",
        ("status_required", Language::It) => "Richiesto",
        ("status_required", Language::Ru) => "Обяз.",
        ("status_required", Language::Zh) => "必需",
        ("status_required", Language::Ja) => "必須",
        ("status_required", _) => "Required",

        ("status_optional", Language::Tr) => "Opsiyonel",
        ("status_optional", Language::De) => "Optional",
        ("status_optional", Language::Fr) => "Optionnel",
        ("status_optional", Language::Es) => "Opcional",
        ("status_optional", Language::It) => "Opzionale",
        ("status_optional", Language::Ru) => "Опц.",
        ("status_optional", Language::Zh) => "可选",
        ("status_optional", Language::Ja) => "任意",
        ("status_optional", _) => "Optional",

        // Doctor Report
        ("doctor_header", Language::Tr) => "=== Konvert Sistem Teşhisi (--doctor) ===",
        ("doctor_header", Language::De) => "=== Konvert Systemdiagnose (--doctor) ===",
        ("doctor_header", Language::Fr) => "=== Diagnostic système Konvert (--doctor) ===",
        ("doctor_header", Language::Es) => "=== Diagnóstico del sistema Konvert (--doctor) ===",
        ("doctor_header", _) => "=== Konvert System Diagnostics (--doctor) ===",

        ("doctor_os", Language::Tr) => "İşletim Sistemi:    ",
        ("doctor_os", Language::De) => "Betriebssystem:     ",
        ("doctor_os", Language::Fr) => "Système :           ",
        ("doctor_os", Language::Es) => "Sistema operativo:  ",
        ("doctor_os", _) => "Operating System:   ",

        ("doctor_desktop", Language::Tr) => "Masaüstü Ortamı:    ",
        ("doctor_desktop", Language::De) => "Desktop-Umgebung:   ",
        ("doctor_desktop", Language::Fr) => "Bureau :            ",
        ("doctor_desktop", Language::Es) => "Entorno de escrit.: ",
        ("doctor_desktop", _) => "Desktop Env:        ",

        ("doctor_hwaccel", Language::Tr) => "Donanım Hızlandırma:",
        ("doctor_hwaccel", Language::De) => "Hardwarebeschl.:    ",
        ("doctor_hwaccel", Language::Fr) => "Accél. matérielle : ",
        ("doctor_hwaccel", Language::Es) => "Aceleración HW:     ",
        ("doctor_hwaccel", _) => "Hardware Accel:     ",

        ("doctor_config", Language::Tr) => "Ayar Dosyası:       ",
        ("doctor_config", Language::De) => "Konfigurationsdatei:",
        ("doctor_config", Language::Fr) => "Fichier config :    ",
        ("doctor_config", Language::Es) => "Archivo config.:    ",
        ("doctor_config", _) => "Config File:        ",

        ("doctor_unknown_desktop", Language::Tr) => "Bilinmeyen Masaüstü Ortamı",
        ("doctor_unknown_desktop", Language::De) => "Unbekannte Desktop-Umgebung",
        ("doctor_unknown_desktop", Language::Fr) => "Environnement de bureau inconnu",
        ("doctor_unknown_desktop", Language::Es) => "Entorno de escritorio desconocido",
        ("doctor_unknown_desktop", _) => "Unknown Desktop Environment",

        ("doctor_hwaccel_no_ffmpeg", Language::Tr) => "FFmpeg eksik",
        ("doctor_hwaccel_no_ffmpeg", Language::De) => "FFmpeg fehlt",
        ("doctor_hwaccel_no_ffmpeg", Language::Fr) => "FFmpeg manquant",
        ("doctor_hwaccel_no_ffmpeg", Language::Es) => "Falta FFmpeg",
        ("doctor_hwaccel_no_ffmpeg", _) => "FFmpeg missing",

        ("doctor_hwaccel_cpu", Language::Tr) => "CPU (Yazılım kodlama)",
        ("doctor_hwaccel_cpu", Language::De) => "CPU (Software-Kodierung)",
        ("doctor_hwaccel_cpu", Language::Fr) => "CPU (encodage logiciel)",
        ("doctor_hwaccel_cpu", Language::Es) => "CPU (codificación por software)",
        ("doctor_hwaccel_cpu", _) => "CPU (Software encoding)",

        ("doctor_col_component", Language::Tr) => "BİLEŞEN",
        ("doctor_col_component", Language::De) => "KOMPONENTE",
        ("doctor_col_component", Language::Fr) => "COMPOSANT",
        ("doctor_col_component", Language::Es) => "COMPONENTE",
        ("doctor_col_component", _) => "COMPONENT",

        ("doctor_col_status", Language::Tr) => "DURUM",
        ("doctor_col_status", Language::De) => "STATUS",
        ("doctor_col_status", Language::Fr) => "ÉTAT",
        ("doctor_col_status", Language::Es) => "ESTADO",
        ("doctor_col_status", _) => "STATUS",

        ("doctor_col_type", Language::Tr) => "TÜR",
        ("doctor_col_type", Language::De) => "TYP",
        ("doctor_col_type", Language::Fr) => "TYPE",
        ("doctor_col_type", Language::Es) => "TIPO",
        ("doctor_col_type", _) => "TYPE",

        ("doctor_col_detail", Language::Tr) => "AÇIKLAMA / SÜRÜM",
        ("doctor_col_detail", Language::De) => "BESCHREIBUNG / VERSION",
        ("doctor_col_detail", Language::Fr) => "DESCRIPTION / VERSION",
        ("doctor_col_detail", Language::Es) => "DESCRIPCIÓN / VERSIÓN",
        ("doctor_col_detail", _) => "DESCRIPTION / VERSION",

        ("doctor_all_ok", Language::Tr) => {
            "[OK] Tüm çekirdek ve isteğe bağlı araçlar kurulu. Konvert tüm özellikleriyle hazır."
        }
        ("doctor_all_ok", Language::De) => {
            "[OK] Alle erforderlichen und optionalen Werkzeuge sind installiert."
        }
        ("doctor_all_ok", Language::Fr) => {
            "[OK] Tous les outils requis et optionnels sont installés."
        }
        ("doctor_all_ok", Language::Es) => {
            "[OK] Todas las herramientas requeridas y opcionales están instaladas."
        }
        ("doctor_all_ok", _) => {
            "[OK] All required and optional tools are installed. Konvert is ready."
        }

        ("doctor_missing_optional", Language::Tr) => {
            "[WARN] Temel dönüşümler çalışabilir, ancak bazı isteğe bağlı bileşenler eksik."
        }
        ("doctor_missing_optional", Language::De) => {
            "[WARN] Grundfunktionen verfügbar, jedoch fehlen einige optionale Werkzeuge."
        }
        ("doctor_missing_optional", Language::Fr) => {
            "[WARN] Conversions de base prêtes, mais certains outils optionnels manquent."
        }
        ("doctor_missing_optional", Language::Es) => {
            "[WARN] Las conversiones básicas funcionan, pero faltan herramientas opcionales."
        }
        ("doctor_missing_optional", _) => {
            "[WARN] Core conversions work, but some optional tools are missing."
        }

        ("doctor_missing_required", Language::Tr) => {
            "[FAIL] Zorunlu bağımlılıklar eksik! Video ve ses dönüşümleri çalışmayacaktır."
        }
        ("doctor_missing_required", Language::De) => {
            "[FAIL] Erforderliche Abhängigkeiten fehlen! Video-/Audiokonvertierung deaktiviert."
        }
        ("doctor_missing_required", Language::Fr) => {
            "[FAIL] Dépendances requises manquantes ! Conversions vidéo/audio impossibles."
        }
        ("doctor_missing_required", Language::Es) => {
            "[FAIL] ¡Faltan dependencias requeridas! Video y audio no funcionarán."
        }
        ("doctor_missing_required", _) => {
            "[FAIL] Required dependencies are missing! Video and audio conversions will not work."
        }

        ("doctor_install_hint", Language::Tr) => "Eksik paketleri kurmak için komutlar:",
        ("doctor_install_hint", Language::De) => "Befehle zum Installieren fehlender Pakete:",
        ("doctor_install_hint", Language::Fr) => "Commandes pour installer les paquets manquants :",
        ("doctor_install_hint", Language::Es) => "Comandos para instalar paquetes faltantes:",
        ("doctor_install_hint", _) => "Commands to install missing packages:",

        ("err_missing_deps", Language::Tr) => {
            "Zorunlu bağımlılıklar eksik; video/ses dönüşümleri çalışmaz."
        }
        ("err_missing_deps", Language::De) => {
            "Erforderliche Abhängigkeiten fehlen; Konvertierungen können nicht fortgesetzt werden."
        }
        ("err_missing_deps", Language::Fr) => {
            "Dépendances requises manquantes; les conversions ne peuvent pas continuer."
        }
        ("err_missing_deps", Language::Es) => {
            "Faltan dependencias requeridas; no se pueden realizar las conversiones."
        }
        ("err_missing_deps", Language::It) => {
            "Dipendenze necessarie mancanti; le conversioni non possono procedere."
        }
        ("err_missing_deps", Language::Ru) => {
            "Необходимые зависимости отсутствуют; конвертация невозможна."
        }
        ("err_missing_deps", Language::Zh) => "缺少必需的依赖项；无法继续转换。",
        ("err_missing_deps", Language::Ja) => {
            "必要な依存関係が見つかりません。変換を続行できません。"
        }
        ("err_missing_deps", _) => "Required dependencies are missing; conversions cannot proceed.",

        ("err_no_integrations_installed", Language::Tr) => {
            "Hiçbir masaüstü entegrasyonu kurulamadı."
        }
        ("err_no_integrations_installed", Language::De) => {
            "Es konnte keine Desktop-Integration installiert werden."
        }
        ("err_no_integrations_installed", Language::Fr) => {
            "Aucune intégration de bureau n'a pu être installée."
        }
        ("err_no_integrations_installed", Language::Es) => {
            "No se pudo instalar ninguna integración de escritorio."
        }
        ("err_no_integrations_installed", Language::It) => {
            "Nessuna integrazione desktop installata."
        }
        ("err_no_integrations_installed", Language::Ru) => {
            "Не удалось установить интеграции рабочего стола."
        }
        ("err_no_integrations_installed", Language::Zh) => "未能安装任何桌面集成。",
        ("err_no_integrations_installed", Language::Ja) => {
            "デスクトップ統合をインストールできませんでした。"
        }
        ("err_no_integrations_installed", _) => "No desktop integration could be installed.",

        ("action_compress", Language::Tr) => "Sıkıştırma",
        ("action_compress", Language::De) => "Komprimieren",
        ("action_compress", Language::Fr) => "Compresser",
        ("action_compress", Language::Es) => "Comprimir",
        ("action_compress", Language::It) => "Comprimi",
        ("action_compress", Language::Ru) => "Сжатие",
        ("action_compress", Language::Zh) => "压缩",
        ("action_compress", Language::Ja) => "圧縮",
        ("action_compress", _) => "Compress",

        ("action_resize", Language::Tr) => "Boyutlandırma",
        ("action_resize", Language::De) => "Größe ändern",
        ("action_resize", Language::Fr) => "Redimensionner",
        ("action_resize", Language::Es) => "Redimensionar",
        ("action_resize", Language::It) => "Ridimensiona",
        ("action_resize", Language::Ru) => "Изменение размера",
        ("action_resize", Language::Zh) => "缩放",
        ("action_resize", Language::Ja) => "リサイズ",
        ("action_resize", _) => "Resize",

        ("action_strip_metadata", Language::Tr) => "Metadata Temizleme",
        ("action_strip_metadata", Language::De) => "Metadaten entfernen",
        ("action_strip_metadata", Language::Fr) => "Supprimer les métadonnées",
        ("action_strip_metadata", Language::Es) => "Eliminar metadatos",
        ("action_strip_metadata", Language::It) => "Rimuovi metadati",
        ("action_strip_metadata", Language::Ru) => "Очистить метаданные",
        ("action_strip_metadata", Language::Zh) => "清除元数据",
        ("action_strip_metadata", Language::Ja) => "メタデータ削除",
        ("action_strip_metadata", _) => "Strip Metadata",

        ("action_target_size", Language::Tr) => "Hedef Boyut",
        ("action_target_size", Language::De) => "Zielgröße",
        ("action_target_size", Language::Fr) => "Taille cible",
        ("action_target_size", Language::Es) => "Tamaño objetivo",
        ("action_target_size", Language::It) => "Dimensione target",
        ("action_target_size", Language::Ru) => "Целевой размер",
        ("action_target_size", Language::Zh) => "目标大小",
        ("action_target_size", Language::Ja) => "目標サイズ",
        ("action_target_size", _) => "Target Size",

        ("action_web_opt", Language::Tr) => "Web Optimizasyonu",
        ("action_web_opt", Language::De) => "Web-Optimierung",
        ("action_web_opt", Language::Fr) => "Optimisation Web",
        ("action_web_opt", Language::Es) => "Optimizar para Web",
        ("action_web_opt", Language::It) => "Ottimizzazione Web",
        ("action_web_opt", Language::Ru) => "Оптимизация для Web",
        ("action_web_opt", Language::Zh) => "网页优化",
        ("action_web_opt", Language::Ja) => "ウェブ最適化",
        ("action_web_opt", _) => "Web Optimize",

        ("err_unexpected_exit", Language::Tr) => "Dönüştürme işlemi beklenmeyen şekilde sonlandı",
        ("err_unexpected_exit", Language::De) => "Konvertierungsprozess unerwartet beendet",
        ("err_unexpected_exit", Language::Fr) => {
            "Le processus de conversion s'est terminé de manière inattendue"
        }
        ("err_unexpected_exit", Language::Es) => {
            "El proceso de conversión finalizó inesperadamente"
        }
        ("err_unexpected_exit", Language::It) => {
            "Processo di conversione terminato in modo imprevisto"
        }
        ("err_unexpected_exit", Language::Ru) => "Процесс конвертации неожиданно завершился",
        ("err_unexpected_exit", Language::Zh) => "转换进程意外终止",
        ("err_unexpected_exit", Language::Ja) => "変換処理が予期せず終了しました",
        ("err_unexpected_exit", _) => "Conversion process terminated unexpectedly",

        ("err_input_not_found", Language::Tr) => "Girdi dosyası mevcut değil",
        ("err_input_not_found", Language::De) => "Eingabedatei existiert nicht",
        ("err_input_not_found", Language::Fr) => "Le fichier d'entrée n'existe pas",
        ("err_input_not_found", Language::Es) => "El archivo de entrada no existe",
        ("err_input_not_found", Language::It) => "Il file di input non esiste",
        ("err_input_not_found", Language::Ru) => "Входной файл не существует",
        ("err_input_not_found", Language::Zh) => "输入文件不存在",
        ("err_input_not_found", Language::Ja) => "入力ファイルが存在しません",
        ("err_input_not_found", _) => "Input file does not exist",

        ("err_unsupported_conversion", Language::Tr) => "Desteklenmeyen dönüştürme",
        ("err_unsupported_conversion", Language::De) => "Nicht unterstützte Konvertierung",
        ("err_unsupported_conversion", Language::Fr) => "Conversion non prise en charge",
        ("err_unsupported_conversion", Language::Es) => "Conversión no compatible",
        ("err_unsupported_conversion", Language::It) => "Conversione non supportata",
        ("err_unsupported_conversion", Language::Ru) => "Неподдерживаемая конвертация",
        ("err_unsupported_conversion", Language::Zh) => "不支持的转换",
        ("err_unsupported_conversion", Language::Ja) => "サポートされていない変換",
        ("err_unsupported_conversion", _) => "Unsupported conversion",

        ("err_jxl_decode", Language::Tr) => "JXL görseli çözülemedi",
        ("err_jxl_decode", _) => "Failed to decode JXL image",

        ("err_svg_rasterize", Language::Tr) => "SVG görseli rasterleştirilemedi",
        ("err_svg_rasterize", _) => "Failed to rasterize SVG image",

        ("err_heic_decode", Language::Tr) => "HEIC/HEIF görseli çözülemedi",
        ("err_heic_decode", _) => "Failed to decode HEIC/HEIF image",

        ("err_svg_convert", Language::Tr) => "SVG formatına dönüştürülemedi",
        ("err_svg_convert", _) => "Failed to convert to SVG format",

        ("err_heic_convert", Language::Tr) => "HEIC formatına dönüştürülemedi",
        ("err_heic_convert", _) => "Failed to convert to HEIC format",

        ("err_cjxl_failed", Language::Tr) => "cjxl ile JPEG XL dönüştürme başarısız oldu",
        ("err_cjxl_failed", _) => "JPEG XL conversion via cjxl failed",

        ("err_no_images_to_merge", Language::Tr) => "Birleştirilecek görsel bulunamadı",
        ("err_no_images_to_merge", _) => "No images found to merge",

        ("err_gs_merge_pdf", Language::Tr) => "Ghostscript ile PDF dosyaları birleştirilemedi",
        ("err_gs_merge_pdf", _) => "Failed to merge PDF files via Ghostscript",

        ("err_gs_merge_parts", Language::Tr) => "Ghostscript ile parçalar birleştirilemedi",
        ("err_gs_merge_parts", _) => "Failed to merge parts via Ghostscript",

        ("err_target_size_zero", Language::Tr) => "Hedef dosya boyutu 0 olamaz",
        ("err_target_size_zero", _) => "Target file size cannot be zero",

        ("err_file_empty_or_unreadable", Language::Tr) => "Dosya boş veya boyutu okunamadı",
        ("err_file_empty_or_unreadable", _) => "File is empty or size could not be determined",

        ("err_file_already_smaller", Language::Tr) => "Dosya zaten hedef boyuttan daha küçük",
        ("err_file_already_smaller", _) => "File is already smaller than the target size",

        ("err_target_size_not_reached", Language::Tr) => "Hedef boyuta sıkıştırılamadı",
        ("err_target_size_not_reached", _) => "Could not compress to target size",

        ("err_soffice_failed", Language::Tr) => {
            "LibreOffice (soffice) ile PDF dönüşümü başarısız oldu veya soffice sistemde kurulu değil"
        }
        ("err_soffice_failed", _) => {
            "LibreOffice (soffice) PDF conversion failed or soffice is not installed"
        }

        ("err_doc_size_limit", Language::Tr) => "Belge boyutu güvenlik sınırını aşıyor",
        ("err_doc_size_limit", _) => "Document size exceeds security limit",

        ("err_pdf_render_images", Language::Tr) => "PDF sayfaları görsele dönüştürülemedi",
        ("err_pdf_render_images", _) => "Failed to render PDF pages to images",

        ("err_udf_read_text", Language::Tr) => "UDF belgesinden metin okunamadı",
        ("err_udf_read_text", _) => "Failed to extract text from UDF document",

        ("err_docx_read_text", Language::Tr) => "DOCX belgesinden metin okunamadı",
        ("err_docx_read_text", _) => "Failed to extract text from DOCX document",

        ("err_gs_compress_failed", Language::Tr) => "Ghostscript PDF sıkıştırma başarısız oldu",
        ("err_gs_compress_failed", _) => "Ghostscript PDF compression failed",

        ("err_gs_not_installed", Language::Tr) => {
            "PDF hedef boyut sıkıştırması için ghostscript (gs) kurulu olmalı, ancak bulunamadı"
        }
        ("err_gs_not_installed", _) => {
            "Ghostscript (gs) is required for PDF compression but not installed"
        }

        ("err_ffmpeg_failed", Language::Tr) => "ffmpeg işlemi başarısız sonuçlandı",
        ("err_ffmpeg_failed", _) => "FFmpeg process failed",

        ("err_extract_frames", Language::Tr) => "Video kareleri PNG olarak çıkarılamadı",
        ("err_extract_frames", _) => "Failed to extract video frames as PNG",

        ("err_video_strip_metadata", Language::Tr) => "Video metadata temizleme başarısız oldu",
        ("err_video_strip_metadata", _) => "Failed to strip video metadata",

        ("err_video_duration", Language::Tr) => "Video süresi okunamadı",
        ("err_video_duration", _) => "Could not read video duration",

        ("err_invalid_duration", Language::Tr) => "Geçersiz video süresi",
        ("err_invalid_duration", _) => "Invalid video duration",

        ("err_video_target_size_failed", Language::Tr) => {
            "Video hedef boyuta sıkıştırılamadı: hiçbir kodlama denemesi sonuç üretmedi"
        }
        ("err_video_target_size_failed", _) => {
            "Failed to compress video to target size: no encoding attempt succeeded"
        }

        ("err_web_video_opt", Language::Tr) => "Web video optimizasyonu başarısız oldu",
        ("err_web_video_opt", _) => "Web video optimization failed",

        ("err_audio_to_video", Language::Tr) => "Sesten video oluşturma başarısız oldu",
        ("err_audio_to_video", _) => "Failed to generate video from audio",

        ("err_audio_strip_metadata", Language::Tr) => "Ses metadata temizleme başarısız oldu",
        ("err_audio_strip_metadata", _) => "Failed to strip audio metadata",

        ("err_imagemagick_failed", Language::Tr) => "ImageMagick komutu başarısız oldu",
        ("err_imagemagick_failed", _) => "ImageMagick command failed",

        ("err_unsupported_compression", Language::Tr) => "Bu format için sıkıştırma desteklenmiyor",
        ("err_unsupported_compression", _) => "Compression is not supported for this format",

        ("err_invalid_resize_spec", Language::Tr) => "Geçersiz boyutlandırma parametresi",
        ("err_invalid_resize_spec", _) => "Invalid resize specification",

        ("err_unsupported_strip_meta", Language::Tr) => {
            "Metadata temizleme bu format için desteklenmiyor"
        }
        ("err_unsupported_strip_meta", _) => "Metadata stripping is not supported for this format",

        ("err_unsupported_resize", Language::Tr) => {
            "Yeniden boyutlandırma sadece görseller için geçerlidir"
        }
        ("err_unsupported_resize", _) => "Resizing is only supported for images",

        ("err_unsupported_web_opt", Language::Tr) => {
            "Web optimizasyonu sadece görsel ve videolar için geçerlidir"
        }
        ("err_unsupported_web_opt", _) => {
            "Web optimization is only supported for image and video formats"
        }

        ("err_unsupported_audio_extract", Language::Tr) => {
            "Videodan ses ayıklama sadece video dosyaları için geçerlidir"
        }
        ("err_unsupported_audio_extract", _) => {
            "Audio extraction is only supported for video files"
        }

        ("err_unsupported_frame_extract", Language::Tr) => {
            "Kare çıkarma sadece video dosyaları için geçerlidir"
        }
        ("err_unsupported_frame_extract", _) => {
            "Frame extraction is only supported for video files"
        }

        ("err_unsupported_gif_convert", Language::Tr) => {
            "GIF dönüşümü sadece video dosyaları için geçerlidir"
        }
        ("err_unsupported_gif_convert", _) => "GIF conversion is only supported for video files",

        ("err_unsupported_audio_to_video", Language::Tr) => {
            "Ses dalgası videosu oluşturmak için bir ses dosyası gereklidir"
        }
        ("err_unsupported_audio_to_video", _) => "Audio waveform video requires a valid audio file",

        ("err_job_cancelled", Language::Tr) => "Dönüştürme kullanıcı tarafından iptal edildi",
        ("err_job_cancelled", Language::De) => "Konvertierung vom Benutzer abgebrochen",
        ("err_job_cancelled", Language::Fr) => "Conversion annulée par l'utilisateur",
        ("err_job_cancelled", Language::Es) => "Conversión cancelada por el usuario",
        ("err_job_cancelled", Language::It) => "Conversione annullata dall'utente",
        ("err_job_cancelled", Language::Ru) => "Конвертация отменена пользователем",
        ("err_job_cancelled", Language::Zh) => "用户已取消转换",
        ("err_job_cancelled", Language::Ja) => "ユーザーによって変換がキャンセルされました",
        ("err_job_cancelled", _) => "Conversion cancelled by user",

        // Notifications
        ("notify_done_single", Language::Tr) => "Dönüştürme Tamamlandı",
        ("notify_done_single", Language::De) => "Konvertierung abgeschlossen",
        ("notify_done_single", Language::Fr) => "Conversion terminée",
        ("notify_done_single", Language::Es) => "Conversión completada",
        ("notify_done_single", Language::It) => "Conversione completata",
        ("notify_done_single", Language::Ru) => "Конвертация завершена",
        ("notify_done_single", Language::Zh) => "转换完成",
        ("notify_done_single", Language::Ja) => "変換完了",
        ("notify_done_single", _) => "Conversion Completed",

        ("notify_done_batch", Language::Tr) => "Toplu Dönüştürme Tamamlandı",
        ("notify_done_batch", Language::De) => "Stapelkonvertierung abgeschlossen",
        ("notify_done_batch", Language::Fr) => "Conversion par lot terminée",
        ("notify_done_batch", Language::Es) => "Conversión por lotes completada",
        ("notify_done_batch", Language::It) => "Conversione batch completata",
        ("notify_done_batch", Language::Ru) => "Пакетная конвертация завершена",
        ("notify_done_batch", Language::Zh) => "批量转换完成",
        ("notify_done_batch", Language::Ja) => "一括変換完了",
        ("notify_done_batch", _) => "Batch Conversion Complete",

        ("label_files_converted", Language::Tr) => "dosya dönüştürüldü",
        ("label_files_converted", Language::De) => "Dateien konvertiert",
        ("label_files_converted", Language::Fr) => "fichiers convertis",
        ("label_files_converted", Language::Es) => "archivos convertidos",
        ("label_files_converted", Language::It) => "file convertiti",
        ("label_files_converted", Language::Ru) => "файлов конвертировано",
        ("label_files_converted", Language::Zh) => "个文件已转换",
        ("label_files_converted", Language::Ja) => "個のファイルを変換完了",
        ("label_files_converted", _) => "files converted",

        ("label_succeeded", Language::Tr) => "başarılı",
        ("label_succeeded", Language::De) => "erfolgreich",
        ("label_succeeded", Language::Fr) => "réussi",
        ("label_succeeded", Language::Es) => "completado",
        ("label_succeeded", Language::It) => "riuscito",
        ("label_succeeded", Language::Ru) => "успешно",
        ("label_succeeded", Language::Zh) => "成功",
        ("label_succeeded", Language::Ja) => "成功",
        ("label_succeeded", _) => "succeeded",

        ("label_failed", Language::Tr) => "başarısız",
        ("label_failed", Language::De) => "fehlgeschlagen",
        ("label_failed", Language::Fr) => "échoué",
        ("label_failed", Language::Es) => "fallido",
        ("label_failed", Language::It) => "non riuscito",
        ("label_failed", Language::Ru) => "ошибка",
        ("label_failed", Language::Zh) => "失败",
        ("label_failed", Language::Ja) => "失敗",
        ("label_failed", _) => "failed",

        ("notify_partial_batch", Language::Tr) => "Toplu Dönüştürme Kısmen Tamamlandı",
        ("notify_partial_batch", Language::De) => "Stapelkonvertierung teilweise abgeschlossen",
        ("notify_partial_batch", Language::Fr) => "Conversion partielle terminée",
        ("notify_partial_batch", Language::Es) => "Conversión por lotes parcialmente completada",
        ("notify_partial_batch", Language::It) => "Conversione parziale completata",
        ("notify_partial_batch", Language::Ru) => "Пакетная конвертация завершена частично",
        ("notify_partial_batch", Language::Zh) => "部分文件转换完成",
        ("notify_partial_batch", Language::Ja) => "一部のファイルのみ変換完了",
        ("notify_partial_batch", _) => "Batch Conversion Partially Completed",

        ("notify_failed", Language::Tr) => "Dönüştürme Başarısız",
        ("notify_failed", Language::De) => "Konvertierung fehlgeschlagen",
        ("notify_failed", Language::Fr) => "Échec de la conversion",
        ("notify_failed", Language::Es) => "Error de conversión",
        ("notify_failed", Language::It) => "Conversione non riuscita",
        ("notify_failed", Language::Ru) => "Ошибка конвертации",
        ("notify_failed", Language::Zh) => "转换失败",
        ("notify_failed", Language::Ja) => "変換に失敗しました",
        ("notify_failed", _) => "Conversion Failed",

        ("notify_cancelled", Language::Tr) => "Dönüştürme İptal Edildi",
        ("notify_cancelled", Language::De) => "Konvertierung abgebrochen",
        ("notify_cancelled", Language::Fr) => "Conversion annulée",
        ("notify_cancelled", Language::Es) => "Conversión cancelada",
        ("notify_cancelled", Language::It) => "Conversione annullata",
        ("notify_cancelled", Language::Ru) => "Конвертация отменена",
        ("notify_cancelled", Language::Zh) => "转换已取消",
        ("notify_cancelled", Language::Ja) => "変換がキャンセルされました",
        ("notify_cancelled", _) => "Conversion Cancelled",

        ("notify_paused", Language::Tr) => "Duraklatıldı",
        ("notify_paused", Language::De) => "Pausiert",
        ("notify_paused", Language::Fr) => "En pause",
        ("notify_paused", Language::Es) => "En pausa",
        ("notify_paused", Language::It) => "In pausa",
        ("notify_paused", Language::Ru) => "Пауза",
        ("notify_paused", Language::Zh) => "已暂停",
        ("notify_paused", Language::Ja) => "一時停止",
        ("notify_paused", _) => "Paused",

        ("notify_integ_title", Language::Tr) => "Konvert Entegrasyonu",
        ("notify_integ_title", Language::De) => "Konvert-Integration",
        ("notify_integ_title", Language::Fr) => "Intégration de Konvert",
        ("notify_integ_title", Language::Es) => "Integración de Konvert",
        ("notify_integ_title", Language::It) => "Integrazione di Konvert",
        ("notify_integ_title", Language::Ru) => "Интеграция Konvert",
        ("notify_integ_title", Language::Zh) => "Konvert 集成",
        ("notify_integ_title", Language::Ja) => "Konvert 統合",
        ("notify_integ_title", _) => "Konvert Integration",

        ("notify_field_source", Language::Tr) => "Kaynak",
        ("notify_field_source", Language::De) => "Quelle",
        ("notify_field_source", Language::Fr) => "Source",
        ("notify_field_source", Language::Es) => "Origen",
        ("notify_field_source", _) => "Source",

        ("notify_field_target", Language::Tr) => "Hedef",
        ("notify_field_target", Language::De) => "Ziel",
        ("notify_field_target", Language::Fr) => "Cible",
        ("notify_field_target", Language::Es) => "Destino",
        ("notify_field_target", _) => "Target",

        ("notify_status_converting", Language::Tr) => "Dönüştürülüyor",
        ("notify_status_converting", Language::De) => "Konvertierung",
        ("notify_status_converting", Language::Fr) => "Conversion",
        ("notify_status_converting", Language::Es) => "Convirtiendo",
        ("notify_status_converting", _) => "Converting",

        ("notify_status_compressing", Language::Tr) => "Sıkıştırılıyor",
        ("notify_status_compressing", Language::De) => "Komprimierung",
        ("notify_status_compressing", Language::Fr) => "Compression",
        ("notify_status_compressing", Language::Es) => "Comprimiendo",
        ("notify_status_compressing", _) => "Compressing",

        ("notify_status_optimizing", Language::Tr) => "Optimize ediliyor",
        ("notify_status_optimizing", Language::De) => "Optimierung",
        ("notify_status_optimizing", Language::Fr) => "Optimisation",
        ("notify_status_optimizing", Language::Es) => "Optimizando",
        ("notify_status_optimizing", _) => "Optimizing",

        ("notify_status_resizing", Language::Tr) => "Yeniden boyutlandırılıyor",
        ("notify_status_resizing", Language::De) => "Größenänderung",
        ("notify_status_resizing", Language::Fr) => "Redimensionnement",
        ("notify_status_resizing", Language::Es) => "Redimensionando",
        ("notify_status_resizing", _) => "Resizing",

        ("notify_status_stripping", Language::Tr) => "Metadata temizleniyor",
        ("notify_status_stripping", Language::De) => "Metadaten werden entfernt",
        ("notify_status_stripping", Language::Fr) => "Suppression des métadonnées",
        ("notify_status_stripping", Language::Es) => "Eliminando metadatos",
        ("notify_status_stripping", _) => "Stripping metadata",

        ("notify_status_extracting", Language::Tr) => "Ayıklanıyor",
        ("notify_status_extracting", Language::De) => "Extraktion",
        ("notify_status_extracting", Language::Fr) => "Extraction",
        ("notify_status_extracting", Language::Es) => "Extrayendo",
        ("notify_status_extracting", _) => "Extracting",

        ("notify_status_merging", Language::Tr) => "PDF birleştiriliyor",
        ("notify_status_merging", Language::De) => "PDF wird zusammengeführt",
        ("notify_status_merging", Language::Fr) => "Fusion PDF",
        ("notify_status_merging", Language::Es) => "Combinando PDF",
        ("notify_status_merging", _) => "Merging PDF",

        ("action_email_25mb", Language::Tr) => "E-Posta (25 MB)",
        ("action_email_25mb", Language::De) => "E-Mail (25 MB)",
        ("action_email_25mb", Language::Fr) => "E-mail (25 Mo)",
        ("action_email_25mb", Language::Es) => "Correo (25 MB)",
        ("action_email_25mb", _) => "Email (25 MB)",

        // Actions (Desktop Menu)
        ("action_to_svg", Language::Tr) => "SVG Olarak Vektörleştir",
        ("action_to_svg", Language::De) => "In SVG vektorisieren",
        ("action_to_svg", Language::Fr) => "Vectoriser en SVG",
        ("action_to_svg", Language::Es) => "Vectorizar a SVG",
        ("action_to_svg", Language::It) => "Vettorizza in SVG",
        ("action_to_svg", Language::Ru) => "Векторизовать в SVG",
        ("action_to_svg", Language::Zh) => "矢量化为 SVG",
        ("action_to_svg", Language::Ja) => "SVGにベクター変換",
        ("action_to_svg", _) => "Trace to SVG (Vectorize)",

        _ => "",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_from_code() {
        assert_eq!(Language::from_code("tr_TR.UTF-8"), Some(Language::Tr));
        assert_eq!(Language::from_code("de_DE"), Some(Language::De));
        assert_eq!(Language::from_code("zh_CN"), Some(Language::Zh));
        assert_eq!(Language::from_code("ja"), Some(Language::Ja));
        assert_eq!(Language::from_code("tr:de"), Some(Language::Tr));
        assert_eq!(Language::from_code("en_US"), Some(Language::En));
        assert_eq!(Language::from_code("xx_YY"), None);
        assert_eq!(Language::from_code(""), None);
    }

    #[test]
    fn test_detect_from_env_gnu_gettext_rules() {
        let lang_fallback = Language::detect_from_env_with(|k| match k {
            "LANGUAGE" => Some("pt_BR:es:en".to_string()),
            _ => None,
        });
        assert_eq!(lang_fallback, Language::Es);

        let lc_all_c = Language::detect_from_env_with(|k| match k {
            "LANGUAGE" => Some("tr".to_string()),
            "LC_ALL" => Some("C".to_string()),
            _ => None,
        });
        assert_eq!(lc_all_c, Language::En);

        let lc_all_posix = Language::detect_from_env_with(|k| match k {
            "LANGUAGE" => Some("tr".to_string()),
            "LC_ALL" => Some("POSIX".to_string()),
            _ => None,
        });
        assert_eq!(lc_all_posix, Language::En);

        let empty_lc_all = Language::detect_from_env_with(|k| match k {
            "LC_ALL" => Some("".to_string()),
            "LANG" => Some("tr_TR.UTF-8".to_string()),
            _ => None,
        });
        assert_eq!(empty_lc_all, Language::Tr);
    }

    #[test]
    fn test_t_lang_translations() {
        assert_eq!(t_lang("status_ok", Language::En), "OK");
        assert_eq!(t_lang("status_missing", Language::Tr), "EKSİK");
        assert_eq!(t_lang("status_missing", Language::En), "MISSING");
        assert_eq!(t_lang("action_compress", Language::Tr), "Sıkıştırma");
        assert_eq!(t_lang("action_compress", Language::En), "Compress");
        assert_eq!(
            t_lang("action_to_svg", Language::Tr),
            "SVG Olarak Vektörleştir"
        );
        assert_eq!(
            t_lang("action_to_svg", Language::En),
            "Trace to SVG (Vectorize)"
        );
    }
}
