#pragma once

#include <QString>
#include <QLocale>

class AppTranslator {
public:
    static QString getActiveLanguage(const QString &configured) {
        if (configured.isEmpty() || configured == "auto") {
            QString sys = QLocale::system().name().left(2).toLower();
            if (sys == "tr" || sys == "de" || sys == "fr" || sys == "es" ||
                sys == "it" || sys == "ru" || sys == "zh" || sys == "ja") {
                return sys;
            }
            return "en";
        }
        return configured.toLower();
    }

    static QString tr(const QString &key, const QString &langCode) {
        QString lang = getActiveLanguage(langCode);

        // --- MAIN TABS ---
        if (key == "tab_convert") {
            if (lang == "tr") return "Dönüştür";
            if (lang == "de") return "Konvertieren";
            if (lang == "fr") return "Convertir";
            if (lang == "es") return "Convertir";
            return "Convert";
        }
        if (key == "tab_history") {
            if (lang == "tr") return "İşlem Günlüğü";
            if (lang == "de") return "Verlauf";
            if (lang == "fr") return "Historique";
            if (lang == "es") return "Historial";
            return "History";
        }
        if (key == "tab_about") {
            if (lang == "tr") return "Hakkında";
            if (lang == "de") return "Über";
            if (lang == "fr") return "À propos";
            if (lang == "es") return "Acerca de";
            return "About";
        }
        if (key == "tab_settings") {
            if (lang == "tr") return "Ayarlar";
            if (lang == "de") return "Einstellungen";
            if (lang == "fr") return "Paramètres";
            if (lang == "es") return "Configuración";
            return "Settings";
        }

        // --- CATEGORIES ---
        if (key == "cat_photos") {
            if (lang == "tr") return "Görseller";
            if (lang == "de") return "Bilder";
            if (lang == "fr") return "Images";
            if (lang == "es") return "Imágenes";
            return "Photos";
        }
        if (key == "cat_videos") {
            if (lang == "tr") return "Videolar";
            if (lang == "de") return "Videos";
            if (lang == "fr") return "Vidéos";
            if (lang == "es") return "Videos";
            return "Videos";
        }
        if (key == "cat_music") {
            if (lang == "tr") return "Müzikler";
            if (lang == "de") return "Audio";
            if (lang == "fr") return "Audio";
            if (lang == "es") return "Audio";
            return "Audio";
        }
        if (key == "cat_documents") {
            if (lang == "tr") return "Belgeler";
            if (lang == "de") return "Dokumente";
            if (lang == "fr") return "Documents";
            if (lang == "es") return "Documentos";
            return "Documents";
        }

        // --- CARD TITLES ---
        if (key == "card_actions") {
            if (lang == "tr") return "Sağ Tık Menüsü Eylemleri";
            if (lang == "de") return "Aktionen im Kontextmenü";
            if (lang == "fr") return "Actions du menu contextuel";
            if (lang == "es") return "Acciones del menú contextual";
            return "Context Menu Actions";
        }
        if (key == "card_photo_comp") {
            if (lang == "tr") return "Görsel Kalite ve Sıkıştırma";
            if (lang == "de") return "Bildqualität & Komprimierung";
            if (lang == "fr") return "Qualité d'image et compression";
            if (lang == "es") return "Calidad de imagen y compresión";
            return "Image Quality & Compression";
        }
        if (key == "card_video_comp") {
            if (lang == "tr") return "Video Sıkıştırma Parametreleri";
            if (lang == "de") return "Videokomprimierung";
            if (lang == "fr") return "Compression vidéo";
            if (lang == "es") return "Compresión de video";
            return "Video Compression";
        }
        if (key == "card_audio_comp") {
            if (lang == "tr") return "Ses Kalite Ayarları";
            if (lang == "de") return "Audioqualität";
            if (lang == "fr") return "Qualité audio";
            if (lang == "es") return "Calidad de audio";
            return "Audio Quality";
        }
        if (key == "card_doc_comp") {
            if (lang == "tr") return "Belge İşleme Parametreleri";
            if (lang == "de") return "Dokumentenverarbeitung";
            if (lang == "fr") return "Traitement des documents";
            if (lang == "es") return "Procesamiento de documentos";
            return "Document Processing";
        }
        if (key == "card_general_settings") {
            if (lang == "tr") return "Genel Tercihler";
            if (lang == "de") return "Allgemeine Einstellungen";
            if (lang == "fr") return "Préférences générales";
            if (lang == "es") return "Preferencias generales";
            return "General Preferences";
        }
        if (key == "card_process_monitor") {
            if (lang == "tr") return "Aktif Görev İzleyici";
            if (lang == "de") return "Aktive Prozessüberwachung";
            if (lang == "fr") return "Surveillance des tâches";
            if (lang == "es") return "Monitor de tareas activas";
            return "Active Task Monitor";
        }
        if (key == "card_desktop_integration") {
            if (lang == "tr") return "Masaüstü Entegrasyonları & Sistem Sağlığı";
            if (lang == "de") return "Desktop-Integrationen & Systemstatus";
            if (lang == "fr") return "Intégrations de bureau & Santé du système";
            if (lang == "es") return "Integraciones de escritorio y estado";
            return "Desktop Integrations & System Health";
        }

        // --- COMMON LABELS & CONTROLS ---
        if (key == "lbl_photo_quality") {
            if (lang == "tr") return "Varsayılan Görsel Kalitesi:";
            if (lang == "de") return "Standard-Bildqualität:";
            if (lang == "fr") return "Qualité d'image par défaut :";
            if (lang == "es") return "Calidad de imagen predeterminada:";
            return "Default Image Quality:";
        }
        if (key == "lbl_lossless_photo") {
            if (lang == "tr") return "Kayıpsız Modu Zorla";
            if (lang == "de") return "Verlustfreien Modus erzwingen";
            if (lang == "fr") return "Forcer le mode sans perte";
            if (lang == "es") return "Forzar modo sin pérdidas";
            return "Force Lossless Mode";
        }
        if (key == "lbl_video_crf") {
            if (lang == "tr") return "CRF Kalitesi:";
            if (lang == "de") return "CRF-Qualität:";
            if (lang == "fr") return "Qualité CRF :";
            if (lang == "es") return "Calidad CRF:";
            return "CRF Quality:";
        }
        if (key == "lbl_video_preset") {
            if (lang == "tr") return "Sıkıştırma Hız / Önayar:";
            if (lang == "de") return "Geschwindigkeit / Preset:";
            if (lang == "fr") return "Vitesse / Préréglage :";
            if (lang == "es") return "Velocidad / Ajuste:";
            return "Compression Speed / Preset:";
        }
        if (key == "lbl_video_hwaccel") {
            if (lang == "tr") return "Donanım Hızlandırma (GPU):";
            if (lang == "de") return "Hardwarebeschleunigung (GPU):";
            if (lang == "fr") return "Accélération matérielle (GPU) :";
            if (lang == "es") return "Aceleración por hardware (GPU):";
            return "Hardware Acceleration (GPU):";
        }
        if (key == "hwaccel_auto") {
            if (lang == "tr") return "Otomatik (VA-API / NVENC)";
            if (lang == "de") return "Automatisch (VA-API / NVENC)";
            if (lang == "fr") return "Automatique (VA-API / NVENC)";
            if (lang == "es") return "Automático (VA-API / NVENC)";
            return "Auto (VA-API / NVENC)";
        }
        if (key == "hwaccel_none") {
            if (lang == "tr") return "Devre Dışı (Sadece CPU)";
            if (lang == "de") return "Deaktiviert (Nur CPU)";
            if (lang == "fr") return "Désactivé (CPU uniquement)";
            if (lang == "es") return "Desactivado (Solo CPU)";
            return "Disabled (CPU Only)";
        }
        if (key == "lbl_video_lossless") {
            if (lang == "tr") return "Kayıpsız Video Modu";
            if (lang == "de") return "Verlustfreier Videomodus";
            if (lang == "fr") return "Mode vidéo sans perte";
            if (lang == "es") return "Modo de video sin pérdidas";
            return "Lossless Video Mode";
        }
        if (key == "lbl_audio_bitrate") {
            if (lang == "tr") return "Hedef Ses Bitrate:";
            if (lang == "de") return "Ziel-Bitrate:";
            if (lang == "fr") return "Débit audio cible :";
            if (lang == "es") return "Tasa de bits de audio:";
            return "Target Audio Bitrate:";
        }
        if (key == "lbl_pdf_dpi") {
            if (lang == "tr") return "PDF Döküm Çözünürlüğü (DPI):";
            if (lang == "de") return "PDF-Auflösung (DPI):";
            if (lang == "fr") return "Résolution de rendu PDF (DPI) :";
            if (lang == "es") return "Resolución de renderizado PDF (DPI):";
            return "PDF Render Resolution (DPI):";
        }
        if (key == "lbl_output_dir") {
            if (lang == "tr") return "Varsayılan Çıktı Klasörü:";
            if (lang == "de") return "Standard-Ausgabeverzeichnis:";
            if (lang == "fr") return "Dossier de sortie par défaut :";
            if (lang == "es") return "Carpeta de salida predeterminada:";
            return "Default Output Directory:";
        }
        if (key == "lbl_output_dir_default") {
            if (lang == "tr") return "Varsayılan (Girdi dosyasının bulunduğu klasör)";
            if (lang == "de") return "Standard (Ordner der Eingabedatei)";
            if (lang == "fr") return "Par défaut (Dossier du fichier source)";
            if (lang == "es") return "Predeterminado (Carpeta del archivo origen)";
            return "Default (Same directory as input file)";
        }
        if (key == "lbl_process_idle") {
            if (lang == "tr") return "Durum: Boşta (Aktif işlem yok)";
            if (lang == "de") return "Status: Leerlauf (Keine aktive Konvertierung)";
            if (lang == "fr") return "État : Inactif (Aucune conversion en cours)";
            if (lang == "es") return "Estado: Inactivo (Sin tareas en curso)";
            return "Status: Idle (No active conversion)";
        }
        if (key == "lbl_process_running") {
            if (lang == "tr") return "Durum: Dönüştürme çalışıyor (PID: %1)";
            if (lang == "de") return "Status: Konvertierung läuft (PID: %1)";
            if (lang == "fr") return "État : Conversion en cours (PID : %1)";
            if (lang == "es") return "Estado: Conversión en curso (PID: %1)";
            return "Status: Conversion in progress (PID: %1)";
        }
        if (key == "lbl_process_paused") {
            if (lang == "tr") return "Durum: Duraklatıldı (PID: %1)";
            if (lang == "de") return "Status: Konvertierung pausiert (PID: %1)";
            if (lang == "fr") return "État : Conversion en pause (PID : %1)";
            if (lang == "es") return "Estado: Conversión en pausa (PID: %1)";
            return "Status: Conversion paused (PID: %1)";
        }
        if (key == "lbl_notifications") {
            if (lang == "tr") return "Sistem Bildirimlerini Göster";
            if (lang == "de") return "Systembenachrichtigungen anzeigen";
            if (lang == "fr") return "Afficher les notifications système";
            if (lang == "es") return "Mostrar notificaciones del sistema";
            return "Show System Notifications";
        }
        if (key == "lbl_language") {
            if (lang == "tr") return "Uygulama Dili:";
            if (lang == "de") return "Anwendungssprache:";
            if (lang == "fr") return "Langue de l'application :";
            if (lang == "es") return "Idioma de la aplicación:";
            return "Application Language:";
        }

        // --- BUTTONS ---
        if (key == "btn_browse") {
            if (lang == "tr") return "Gözat...";
            if (lang == "de") return "Durchsuchen...";
            if (lang == "fr") return "Parcourir...";
            if (lang == "es") return "Examinar...";
            return "Browse...";
        }
        if (key == "btn_reset") {
            if (lang == "tr") return "Sıfırla";
            if (lang == "de") return "Zurücksetzen";
            if (lang == "fr") return "Réinitialiser";
            if (lang == "es") return "Restablecer";
            return "Reset";
        }
        if (key == "btn_pause_process") {
            if (lang == "tr") return "Duraklat";
            if (lang == "de") return "Pausieren";
            if (lang == "fr") return "Mettre en pause";
            if (lang == "es") return "Pausar";
            return "Pause";
        }
        if (key == "btn_resume_process") {
            if (lang == "tr") return "Devam Et";
            if (lang == "de") return "Fortsetzen";
            if (lang == "fr") return "Reprendre";
            if (lang == "es") return "Reanudar";
            return "Resume";
        }
        if (key == "btn_cancel_process") {
            if (lang == "tr") return "Görevi İptal Et";
            if (lang == "de") return "Aufgabe abbrechen";
            if (lang == "fr") return "Annuler la tâche";
            if (lang == "es") return "Cancelar tarea";
            return "Cancel Task";
        }
        if (key == "btn_install_integration") {
            if (lang == "tr") return "Entegrasyonları Kur";
            if (lang == "de") return "Integrationen installieren";
            if (lang == "fr") return "Installer les intégrations";
            if (lang == "es") return "Instalar integraciones";
            return "Install Integrations";
        }
        if (key == "btn_uninstall_integration") {
            if (lang == "tr") return "Entegrasyonları Kaldır";
            if (lang == "de") return "Integrationen entfernen";
            if (lang == "fr") return "Désinstaller les intégrations";
            if (lang == "es") return "Desinstalar integraciones";
            return "Uninstall Integrations";
        }
        if (key == "btn_check_deps") {
            if (lang == "tr") return "Bağımlılıkları Denetle";
            if (lang == "de") return "Abhängigkeiten prüfen";
            if (lang == "fr") return "Vérifier les dépendances";
            if (lang == "es") return "Comprobar dependencias";
            return "Check Dependencies";
        }
        if (key == "btn_refresh") {
            if (lang == "tr") return "Yenile";
            if (lang == "de") return "Aktualisieren";
            if (lang == "fr") return "Actualiser";
            if (lang == "es") return "Actualizar";
            return "Refresh";
        }
        if (key == "btn_clear_history") {
            if (lang == "tr") return "Geçmişi Temizle";
            if (lang == "de") return "Verlauf löschen";
            if (lang == "fr") return "Effacer l'historique";
            if (lang == "es") return "Borrar historial";
            return "Clear History";
        }

        // --- ABOUT TABS ---
        if (key == "about_tab_info") {
            if (lang == "tr") return "Hakkında";
            if (lang == "de") return "Über";
            if (lang == "fr") return "À propos";
            if (lang == "es") return "Acerca de";
            return "About";
        }
        if (key == "about_tab_authors" || key == "about_tab_community") {
            if (lang == "tr") return "Yazarlar";
            if (lang == "de") return "Autoren";
            if (lang == "fr") return "Auteurs";
            if (lang == "es") return "Autores";
            return "Authors";
        }
        if (key == "about_tab_components") {
            if (lang == "tr") return "Bileşenler & Teşekkür";
            if (lang == "de") return "Komponenten & Danksagung";
            if (lang == "fr") return "Composants & Remerciements";
            if (lang == "es") return "Componentes y Agradecimientos";
            return "Components & Thanks";
        }
        if (key == "about_tab_license") {
            if (lang == "tr") return "Lisans";
            if (lang == "de") return "Lizenz";
            if (lang == "fr") return "Licence";
            if (lang == "es") return "Licencia";
            return "License";
        }

        if (key == "lang_auto") {
            return pickLang(lang, "Auto Detect (System)", "Otomatik Algıla (Sistem Dili)", "Automatisch (Systemsprache)", "Détection automatique (Système)", "Detección automática (Sistema)");
        }
        if (key == "card_history_stats") {
            return pickLang(lang, "Conversion Statistics", "Dönüştürme İstatistikleri", "Konvertierungsstatistiken", "Statistiques de conversion", "Estadísticas de conversión");
        }
        if (key == "card_history_preview") {
            return pickLang(lang, "File Preview & Location", "Dosya Önizleme ve Konum", "Dateivorschau & Speicherort", "Aperçu du fichier et emplacement", "Vista previa y ubicación");
        }
        if (key == "lbl_total_operations_init") {
            return pickLang(lang, "<b>Total Conversions:</b> 0", "<b>Toplam İşlem:</b> 0", "<b>Konvertierungen gesamt:</b> 0", "<b>Total des conversions :</b> 0", "<b>Conversiones totales:</b> 0");
        }
        if (key == "lbl_total_operations_fmt") {
            return pickLang(lang, "<b>Total Conversions:</b> %1 (%2 files)", "<b>Toplam İşlem:</b> %1 (%2 dosya)", "<b>Konvertierungen gesamt:</b> %1 (%2 Dateien)", "<b>Total des conversions :</b> %1 (%2 fichiers)", "<b>Conversiones totales:</b> %1 (%2 archivos)");
        }
        if (key == "lbl_total_savings_init") {
            return pickLang(lang, "<b>Total Saved:</b> 0 MB", "<b>Toplam Tasarruf:</b> 0 MB", "<b>Gesamtersparnis:</b> 0 MB", "<b>Espace économisé :</b> 0 Mo", "<b>Ahorro total:</b> 0 MB");
        }
        if (key == "lbl_total_savings_fmt") {
            return pickLang(lang, "<b>Total Saved:</b> %1", "<b>Toplam Tasarruf:</b> %1", "<b>Gesamtersparnis:</b> %1", "<b>Espace économisé :</b> %1", "<b>Ahorro total:</b> %1");
        }
        if (key == "hdr_date_time") {
            return pickLang(lang, "Date / Time", "Tarih / Saat", "Datum / Zeit", "Date / Heure", "Fecha / Hora");
        }
        if (key == "hdr_action") {
            return pickLang(lang, "Action", "İşlem", "Aktion", "Action", "Acción");
        }
        if (key == "hdr_files") {
            return pickLang(lang, "Files", "Dosya", "Dateien", "Fichiers", "Archivos");
        }
        if (key == "hdr_original") {
            return pickLang(lang, "Original", "Orijinal", "Original", "Original", "Original");
        }
        if (key == "hdr_output") {
            return pickLang(lang, "Output", "Çıktı", "Ausgabe", "Sortie", "Salida");
        }
        if (key == "hdr_savings") {
            return pickLang(lang, "Savings", "Tasarruf", "Ersparnis", "Gain", "Ahorro");
        }
        if (key == "hdr_status") {
            return pickLang(lang, "Status", "Durum", "Status", "Statut", "Estado");
        }
        if (key == "lbl_preview_select") {
            return pickLang(lang, "Select a row from the table to preview file.", "Önizleme için tablodan bir satır seçin.", "Wählen Sie eine Zeile für die Vorschau aus.", "Sélectionnez une ligne pour l'aperçu.", "Seleccione una fila para ver la vista previa.");
        }
        if (key == "lbl_preview_no_path") {
            return pickLang(lang, "No file path recorded for this operation.", "Bu işlem için dosya yolu kaydedilmemiş.", "Für diesen Vorgang wurde kein Dateipfad gespeichert.", "Aucun chemin enregistré pour cette opération.", "No hay ruta registrada para esta operación.");
        }
        if (key == "lbl_preview_missing") {
            return pickLang(lang, "File not found on disk (may have been moved or deleted).", "Dosya diskte bulunamadı (taşınmış veya silinmiş olabilir).", "Datei nicht gefunden (möglicherweise verschoben oder gelöscht).", "Fichier introuvable (peut-être déplacé ou supprimé).", "Archivo no encontrado (puede haberse movido o eliminado).");
        }
        if (key == "btn_open_file") {
            return pickLang(lang, "Open File", "Dosyayı Aç", "Datei öffnen", "Ouvrir le fichier", "Abrir archivo");
        }
        if (key == "btn_show_in_fm") {
            return pickLang(lang, "Show in File Manager", "Dolphin'de Göster", "Im Dateimanager zeigen", "Afficher dans le dossier", "Mostrar en carpeta");
        }
        if (key == "dlg_clear_history_title") {
            return pickLang(lang, "Clear history", "Geçmişi temizle", "Verlauf löschen", "Effacer l'historique", "Borrar historial");
        }
        if (key == "dlg_clear_history_body") {
            return pickLang(lang, "All conversion history will be deleted. Continue?", "Tüm dönüşüm geçmişi silinecek. Emin misiniz?", "Der gesamte Konvertierungsverlauf wird gelöscht. Fortfahren?", "Tout l'historique des conversions sera supprimé. Continuer ?", "Se eliminará todo el historial de conversiones. ¿Continuar?");
        }
        if (key == "dlg_error_title") {
            return pickLang(lang, "Error", "Hata", "Fehler", "Erreur", "Error");
        }
        if (key == "dlg_clear_history_err") {
            return pickLang(lang, "Could not delete the history file.", "Geçmiş dosyası silinemedi.", "Verlaufsdatei konnte nicht gelöscht werden.", "Impossible de supprimer le fichier d'historique.", "No se pudo eliminar el archivo de historial.");
        }
        if (key == "about_subtitle") {
            return pickLang(lang, "Native file converter for Linux and KDE Plasma", "Linux ve KDE Plasma için yerel dosya dönüştürücü", "Nativer Dateikonverter für Linux und KDE Plasma", "Convertisseur de fichiers natif pour Linux et KDE Plasma", "Conversor de archivos nativo para Linux y KDE Plasma");
        }
        if (key == "about_description") {
            return pickLang(lang,
                "Konvert is a fast, offline, and native file conversion utility designed for Linux and KDE Plasma. All processing runs entirely on your local machine with strict zero-telemetry privacy.",
                "Konvert; Linux masaüstü ortamları ve KDE Plasma için sıfır telemetri ve tam gizlilik esasıyla geliştirilmiş yerel bir dosya işleme aracıdır. Dosyalarınız cihazınızdan asla ayrılmaz; tüm dönüştürme ve optimizasyon işlemleri yerel donanım kaynaklarıyla gerçekleştirilir.",
                "Konvert ist ein schnelles, natives Offline-Konvertierungswerkzeug für Linux und KDE Plasma. Alle Vorgänge laufen lokal und ohne Telemetrie ab.",
                "Konvert est un outil de conversion de fichiers natif, rapide et hors ligne conçu pour Linux et KDE Plasma, sans aucune télémétrie.",
                "Konvert es una utilidad de conversión de archivos nativa, rápida y sin conexión diseñada para Linux y KDE Plasma, sin telemetría.");
        }
        if (key == "about_spec_version") {
            return pickLang(lang, "Version:", "Sürüm:", "Version:", "Version :", "Versión:");
        }
        if (key == "about_spec_desktop") {
            return pickLang(lang, "Desktop Environment:", "Masaüstü Ortamı:", "Desktop-Umgebung:", "Environnement :", "Entorno de escritorio:");
        }
        if (key == "about_spec_gui") {
            return pickLang(lang, "GUI Framework:", "Grafik Arayüzü:", "GUI-Framework:", "Interface graphique :", "Interfaz gráfica:");
        }
        if (key == "about_spec_gui_val") {
            return pickLang(lang, "Qt 6 (KDE Breeze Compliant)", "Qt 6 (KDE Breeze Uyumlu)", "Qt 6 (KDE Breeze-konform)", "Qt 6 (Compatible KDE Breeze)", "Qt 6 (Compatible con KDE Breeze)");
        }
        if (key == "about_spec_engine") {
            return pickLang(lang, "Core Engine:", "Çekirdek Motor:", "Kern-Engine:", "Moteur principal :", "Motor principal:");
        }
        if (key == "about_spec_engine_val") {
            return pickLang(lang, "Rust 2024 & Rayon Multi-Core Engine", "Rust 2024 & Rayon Çok Çekirdekli Motor", "Rust 2024 & Rayon Multi-Core-Engine", "Moteur multicœur Rust 2024 & Rayon", "Motor multinúcleo Rust 2024 y Rayon");
        }
        if (key == "about_spec_homepage") {
            return pickLang(lang, "Homepage:", "Web Sitesi:", "Webseite:", "Site web :", "Sitio web:");
        }
        if (key == "about_spec_bugs") {
            return pickLang(lang, "Bug Tracker:", "Hata Bildirimi:", "Fehlerberichte:", "Suivi des bogues :", "Registro de errores:");
        }
        if (key == "about_author_title") {
            return pickLang(lang, "Author", "Geliştirici", "Autor", "Auteur", "Autor");
        }
        if (key == "about_author_desc") {
            return pickLang(lang, "Core architecture, Rust conversion engine, and Qt6 GUI design.", "Çekirdek mimari, Rust dönüştürme motoru ve Qt6 arayüz tasarımı.", "Kernarchitektur, Rust-Konvertierungs-Engine und Qt6-GUI-Design.", "Architecture principale, moteur de conversion Rust et interface Qt6.", "Arquitectura principal, motor de conversión Rust e interfaz Qt6.");
        }
        if (key == "about_comp_rust") {
            return pickLang(lang, "Multi-threaded parallel systems engine and memory safety.", "Paralel çok çekirdekli sistem motoru ve güvenli bellek yönetimi.", "Parallele Multi-Thread-System-Engine und Speichersicherheit.", "Moteur système parallèle multicœur et sûreté mémoire.", "Motor de sistemas paralelo multihilo y seguridad de memoria.");
        }
        if (key == "about_comp_qt") {
            return pickLang(lang, "Native KDE Breeze compliant graphical user interface.", "KDE Breeze ile kusursuz uyumlu yerel grafik kullanıcı arayüzü.", "Native KDE-Breeze-konforme grafische Benutzeroberfläche.", "Interface graphique native compatible avec KDE Breeze.", "Interfaz gráfica nativa compatible con KDE Breeze.");
        }
        if (key == "about_comp_zbus") {
            return pickLang(lang, "Native KDE Plasma 6 JobView (kuiserver) progress and notification integration.", "KDE Plasma 6 JobView (kuiserver) yerel ilerleme ve bildirim entegrasyonu.", "Native KDE-Plasma-6-JobView-(kuiserver)-Fortschrittsintegration.", "Intégration native KDE Plasma 6 JobView (kuiserver) et notifications.", "Integración nativa de progreso y notificaciones KDE Plasma 6 JobView.");
        }
        if (key == "about_comp_ffmpeg") {
            return pickLang(lang, "High performance video & audio transcoding framework.", "Gelişmiş video ve ses kodlama altyapısı, dalga formu üretimi.", "Leistungsstarkes Video- und Audio-Transkodierungs-Framework.", "Infrastructure haute performance de transcodage vidéo et audio.", "Plataforma de transcodificación de audio y video de alto rendimiento.");
        }
        if (key == "about_comp_potrace") {
            return pickLang(lang, "Vector tracing (bitmap to SVG) and lossless PNG optimization.", "Vektörleştirme (bitmap'ten SVG'ye) ve kayıpsız PNG optimizasyonu.", "Vektorisierung (Bitmap zu SVG) und verlustfreie PNG-Optimierung.", "Vectorisation (bitmap vers SVG) et optimisation PNG sans perte.", "Vectorización (mapa de bits a SVG) y optimización PNG sin pérdidas.");
        }
        if (key == "about_comp_jxl") {
            return pickLang(lang, "Next-gen high efficiency image coding and compression.", "Yeni nesil yüksek verimli görsel sıkıştırma ve kod çözücü.", "Hocheffiziente Bildkodierung und -komprimierung der nächsten Generation.", "Codage et compression d'image haute efficacité nouvelle génération.", "Codificación y compresión de imágenes de alta eficiencia.");
        }
        if (key == "about_comp_pdf") {
            return pickLang(lang, "PDF page rasterization, text extraction, and document compression.", "PDF sayfa ayrıştırma, metin çıkarma ve belge sıkıştırma.", "PDF-Rasterisierung, Textextraktion und Dokumentenkomprimierung.", "Rastérisation PDF, extraction de texte et compression de documents.", "Rasterización PDF, extracción de texto y compresión de documentos.");
        }
        if (key == "about_comp_im") {
            return pickLang(lang, "Image manipulation, PDF merging, and metadata stripping.", "Görsel format dönüşümleri, PDF birleştirme ve metadata temizleme.", "Bildbearbeitung, PDF-Zusammenführung und Metadatenbereinigung.", "Traitement d'images, fusion PDF et suppression des métadonnées.", "Procesamiento de imágenes, fusión de PDF y eliminación de metadatos.");
        }
        if (key == "msg_desk_desc") {
            return pickLang(lang,
                "Install or remove context menu integrations for KDE Dolphin, GNOME Nautilus, Cinnamon Nemo, and XFCE Thunar.",
                "KDE Dolphin, GNOME Nautilus, Cinnamon Nemo ve XFCE Thunar dosya yöneticisi entegrasyonlarını kurabilir veya kaldırabilirsiniz.",
                "Kontextmenü-Integrationen für KDE Dolphin, GNOME Nautilus, Cinnamon Nemo und XFCE Thunar installieren oder entfernen.",
                "Installer ou supprimer les intégrations de menu contextuel pour KDE Dolphin, GNOME Nautilus, Cinnamon Nemo et XFCE Thunar.",
                "Instale o elimine las integraciones del menú contextual para KDE Dolphin, GNOME Nautilus, Cinnamon Nemo y XFCE Thunar.");
        }
        if (key == "msg_install_ok") {
            return pickLang(lang, "Integrations installed successfully.", "Entegrasyonlar başarıyla kuruldu.", "Integrationen erfolgreich installiert.", "Intégrations installées avec succès.", "Integraciones instaladas correctamente.");
        }
        if (key == "msg_install_fail") {
            return pickLang(lang, "Failed to install integrations.", "Entegrasyon kurulumu başarısız oldu.", "Installation der Integrationen fehlgeschlagen.", "Échec de l'installation des intégrations.", "Error al instalar las integraciones.");
        }
        if (key == "msg_uninstall_confirm") {
            return pickLang(lang,
                "Are you sure you want to remove Konvert context menu actions from all file managers?",
                "Konvert sağ tık menüsü eylemlerini tüm dosya yöneticilerinden kaldırmak istediğinize emin misiniz?",
                "Möchten Sie die Konvert-Kontextmenüaktionen wirklich aus allen Dateimanagern entfernen?",
                "Voulez-vous vraiment supprimer les actions Konvert de tous les gestionnaires de fichiers ?",
                "¿Está seguro de que desea eliminar las acciones de Konvert de todos los gestores de archivos?");
        }
        if (key == "msg_uninstall_ok") {
            return pickLang(lang, "Integrations uninstalled.", "Entegrasyonlar kaldırıldı.", "Integrationen entfernt.", "Intégrations désinstallées.", "Integraciones desinstaladas.");
        }
        if (key == "msg_uninstall_fail") {
            return pickLang(lang, "Failed to uninstall integrations.", "Entegrasyonlar kaldırılamadı.", "Entfernen der Integrationen fehlgeschlagen.", "Échec de la désinstallation des intégrations.", "Error al desinstalar las integraciones.");
        }
        if (key == "msg_doctor_fail") {
            return pickLang(lang, "Could not collect diagnostics output.", "Tanılama çıktısı alınamadı.", "Diagnoseausgabe konnte nicht abgerufen werden.", "Impossible de récupérer le diagnostic.", "No se pudo obtener el diagnóstico.");
        }
        if (key == "msg_save_err_title") {
            return pickLang(lang, "Could not save settings", "Ayarlar kaydedilemedi", "Einstellungen konnten nicht gespeichert werden", "Impossible d'enregistrer les paramètres", "No se pudieron guardar los ajustes");
        }
        if (key == "msg_save_err_body") {
            return pickLang(lang, "The settings file could not be written. Check file permissions.", "Ayar dosyası yazılamadı. Disk izinlerini kontrol edin.", "Einstellungsdatei konnte nicht geschrieben werden. Dateirechte prüfen.", "Impossible d'écrire le fichier de configuration. Vérifiez les permissions.", "No se pudo escribir el archivo de configuración. Compruebe los permisos.");
        }
        if (key == "warn_video_lossless") {
            return pickLang(lang,
                "Lossless mode active: CRF is ignored and output files can be 5x-15x larger.",
                "Kayıpsız mod aktif: CRF ayarı yok sayılır ve çıktı dosyaları 5x-15x daha büyük olabilir.",
                "Verlustfreier Modus aktiv: CRF wird ignoriert und Dateien können 5x-15x größer sein.",
                "Mode sans perte actif : CRF ignoré et les fichiers peuvent être 5 à 15 fois plus volumineux.",
                "Modo sin pérdidas activo: se ignora CRF y los archivos pueden ser 5x-15x más grandes.");
        }
        if (key == "tip_photo_quality") {
            return pickLang(lang, "Target image quality percentage (1-100). 85 is recommended.", "Hedef görsel kalitesi (1-100). 85 önerilir.", "Ziel-Bildqualität (1-100). Empfohlen: 85.", "Qualité d'image cible (1-100). 85 recommandé.", "Calidad de imagen objetivo (1-100). Se recomienda 85.");
        }
        if (key == "tip_lossless_photo") {
            return pickLang(lang, "Forces mathematically lossless mode in WebP, PNG, and JXL conversions.", "WebP, PNG ve JXL dönüşümlerinde tam kayıpsız modu zorlar.", "Erzwingt verlustfreien Modus bei WebP-, PNG- und JXL-Konvertierungen.", "Force le mode sans perte pour WebP, PNG et JXL.", "Fuerza el modo sin pérdidas en WebP, PNG y JXL.");
        }
        if (key == "tip_video_crf") {
            return pickLang(lang, "Constant Rate Factor (0-51): 0 is lossless, 18-23 is balanced high quality, 28+ is high compression.", "Sabit Oran Faktörü (0-51): 0 kayıpsız, 18-23 dengeli yüksek kalite, 28+ yüksek sıkıştırma.", "Constant Rate Factor (0-51): 0 verlustfrei, 18-23 ausgewogen, 28+ hohe Komprimierung.", "Facteur de qualité CRF (0-51) : 0 sans perte, 18-23 équilibré, 28+ compression élevée.", "Factor de tasa constante (0-51): 0 sin pérdidas, 18-23 equilibrado, 28+ alta compresión.");
        }
        if (key == "tip_video_preset") {
            return pickLang(lang, "Encoder speed preset: 'slow' or 'medium' offers the best compression-to-time ratio.", "Kodlayıcı hız önayarı: 'slow' veya 'medium' en iyi sıkıştırma/süre dengesini sunar.", "Encoder-Geschwindigkeit: 'slow' oder 'medium' bietet das beste Verhältnis.", "Préréglage de vitesse : 'slow' ou 'medium' offre le meilleur compromis.", "Preajuste de velocidad: 'slow' o 'medium' ofrece el mejor equilibrio.");
        }
        if (key == "tip_video_hwaccel") {
            return pickLang(lang, "GPU hardware acceleration encoder (NVIDIA NVENC or Intel/AMD VA-API). Automatically falls back to CPU if unsupported.", "GPU donanım hızlandırma kodlayıcısı (NVIDIA NVENC veya Intel/AMD VA-API). Desteklenmeyen sistemlerde otomatik olarak CPU'ya geri döner.", "GPU-Hardwarebeschleunigung (NVIDIA NVENC oder VA-API). Fällt automatisch auf CPU zurück.", "Accélération matérielle GPU (NVENC ou VA-API). Repli automatique sur CPU.", "Aceleración por GPU (NVENC o VA-API). Vuelve a CPU si no es compatible.");
        }
        if (key == "tip_video_lossless") {
            return pickLang(lang, "Encodes using x264 RGB with CRF 0 without lossy compression artifacts.", "CRF 0 ile x264 RGB kayıpsız kodlama yapar.", "Kodiert mit x264 RGB und CRF 0 verlustfrei.", "Encode en x264 RGB avec CRF 0 sans perte.", "Codifica con x264 RGB CRF 0 sin pérdida.");
        }
        if (key == "tip_audio_bitrate") {
            return pickLang(lang, "Target audio bitrate for MP3 and AAC encoding. 320k provides transparent CD quality.", "MP3 ve AAC kodlama için hedef ses bit hızı. 320k şeffaf stüdyo kalitesi sunar.", "Ziel-Audiobitrate für MP3 und AAC. 320k bietet transparente CD-Qualität.", "Débit audio cible pour MP3 et AAC. 320k offre une qualité CD transparente.", "Tasa de bits de audio objetivo para MP3 y AAC. 320k ofrece calidad CD.");
        }
        if (key == "tip_pdf_dpi") {
            return pickLang(lang, "DPI resolution when rendering PDF pages to images. 300 DPI is standard print quality.", "PDF sayfaları görsele aktarılırken kullanılacak DPI çözünürlüğü. 300 DPI standart baskı kalitesidir.", "DPI-Auflösung beim Rendern von PDF-Seiten. 300 DPI entspricht Druckqualität.", "Résolution DPI lors du rendu des pages PDF. 300 DPI est standard pour l'impression.", "Resolución DPI al convertir páginas PDF a imagen. 300 DPI es calidad de impresión.");
        }
        if (key == "tip_notifications") {
            return pickLang(lang, "Displays real-time D-Bus notification with progress bar during conversion.", "Dönüştürme esnasında yerel ilerleme çubuğu ve D-Bus bildirimlerini gösterir.", "Zeigt während der Konvertierung D-Bus-Benachrichtigungen mit Fortschrittsbalken.", "Affiche des notifications D-Bus avec barre de progression en temps réel.", "Muestra notificaciones D-Bus en tiempo real con barra de progreso.");
        }
        if (key == "tip_output_dir") {
            return pickLang(lang, "Default directory where converted files will be saved. If left empty, files are saved in the same folder as the input file.", "Dönüştürülen tüm dosyaların kaydedileceği varsayılan klasör. Boş bırakıldığında girdi dosyasıyla aynı klasöre kaydedilir.", "Standardverzeichnis für konvertierte Dateien. Leer speichert im Quellordner.", "Dossier par défaut des fichiers convertis. Vide enregistre dans le dossier source.", "Carpeta predeterminada de salida. Si está vacía se guarda junto al original.");
        }

        return key;
    }

