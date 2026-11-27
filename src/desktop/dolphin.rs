use anyhow::{Context, Result};
use directories::BaseDirs;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

pub struct ActionDef {
    pub id: &'static str,
    pub action_key: &'static str,
    pub category: &'static str, // "photos", "videos", "music", "pdf", "udf", "word"
    pub name_en: &'static str,
    pub name_tr: &'static str,
    pub name_de: &'static str,
    pub name_fr: &'static str,
    pub name_es: &'static str,
    pub icon: &'static str,
    pub cmd_args: &'static str,
    pub default_enabled: bool,
}

pub const ALL_ACTIONS: &[ActionDef] = &[
    // PHOTOS - Core Optimization & Privacy (Default Enabled)
    ActionDef {
        id: "compress_photo",
        action_key: "01_compressPhoto",
        category: "photos",
        name_en: "Compress Image",
        name_tr: "Görseli Sıkıştır",
        name_de: "Bild komprimieren",
        name_fr: "Compresser l'image",
        name_es: "Comprimir imagen",
        icon: "konvert-compress",
        cmd_args: "-t compress %F",
        default_enabled: true,
    },
    ActionDef {
        id: "web_opt",
        action_key: "02_webOpt",
        category: "photos",
        name_en: "Optimize for Web",
        name_tr: "Web İçin Optimize Et",
        name_de: "Für Web optimieren",
        name_fr: "Optimiser pour le Web",
        name_es: "Optimizar para la Web",
        icon: "konvert-globe",
        cmd_args: "-t web_opt %F",
        default_enabled: true,
    },
    ActionDef {
        id: "strip_metadata",
        action_key: "03_stripMetadata",
        category: "photos",
        name_en: "Strip Metadata",
        name_tr: "Metadata Temizle",
        name_de: "Metadaten entfernen",
        name_fr: "Supprimer les métadonnées",
        name_es: "Eliminar metadatos",
        icon: "konvert-shield",
        cmd_args: "-t strip-metadata %F",
        default_enabled: true,
    },
    ActionDef {
        id: "resize_1080p",
        action_key: "04_resize1080p",
        category: "photos",
        name_en: "Resize to 1080p",
        name_tr: "1080p'ye Ölçekle",
        name_de: "Auf 1080p skalieren",
        name_fr: "Redimensionner en 1080p",
        name_es: "Escalar a 1080p",
        icon: "konvert-monitor",
        cmd_args: "-t resize_1080p %F",
        default_enabled: true,
    },
    ActionDef {
        id: "merge_to_pdf",
        action_key: "05_mergeToPDF",
        category: "photos",
        name_en: "Merge to PDF",
        name_tr: "PDF Olarak Birleştir",
        name_de: "In ein PDF zusammenführen",
        name_fr: "Fusionner en un seul PDF",
        name_es: "Combinar en un solo PDF",
        icon: "konvert-layers",
        cmd_args: "--merge-pdf %F",
        default_enabled: true,
    },
    // PHOTOS - Core Popular Formats (Default Enabled)
    ActionDef {
        id: "to_webp",
        action_key: "06_toWebP",
        category: "photos",
        name_en: "Convert to WebP",
        name_tr: "WebP'ye Dönüştür",
        name_de: "In WebP umwandeln",
        name_fr: "Convertir en WebP",
        name_es: "Convertir a WebP",
        icon: "konvert-image",
        cmd_args: "-t webp %F",
        default_enabled: true,
    },
    ActionDef {
        id: "to_jpg",
        action_key: "07_toJPG",
        category: "photos",
        name_en: "Convert to JPG",
        name_tr: "JPG'ye Dönüştür",
        name_de: "In JPG umwandeln",
        name_fr: "Convertir en JPG",
        name_es: "Convertir a JPG",
        icon: "konvert-image",
        cmd_args: "-t jpg %F",
        default_enabled: true,
    },
    ActionDef {
        id: "to_png",
        action_key: "08_toPNG",
        category: "photos",
        name_en: "Convert to PNG",
        name_tr: "PNG'ye Dönüştür",
        name_de: "In PNG umwandeln",
        name_fr: "Convertir en PNG",
        name_es: "Convertir a PNG",
        icon: "konvert-image",
        cmd_args: "-t png %F",
        default_enabled: true,
    },
    ActionDef {
        id: "to_avif",
        action_key: "09_toAVIF",
        category: "photos",
        name_en: "Convert to AVIF",
        name_tr: "AVIF'e Dönüştür",
        name_de: "In AVIF umwandeln",
        name_fr: "Convertir en AVIF",
        name_es: "Convertir a AVIF",
        icon: "konvert-image",
        cmd_args: "-t avif %F",
        default_enabled: true,
    },
    ActionDef {
        id: "to_jxl",
        action_key: "10_toJXL",
        category: "photos",
        name_en: "Convert to JXL",
        name_tr: "JXL'e Dönüştür",
        name_de: "In JXL umwandeln",
        name_fr: "Convertir en JXL",
        name_es: "Convertir a JXL",
        icon: "konvert-image",
        cmd_args: "-t jxl %F",
        default_enabled: true,
    },
    // PHOTOS - Opt-in Niche Formats & Presets (Toggleable in Settings)
    ActionDef {
        id: "to_svg",
        action_key: "11_toSVG",
        category: "photos",
        name_en: "Trace to SVG",
        name_tr: "SVG'ye Vektörleştir",
        name_de: "In SVG vektorisieren",
        name_fr: "Vectoriser en SVG",
        name_es: "Vectorizar a SVG",
        icon: "konvert-image",
        cmd_args: "-t svg %F",
        default_enabled: false,
    },
    ActionDef {
        id: "to_ico",
        action_key: "12_toICO",
        category: "photos",
        name_en: "Convert to ICO",
        name_tr: "ICO'ya Dönüştür",
        name_de: "In ICO umwandeln",
        name_fr: "Convertir en ICO",
        name_es: "Convertir a ICO",
        icon: "konvert-image",
        cmd_args: "-t ico %F",
        default_enabled: false,
    },
    ActionDef {
        id: "to_heic",
        action_key: "13_toHEIC",
        category: "photos",
        name_en: "Convert to HEIC",
        name_tr: "HEIC'e Dönüştür",
        name_de: "In HEIC umwandeln",
        name_fr: "Convertir en HEIC",
        name_es: "Convertir a HEIC",
        icon: "konvert-image",
        cmd_args: "-t heic %F",
        default_enabled: false,
    },
    ActionDef {
        id: "to_bmp",
        action_key: "14_toBMP",
        category: "photos",
        name_en: "Convert to BMP",
        name_tr: "BMP'ye Dönüştür",
        name_de: "In BMP umwandeln",
        name_fr: "Convertir en BMP",
        name_es: "Convertir a BMP",
        icon: "konvert-image",
        cmd_args: "-t bmp %F",
        default_enabled: false,
    },
    ActionDef {
        id: "to_jxl_lossless",
        action_key: "15_toJXLLossless",
        category: "photos",
        name_en: "Convert to Lossless JXL",
        name_tr: "Kayıpsız JXL'e Dönüştür",
        name_de: "In JXL umwandeln (Verlustfrei)",
        name_fr: "Convertir en JXL (Sans perte)",
        name_es: "Convertir a JXL (Sin pérdidas)",
        icon: "konvert-image",
        cmd_args: "-t jxl --lossless %F",
        default_enabled: false,
    },
    ActionDef {
        id: "resize_50",
        action_key: "16_resize50",
        category: "photos",
        name_en: "Resize 50%",
        name_tr: "%50 Küçült",
        name_de: "Um 50 % verkleinern",
        name_fr: "Redimensionner de 50 %",
        name_es: "Redimensionar al 50%",
        icon: "konvert-scaling",
        cmd_args: "-t resize_50 %F",
        default_enabled: false,
    },
    ActionDef {
        id: "discord_8mb_photo",
        action_key: "17_discord8MBPhoto",
        category: "photos",
        name_en: "Compress for Discord (8 MB)",
        name_tr: "Discord İçin Sıkıştır (8 MB)",
        name_de: "Für Discord komprimieren (8 MB)",
        name_fr: "Compresser pour Discord (8 Mo)",
        name_es: "Comprimir para Discord (8 MB)",
        icon: "konvert-mail",
        cmd_args: "-t discord_8mb %F",
        default_enabled: false,
    },
    ActionDef {
        id: "email_25mb_photo",
        action_key: "18_email25MBPhoto",
        category: "photos",
        name_en: "Compress for Email (25 MB)",
        name_tr: "E-Posta İçin Sıkıştır (25 MB)",
        name_de: "Für E-Mail komprimieren (25 MB)",
        name_fr: "Compresser pour e-mail (25 Mo)",
        name_es: "Comprimir para correo (25 MB)",
        icon: "konvert-mail",
        cmd_args: "-t email_25mb %F",
        default_enabled: false,
    },
    // VIDEOS
    ActionDef {
        id: "compress_video",
        action_key: "01_compressVideo",
        category: "videos",
        name_en: "Compress Video",
        name_tr: "Videoyu Sıkıştır",
        name_de: "Video komprimieren",
        name_fr: "Compresser la vidéo",
        name_es: "Comprimir video",
        icon: "konvert-compress",
        cmd_args: "-t compress %F",
        default_enabled: true,
    },
    ActionDef {
        id: "discord_8mb_video",
        action_key: "02_discord8MBVideo",
        category: "videos",
        name_en: "Compress for Discord (8 MB)",
        name_tr: "Discord İçin Sıkıştır (8 MB)",
        name_de: "Für Discord komprimieren (8 MB)",
        name_fr: "Compresser pour Discord (8 Mo)",
        name_es: "Comprimir para Discord (8 MB)",
        icon: "konvert-mail",
        cmd_args: "-t discord_8mb %F",
        default_enabled: true,
    },
    ActionDef {
        id: "email_25mb_video",
        action_key: "03_email25MBVideo",
        category: "videos",
        name_en: "Compress for Email (25 MB)",
        name_tr: "E-Posta İçin Sıkıştır (25 MB)",
        name_de: "Für E-Mail komprimieren (25 MB)",
        name_fr: "Compresser pour e-mail (25 Mo)",
        name_es: "Comprimir para correo (25 MB)",
        icon: "konvert-mail",
        cmd_args: "-t email_25mb %F",
        default_enabled: true,
    },
    ActionDef {
        id: "strip_metadata_video",
        action_key: "04_stripMetadataVideo",
        category: "videos",
        name_en: "Strip Metadata",
        name_tr: "Metadata Temizle",
        name_de: "Metadaten entfernen",
        name_fr: "Supprimer les métadonnées",
        name_es: "Eliminar metadatos",
        icon: "konvert-shield",
        cmd_args: "-t strip-metadata %F",
        default_enabled: true,
    },
    ActionDef {
        id: "extract_audio",
        action_key: "05_extractAudio",
        category: "videos",
        name_en: "Extract Audio",
        name_tr: "Videodan Sesi Ayıkla",
        name_de: "Audio extrahieren",
        name_fr: "Extraire l'audio",
        name_es: "Extraer audio",
        icon: "konvert-audio",
        cmd_args: "-t extract-audio %F",
        default_enabled: true,
    },
    ActionDef {
        id: "to_mp4",
        action_key: "06_toMP4",
        category: "videos",
        name_en: "Convert to MP4",
        name_tr: "MP4'e Dönüştür",
        name_de: "In MP4 umwandeln",
        name_fr: "Convertir en MP4",
        name_es: "Convertir a MP4",
        icon: "konvert-video",
        cmd_args: "-t mp4 %F",
        default_enabled: true,
    },
    ActionDef {
        id: "to_av1",
        action_key: "07_toAV1",
        category: "videos",
        name_en: "Convert to AV1",
        name_tr: "AV1'e Dönüştür",
        name_de: "In AV1 umwandeln",
        name_fr: "Convertir en AV1",
        name_es: "Convertir a AV1",
        icon: "konvert-video",
        cmd_args: "-t av1 %F",
        default_enabled: true,
    },
    ActionDef {
        id: "to_webm",
        action_key: "08_toWebM",
        category: "videos",
        name_en: "Convert to WebM",
        name_tr: "WebM'ye Dönüştür",
        name_de: "In WebM umwandeln",
        name_fr: "Convertir en WebM",
        name_es: "Convertir a WebM",
        icon: "konvert-video",
        cmd_args: "-t webm %F",
        default_enabled: true,
    },
    ActionDef {
        id: "to_mkv",
        action_key: "09_toMKV",
        category: "videos",
        name_en: "Convert to MKV",
        name_tr: "MKV'ye Dönüştür",
        name_de: "In MKV umwandeln",
        name_fr: "Convertir en MKV",
        name_es: "Convertir a MKV",
        icon: "konvert-video",
        cmd_args: "-t mkv %F",
        default_enabled: true,
    },
    ActionDef {
        id: "to_gif",
        action_key: "10_toGIF",
        category: "videos",
        name_en: "Convert to GIF",
        name_tr: "GIF'e Dönüştür",
        name_de: "In GIF umwandeln",
        name_fr: "Convertir en GIF",
        name_es: "Convertir a GIF",
        icon: "konvert-video",
        cmd_args: "-t gif %F",
        default_enabled: true,
    },
    ActionDef {
        id: "extract_frames",
        action_key: "11_extractFrames",
        category: "videos",
        name_en: "Export Frames",
        name_tr: "Kareleri Klasöre Aktar",
        name_de: "Frames in Ordner exportieren",
        name_fr: "Exporter les images vers un dossier",
        name_es: "Exportar fotogramas a carpeta",
        icon: "konvert-folder",
        cmd_args: "--extract-frames %F",
        default_enabled: false,
    },
    // MUSIC / AUDIO
    ActionDef {
        id: "to_mp3",
        action_key: "01_toMP3",
        category: "music",
        name_en: "Convert to MP3",
        name_tr: "MP3'e Dönüştür",
        name_de: "In MP3 umwandeln",
        name_fr: "Convertir en MP3",
        name_es: "Convertir a MP3",
        icon: "konvert-audio",
        cmd_args: "-t mp3 %F",
        default_enabled: true,
    },
    ActionDef {
        id: "to_wav",
        action_key: "02_toWAV",
        category: "music",
        name_en: "Convert to WAV",
        name_tr: "WAV'a Dönüştür",
        name_de: "In WAV umwandeln",
        name_fr: "Convertir en WAV",
        name_es: "Convertir a WAV",
        icon: "konvert-audio",
        cmd_args: "-t wav %F",
        default_enabled: true,
    },
    ActionDef {
        id: "audio_to_video",
        action_key: "03_audioToVideo",
        category: "music",
        name_en: "Convert to Video",
        name_tr: "Videoya Dönüştür",
        name_de: "In Video umwandeln",
        name_fr: "Convertir en vidéo",
        name_es: "Convertir a video",
        icon: "konvert-video",
        cmd_args: "-t audio-to-video %F",
        default_enabled: true,
    },
    // DOCUMENTS - PDF
    ActionDef {
        id: "compress_doc",
        action_key: "01_compressDoc",
        category: "pdf",
        name_en: "Compress PDF",
        name_tr: "PDF Sıkıştır",
        name_de: "PDF komprimieren",
        name_fr: "Compresser le PDF",
        name_es: "Comprimir PDF",
        icon: "konvert-compress",
        cmd_args: "-t compress %F",
        default_enabled: true,
    },
    ActionDef {
        id: "pdf_to_images",
        action_key: "02_pdfToImages",
        category: "pdf",
        name_en: "Extract Pages to PNG",
        name_tr: "Sayfaları PNG'ye Çıkar",
        name_de: "Seiten als PNG extrahieren",
        name_fr: "Extraire les pages en PNG",
        name_es: "Extraer páginas a PNG",
        icon: "konvert-image",
        cmd_args: "-t png %F",
        default_enabled: true,
    },
    // DOCUMENTS - UDF
    ActionDef {
        id: "udf_to_pdf",
        action_key: "01_udfToPDF",
        category: "udf",
        name_en: "Convert UDF to PDF",
        name_tr: "UDF'i PDF'e Dönüştür",
        name_de: "UDF in PDF umwandeln",
        name_fr: "Convertir UDF en PDF",
        name_es: "Convertir UDF a PDF",
        icon: "konvert-document",
        cmd_args: "-t pdf %F",
        default_enabled: true,
    },
    ActionDef {
        id: "udf_to_txt",
        action_key: "02_udfToTXT",
        category: "udf",
        name_en: "Convert UDF to Text",
        name_tr: "UDF'i Metne Dönüştür",
        name_de: "UDF in Text umwandeln",
        name_fr: "Convertir UDF en Texte",
        name_es: "Convertir UDF a Texto",
        icon: "konvert-document",
        cmd_args: "-t txt %F",
        default_enabled: true,
    },
    // DOCUMENTS - WORD
    ActionDef {
        id: "docx_to_pdf",
        action_key: "01_docxToPDF",
        category: "word",
        name_en: "Convert DOCX to PDF",
        name_tr: "DOCX'i PDF'e Dönüştür",
        name_de: "DOCX in PDF umwandeln",
        name_fr: "Convertir DOCX en PDF",
        name_es: "Convertir DOCX a PDF",
        icon: "konvert-document",
        cmd_args: "-t pdf %F",
        default_enabled: true,
    },
    ActionDef {
        id: "docx_to_txt",
        action_key: "02_docxToTXT",
        category: "word",
        name_en: "Convert DOCX to Text",
        name_tr: "DOCX'i Metne Dönüştür",
        name_de: "DOCX in Text umwandeln",
        name_fr: "Convertir DOCX en Texte",
        name_es: "Convertir DOCX a Texto",
        icon: "konvert-document",
        cmd_args: "-t txt %F",
        default_enabled: true,
    },
];

