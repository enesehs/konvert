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

    fn detect_from_env() -> Self {
        // GNU sırası: LANGUAGE > LC_ALL > LC_MESSAGES > LANG.
        // Boş tanımlı değişkenler atlanır (ör. LC_ALL="" yaygın).
        for key in ["LANGUAGE", "LC_ALL", "LC_MESSAGES", "LANG"] {
            let Ok(value) = std::env::var(key) else {
                continue;
            };
            let value = value.trim();
            if value.is_empty() {
                continue;
            }
            if let Some(lang) = Self::from_code(value) {
                return lang;
            }
        }
        Language::En
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
        // App & Window
        ("app_name", _) => "Konvert",
        ("settings_title", Language::Tr) => "Konvert Ayarları",
        ("settings_title", Language::De) => "Konvert Einstellungen",
        ("settings_title", Language::Fr) => "Paramètres de Konvert",
        ("settings_title", Language::Es) => "Ajustes de Konvert",
        ("settings_title", Language::It) => "Impostazioni di Konvert",
        ("settings_title", Language::Ru) => "Настройки Konvert",
        ("settings_title", Language::Zh) => "Konvert 设置",
        ("settings_title", Language::Ja) => "Konvert 設定",
        ("settings_title", _) => "Konvert Settings",

        ("tab_general", Language::Tr) => "Genel",
        ("tab_general", Language::De) => "Allgemein",
        ("tab_general", Language::Fr) => "Général",
        ("tab_general", Language::Es) => "General",
        ("tab_general", Language::It) => "Generale",
        ("tab_general", Language::Ru) => "Общие",
        ("tab_general", Language::Zh) => "常规",
        ("tab_general", Language::Ja) => "一般",
        ("tab_general", _) => "General",

        // Integrations Group
        ("group_integrations", Language::Tr) => "Masaüstü Entegrasyonları",
        ("group_integrations", Language::De) => "Desktop-Integrationen",
        ("group_integrations", Language::Fr) => "Intégrations de bureau",
        ("group_integrations", Language::Es) => "Integraciones de escritorio",
        ("group_integrations", Language::It) => "Integrazioni desktop",
        ("group_integrations", Language::Ru) => "Интеграция с рабочим столом",
        ("group_integrations", Language::Zh) => "桌面集成",
        ("group_integrations", Language::Ja) => "デスクトップ統合",
        ("group_integrations", _) => "Desktop Integrations",

        ("group_integrations_desc", Language::Tr) => {
            "Sağ tık bağlam menüsü ve sistem bildirim tercihleri"
        }
        ("group_integrations_desc", Language::De) => {
            "Kontextmenü- und Systembenachrichtigungseinstellungen"
        }
        ("group_integrations_desc", Language::Fr) => {
            "Préférences du menu contextuel et des notifications système"
        }
        ("group_integrations_desc", Language::Es) => {
            "Preferencias del menú contextual y notificaciones del sistema"
        }
        ("group_integrations_desc", Language::It) => {
            "Preferenze del menu contestuale e notifiche di sistema"
        }
        ("group_integrations_desc", Language::Ru) => {
            "Параметры контекстного меню и системных уведомлений"
        }
        ("group_integrations_desc", Language::Zh) => "右键菜单与系统通知偏好设置",
        ("group_integrations_desc", Language::Ja) => "コンテキストメニューとシステム通知の設定",
        ("group_integrations_desc", _) => "Context menu and system notification preferences",

        ("dolphin_title", _) => "Dolphin (KDE Plasma)",
        ("dolphin_sub", Language::Tr) => {
            "KDE dosya yöneticisinde sağ tık 'Konvert' menüsünü göster"
        }
        ("dolphin_sub", Language::De) => "Rechtsklick-Menü 'Konvert' im KDE-Dateimanager anzeigen",
        ("dolphin_sub", Language::Fr) => {
            "Afficher le menu contextuel 'Konvert' dans le gestionnaire KDE"
        }
        ("dolphin_sub", Language::Es) => "Mostrar el menú contextual 'Konvert' en el gestor de KDE",
        ("dolphin_sub", Language::It) => {
            "Mostra il menu contestuale 'Konvert' nel gestore di file KDE"
        }
        ("dolphin_sub", Language::Ru) => "Показывать меню 'Konvert' в контекстном меню KDE",
        ("dolphin_sub", Language::Zh) => "在 KDE 文件管理器右键菜单中显示 'Konvert'",
        ("dolphin_sub", Language::Ja) => {
            "KDEファイルマネージャーで右クリック「Konvert」メニューを表示"
        }
        ("dolphin_sub", _) => "Show 'Konvert' in KDE file manager right-click menu",

        ("nautilus_title", _) => "Nautilus (GNOME)",
        ("nautilus_sub", Language::Tr) => {
            "GNOME dosya yöneticisinde 'Betikler > Konvert' menüsünü göster"
        }
        ("nautilus_sub", Language::De) => "'Skripte > Konvert'-Menü im GNOME-Dateimanager anzeigen",
        ("nautilus_sub", Language::Fr) => "Afficher 'Scripts > Konvert' dans le gestionnaire GNOME",
        ("nautilus_sub", Language::Es) => "Mostrar 'Scripts > Konvert' en el gestor de GNOME",
        ("nautilus_sub", Language::It) => "Mostra 'Script > Konvert' nel gestore di file GNOME",
        ("nautilus_sub", Language::Ru) => "Показывать 'Скрипты > Konvert' в GNOME Nautilus",
        ("nautilus_sub", Language::Zh) => "在 GNOME 文件管理器中显示 '脚本 > Konvert'",
        ("nautilus_sub", Language::Ja) => {
            "GNOMEファイルマネージャーで「スクリプト > Konvert」を表示"
        }
        ("nautilus_sub", _) => "Show 'Scripts > Konvert' in GNOME file manager",

        ("thunar_title", _) => "Thunar (XFCE)",
        ("thunar_sub", Language::Tr) => {
            "XFCE dosya yöneticisinde sağ tık 'Konvert' eylemlerini göster"
        }
        ("thunar_sub", Language::De) => {
            "Rechtsklick-Aktionen 'Konvert' im XFCE-Dateimanager anzeigen"
        }
        ("thunar_sub", Language::Fr) => "Afficher les actions 'Konvert' dans le gestionnaire XFCE",
        ("thunar_sub", Language::Es) => "Mostrar acciones 'Konvert' en el gestor de XFCE",
        ("thunar_sub", Language::It) => "Mostra le azioni 'Konvert' nel gestore di file XFCE",
        ("thunar_sub", Language::Ru) => "Показывать действия 'Konvert' в XFCE Thunar",
        ("thunar_sub", Language::Zh) => "在 XFCE 文件管理器右键菜单中显示 'Konvert' 操作",
        ("thunar_sub", Language::Ja) => "XFCEファイルマネージャーで右クリック「Konvert」操作を表示",
        ("thunar_sub", _) => "Show 'Konvert' actions in XFCE file manager",

        ("notify_switch_title", Language::Tr) => "Sistem Bildirimleri",
        ("notify_switch_title", Language::De) => "Systembenachrichtigungen",
        ("notify_switch_title", Language::Fr) => "Notifications système",
        ("notify_switch_title", Language::Es) => "Notificaciones del sistema",
        ("notify_switch_title", Language::It) => "Notifiche di sistema",
        ("notify_switch_title", Language::Ru) => "Системные уведомления",
        ("notify_switch_title", Language::Zh) => "系统通知",
        ("notify_switch_title", Language::Ja) => "システム通知",
        ("notify_switch_title", _) => "System Notifications",

        ("notify_switch_sub", Language::Tr) => {
            "İşlem tamamlandığında veya hata oluştuğunda bildirim göster"
        }
        ("notify_switch_sub", Language::De) => {
            "Benachrichtigen, wenn Vorgänge abgeschlossen sind oder fehlschlagen"
        }
        ("notify_switch_sub", Language::Fr) => {
            "Notifier lorsque les opérations se terminent ou échouent"
        }
        ("notify_switch_sub", Language::Es) => {
            "Notificar cuando las operaciones finalicen o fallen"
        }
        ("notify_switch_sub", Language::It) => {
            "Notifica quando le operazioni sono completate o non riescono"
        }
        ("notify_switch_sub", Language::Ru) => "Уведомлять о завершении или ошибках конвертации",
        ("notify_switch_sub", Language::Zh) => "转换完成或发生错误时显示系统通知",
        ("notify_switch_sub", Language::Ja) => "処理完了時またはエラー発生時に通知を表示",
        ("notify_switch_sub", _) => "Notify when operations complete or fail",

        // Formats Group
        ("group_formats", Language::Tr) => "Format Kalite Düzeyleri",
        ("group_formats", Language::De) => "Format-Qualitätsstufen",
        ("group_formats", Language::Fr) => "Niveaux de qualité des formats",
        ("group_formats", Language::Es) => "Niveles de calidad de formato",
        ("group_formats", Language::It) => "Livelli di qualità del formato",
        ("group_formats", Language::Ru) => "Качество форматов",
        ("group_formats", Language::Zh) => "格式质量设置",
        ("group_formats", Language::Ja) => "フォーマット品質設定",
        ("group_formats", _) => "Format Quality Levels",

        ("group_formats_desc", Language::Tr) => {
            "Dönüştürme sırasında kullanılacak varsayılan sıkıştırma oranları (1-100)"
        }
        ("group_formats_desc", Language::De) => {
            "Standardmäßige Komprimierungsraten für die Konvertierung (1-100)"
        }
        ("group_formats_desc", Language::Fr) => {
            "Taux de compression par défaut pour les conversions (1-100)"
        }
        ("group_formats_desc", Language::Es) => {
            "Tasas de compresión predeterminadas para conversiones (1-100)"
        }
        ("group_formats_desc", Language::It) => {
            "Rapporti di compressione predefiniti per le conversioni (1-100)"
        }
        ("group_formats_desc", Language::Ru) => {
            "Уровни сжатия по умолчанию при конвертации (1-100)"
        }
        ("group_formats_desc", Language::Zh) => "转换时使用的默认压缩率 (1-100)",
        ("group_formats_desc", Language::Ja) => "変換時のデフォルト圧縮レベル (1-100)",
        ("group_formats_desc", _) => "Default compression rates used during conversion (1-100)",

        ("webp_quality_title", Language::Tr) => "WebP Kalitesi",
        ("webp_quality_title", Language::De) => "WebP-Qualität",
        ("webp_quality_title", Language::Fr) => "Qualité WebP",
        ("webp_quality_title", Language::Es) => "Calidad WebP",
        ("webp_quality_title", Language::It) => "Qualità WebP",
        ("webp_quality_title", Language::Ru) => "Качество WebP",
        ("webp_quality_title", Language::Zh) => "WebP 质量",
        ("webp_quality_title", Language::Ja) => "WebP 品質",
        ("webp_quality_title", _) => "WebP Quality",

        ("webp_quality_sub", Language::Tr) => {
            "Önerilen: 85. Düşük değerler dosya boyutunu küçültür."
        }
        ("webp_quality_sub", Language::De) => {
            "Empfohlen: 85. Niedrigere Werte verringern die Dateigröße."
        }
        ("webp_quality_sub", Language::Fr) => {
            "Recommandé: 85. Des valeurs inférieures réduisent la taille."
        }
        ("webp_quality_sub", Language::Es) => {
            "Recomendado: 85. Valores más bajos reducen el tamaño del archivo."
        }
        ("webp_quality_sub", Language::It) => {
            "Consigliato: 85. Valori più bassi riducono la dimensione del file."
        }
        ("webp_quality_sub", Language::Ru) => {
            "Рекомендуется: 85. Меньшие значения уменьшают размер файла."
        }
        ("webp_quality_sub", Language::Zh) => "推荐值：85。较低的数值可减小文件大小。",
        ("webp_quality_sub", Language::Ja) => {
            "推奨値: 85。低い値にするとファイルサイズが小さくなります。"
        }
        ("webp_quality_sub", _) => "Recommended: 85. Lower values reduce file size.",

        ("jpeg_quality_title", Language::Tr) => "JPEG Kalitesi",
        ("jpeg_quality_title", Language::De) => "JPEG-Qualität",
        ("jpeg_quality_title", Language::Fr) => "Qualité JPEG",
        ("jpeg_quality_title", Language::Es) => "Calidad JPEG",
        ("jpeg_quality_title", Language::It) => "Qualità JPEG",
        ("jpeg_quality_title", Language::Ru) => "Качество JPEG",
        ("jpeg_quality_title", Language::Zh) => "JPEG 质量",
        ("jpeg_quality_title", Language::Ja) => "JPEG 品質",
        ("jpeg_quality_title", _) => "JPEG Quality",

        ("jpeg_quality_sub", Language::Tr) => "Önerilen: 85. Standart kayıplı sıkıştırma oranı.",
        ("jpeg_quality_sub", Language::De) => "Empfohlen: 85. Standardkomprimierung mit Verlusten.",
        ("jpeg_quality_sub", Language::Fr) => {
            "Recommandé: 85. Taux de compression avec perte standard."
        }
        ("jpeg_quality_sub", Language::Es) => {
            "Recomendado: 85. Tasa de compresión estándar con pérdidas."
        }
        ("jpeg_quality_sub", Language::It) => {
            "Consigliato: 85. Tasso di compressione con perdita standard."
        }
        ("jpeg_quality_sub", Language::Ru) => "Рекомендуется: 85. Стандартное сжатие с потерями.",
        ("jpeg_quality_sub", Language::Zh) => "推荐值：85。标准有损压缩比例。",
        ("jpeg_quality_sub", Language::Ja) => "推奨値: 85。標準の非可逆圧縮比率。",
        ("jpeg_quality_sub", _) => "Recommended: 85. Standard lossy compression rate.",

        // About Group
        ("group_about", Language::Tr) => "Hakkında ve Geliştirici",
        ("group_about", Language::De) => "Über & Entwickler",
        ("group_about", Language::Fr) => "À propos & Développeur",
        ("group_about", Language::Es) => "Acerca de & Desarrollador",
        ("group_about", Language::It) => "Informazioni & Sviluppatore",
        ("group_about", Language::Ru) => "О программе и разработчик",
        ("group_about", Language::Zh) => "关于与开发者",
        ("group_about", Language::Ja) => "情報と開発者",
        ("group_about", _) => "About & Developer",

        ("author_title", Language::Tr) => "Geliştirici",
        ("author_title", Language::De) => "Entwickler",
        ("author_title", Language::Fr) => "Développeur",
        ("author_title", Language::Es) => "Desarrollador",
        ("author_title", Language::It) => "Sviluppatore",
        ("author_title", Language::Ru) => "Разработчик",
        ("author_title", Language::Zh) => "开发者",
        ("author_title", Language::Ja) => "開発者",
        ("author_title", _) => "Developer",

        ("contact_title", Language::Tr) => "İletişim",
        ("contact_title", Language::De) => "Kontakt",
        ("contact_title", Language::Fr) => "Contact",
        ("contact_title", Language::Es) => "Contacto",
        ("contact_title", Language::It) => "Contatto",
        ("contact_title", Language::Ru) => "Контакты",
        ("contact_title", Language::Zh) => "联系方式",
        ("contact_title", Language::Ja) => "連絡先",
        ("contact_title", _) => "Contact",

        ("version_title", Language::Tr) => "Konvert Sürümü",
        ("version_title", Language::De) => "Konvert-Version",
        ("version_title", Language::Fr) => "Version de Konvert",
        ("version_title", Language::Es) => "Versión de Konvert",
        ("version_title", Language::It) => "Versione di Konvert",
        ("version_title", Language::Ru) => "Версия Konvert",
        ("version_title", Language::Zh) => "Konvert 版本",
        ("version_title", Language::Ja) => "Konvert バージョン",
        ("version_title", _) => "Konvert Version",

        ("version_sub", Language::Tr) => "0.1.0 (Rust Yerel Derleme)",
        ("version_sub", Language::De) => "0.1.0 (Rust Native Build)",
        ("version_sub", Language::Fr) => "0.1.0 (Compilation native Rust)",
        ("version_sub", Language::Es) => "0.1.0 (Compilación nativa Rust)",
        ("version_sub", Language::It) => "0.1.0 (Compilazione nativa Rust)",
        ("version_sub", Language::Ru) => "0.1.0 (Нативная сборка на Rust)",
        ("version_sub", Language::Zh) => "0.1.0 (Rust 原生编译)",
        ("version_sub", Language::Ja) => "0.1.0 (Rust ネイティブビルド)",
        ("version_sub", _) => "0.1.0 (Rust Native Build)",

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

        ("notify_done_batch", Language::Tr) => "Toplu Dönüştürme Başarılı",
        ("notify_done_batch", Language::De) => "Stapelkonvertierung erfolgreich",
        ("notify_done_batch", Language::Fr) => "Conversion par lot réussie",
        ("notify_done_batch", Language::Es) => "Conversión por lotes completada",
        ("notify_done_batch", Language::It) => "Conversione batch completata",
        ("notify_done_batch", Language::Ru) => "Пакетная конвертация завершена",
        ("notify_done_batch", Language::Zh) => "批量转换成功",
        ("notify_done_batch", Language::Ja) => "一括変換完了",
        ("notify_done_batch", _) => "Batch Conversion Completed",

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

        ("notify_integ_title", Language::Tr) => "Konvert Entegrasyonu",
        ("notify_integ_title", Language::De) => "Konvert-Integration",
        ("notify_integ_title", Language::Fr) => "Intégration de Konvert",
        ("notify_integ_title", Language::Es) => "Integración de Konvert",
        ("notify_integ_title", Language::It) => "Integrazione di Konvert",
        ("notify_integ_title", Language::Ru) => "Интеграция Konvert",
        ("notify_integ_title", Language::Zh) => "Konvert 集成",
        ("notify_integ_title", Language::Ja) => "Konvert 統合",
        ("notify_integ_title", _) => "Konvert Integration",

        // Actions (Desktop Menu)
        ("action_to_webp", Language::Tr) => "WebP'ye Dönüştür",
        ("action_to_webp", Language::De) => "In WebP umwandeln",
        ("action_to_webp", Language::Fr) => "Convertir en WebP",
        ("action_to_webp", Language::Es) => "Convertir a WebP",
        ("action_to_webp", Language::It) => "Converti in WebP",
        ("action_to_webp", Language::Ru) => "Конвертировать в WebP",
        ("action_to_webp", Language::Zh) => "转换为 WebP",
        ("action_to_webp", Language::Ja) => "WebPに変換",
        ("action_to_webp", _) => "Convert to WebP",

        ("action_to_png", Language::Tr) => "PNG'ye Dönüştür",
        ("action_to_png", Language::De) => "In PNG umwandeln",
        ("action_to_png", Language::Fr) => "Convertir en PNG",
        ("action_to_png", Language::Es) => "Convertir a PNG",
        ("action_to_png", Language::It) => "Converti in PNG",
        ("action_to_png", Language::Ru) => "Конвертировать в PNG",
        ("action_to_png", Language::Zh) => "转换为 PNG",
        ("action_to_png", Language::Ja) => "PNGに変換",
        ("action_to_png", _) => "Convert to PNG",

        ("action_to_jpg", Language::Tr) => "JPG'ye Dönüştür",
        ("action_to_jpg", Language::De) => "In JPG umwandeln",
        ("action_to_jpg", Language::Fr) => "Convertir en JPG",
        ("action_to_jpg", Language::Es) => "Convertir a JPG",
        ("action_to_jpg", Language::It) => "Converti in JPG",
        ("action_to_jpg", Language::Ru) => "Конвертировать в JPG",
        ("action_to_jpg", Language::Zh) => "转换为 JPG",
        ("action_to_jpg", Language::Ja) => "JPGに変換",
        ("action_to_jpg", _) => "Convert to JPG",

        ("action_to_svg", Language::Tr) => "SVG Olarak Vektörleştir",
        ("action_to_svg", Language::De) => "In SVG vektorisieren",
        ("action_to_svg", Language::Fr) => "Vectoriser en SVG",
        ("action_to_svg", Language::Es) => "Vectorizar a SVG",
        ("action_to_svg", Language::It) => "Vettorizza in SVG",
        ("action_to_svg", Language::Ru) => "Векторизовать в SVG",
        ("action_to_svg", Language::Zh) => "矢量化为 SVG",
        ("action_to_svg", Language::Ja) => "SVGにベクター変換",
        ("action_to_svg", _) => "Trace to SVG (Vectorize)",

        ("action_to_avif", Language::Tr) => "AVIF'e Dönüştür",
        ("action_to_avif", Language::De) => "In AVIF umwandeln",
        ("action_to_avif", Language::Fr) => "Convertir en AVIF",
        ("action_to_avif", Language::Es) => "Convertir a AVIF",
        ("action_to_avif", Language::It) => "Converti in AVIF",
        ("action_to_avif", Language::Ru) => "Конвертировать в AVIF",
        ("action_to_avif", Language::Zh) => "转换为 AVIF",
        ("action_to_avif", Language::Ja) => "AVIFに変換",
        ("action_to_avif", _) => "Convert to AVIF",

        ("action_to_mp3", Language::Tr) => "MP3'e Dönüştür",
        ("action_to_mp3", Language::De) => "In MP3 umwandeln",
        ("action_to_mp3", Language::Fr) => "Convertir en MP3",
        ("action_to_mp3", Language::Es) => "Convertir a MP3",
        ("action_to_mp3", Language::It) => "Converti in MP3",
        ("action_to_mp3", Language::Ru) => "Конвертировать в MP3",
        ("action_to_mp3", Language::Zh) => "转换为 MP3",
        ("action_to_mp3", Language::Ja) => "MP3に変換",
        ("action_to_mp3", _) => "Convert to MP3",

        ("action_to_gif", Language::Tr) => "GIF'e Dönüştür",
        ("action_to_gif", Language::De) => "In GIF umwandeln",
        ("action_to_gif", Language::Fr) => "Convertir en GIF",
        ("action_to_gif", Language::Es) => "Convertir a GIF",
        ("action_to_gif", Language::It) => "Converti in GIF",
        ("action_to_gif", Language::Ru) => "Конвертировать в GIF",
        ("action_to_gif", Language::Zh) => "转换为 GIF",
        ("action_to_gif", Language::Ja) => "GIFに変換",
        ("action_to_gif", _) => "Convert to GIF",

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
}