    static QString trActionLabel(const QString &id, const QString &lang) {
        struct Entry {
            const char *id;
            const char *en;
            const char *tr;
            const char *de;
            const char *fr;
            const char *es;
        };
        static const Entry entries[] = {
            {"compress_photo", "Compress Image", "Görseli Sıkıştır", "Bild komprimieren", "Compresser l'image", "Comprimir imagen"},
            {"to_webp", "Convert to WebP", "WebP'ye Dönüştür", "In WebP umwandeln", "Convertir en WebP", "Convertir a WebP"},
            {"to_png", "Convert to PNG", "PNG'ye Dönüştür", "In PNG umwandeln", "Convertir en PNG", "Convertir a PNG"},
            {"to_jpg", "Convert to JPG", "JPG'ye Dönüştür", "In JPG umwandeln", "Convertir en JPG", "Convertir a JPG"},
            {"to_svg", "Trace to SVG", "SVG'ye Vektörleştir", "In SVG vektorisieren", "Vectoriser en SVG", "Vectorizar a SVG"},
            {"to_ico", "Convert to ICO", "ICO'ya Dönüştür", "In ICO umwandeln", "Convertir en ICO", "Convertir a ICO"},
            {"to_heic", "Convert to HEIC", "HEIC'e Dönüştür", "In HEIC umwandeln", "Convertir en HEIC", "Convertir a HEIC"},
            {"to_bmp", "Convert to BMP", "BMP'ye Dönüştür", "In BMP umwandeln", "Convertir en BMP", "Convertir a BMP"},
            {"to_avif", "Convert to AVIF", "AVIF'e Dönüştür", "In AVIF umwandeln", "Convertir en AVIF", "Convertir a AVIF"},
            {"to_jxl", "Convert to JXL", "JXL'e Dönüştür", "In JXL umwandeln", "Convertir en JXL", "Convertir a JXL"},
            {"to_jxl_lossless", "Convert to Lossless JXL", "Kayıpsız JXL'e Dönüştür", "In JXL umwandeln (Verlustfrei)", "Convertir en JXL (Sans perte)", "Convertir a JXL (Sin pérdidas)"},
            {"merge_to_pdf", "Merge to PDF", "PDF Olarak Birleştir", "In ein PDF zusammenführen", "Fusionner en un seul PDF", "Combinar en un solo PDF"},
            {"strip_metadata", "Strip Metadata", "Metadata Temizle", "Metadaten entfernen", "Supprimer les métadonnées", "Eliminar metadatos"},
            {"resize_50", "Resize 50%", "%50 Küçült", "Um 50 % verkleinern", "Redimensionner de 50 %", "Redimensionar al 50%"},
            {"resize_1080p", "Resize to 1080p", "1080p'ye Ölçekle", "Auf 1080p skalieren", "Redimensionner en 1080p", "Escalar a 1080p"},
            {"web_opt", "Optimize for Web", "Web İçin Optimize Et", "Für Web optimieren", "Optimiser pour le Web", "Optimizar para la Web"},
            {"discord_8mb_photo", "Compress for Discord (8 MB)", "Discord İçin Sıkıştır (8 MB)", "Für Discord komprimieren (8 MB)", "Compresser pour Discord (8 Mo)", "Comprimir para Discord (8 MB)"},
            {"email_25mb_photo", "Compress for Email (25 MB)", "E-Posta İçin Sıkıştır (25 MB)", "Für E-Mail komprimieren (25 MB)", "Compresser pour e-mail (25 Mo)", "Comprimir para correo (25 MB)"},
            {"compress_video", "Compress Video", "Videoyu Sıkıştır", "Video komprimieren", "Compresser la vidéo", "Comprimir video"},
            {"to_mp4", "Convert to MP4", "MP4'e Dönüştür", "In MP4 umwandeln", "Convertir en MP4", "Convertir a MP4"},
            {"to_mkv", "Convert to MKV", "MKV'ye Dönüştür", "In MKV umwandeln", "Convertir en MKV", "Convertir a MKV"},
            {"to_webm", "Convert to WebM", "WebM'ye Dönüştür", "In WebM umwandeln", "Convertir en WebM", "Convertir a WebM"},
            {"to_av1", "Convert to AV1", "AV1'e Dönüştür", "In AV1 umwandeln", "Convertir en AV1", "Convertir a AV1"},
            {"to_gif", "Convert to GIF", "GIF'e Dönüştür", "In GIF umwandeln", "Convertir en GIF", "Convertir a GIF"},
            {"extract_audio", "Extract Audio", "Videodan Sesi Ayıkla", "Audio extrahieren", "Extraire l'audio", "Extraer audio"},
            {"discord_8mb_video", "Compress for Discord (8 MB)", "Discord İçin Sıkıştır (8 MB)", "Für Discord komprimieren (8 MB)", "Compresser pour Discord (8 Mo)", "Comprimir para Discord (8 MB)"},
            {"email_25mb_video", "Compress for Email (25 MB)", "E-Posta İçin Sıkıştır (25 MB)", "Für E-Mail komprimieren (25 MB)", "Compresser pour e-mail (25 Mo)", "Comprimir para correo (25 MB)"},
            {"strip_metadata_video", "Strip Metadata", "Metadata Temizle", "Metadaten entfernen", "Supprimer les métadonnées", "Eliminar metadatos"},
            {"extract_frames", "Export Frames", "Kareleri Klasöre Aktar", "Frames in Ordner exportieren", "Exporter les images vers un dossier", "Exportar fotogramas a carpeta"},
            {"to_mp3", "Convert to MP3", "MP3'e Dönüştür", "In MP3 umwandeln", "Convertir en MP3", "Convertir a MP3"},
            {"to_wav", "Convert to WAV", "WAV'a Dönüştür", "In WAV umwandeln", "Convertir en WAV", "Convertir a WAV"},
            {"audio_to_video", "Convert to Video", "Videoya Dönüştür", "In Video umwandeln", "Convertir en vidéo", "Convertir a video"},
            {"compress_doc", "Compress PDF", "PDF Sıkıştır", "PDF komprimieren", "Compresser le PDF", "Comprimir PDF"},
            {"udf_to_pdf", "Convert UDF to PDF", "UDF'i PDF'e Dönüştür", "UDF in PDF umwandeln", "Convertir UDF en PDF", "Convertir UDF a PDF"},
            {"udf_to_txt", "Convert UDF to Text", "UDF'i Metne Dönüştür", "UDF in Text umwandeln", "Convertir UDF en Texte", "Convertir UDF a Texto"},
            {"pdf_to_images", "Extract Pages to PNG", "Sayfaları PNG'ye Çıkar", "Seiten als PNG extrahieren", "Extraire les pages en PNG", "Extraer páginas a PNG"},
            {"docx_to_pdf", "Convert DOCX to PDF", "DOCX'i PDF'e Dönüştür", "DOCX in PDF umwandeln", "Convertir DOCX en PDF", "Convertir DOCX a PDF"},
            {"docx_to_txt", "Convert DOCX to Text", "DOCX'i Metne Dönüştür", "DOCX in Text umwandeln", "Convertir DOCX en Texte", "Convertir DOCX a Texto"},
        };
        for (const auto &entry : entries) {
            if (id == entry.id) {
                return pickLang(lang, entry.en, entry.tr, entry.de, entry.fr, entry.es);
            }
        }
        return id;
    }