/// Nautilus/Thunar gibi basit entegrasyonlarda sunulan ortak action kümesi.
/// Tek kaynak: `ALL_ACTIONS` (isim, ikon ve komutlar oradan gelir).
pub const QUICK_ACTION_IDS: &[&str] =
    &["to_webp", "to_png", "to_jpg", "to_avif", "to_mp3", "to_gif"];

pub fn find_action(id: &str) -> Option<&'static ActionDef> {
    ALL_ACTIONS.iter().find(|action| action.id == id)
}

pub fn get_service_menus_dir() -> Result<PathBuf> {
    let base_dirs = BaseDirs::new().context("Kullanıcı ev dizini bulunamadı")?;
    let path = base_dirs.data_local_dir().join("kio").join("servicemenus");
    Ok(path)
}

pub fn is_dolphin_installed() -> bool {
    let Ok(dir) = get_service_menus_dir() else {
        return false;
    };
    let Ok(entries) = fs::read_dir(dir) else {
        return false;
    };
    entries.flatten().any(|e| {
        let name = e.file_name();
        let s = name.to_string_lossy();
        s.starts_with("konvert_") && s.ends_with(".desktop")
    })
}

struct CategoryInfo {
    key: &'static str,
    file_name: &'static str,
    mime_types: &'static str,
}

