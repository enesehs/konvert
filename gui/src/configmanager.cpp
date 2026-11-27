#include "configmanager.h"
#include <QFile>
#include <QFileInfo>
#include <QDir>
#include <QSaveFile>
#include <QTextStream>
#include <QStandardPaths>
#include <QProcess>
#include <QCoreApplication>
#include <QDebug>
#include <QJsonObject>
#include <QJsonDocument>
#include <QJsonArray>
#include <QJsonValue>
#include <QSharedPointer>

static QString stripTomlComment(const QString &raw) {
    bool inDouble = false;
    bool inSingle = false;
    bool escaped = false;
    for (int i = 0; i < raw.size(); ++i) {
        const QChar ch = raw.at(i);
        if (escaped) {
            escaped = false;
            continue;
        }
        if (inDouble && ch == '\\') {
            escaped = true;
            continue;
        }
        if (!inSingle && ch == '"') {
            inDouble = !inDouble;
        } else if (!inDouble && ch == '\'') {
            inSingle = !inSingle;
        } else if (!inDouble && !inSingle && ch == '#') {
            return raw.left(i).trimmed();
        }
    }
    return raw.trimmed();
}

static QString unquoteTomlString(QString val) {
    val = val.trimmed();
    if ((val.startsWith('"') && val.endsWith('"') && val.size() >= 2) ||
        (val.startsWith('\'') && val.endsWith('\'') && val.size() >= 2)) {
        val = val.mid(1, val.size() - 2);
    }
    val.replace("\\\"", "\"");
    val.replace("\\\\", "\\");
    return val;
}

static QString escapeTomlString(QString val) {
    val.replace('\\', "\\\\");
    val.replace('"', "\\\"");
    return val;
}

ConfigManager::ConfigManager() {
    resetDefaults();
    load();
}

void ConfigManager::resetDefaults() {
    notificationEnabled = true;
    outputDirectory.clear();
    language = "auto";
    enabledActions = {
        "compress_photo", "web_opt", "strip_metadata", "resize_1080p", "merge_to_pdf",
        "to_webp", "to_jpg", "to_png", "to_avif", "to_jxl",
        "compress_video", "discord_8mb_video", "email_25mb_video", "strip_metadata_video", "extract_audio",
        "to_mp4", "to_av1", "to_webm", "to_mkv", "to_gif",
        "to_mp3", "to_wav", "audio_to_video",
        "compress_doc", "pdf_to_images",
        "udf_to_pdf", "udf_to_txt",
        "docx_to_pdf", "docx_to_txt"
    };
    imageQuality = 85;
    imageLossless = false;
    videoCrf = 23;
    videoPreset = "medium";
    videoLossless = false;
    videoHwaccel = "auto";
    audioBitrate = "320k";
    pdfDpi = 300;
}

QString ConfigManager::getConfigPath() {
    QString configDir = QStandardPaths::writableLocation(QStandardPaths::ConfigLocation);
    return QDir(configDir).filePath("konvert/config.toml");
}

QString ConfigManager::findKonvertBinary() {
    const QString appDir = QCoreApplication::applicationDirPath();
    const QStringList candidates = {
        QDir(appDir).filePath("konvert"),
        QDir::homePath() + "/.local/bin/konvert",
        QStringLiteral("/usr/bin/konvert"),
        QDir(appDir).filePath("../../target/release/konvert"),
        QDir(appDir).filePath("../../target/debug/konvert")
    };
    for (const QString &candidate : candidates) {
        const QFileInfo info(candidate);
        if (info.exists() && info.isExecutable()) {
            return info.canonicalFilePath().isEmpty() ? info.absoluteFilePath() : info.canonicalFilePath();
        }
    }
    const QString inPath = QStandardPaths::findExecutable("konvert");
    return inPath.isEmpty() ? QStringLiteral("konvert") : inPath;
}

