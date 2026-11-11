#pragma once

#include <QString>
#include <QLocale>

class AppTranslator {
public:
    static QString getActiveLanguage(const QString &configured) {
        if (configured.isEmpty() || configured == "auto") {
            QString sys = QLocale::system().name().left(2).toLower();
            if (sys == "tr" || sys == "de" || sys == "fr" || sys == "es") {
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
            return "CRF Quality:";
        }
        if (key == "lbl_video_preset") {
            if (lang == "tr") return "Sıkıştırma Hız / Önayar:";
            if (lang == "de") return "Geschwindigkeit / Preset:";
            if (lang == "fr") return "Vitesse / Préréglage :";
            if (lang == "es") return "Velocidad / Ajuste:";
            return "Compression Speed / Preset:";
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
        if (key == "about_tab_community") {
            if (lang == "tr") return "Yazarlar & Topluluk";
            if (lang == "de") return "Autoren & Community";
            if (lang == "fr") return "Auteurs & Communauté";
            if (lang == "es") return "Autores y Comunidad";
            return "Authors & Community";
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

        return key;
    }
};
