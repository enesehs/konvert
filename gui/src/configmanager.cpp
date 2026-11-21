#include "configmanager.h"
#include <QFile>
#include <QFileInfo>
#include <QDir>
#include <QTextStream>
#include <QStandardPaths>
#include <QProcess>
#include <QCoreApplication>
#include <QDebug>
#include <QJsonObject>
#include <QJsonDocument>
#include <QJsonArray>

ConfigManager::ConfigManager() {
    enabledActions = {
        "compress_photo", "to_webp", "to_png", "to_jpg", "to_svg", "to_ico", "to_heic", "to_bmp",
        "to_avif", "to_jxl", "to_jxl_lossless",
        "merge_to_pdf", "strip_metadata", "resize_50", "resize_1080p",
        "web_opt", "discord_8mb_photo", "email_25mb_photo",
        "compress_video", "to_mp4", "to_mkv", "to_webm", "to_gif", "extract_audio",
        "discord_8mb_video", "email_25mb_video",
        "to_mp3", "to_wav", "audio_to_video",
        "compress_doc", "udf_to_pdf", "udf_to_txt", "pdf_to_images",
        "docx_to_pdf", "docx_to_txt"
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
            } else if (line.startsWith("hwaccel")) {
                videoHwaccel = line.section('=', 1).trimmed();
                videoHwaccel.remove('"');
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
    // 1. Rust CLI'yı Tek Gerçeklik Kaynağı (Single Source of Truth) olarak kullan:
    // JSON yükü oluşturulur ve `konvert --save-config-json -` çağrılır.
    // Bu sayede `[profiles.*]` ve tanınmayan tüm TOML tabloları eksiksiz korunur.
    QJsonObject root;

    QJsonObject general;
    general["notification_enabled"] = notificationEnabled;
    if (!outputDirectory.isEmpty()) {
        general["output_directory"] = outputDirectory;
    }
    general["language"] = language;
    root["general"] = general;

    QJsonObject contextMenu;
    QJsonArray actions;
    for (const QString &action : enabledActions) {
        actions.append(action);
    }
    contextMenu["enabled_actions"] = actions;
    root["context_menu"] = contextMenu;

    QJsonObject image;
    image["quality"] = imageQuality;
    image["lossless"] = imageLossless;
    root["image"] = image;

    QJsonObject video;
    video["crf"] = videoCrf;
    video["preset"] = videoPreset;
    video["lossless"] = videoLossless;
    video["hwaccel"] = videoHwaccel;
    root["video"] = video;

    QJsonObject audio;
    audio["bitrate"] = audioBitrate;
    root["audio"] = audio;

    QJsonObject document;
    document["pdf_dpi"] = pdfDpi;
    root["document"] = document;

    QJsonDocument jsonDoc(root);
    QByteArray jsonBytes = jsonDoc.toJson(QJsonDocument::Compact);

    QString program = "konvert";
    const QFileInfo sibling(QDir(QCoreApplication::applicationDirPath()).filePath("konvert"));
    if (sibling.exists() && sibling.isExecutable()) {
        program = sibling.absoluteFilePath();
    }

    QProcess process;
    process.start(program, QStringList() << "--save-config-json" << "-");
    if (process.waitForStarted(2000)) {
        process.write(jsonBytes);
        process.closeWriteChannel();
        if (process.waitForFinished(5000) && process.exitCode() == 0) {
            return true;
        }
    }

    // 2. Acil durum yedeği (Fallback): Eğer konvert CLI çalıştırılamadıysa,
    // mevcut dosyadaki [profiles.*] ve bilinmeyen diğer bölümleri koruyarak yaz.
    QString path = getConfigPath();
    QFileInfo fi(path);
    QDir().mkpath(fi.absolutePath());

    QString preservedExtra;
    QFile existingFile(path);
    if (existingFile.open(QIODevice::ReadOnly | QIODevice::Text)) {
        QTextStream in(&existingFile);
        QString curSec = "";
        QString block = "";
        while (!in.atEnd()) {
            QString line = in.readLine();
            QString trimmed = line.trimmed();
            if (trimmed.startsWith('[') && trimmed.endsWith(']')) {
                curSec = trimmed.mid(1, trimmed.length() - 2);
            }
            if (curSec.startsWith("profiles") ||
                (!curSec.isEmpty() && curSec != "general" && curSec != "context_menu" &&
                 curSec != "image" && curSec != "video" && curSec != "audio" && curSec != "document")) {
                block += line + "\n";
            }
        }
        preservedExtra = block.trimmed();
        existingFile.close();
    }

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
    out << "lossless = " << (videoLossless ? "true" : "false") << "\n";
    out << "hwaccel = \"" << videoHwaccel << "\"\n\n";

    out << "[audio]\n";
    out << "bitrate = \"" << audioBitrate << "\"\n\n";

    out << "[document]\n";
    out << "pdf_dpi = " << pdfDpi << "\n";

    if (!preservedExtra.isEmpty()) {
        out << "\n" << preservedExtra << "\n";
    }

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