bool ConfigManager::applyFromJsonBytes(const QByteArray &jsonBytes) {
    QJsonParseError parseErr;
    QJsonDocument doc = QJsonDocument::fromJson(jsonBytes, &parseErr);
    if (parseErr.error != QJsonParseError::NoError || !doc.isObject()) {
        return false;
    }

    const QJsonObject root = doc.object();
    if (root.contains("general") && root.value("general").isObject()) {
        const QJsonObject g = root.value("general").toObject();
        if (g.contains("notification_enabled")) {
            notificationEnabled = g.value("notification_enabled").toBool(notificationEnabled);
        }
        const QJsonValue outVal = g.value("output_directory");
        outputDirectory = (outVal.isNull() || outVal.isUndefined()) ? QString() : outVal.toString();
        const QJsonValue langVal = g.value("language");
        const QString parsedLang = (langVal.isNull() || langVal.isUndefined()) ? QString() : langVal.toString().trimmed();
        language = parsedLang.isEmpty() ? QStringLiteral("auto") : parsedLang;
    }

    if (root.contains("context_menu") && root.value("context_menu").isObject()) {
        const QJsonObject ctx = root.value("context_menu").toObject();
        if (ctx.contains("enabled_actions") && ctx.value("enabled_actions").isArray()) {
            QStringList list;
            for (const QJsonValue &v : ctx.value("enabled_actions").toArray()) {
                const QString item = v.toString().trimmed();
                if (!item.isEmpty()) {
                    list.append(item);
                }
            }
            enabledActions = list;
        }
    }

    if (root.contains("image") && root.value("image").isObject()) {
        const QJsonObject img = root.value("image").toObject();
        if (img.contains("quality")) imageQuality = img.value("quality").toInt(imageQuality);
        if (img.contains("lossless")) imageLossless = img.value("lossless").toBool(imageLossless);
    }

    if (root.contains("video") && root.value("video").isObject()) {
        const QJsonObject vid = root.value("video").toObject();
        if (vid.contains("crf")) videoCrf = vid.value("crf").toInt(videoCrf);
        if (vid.contains("preset")) videoPreset = vid.value("preset").toString(videoPreset);
        if (vid.contains("lossless")) videoLossless = vid.value("lossless").toBool(videoLossless);
        if (vid.contains("hwaccel")) videoHwaccel = vid.value("hwaccel").toString(videoHwaccel);
    }

    if (root.contains("audio") && root.value("audio").isObject()) {
        const QJsonObject aud = root.value("audio").toObject();
        if (aud.contains("bitrate")) audioBitrate = aud.value("bitrate").toString(audioBitrate);
    }

    if (root.contains("document") && root.value("document").isObject()) {
        const QJsonObject d = root.value("document").toObject();
        if (d.contains("pdf_dpi")) pdfDpi = d.value("pdf_dpi").toInt(pdfDpi);
    }

    return true;
}