fn quote_desktop_exec(exec: &str) -> String {
    let needs_quoting = exec.is_empty()
        || exec.chars().any(|c| {
            c.is_whitespace()
                || matches!(
                    c,
                    '\\' | '"'
                        | '\''
                        | '>'
                        | '<'
                        | '~'
                        | '|'
                        | '&'
                        | ';'
                        | '$'
                        | '*'
                        | '?'
                        | '#'
                        | '('
                        | ')'
                        | '`'
                )
        });
    if !needs_quoting {
        return exec.replace('%', "%%");
    }

    let mut quoted = String::with_capacity(exec.len() + 4);
    quoted.push('"');
    for ch in exec.chars() {
        match ch {
            '"' | '`' | '$' | '\\' => {
                quoted.push('\\');
                quoted.push(ch);
            }
            '%' => quoted.push_str("%%"),
            _ => quoted.push(ch),
        }
    }
    quoted.push('"');
    quoted
}

const CATEGORIES: &[CategoryInfo] = &[
    CategoryInfo {
        key: "photos",
        file_name: "konvert_photos.desktop",
        mime_types: "image/png;image/jpeg;image/webp;image/bmp;image/svg+xml;image/x-icon;image/vnd.microsoft.icon;image/tiff;image/avif;image/jxl;image/qoi;image/heic;image/heif;image/gif;",
    },
    CategoryInfo {
        key: "videos",
        file_name: "konvert_videos.desktop",
        mime_types: "video/mp4;video/webm;video/x-matroska;video/avi;video/quicktime;video/x-msvideo;video/x-flv;video/mp2t;video/x-ms-wmv;video/x-m4v;",
    },
    CategoryInfo {
        key: "music",
        file_name: "konvert_music.desktop",
        mime_types: "audio/mpeg;audio/x-wav;audio/wav;audio/flac;audio/ogg;audio/opus;audio/mp4;audio/aac;audio/x-m4a;audio/x-ms-wma;audio/x-aiff;audio/x-opus+ogg;audio/x-vorbis+ogg;",
    },
    CategoryInfo {
        key: "pdf",
        file_name: "konvert_pdf.desktop",
        mime_types: "application/pdf;",
    },
    CategoryInfo {
        key: "udf",
        file_name: "konvert_udf.desktop",
        mime_types: "application/x-udf;",
    },
    CategoryInfo {
        key: "word",
        file_name: "konvert_word.desktop",
        mime_types: "application/vnd.openxmlformats-officedocument.wordprocessingml.document;",
    },
];

