#pragma once

#include <QByteArray>
#include <QString>
#include <QStringList>
#include <functional>

class QObject;

class ConfigManager {
public:
    ConfigManager();

    void resetDefaults();
    bool load();
    bool save();
    void saveAsync(QObject *context, std::function<void(bool)> callback);
    void syncContextMenu();

    // General
    bool notificationEnabled = true;
    QString outputDirectory; // boş = giriş dosyasının klasörü
    QString language = "auto";

    // Context Menu Actions
    QStringList enabledActions;

    // Image
    int imageQuality = 85;
    bool imageLossless = false;

    // Video
    int videoCrf = 23;
    QString videoPreset = "medium";
    bool videoLossless = false;
    QString videoHwaccel = "auto";

    // Audio
    QString audioBitrate = "320k";

    // Document
    int pdfDpi = 300;

    static QString getConfigPath();
    static QString findKonvertBinary();

private:
    QByteArray buildConfigJson() const;
    bool applyFromJsonBytes(const QByteArray &jsonBytes);
    bool saveFallback();
};