bool ConfigManager::load() {
    resetDefaults();

    // 1. Öncelikli Tek Gerçeklik Kaynağı (SSoT): Rust CLI --dump-config-json
    QProcess dumpProc;
    dumpProc.start(findKonvertBinary(), {"--dump-config-json"});
    if (dumpProc.waitForFinished(2500)
        && dumpProc.exitStatus() == QProcess::NormalExit
        && dumpProc.exitCode() == 0
        && applyFromJsonBytes(dumpProc.readAllStandardOutput()))
    {
        return true;
    }

    // 2. Acil durum yedeği: sertleştirilmiş TOML okuyucu
    QString path = getConfigPath();
    QFile file(path);
    if (!file.open(QIODevice::ReadOnly | QIODevice::Text)) {
        return false;
    }

    QTextStream in(&file);
    QString currentSection;
    QStringList parsedActions;
    bool seenEnabledActions = false;
    bool inActionsArray = false;

    while (!in.atEnd()) {
        QString line = stripTomlComment(in.readLine());
        if (line.isEmpty()) continue;

        if (line.startsWith('[') && line.endsWith(']')) {
            currentSection = line.mid(1, line.length() - 2).trimmed();
            inActionsArray = false;
            continue;
        }

        if (currentSection == "context_menu") {
            if (line.section('=', 0, 0).trimmed() == "enabled_actions") {
                seenEnabledActions = true;
                parsedActions.clear();
                inActionsArray = true;
                int startBracket = line.indexOf('[');
                int endBracket = line.indexOf(']');
                if (startBracket != -1 && endBracket != -1) {
                    QString itemsStr = line.mid(startBracket + 1, endBracket - startBracket - 1);
                    for (const QString &item : itemsStr.split(',')) {
                        QString cleaned = unquoteTomlString(item);
                        if (!cleaned.isEmpty()) parsedActions.append(cleaned);
                    }
                    inActionsArray = false;
                }
            } else if (inActionsArray) {
                if (line.contains(']')) {
                    inActionsArray = false;
                }
                QString cleaned = line;
                cleaned.remove('[').remove(']').remove(',');
                cleaned = unquoteTomlString(cleaned);
                if (!cleaned.isEmpty()) parsedActions.append(cleaned);
            }
            continue;
        }

        const int eqIdx = line.indexOf('=');
        if (eqIdx <= 0) continue;
        const QString key = line.left(eqIdx).trimmed();
        const QString val = line.mid(eqIdx + 1).trimmed();

        if (currentSection == "general") {
            if (key == "notification_enabled") {
                notificationEnabled = (val == "true");
            } else if (key == "output_directory") {
                outputDirectory = unquoteTomlString(val);
            } else if (key == "language") {
                QString parsed = unquoteTomlString(val);
                language = parsed.isEmpty() ? QStringLiteral("auto") : parsed;
            }
        } else if (currentSection == "image") {
            if (key == "quality") {
                bool ok = false;
                int q = val.toInt(&ok);
                if (ok && q >= 1 && q <= 100) imageQuality = q;
            } else if (key == "lossless") {
                imageLossless = (val == "true");
            }
        } else if (currentSection == "video") {
            if (key == "crf") {
                bool ok = false;
                int c = val.toInt(&ok);
                if (ok && c >= 0 && c <= 51) videoCrf = c;
            } else if (key == "preset") {
                videoPreset = unquoteTomlString(val);
            } else if (key == "lossless") {
                videoLossless = (val == "true");
            } else if (key == "hwaccel") {
                videoHwaccel = unquoteTomlString(val);
            }
        } else if (currentSection == "audio") {
            if (key == "bitrate") {
                audioBitrate = unquoteTomlString(val);
            }
        } else if (currentSection == "document") {
            if (key == "pdf_dpi") {
                bool ok = false;
                int d = val.toInt(&ok);
                if (ok && d >= 1 && d <= 2400) pdfDpi = d;
            }
        }
    }

    if (seenEnabledActions) {
        enabledActions = parsedActions;
    }

    return true;
}

QByteArray ConfigManager::buildConfigJson() const {
    QJsonObject root;

    QJsonObject general;
    general["notification_enabled"] = notificationEnabled;
    if (outputDirectory.trimmed().isEmpty()) {
        general["output_directory"] = QJsonValue::Null;
    } else {
        general["output_directory"] = outputDirectory.trimmed();
    }
    if (language.trimmed().isEmpty() || language == "auto") {
        general["language"] = QJsonValue::Null;
    } else {
        general["language"] = language;
    }
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

    return QJsonDocument(root).toJson(QJsonDocument::Compact);
}

bool ConfigManager::save() {
    const QByteArray jsonBytes = buildConfigJson();
    QProcess process;
    process.start(findKonvertBinary(), QStringList() << "--save-config-json" << "-");
    if (process.waitForStarted(2000)) {
        process.write(jsonBytes);
        process.closeWriteChannel();
        if (process.waitForFinished(5000) && process.exitStatus() == QProcess::NormalExit && process.exitCode() == 0) {
            return true;
        }
    }
    return saveFallback();
}