pub fn install_dolphin_service(custom_exec: Option<&str>) -> Result<PathBuf> {
    let cfg = crate::config::Config::load();
    install_dolphin_service_with_options(&cfg.context_menu.enabled_actions, custom_exec, None)
}

pub fn install_dolphin_service_with_actions(
    enabled_action_ids: &[String],
    custom_exec: Option<&str>,
) -> Result<PathBuf> {
    install_dolphin_service_with_options(enabled_action_ids, custom_exec, None)
}

pub fn install_dolphin_service_with_options(
    enabled_action_ids: &[String],
    custom_exec: Option<&str>,
    custom_dir: Option<PathBuf>,
) -> Result<PathBuf> {
    let dir = match custom_dir {
        Some(d) => d,
        None => get_service_menus_dir()?,
    };
    fs::create_dir_all(&dir)?;

    let legacy_files = ["konvert.desktop", "konvert_documents.desktop"];
    for legacy in &legacy_files {
        let p = dir.join(legacy);
        if p.exists() {
            let _ = fs::remove_file(&p);
        }
    }

    let exec = match custom_exec {
        Some(e) => e.to_string(),
        None => {
            let p = std::env::current_exe()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| "konvert".to_string());
            if p.contains("target/release") || p.contains("target/debug") {
                let local_bin = dirs_bin();
                if local_bin.exists() {
                    local_bin.to_string_lossy().to_string()
                } else {
                    p
                }
            } else {
                p
            }
        }
    };

    for cat in CATEGORIES {
        let active_actions: Vec<&ActionDef> = ALL_ACTIONS
            .iter()
            .filter(|act| {
                act.category == cat.key && enabled_action_ids.iter().any(|id| id == act.id)
            })
            .collect();

        let file_path = dir.join(cat.file_name);

        if active_actions.is_empty() {
            if file_path.exists() {
                let _ = fs::remove_file(&file_path);
            }
            continue;
        }

        let quoted_exec = quote_desktop_exec(&exec);
        let actions_keys = active_actions
            .iter()
            .map(|a| a.action_key)
            .collect::<Vec<_>>()
            .join(";");

        let mut content = format!(
            r#"[Desktop Entry]
Type=Service
X-KDE-ServiceTypes=KonqPopupMenu/Plugin
MimeType={mime_types}
Actions=00_openSettings;{actions_keys};
X-KDE-Submenu=Konvert
Icon=konvert-symbolic

[Desktop Action 00_openSettings]
Name=Konvert Settings
Name[tr]=Konvert Ayarları
Name[de]=Konvert-Einstellungen
Name[fr]=Paramètres Konvert
Name[es]=Ajustes de Konvert
Icon=konvert-symbolic
Exec={quoted_exec} -s

"#,
            mime_types = cat.mime_types,
            actions_keys = actions_keys,
            quoted_exec = quoted_exec,
        );

        for act in &active_actions {
            content.push_str(&format!(
                r#"[Desktop Action {action_key}]
Name={name_en}
Name[tr]={name_tr}
Name[de]={name_de}
Name[fr]={name_fr}
Name[es]={name_es}
Icon={icon}
Exec={exec} {cmd_args}

"#,
                action_key = act.action_key,
                name_en = act.name_en,
                name_tr = act.name_tr,
                name_de = act.name_de,
                name_fr = act.name_fr,
                name_es = act.name_es,
                icon = act.icon,
                exec = quoted_exec,
                cmd_args = act.cmd_args
            ));
        }

        fs::write(&file_path, content)?;

        if let Ok(mut perms) = fs::metadata(&file_path).map(|m| m.permissions()) {
            perms.set_mode(0o755);
            let _ = fs::set_permissions(&file_path, perms);
        }
    }

    Ok(dir)
}