    static QString trActionDesc(const QString &id, const QString &lang) {
        struct Entry {
            const char *id;
            const char *en;
            const char *tr;
            const char *de;
            const char *fr;
            const char *es;
        };
        static const Entry entries[] = {
            {"compress_photo", "Compresses image to WebP with optimal balance of maximum visual quality and minimum file size.", "Görseli insan gözünün fark edemeyeceği maksimum görsel kalite ve en küçük dosya boyutu dengesiyle WebP formatında sıkıştırır.", "Komprimiert das Bild in WebP mit optimaler Balance aus Qualität und Dateigröße.", "Compresse l'image en WebP avec un équilibre optimal qualité/poids.", "Comprime la imagen a WebP equilibrando calidad visual y tamaño mínimo."},
            {"to_webp", "Converts to modern WebP format, providing significant size reduction compared to JPEG and PNG.", "Modern WebP formatına dönüştürür. JPEG ve PNG'ye kıyasla yüksek oranda boyut tasarrufu sağlar.", "Wandelt in modernes WebP-Format mit deutlich kleinerer Dateigröße um.", "Convertit au format moderne WebP, bien plus léger que JPEG et PNG.", "Convierte al formato WebP moderno, reduciendo el tamaño frente a JPEG y PNG."},
            {"to_png", "Converts to lossless standard PNG with transparency support.", "Kayıpsız ve şeffaflık (alpha) kanalını destekleyen standart PNG formatına dönüştürür.", "Wandelt in verlustfreies PNG mit Transparenzunterstützung um.", "Convertit en PNG sans perte avec gestion de la transparence.", "Convierte a PNG estándar sin pérdidas con transparencia."},
            {"to_jpg", "Converts to universal standard JPEG image.", "Evrensel uyumluluğa sahip standart JPEG formatına dönüştürür.", "Wandelt in das universelle JPEG-Bildformat um.", "Convertit au format JPEG standard universel.", "Convierte al formato estándar universal JPEG."},
            {"to_svg", "Traces raster image to scalable vector graphic (SVG) format using potrace.", "Görseli potrace ile izleyerek ölçeklenebilir vektör (SVG) grafiğine dönüştürür.", "Vektorisiert Rasterbilder mithilfe von Potrace als skalierbare SVG-Grafik.", "Vectorise une image matricielle en graphique vectoriel SVG via potrace.", "Vectoriza la imagen rasterizada en un gráfico vectorial SVG con potrace."},
            {"to_ico", "Converts to standard Windows and favicon ICO format.", "Web sitesi favicon veya uygulama simgesi olarak kullanılan ICO formatına dönüştürür.", "Wandelt in das Standard-Windows- und Favicon-ICO-Format um.", "Convertit au format ICO standard pour icônes et favicons.", "Convierte al formato ICO estándar para iconos y favicons."},
            {"to_heic", "Converts to high-efficiency HEIC format.", "Apple ve modern cihazlarda kullanılan yüksek verimli HEIC formatına dönüştürür.", "Wandelt in das hocheffiziente HEIC-Format um.", "Convertit au format haute efficacité HEIC.", "Convierte al formato de alta eficiencia HEIC."},
            {"to_bmp", "Converts to uncompressed standard Windows Bitmap format.", "Sıkıştırmasız standart Windows Bitmap formatına dönüştürür.", "Wandelt in unkomprimiertes Windows-Bitmap-Format um.", "Convertit au format Windows Bitmap non compressé.", "Convierte al formato Windows Bitmap sin compresión."},
            {"to_avif", "Next-generation image format based on AV1, providing superior compression.", "AV1 video kodlayıcı tabanlı, yüksek sıkıştırma verimliliği sunan yeni nesil görsel formatı.", "AV1-basiertes Bildformat der nächsten Generation mit überragender Komprimierung.", "Format d'image nouvelle génération basé sur AV1.", "Formato de imagen de nueva generación basado en AV1."},
            {"to_jxl", "Converts to next-generation JPEG XL format with wide color gamut and high compression.", "Yeni nesil JPEG XL formatına dönüştürür. Geniş renk gamutu ve üstün sıkıştırma oranı sunar.", "Wandelt in das moderne JPEG-XL-Format mit hohem Farbraum um.", "Convertit en JPEG XL nouvelle génération avec large gamme chromatique.", "Convierte al formato JPEG XL con amplia gama de color y alta compresión."},
            {"to_jxl_lossless", "Mathematically lossless JPEG XL compression without pixel loss.", "Piksel kaybı olmaksızın matematiksel olarak tam kayıpsız JPEG XL sıkıştırması uygular.", "Mathematisch verlustfreie JPEG-XL-Komprimierung ohne Pixelverlust.", "Compression JPEG XL mathématiquement sans perte.", "Compresión JPEG XL matemáticamente sin pérdidas."},
            {"merge_to_pdf", "Merges all selected images into a single multi-page PDF document.", "Seçilen tüm görselleri tek bir PDF belgesi içerisinde sayfalar halinde birleştirir.", "Führt alle ausgewählten Bilder in einem mehrseitigen PDF-Dokument zusammen.", "Fusionne toutes les images sélectionnées en un seul document PDF.", "Combina todas las imágenes seleccionadas en un único documento PDF."},
            {"strip_metadata", "Cleans GPS coordinates, camera model, shooting date, and personal tags for privacy.", "Görseldeki GPS konumu, çekim tarihi, kamera modeli ve etiketleri temizleyerek gizliliği korur.", "Entfernt GPS-Koordinaten, Kameramodell, Aufnahmedatum und Tags für Datenschutz.", "Supprime coordonnées GPS, appareil et date pour protéger la vie privée.", "Elimina coordenadas GPS, cámara y fecha para proteger la privacidad."},
            {"resize_50", "Reduces image dimensions by half (50%) to shrink file size.", "Görselin genişlik ve yükseklik çözünürlüğünü yarıya indirerek dosya boyutunu büyük oranda azaltır.", "Halbiert Bildbreite und -höhe zur starken Verkleinerung der Datei.", "Réduit les dimensions de moitié (50 %) pour alléger le fichier.", "Reduce las dimensiones a la mitad (50 %) para disminuir el tamaño."},
            {"resize_1080p", "Scales image proportionally to fit within 1920x1080 Full HD bounds.", "Görselin en-boy oranını koruyarak 1920x1080 Full HD sınırlarına sığacak şekilde ölçekler.", "Skaliert das Bild proportional auf maximal 1920x1080 Full HD.", "Redimensionne proportionnellement dans la limite Full HD 1920x1080.", "Escala proporcionalmente la imagen dentro de 1920x1080 Full HD."},
            {"web_opt", "Converts to WebP with 82% quality and stripped EXIF for fast web delivery.", "WebP formatında %82 kalite ve sıfır EXIF ile web için ideal boyuta getirir.", "Wandelt in WebP (Qualität 82 %) ohne EXIF für das Web um.", "Convertit en WebP qualité 82 % sans EXIF pour le Web.", "Convierte a WebP al 82 % de calidad y sin EXIF para la Web."},
            {"discord_8mb_photo", "Optimizes image to guarantee size under 8 MB.", "Görseli 8 MB sınırına sığacak şekilde optimize eder.", "Optimiert das Bild sicher unter die 8-MB-Grenze.", "Optimise l'image pour rester sous 8 Mo.", "Optimiza la imagen para que no supere los 8 MB."},
            {"email_25mb_photo", "Optimizes image to fit standard 25 MB email attachment limits.", "Görseli 25 MB e-posta ek sınırına uyacak şekilde optimize eder.", "Optimiert das Bild für E-Mail-Anhänge bis 25 MB.", "Optimise l'image pour la limite de 25 Mo des e-mails.", "Optimiza la imagen para el límite de 25 MB en correos electrónicos."},
            {"compress_video", "Compresses video with H.264 CRF 28 and slow preset preserving visual quality.", "Videoyu H.264 CRF 28 ve slow kodlama önayarı ile görsel kalitesini koruyarak %80'e varan oranda sıkıştırır.", "Komprimiert Video mit H.264 CRF 28 und Slow-Preset bei hoher Qualität.", "Compresse la vidéo en H.264 CRF 28 (preset slow) en préservant la qualité.", "Comprime el video con H.264 CRF 28 y preajuste slow preservando la calidad."},
            {"to_mp4", "Converts to standard H.264/AAC MP4 for universal device and browser compatibility.", "Evrensel cihaz ve tarayıcı uyumluluğu için standart H.264/AAC MP4 formatına dönüştürür.", "Wandelt in universelles H.264/AAC-MP4 für Geräte und Browser um.", "Convertit en MP4 H.264/AAC compatible avec tous appareils et navigateurs.", "Convierte a MP4 H.264/AAC estándar compatible con todos los dispositivos."},
            {"to_mkv", "Converts to versatile Matroska video container supporting multiple audio and subtitle streams.", "Çoklu ses ve altyazı izlerini destekleyen esnek Matroska video kapsayıcısına dönüştürür.", "Wandelt in den flexiblen Matroska-Container für Audio- und Untertitelspuren um.", "Convertit au conteneur Matroska prenant en charge plusieurs pistes audio et sous-titres.", "Convierte al contenedor Matroska compatible con múltiples pistas de audio y subtítulos."},
            {"to_webm", "Converts to open-source VP9 WebM format for web and HTML5 streaming.", "Web ve HTML5 uyumlu açık kaynaklı VP9 video formatına dönüştürür.", "Wandelt in quelloffenes VP9-WebM für Web- und HTML5-Streaming um.", "Convertit au format ouvert VP9 WebM pour le streaming Web et HTML5.", "Convierte al formato abierto VP9 WebM para la Web y HTML5."},
            {"to_av1", "Converts to modern open AV1 video format with high compression efficiency via hardware acceleration or SVT-AV1.", "Modern açık AV1 formatına dönüştürür. Donanım hızlandırma veya SVT-AV1 ile yüksek sıkıştırma verimliliği sağlar.", "Wandelt in modernes AV1-Videoformat per Hardware-Beschleunigung oder SVT-AV1 um.", "Convertit au format vidéo ouvert AV1 avec SVT-AV1 ou accélération matérielle.", "Convierte al formato de video AV1 moderno con SVT-AV1 o aceleración GPU."},
            {"to_gif", "Converts the first 15 seconds of video to a high-quality animated GIF using two-pass palette optimization.", "İki geçişli renk paleti analiziyle videonun ilk 15 saniyesini yüksek kaliteli animasyonlu GIF'e dönüştürür.", "Wandelt die ersten 15 Sekunden per Zwei-Pass-Palette in ein animiertes GIF um.", "Convertit les 15 premières secondes en GIF animé via palette deux passes.", "Convierte los primeros 15 segundos en GIF animado con paleta de dos pasadas."},
            {"extract_audio", "Demuxes and extracts the audio stream directly into an MP3 file.", "Videonun ses izini doğrudan ayrı bir MP3 ses dosyası olarak dışa aktarır.", "Extrahiert die Audiospur direkt als MP3-Datei.", "Extrait directement la piste audio vers un fichier MP3.", "Extrae directamente la pista de audio a un archivo MP3."},
            {"discord_8mb_video", "Automatically computes video bitrate according to length to fit within 8 MB.", "Videonun süresine göre bit hızını otomatik ayarlayarak 8 MB boyutuna sığdırır.", "Berechnet die Bitrate automatisch passend für das 8-MB-Limit.", "Calcule le débit automatiquement pour rester sous 8 Mo.", "Calcula automáticamente el bitrate para ajustarse a 8 MB."},
            {"email_25mb_video", "Automatically calculates bitrate to fit within 25 MB email attachment limit.", "Videonun süresine göre bit hızını otomatik ayarlayarak 25 MB e-posta ek sınırına sığdırır.", "Berechnet die Bitrate automatisch für E-Mail-Anhänge bis 25 MB.", "Calcule automatiquement le débit pour la limite e-mail de 25 Mo.", "Calcula el bitrate para el límite de 25 MB de correo."},
            {"strip_metadata_video", "Removes all metadata tags via lossless stream copy without re-encoding or inflating file size.", "Yeniden kodlama yapmadan ve dosya boyutunu büyütmeden tüm metadata etiketlerini kopya akışla temizler.", "Entfernt alle Metadaten verlustfrei per Stream-Copy ohne Neukodierung.", "Supprime toutes les métadonnées sans réencodage par copie de flux.", "Elimina todos los metadatos sin recodificar mediante copia directa de flujo."},
            {"extract_frames", "Exports all video frames as individual PNG images to a dedicated folder.", "Videonun her video karesini PNG resimleri olarak yeni bir klasöre döker.", "Exportiert alle Videobilder als PNG-Dateien in einen eigenen Ordner.", "Exporte toutes les images de la vidéo en PNG dans un dossier dédié.", "Exporta todos los fotogramas del video como PNG a una carpeta dedicada."},
            {"to_mp3", "Converts to universal standard MP3 audio.", "En yaygın evrensel ses formatı olan MP3'e dönüştürür.", "Wandelt in universelles Standard-MP3-Audio um.", "Convertit en audio MP3 universel.", "Convierte a audio MP3 estándar universal."},
            {"to_wav", "Converts to uncompressed studio-quality PCM WAV audio.", "Stüdyo kalitesinde, tam kayıpsız ve sıkıştırmasız PCM WAV ses dosyası üretir.", "Wandelt in unkomprimiertes PCM-WAV-Audio in Studioqualität um.", "Convertit en audio PCM WAV sans compression de qualité studio.", "Convierte a audio PCM WAV sin compresión con calidad de estudio."},
            {"audio_to_video", "Generates an aesthetic KDE-blue waveform visualization video in MP4 format from audio.", "Ses dosyası için arka planda KDE mavisi ses dalga formu üreterek paylaşılabilir bir MP4 videosu oluşturur.", "Erzeugt ein MP4-Video mit blauer KDE-Wellenform-Visualisierung.", "Génère une vidéo MP4 avec forme d'onde bleue KDE depuis l'audio.", "Genera un video MP4 con visualización de onda en azul KDE."},
            {"compress_doc", "Compresses high-resolution embedded images and streams in PDF files to minimize file size.", "PDF belgesinin içerisindeki yüksek çözünürlüklü görselleri ve veri akışlarını sıkıştırarak dosya boyutunu düşürür.", "Komprimiert eingebettete Bilder und Datenströme in PDF-Dateien.", "Compresse les images intégrées et les flux des PDF pour réduire la taille.", "Comprime imágenes incrustadas y flujos de PDF para reducir el tamaño."},
            {"udf_to_pdf", "Converts Turkish National Judicial Network (.udf) legal documents directly into standard PDF.", "Ulusal Yargı Ağı Bilişim Sistemi (.udf) belgelerini doğrudan görüntülenebilir PDF belgesine dönüştürür.", "Wandelt UYAP-.udf-Rechtsdokumente direkt in Standard-PDF um.", "Convertit directement les documents UYAP .udf en PDF standard.", "Convierte documentos legales .udf de UYAP directamente a PDF."},
            {"udf_to_txt", "Extracts clean text content from .udf documents into a standard text file.", "UYAP .udf belgesinin içindeki saf metin içeriğini ayrıştırarak .txt dosyası olarak kaydeder.", "Extrahiert reinen Text aus .udf-Dokumenten als Textdatei.", "Extrait le texte brut des documents .udf vers un fichier texte.", "Extrae el texto limpio de documentos .udf a un archivo de texto."},
            {"pdf_to_images", "Renders all pages of a PDF document into high-resolution PNG image files.", "PDF belgesinin her bir sayfasını yüksek çözünürlüklü PNG resimleri olarak ayrı ayrı klasöre aktarır.", "Rendert alle PDF-Seiten als hochauflösende PNG-Dateien in einen Ordner.", "Convertit toutes les pages PDF en fichiers PNG haute résolution.", "Convierte todas las páginas PDF en imágenes PNG de alta resolución."},
            {"docx_to_pdf", "Converts Microsoft Word (.docx) documents to PDF using LibreOffice.", "Microsoft Word (.docx) belgelerini LibreOffice motoruyla PDF belgesine dönüştürür.", "Wandelt Word-(.docx)-Dokumente mit LibreOffice in PDF um.", "Convertit les documents Word (.docx) en PDF via LibreOffice.", "Convierte documentos Word (.docx) a PDF usando LibreOffice."},
            {"docx_to_txt", "Extracts text content from .docx documents into a standard text file.", "DOCX belgesinin metin içeriğini ayrıştırarak .txt dosyası olarak kaydeder.", "Extrahiert Textinhalte aus .docx-Dokumenten in eine Textdatei.", "Extrait le texte des documents .docx vers un fichier texte.", "Extrae el texto de documentos .docx a un archivo de texto."},
        };
        for (const auto &entry : entries) {
            if (id == entry.id) {
                return pickLang(lang, entry.en, entry.tr, entry.de, entry.fr, entry.es);
            }
        }
        return "";
    }

private:
    static QString pickLang(
        const QString &lang,
        const char *en,
        const char *tr,
        const char *de,
        const char *fr,
        const char *es
    ) {
        if (lang == "tr") return QString::fromUtf8(tr);
        if (lang == "de") return QString::fromUtf8(de);
        if (lang == "fr") return QString::fromUtf8(fr);
        if (lang == "es") return QString::fromUtf8(es);
        return QString::fromUtf8(en);
    }
};