void ConfigManager::saveAsync(QObject *context, std::function<void(bool)> callback) {
    const QByteArray jsonBytes = buildConfigJson();
    auto *process = new QProcess(context);
    auto done = QSharedPointer<bool>::create(false);

    QObject::connect(process, QOverload<int, QProcess::ExitStatus>::of(&QProcess::finished),
        context, [this, process, done, cb = callback](int exitCode, QProcess::ExitStatus exitStatus) {
            if (*done) return;
            *done = true;
            const bool cliOk = (exitStatus == QProcess::NormalExit && exitCode == 0);
            const bool result = cliOk ? true : saveFallback();
            process->deleteLater();
            if (cb) cb(result);
        });

    QObject::connect(process, &QProcess::errorOccurred,
        context, [this, process, done, cb = callback](QProcess::ProcessError) {
            if (*done) return;
            *done = true;
            const bool result = saveFallback();
            process->deleteLater();
            if (cb) cb(result);
        });

    process->start(findKonvertBinary(), QStringList() << "--save-config-json" << "-");
    if (process->waitForStarted(500)) {
        process->write(jsonBytes);
        process->closeWriteChannel();
    }
}

bool ConfigManager::saveFallback() {
    QString path = getConfigPath();
    QFileInfo fi(path);
    QDir().mkpath(fi.absolutePath());

    QString preservedExtra;
    QFile existingFile(path);
    if (existingFile.open(QIODevice::ReadOnly | QIODevice::Text)) {
        QTextStream in(&existingFile);
        QString curSec;
        QString block;
        while (!in.atEnd()) {
            QString line = in.readLine();
            QString trimmed = line.trimmed();
            if (trimmed.startsWith('[') && trimmed.endsWith(']')) {
                curSec = trimmed.mid(1, trimmed.length() - 2).trimmed();
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

    QSaveFile file(path);
    if (!file.open(QIODevice::WriteOnly | QIODevice::Text)) {
        return false;
    }

    QTextStream out(&file);

    out << "[general]\n";
    out << "notification_enabled = " << (notificationEnabled ? "true" : "false") << "\n";
    if (!outputDirectory.trimmed().isEmpty()) {
        out << "output_directory = \"" << escapeTomlString(outputDirectory.trimmed()) << "\"\n";
    }
    if (!language.trimmed().isEmpty() && language != "auto") {
        out << "language = \"" << escapeTomlString(language) << "\"\n";
    }
    out << "\n";

    out << "[context_menu]\n";
    out << "enabled_actions = [\n";
    for (int i = 0; i < enabledActions.size(); ++i) {
        out << "    \"" << escapeTomlString(enabledActions[i]) << "\""
            << (i + 1 < enabledActions.size() ? "," : "") << "\n";
    }
    out << "]\n\n";

    out << "[image]\n";
    out << "quality = " << imageQuality << "\n";
    out << "lossless = " << (imageLossless ? "true" : "false") << "\n\n";

    out << "[video]\n";
    out << "crf = " << videoCrf << "\n";
    out << "preset = \"" << escapeTomlString(videoPreset) << "\"\n";
    out << "lossless = " << (videoLossless ? "true" : "false") << "\n";
    out << "hwaccel = \"" << escapeTomlString(videoHwaccel) << "\"\n\n";

    out << "[audio]\n";
    out << "bitrate = \"" << escapeTomlString(audioBitrate) << "\"\n\n";

    out << "[document]\n";
    out << "pdf_dpi = " << pdfDpi << "\n";

    if (!preservedExtra.isEmpty()) {
        out << "\n" << preservedExtra << "\n";
    }

    out.flush();
    if (!file.commit()) {
        return false;
    }

    syncContextMenu();
    return true;
}

void ConfigManager::syncContextMenu() {
    const QString program = findKonvertBinary();
    const bool started = QProcess::startDetached(program, QStringList() << "--sync-menu");
    if (!started) {
        qWarning("Konvert: context menu could not be synchronized (%s)", qPrintable(program));
    }
}
