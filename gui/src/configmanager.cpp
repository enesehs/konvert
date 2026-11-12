#include "configmanager.h"
#include <QFile>
#include <QFileInfo>
#include <QDir>
#include <QTextStream>
#include <QStandardPaths>
#include <QProcess>
#include <QCoreApplication>
#include <QDebug>

ConfigManager::ConfigManager() {
    enabledActions = {
        "compress_photo", "to_webp", "to_png", "to_jpg", "to_svg", "to_ico", "to_heic", "to_bmp",
        "to_avif", "to_jxl", "to_jxl_lossless",
        "merge_to_pdf", "strip_metadata", "resize_50", "resize_1080p",
        "web_opt", "discord_8mb_photo", "email_25mb_photo",
        "compress_video", "to_mp4", "to_mkv", "to_webm", "to_gif", "extract_frames", "extract_audio",
        "discord_8mb_video", "email_25mb_video",
        "to_mp3", "to_wav", "audio_to_video",
        "compress_doc", "udf_to_pdf", "udf_to_txt", "pdf_to_images"
    };
    load();
}

QString ConfigManager::getConfigPath() {
    QString configDir = QStandardPaths::writableLocation(QStandardPaths::ConfigLocation);
    return QDir(configDir).filePath("konvert/config.toml");
}

bool ConfigManager::load() {
    QString path = getConfigPath();
    QFile file(path);
    if (!file.open(QIODevice::ReadOnly | QIODevice::Text)) {
        return false;
    }

    QTextStream in(&file);
    QString currentSection = "";
    QStringList parsedActions;
    bool inActionsArray = false;

    while (!in.atEnd()) {
        QString line = in.readLine().trimmed();
        if (line.startsWith('#') || line.isEmpty()) continue;

        if (line.startsWith('[') && line.endsWith(']')) {
            currentSection = line.mid(1, line.length() - 2);
            inActionsArray = false;
            continue;
        }

        if (currentSection == "context_menu") {
            if (line.startsWith("enabled_actions")) {
                parsedActions.clear();
                inActionsArray = true;
                int startBracket = line.indexOf('[');
                int endBracket = line.indexOf(']');
                if (startBracket != -1 && endBracket != -1) {
                    QString itemsStr = line.mid(startBracket + 1, endBracket - startBracket - 1);
                    for (const QString &item : itemsStr.split(',')) {
                        QString cleaned = item.trimmed();
                        cleaned.remove('"').remove('\'');
                        if (!cleaned.isEmpty()) parsedActions.append(cleaned);
                    }
                    inActionsArray = false;
                }
            } else if (inActionsArray) {
                if (line.contains(']')) {
                    inActionsArray = false;
                }
                QString cleaned = line;
                cleaned = cleaned.remove('"').remove('\'').remove(',').remove(']').remove('[').trimmed();
                if (!cleaned.isEmpty()) parsedActions.append(cleaned);
            }
        } else if (currentSection == "general") {
            if (line.startsWith("notification_enabled")) {
                notificationEnabled = line.contains("true");
            } else if (line.startsWith("output_directory")) {
                // Rust çekirdeğinin kullandığı çıktı klasörünü koru (GUI'de henüz seçici yok).
                outputDirectory = line.section('=', 1).trimmed();
                outputDirectory.remove('"');
            } else if (line.startsWith("language")) {
                language = line.section('=', 1).trimmed();
                language.remove('"');
            }
        } else if (currentSection == "image") {
            if (line.startsWith("quality")) {
                imageQuality = line.section('=', 1).trimmed().toInt();
            } else if (line.startsWith("lossless")) {
                imageLossless = line.contains("true");
            }
        } else if (currentSection == "video") {
            if (line.startsWith("crf")) {
                videoCrf = line.section('=', 1).trimmed().toInt();
            } else if (line.startsWith("preset")) {
                videoPreset = line.section('=', 1).trimmed();
                videoPreset.remove('"');
            } else if (line.startsWith("lossless")) {
                videoLossless = line.contains("true");
            }
        } else if (currentSection == "audio") {
            if (line.startsWith("bitrate")) {
                audioBitrate = line.section('=', 1).trimmed();
                audioBitrate.remove('"');
            }
        } else if (currentSection == "document") {
            if (line.startsWith("pdf_dpi")) {
                pdfDpi = line.section('=', 1).trimmed().toInt();
            }
        }
    }

    if (!parsedActions.isEmpty()) {
        enabledActions = parsedActions;
    }

    return true;
}

bool ConfigManager::save() {
    QString path = getConfigPath();
    QFileInfo fi(path);
    QDir().mkpath(fi.absolutePath());

    QFile file(path);
    if (!file.open(QIODevice::WriteOnly | QIODevice::Text)) {
        return false;
    }

    QTextStream out(&file);

    out << "[general]\n";
    out << "notification_enabled = " << (notificationEnabled ? "true" : "false") << "\n";
    if (!outputDirectory.isEmpty()) {
        out << "output_directory = \"" << outputDirectory << "\"\n";
    }
    out << "language = \"" << language << "\"\n\n";

    out << "[context_menu]\n";
    out << "enabled_actions = [\n";
    for (int i = 0; i < enabledActions.size(); ++i) {
        out << "    \"" << enabledActions[i] << "\"" << (i + 1 < enabledActions.size() ? "," : "") << "\n";
    }
    out << "]\n\n";

    out << "[image]\n";
    out << "quality = " << imageQuality << "\n";
    out << "lossless = " << (imageLossless ? "true" : "false") << "\n\n";

    out << "[video]\n";
    out << "crf = " << videoCrf << "\n";
    out << "preset = \"" << videoPreset << "\"\n";
    out << "lossless = " << (videoLossless ? "true" : "false") << "\n\n";

    out << "[audio]\n";
    out << "bitrate = \"" << audioBitrate << "\"\n\n";

    out << "[document]\n";
    out << "pdf_dpi = " << pdfDpi << "\n";

    file.close();

    syncContextMenu();
    return true;
}

void ConfigManager::syncContextMenu() {
    // Geliştirme kurulumunda `konvert` PATH'te olmayabilir: önce kardeş binary'yi dene.
    QString program = "konvert";
    const QFileInfo sibling(QDir(QCoreApplication::applicationDirPath()).filePath("konvert"));
    if (sibling.exists() && sibling.isExecutable()) {
        program = sibling.absoluteFilePath();
    }

    const bool started = QProcess::startDetached(program, QStringList() << "--sync-menu");
    if (!started) {
        qWarning("Konvert: context menu could not be synchronized (%s)", qPrintable(program));
    }
}
