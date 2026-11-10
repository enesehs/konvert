#pragma once

#include <QString>
#include <QStringList>

class ConfigManager {
public:
    ConfigManager();

    bool load();
    bool save();
    void syncContextMenu();

    // General
    bool notificationEnabled = true;
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

    // Audio
    QString audioBitrate = "320k";

    // Document
    int pdfDpi = 300;

    static QString getConfigPath();
};