fn dirs_bin() -> PathBuf {
    BaseDirs::new()
        .map(|b| b.home_dir().join(".local").join("bin").join("konvert"))
        .unwrap_or_else(|| PathBuf::from("/usr/local/bin/konvert"))
}

pub fn uninstall_dolphin_service() -> Result<bool> {
    let dir = get_service_menus_dir()?;
    let mut removed = false;

    for cat in CATEGORIES {
        let file_path = dir.join(cat.file_name);
        if file_path.exists() {
            let _ = fs::remove_file(&file_path);
            removed = true;
        }
    }

    let legacy_files = ["konvert.desktop", "konvert_documents.desktop"];
    for legacy in &legacy_files {
        let file_path = dir.join(legacy);
        if file_path.exists() {
            let _ = fs::remove_file(&file_path);
            removed = true;
        }
    }

    Ok(removed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quick_actions_exist_in_registry() {
        for id in QUICK_ACTION_IDS {
            assert!(find_action(id).is_some(), "kayıtta olmayan action: {id}");
        }
    }

    #[test]
    fn test_all_action_ids_are_unique() {
        let mut ids: Vec<&str> = ALL_ACTIONS.iter().map(|action| action.id).collect();
        let total = ids.len();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), total, "yinelenen action id'si var");
    }

    #[test]
    fn test_dolphin_first_action_uses_konvert_symbolic_and_numeric_order() {
        let temp_dir =
            std::env::temp_dir().join(format!("konvert_test_dolphin_menu_{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp_dir);
        let default_ids: Vec<String> = ALL_ACTIONS
            .iter()
            .filter(|a| a.default_enabled)
            .map(|a| a.id.to_string())
            .collect();
        install_dolphin_service_with_options(
            &default_ids,
            Some("/usr/bin/konvert"),
            Some(temp_dir.clone()),
        )
        .expect("dolphin service install");

        for fname in [
            "konvert_photos.desktop",
            "konvert_videos.desktop",
            "konvert_pdf.desktop",
        ] {
            let content = fs::read_to_string(temp_dir.join(fname)).unwrap();
            let header_block = content.split("[Desktop Action ").next().unwrap();
            assert!(
                header_block.contains("Icon=konvert-symbolic"),
                "{fname} submenu entry must use konvert-symbolic"
            );
            assert!(content.contains("X-KDE-ServiceTypes=KonqPopupMenu/Plugin"));
            let first_action_block = content.split("[Desktop Action ").nth(1).unwrap();
            assert!(
                first_action_block.starts_with("00_openSettings]")
                    && first_action_block.contains("Icon=konvert-symbolic"),
                "{fname} first action must be 00_openSettings with konvert-symbolic"
            );
            let second_action_block = content.split("[Desktop Action ").nth(2).unwrap();
            assert!(
                second_action_block.contains("Icon=konvert-compress"),
                "{fname} compress action must use konvert-compress"
            );
        }

        let photos_content = fs::read_to_string(temp_dir.join("konvert_photos.desktop")).unwrap();
        assert!(photos_content.contains("01_compressPhoto"));
        assert!(!photos_content.contains("toSVG"));

        let videos_content = fs::read_to_string(temp_dir.join("konvert_videos.desktop")).unwrap();
        assert!(videos_content.contains("04_stripMetadataVideo"));
        assert!(videos_content.contains("video/x-m4v;"));

        let music_content = fs::read_to_string(temp_dir.join("konvert_music.desktop")).unwrap();
        assert!(music_content.contains("audio/wav;"));
        assert!(music_content.contains("audio/x-opus+ogg;"));

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_quote_desktop_exec_handles_spaces_and_special_chars() {
        assert_eq!(quote_desktop_exec("/usr/bin/konvert"), "/usr/bin/konvert");
        assert_eq!(
            quote_desktop_exec("/opt/My Apps/konvert$1"),
            "\"/opt/My Apps/konvert\\$1\""
        );
        assert_eq!(
            quote_desktop_exec("/opt/100%/konvert"),
            "/opt/100%%/konvert"
        );
    }
}
