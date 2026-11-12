use anyhow::{Context, Result};
use directories::BaseDirs;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

pub struct ActionDef {
    pub id: &'static str,
    pub action_key: &'static str,
    pub category: &'static str, // "photos", "videos", "music", "documents"
    pub name_en: &'static str,
    pub name_tr: &'static str,
    pub name_de: &'static str,
    pub name_fr: &'static str,
    pub name_es: &'static str,
    pub icon: &'static str,
    pub cmd_args: &'static str,
}

pub const ALL_ACTIONS: &[ActionDef] = &[
    // PHOTOS
    ActionDef {
        id: "compress_photo",
        action_key: "compressPhoto",
        category: "photos",
        name_en: "Compress Image (Max Quality)",
        name_tr: "Görseli Sıkıştır",
        name_de: "Bild komprimieren",
        name_fr: "Compresser l'image",
        name_es: "Comprimir imagen",
        icon: "konvert-compress",
        cmd_args: "-t compress %F",
    },
    ActionDef {
        id: "to_webp",
        action_key: "toWebP",
        category: "photos",
        name_en: "Convert to WebP",
        name_tr: "WebP'ye Dönüştür",
        name_de: "In WebP umwandeln",
        name_fr: "Convertir en WebP",
        name_es: "Convertir a WebP",
        icon: "konvert-image",
        cmd_args: "-t webp %F",
    },
    ActionDef {
        id: "to_png",
        action_key: "toPNG",
        category: "photos",
        name_en: "Convert to PNG",
        name_tr: "PNG'ye Dönüştür",
        name_de: "In PNG umwandeln",
        name_fr: "Convertir en PNG",
        name_es: "Convertir a PNG",
        icon: "konvert-image",
        cmd_args: "-t png %F",
    },
    ActionDef {
        id: "to_jpg",
        action_key: "toJPG",
        category: "photos",
        name_en: "Convert to JPG",
        name_tr: "JPG'ye Dönüştür",
        name_de: "In JPG umwandeln",
        name_fr: "Convertir en JPG",
        name_es: "Convertir a JPG",
        icon: "konvert-image",
        cmd_args: "-t jpg %F",
    },
    ActionDef {
        id: "to_svg",
        action_key: "toSVG",
        category: "photos",
        name_en: "Convert to SVG",
        name_tr: "SVG'ye Dönüştür",
        name_de: "In SVG umwandeln",
        name_fr: "Convertir en SVG",
        name_es: "Convertir a SVG",
        icon: "konvert-image",
        cmd_args: "-t svg %F",
    },
    ActionDef {
        id: "to_ico",
        action_key: "toICO",
        category: "photos",
        name_en: "Convert to ICO",
        name_tr: "ICO'ya Dönüştür",
        name_de: "In ICO umwandeln",
        name_fr: "Convertir en ICO",
        name_es: "Convertir a ICO",
        icon: "konvert-image",
        cmd_args: "-t ico %F",
    },
    ActionDef {
        id: "to_heic",
        action_key: "toHEIC",
        category: "photos",
        name_en: "Convert to HEIC",
        name_tr: "HEIC'e Dönüştür",
        name_de: "In HEIC umwandeln",
        name_fr: "Convertir en HEIC",
        name_es: "Convertir a HEIC",
        icon: "konvert-image",
        cmd_args: "-t heic %F",
    },
    ActionDef {
        id: "to_bmp",
        action_key: "toBMP",
        category: "photos",
        name_en: "Convert to BMP",
        name_tr: "BMP'ye Dönüştür",
        name_de: "In BMP umwandeln",
        name_fr: "Convertir en BMP",
        name_es: "Convertir a BMP",
        icon: "konvert-image",
        cmd_args: "-t bmp %F",
    },
    ActionDef {
        id: "to_avif",
        action_key: "toAVIF",
        category: "photos",
        name_en: "Convert to AVIF",
        name_tr: "AVIF'e Dönüştür",
        name_de: "In AVIF umwandeln",
        name_fr: "Convertir en AVIF",
        name_es: "Convertir a AVIF",
        icon: "konvert-image",
        cmd_args: "-t avif %F",
    },
    ActionDef {
        id: "to_jxl",
        action_key: "toJXL",
        category: "photos",
        name_en: "Convert to JXL",
        name_tr: "JXL'e Dönüştür",
        name_de: "In JXL umwandeln",
        name_fr: "Convertir en JXL",
        name_es: "Convertir a JXL",
        icon: "konvert-image",
        cmd_args: "-t jxl %F",
    },
    ActionDef {
        id: "to_jxl_lossless",
        action_key: "toJXLLossless",
        category: "photos",
        name_en: "Convert to JXL (Lossless)",
        name_tr: "Kayıpsız JXL'e Dönüştür",
        name_de: "In JXL umwandeln (Verlustfrei)",
        name_fr: "Convertir en JXL (Sans perte)",
        name_es: "Convertir a JXL (Sin pérdidas)",
        icon: "konvert-image",
        cmd_args: "-t jxl --lossless %F",
    },
    ActionDef {
        id: "merge_to_pdf",
        action_key: "mergeToPDF",
        category: "photos",
        name_en: "Merge Images to Single PDF",
        name_tr: "Tek PDF Olarak Birleştir",
        name_de: "Bilder in ein PDF zusammenführen",
        name_fr: "Fusionner les images en un seul PDF",
        name_es: "Combinar imágenes en un solo PDF",
        icon: "konvert-layers",
        cmd_args: "--merge-pdf %F",
    },
    ActionDef {
        id: "strip_metadata",
        action_key: "stripMetadata",
        category: "photos",
        name_en: "Strip Metadata & EXIF",
        name_tr: "EXIF ve Metadata Temizle",
        name_de: "Metadaten & EXIF entfernen",
        name_fr: "Supprimer les métadonnées et EXIF",
        name_es: "Eliminar metadatos y EXIF",
        icon: "konvert-shield",
        cmd_args: "-t strip-metadata %F",
    },
    ActionDef {
        id: "resize_50",
        action_key: "resize50",
        category: "photos",
        name_en: "Resize Image by 50%",
        name_tr: "Görseli %50 Küçült",
        name_de: "Bild um 50 % verkleinern",
        name_fr: "Redimensionner l'image de 50 %",
        name_es: "Redimensionar imagen al 50%",
        icon: "konvert-scaling",
        cmd_args: "-t resize_50 %F",
    },
    ActionDef {
        id: "resize_1080p",
        action_key: "resize1080p",
        category: "photos",
        name_en: "Resize to Full HD",
        name_tr: "Full HD'ye Ölçekle",
        name_de: "Auf Full HD skalieren",
        name_fr: "Redimensionner en Full HD",
        name_es: "Escalar a Full HD",
        icon: "konvert-monitor",
        cmd_args: "-t resize_1080p %F",
    },
    ActionDef {
        id: "web_opt",
        action_key: "webOpt",
        category: "photos",
        name_en: "Optimize for Web (WebP + Strip EXIF)",
        name_tr: "Web İçin Optimize Et (WebP + EXIF Temiz)",
        name_de: "Für Web optimieren (WebP + EXIF bereinigt)",
        name_fr: "Optimiser pour le Web (WebP + sans EXIF)",
        name_es: "Optimizar para la Web (WebP + sin EXIF)",
        icon: "konvert-globe",
        cmd_args: "-t web_opt %F",
    },
    ActionDef {
        id: "discord_8mb_photo",
        action_key: "discord8MBPhoto",
        category: "photos",
        name_en: "Compress for Discord (8 MB)",
        name_tr: "Discord İçin Sıkıştır (8 MB)",
        name_de: "Für Discord komprimieren (8 MB)",
        name_fr: "Compresser pour Discord (8 Mo)",
        name_es: "Comprimir para Discord (8 MB)",
        icon: "konvert-mail",
        cmd_args: "-t discord_8mb %F",
    },
    ActionDef {
        id: "email_25mb_photo",
        action_key: "email25MBPhoto",
        category: "photos",
        name_en: "Compress for Email (25 MB)",
        name_tr: "E-Posta İçin Sıkıştır (25 MB)",
        name_de: "Für E-Mail komprimieren (25 MB)",
        name_fr: "Compresser pour e-mail (25 Mo)",
        name_es: "Comprimir para correo (25 MB)",
        icon: "konvert-mail",
        cmd_args: "-t email_25mb %F",
    },
    // VIDEOS
    ActionDef {
        id: "compress_video",
        action_key: "compressVideo",
        category: "videos",
        name_en: "Compress Video (Max Quality)",
        name_tr: "Videoyu Sıkıştır",
        name_de: "Video komprimieren",
        name_fr: "Compresser la vidéo",
        name_es: "Comprimir video",
        icon: "konvert-compress",
        cmd_args: "-t compress %F",
    },
    ActionDef {
        id: "to_mp4",
        action_key: "toMP4",
        category: "videos",
        name_en: "Convert to MP4",
        name_tr: "MP4'e Dönüştür",
        name_de: "In MP4 umwandeln",
        name_fr: "Convertir en MP4",
        name_es: "Convertir a MP4",
        icon: "konvert-video",
        cmd_args: "-t mp4 %F",
    },
    ActionDef {
        id: "to_mkv",
        action_key: "toMKV",
        category: "videos",
        name_en: "Convert to MKV",
        name_tr: "MKV'ye Dönüştür",
        name_de: "In MKV umwandeln",
        name_fr: "Convertir en MKV",
        name_es: "Convertir a MKV",
        icon: "konvert-video",
        cmd_args: "-t mkv %F",
    },
    ActionDef {
        id: "to_webm",
        action_key: "toWebM",
        category: "videos",
        name_en: "Convert to WebM",
        name_tr: "WebM'ye Dönüştür",
        name_de: "In WebM umwandeln",
        name_fr: "Convertir en WebM",
        name_es: "Convertir a WebM",
        icon: "konvert-video",
        cmd_args: "-t webm %F",
    },
    ActionDef {
        id: "to_gif",
        action_key: "toGIF",
        category: "videos",
        name_en: "Convert to GIF",
        name_tr: "GIF'e Dönüştür",
        name_de: "In GIF umwandeln",
        name_fr: "Convertir en GIF",
        name_es: "Convertir a GIF",
        icon: "konvert-video",
        cmd_args: "-t gif %F",
    },
    ActionDef {
        id: "extract_frames",
        action_key: "extractFrames",
        category: "videos",
        name_en: "Export Frames to Folder",
        name_tr: "Kareleri Klasöre Aktar",
        name_de: "Frames in Ordner exportieren",
        name_fr: "Exporter les images vers un dossier",
        name_es: "Exportar fotogramas a carpeta",
        icon: "konvert-folder",
        cmd_args: "--extract-frames %F",
    },
    ActionDef {
        id: "extract_audio",
        action_key: "extractAudio",
        category: "videos",
        name_en: "Extract Audio",
        name_tr: "Videodan Sesi Ayıkla",
        name_de: "Audio extrahieren",
        name_fr: "Extraire l'audio",
        name_es: "Extraer audio",
        icon: "konvert-audio",
        cmd_args: "-t extract-audio %F",
    },
    ActionDef {
        id: "discord_8mb_video",
        action_key: "discord8MBVideo",
        category: "videos",
        name_en: "Compress for Discord (8 MB)",
        name_tr: "Discord İçin Sıkıştır (8 MB)",
        name_de: "Für Discord komprimieren (8 MB)",
        name_fr: "Compresser pour Discord (8 Mo)",
        name_es: "Comprimir para Discord (8 MB)",
        icon: "konvert-mail",
        cmd_args: "-t discord_8mb %F",
    },
    ActionDef {
        id: "email_25mb_video",
        action_key: "email25MBVideo",
        category: "videos",
        name_en: "Compress for Email (25 MB)",
        name_tr: "E-Posta İçin Sıkıştır (25 MB)",
        name_de: "Für E-Mail komprimieren (25 MB)",
        name_fr: "Compresser pour e-mail (25 Mo)",
        name_es: "Comprimir para correo (25 MB)",
        icon: "konvert-mail",
        cmd_args: "-t email_25mb %F",
    },
    // MUSIC / AUDIO
    ActionDef {
        id: "to_mp3",
        action_key: "toMP3",
        category: "music",
        name_en: "Convert to MP3",
        name_tr: "MP3'e Dönüştür",
        name_de: "In MP3 umwandeln",
        name_fr: "Convertir en MP3",
        name_es: "Convertir a MP3",
        icon: "konvert-audio",
        cmd_args: "-t mp3 %F",
    },
    ActionDef {
        id: "to_wav",
        action_key: "toWAV",
        category: "music",
        name_en: "Convert to WAV",
        name_tr: "WAV'a Dönüştür",
        name_de: "In WAV umwandeln",
        name_fr: "Convertir en WAV",
        name_es: "Convertir a WAV",
        icon: "konvert-audio",
        cmd_args: "-t wav %F",
    },
    ActionDef {
        id: "audio_to_video",
        action_key: "audioToVideo",
        category: "music",
        name_en: "Convert to Video",
        name_tr: "Videoya Dönüştür",
        name_de: "In Video umwandeln",
        name_fr: "Convertir en vidéo",
        name_es: "Convertir a video",
        icon: "konvert-video",
        cmd_args: "-t audio-to-video %F",
    },
    // DOCUMENTS
    ActionDef {
        id: "compress_doc",
        action_key: "compressDoc",
        category: "documents",
        name_en: "Compress PDF",
        name_tr: "PDF Sıkıştır",
        name_de: "PDF komprimieren",
        name_fr: "Compresser le PDF",
        name_es: "Comprimir PDF",
        icon: "konvert-compress",
        cmd_args: "-t compress %F",
    },
    ActionDef {
        id: "udf_to_pdf",
        action_key: "udfToPDF",
        category: "documents",
        name_en: "Convert UDF to PDF",
        name_tr: "UDF'i PDF'e Dönüştür",
        name_de: "UDF in PDF umwandeln",
        name_fr: "Convertir UDF en PDF",
        name_es: "Convertir UDF a PDF",
        icon: "konvert-document",
        cmd_args: "-t pdf %F",
    },
    ActionDef {
        id: "udf_to_txt",
        action_key: "udfToTXT",
        category: "documents",
        name_en: "Convert UDF to Text",
        name_tr: "UDF'i Metne Dönüştür",
        name_de: "UDF in Text umwandeln",
        name_fr: "Convertir UDF en Texte",
        name_es: "Convertir UDF a Texto",
        icon: "konvert-document",
        cmd_args: "-t txt %F",
    },
    ActionDef {
        id: "pdf_to_images",
        action_key: "pdfToImages",
        category: "documents",
        name_en: "Extract PDF Pages to PNG",
        name_tr: "PDF Sayfalarını PNG'ye Çıkar",
        name_de: "PDF-Seiten als PNG extrahieren",
        name_fr: "Extraire les pages PDF en PNG",
        name_es: "Extraer páginas PDF a PNG",
        icon: "konvert-image",
        cmd_args: "-t png %F",
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

struct CategoryInfo {
    key: &'static str,
    file_name: &'static str,
    mime_types: &'static str,
    default_icon: &'static str,
}

const CATEGORIES: &[CategoryInfo] = &[
    CategoryInfo {
        key: "photos",
        file_name: "konvert_photos.desktop",
        mime_types: "image/png;image/jpeg;image/webp;image/bmp;image/svg+xml;image/x-icon;image/tiff;image/avif;image/jxl;image/qoi;image/heic;image/heif;",
        default_icon: "konvert-image",
    },
    CategoryInfo {
        key: "videos",
        file_name: "konvert_videos.desktop",
        mime_types: "video/mp4;video/webm;video/x-matroska;video/avi;video/quicktime;video/x-msvideo;video/x-flv;video/x-matroska;",
        default_icon: "konvert-video",
    },
    CategoryInfo {
        key: "music",
        file_name: "konvert_music.desktop",
        mime_types: "audio/mpeg;audio/x-wav;audio/flac;audio/ogg;audio/opus;audio/mp4;audio/aac;audio/x-m4a;",
        default_icon: "konvert-audio",
    },
    CategoryInfo {
        key: "documents",
        file_name: "konvert_documents.desktop",
        mime_types: "application/pdf;application/x-udf;application/vnd.openxmlformats-officedocument.wordprocessingml.document;application/msword;text/plain;text/html;",
        default_icon: "konvert-document",
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

    let legacy_file = dir.join("konvert.desktop");
    if legacy_file.exists() {
        let _ = fs::remove_file(&legacy_file);
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

        let actions_keys = active_actions
            .iter()
            .map(|a| a.action_key)
            .collect::<Vec<_>>()
            .join(";");

        let mut content = format!(
            r#"[Desktop Entry]
Type=Service
ServiceTypes=KonqPopupMenu/Plugin
MimeType={mime_types}
Actions={actions_keys};
X-KDE-Submenu=Konvert
Icon=konvert

"#,
            mime_types = cat.mime_types,
            actions_keys = actions_keys,
        );

        for act in active_actions {
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
                exec = exec,
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

    let legacy_file = dir.join("konvert.desktop");
    if legacy_file.exists() {
        let _ = fs::remove_file(&legacy_file);
        removed = true;
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
}
