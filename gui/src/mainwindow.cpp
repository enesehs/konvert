#include "mainwindow.h"
#include "translator.h"
#include <QApplication>
#include <QVBoxLayout>
#include <QHBoxLayout>
#include <QGridLayout>
#include <QGroupBox>
#include <QLabel>
#include <QPushButton>
#include <QIcon>
#include <QFrame>
#include <QScrollArea>
#include <QJsonDocument>
#include <QJsonArray>
#include <QJsonObject>
#include <QStandardPaths>
#include <QDir>
#include <QPixmap>
#include <QTextBrowser>
#include <QFontDatabase>
#include <QDesktopServices>
#include <QUrl>
#include <QProcess>
#include <QFileIconProvider>
#include <QFileInfo>
#include <QSysInfo>
#include <QFile>
#include <QMessageBox>
#include <QCloseEvent>
#include <QImageReader>
#include <QFileDialog>
#include <QShortcut>
#include <QPointer>
#include <QSharedPointer>
#include <QSignalBlocker>
#include <QThreadPool>
#include <QMetaObject>
#include <cmath>

#ifndef KONVERT_APP_VERSION
#define KONVERT_APP_VERSION "v0.1-beta"
#endif

static QString getCacheBaseDir() {
    return QDir(QStandardPaths::writableLocation(QStandardPaths::GenericCacheLocation)).filePath("konvert");
}

static bool pidIsActiveKonvert(qint64 pid) {
    if (pid <= 0) return false;
    QFile commFile(QString("/proc/%1/comm").arg(pid));
    if (commFile.open(QIODevice::ReadOnly | QIODevice::Text)) {
        QString comm = QTextStream(&commFile).readLine().trimmed();
        if (comm == "konvert") {
            return true;
        }
    }
    QFileInfo exeInfo(QString("/proc/%1/exe").arg(pid));
    if (exeInfo.exists()) {
        QString target = exeInfo.symLinkTarget();
        return QFileInfo(target).fileName() == "konvert";
    }
    return false;
}

static qint64 parsePidFile(const QString &pidPath) {
    QFile file(pidPath);
    if (!file.exists() || !file.open(QIODevice::ReadOnly | QIODevice::Text)) {
        return 0;
    }
    bool ok = false;
    qint64 pid = QTextStream(&file).readLine().trimmed().toLongLong(&ok);
    return (ok && pidIsActiveKonvert(pid)) ? pid : 0;
}

static qint64 getActiveKonvertPid() {
    const QString base = getCacheBaseDir();
    QDir pidsDir(QDir(base).filePath("pids"));
    if (pidsDir.exists()) {
        const QStringList entries = pidsDir.entryList({"*.pid"}, QDir::Files);
        for (const QString &name : entries) {
            qint64 pid = parsePidFile(pidsDir.filePath(name));
            if (pid > 0) return pid;
        }
    }
    return parsePidFile(QDir(base).filePath("running.pid"));
}

static bool isPidPaused(qint64 pid) {
    if (pid <= 0) return false;
    const QString base = getCacheBaseDir();
    return QFileInfo::exists(QDir(QDir(base).filePath("pids")).filePath(QString("%1.paused").arg(pid)));
}

MainWindow::MainWindow(QWidget *parent)
    : QMainWindow(parent)
{
    setupUi();

    // Slider her tik'te config yazıp menü senkronu başlatmasın: 400 ms debounce.
    m_saveTimer = new QTimer(this);
    m_saveTimer->setSingleShot(true);
    m_saveTimer->setInterval(400);
    connect(m_saveTimer, &QTimer::timeout, this, &MainWindow::saveSettingsNow);

    m_reloadGuardTimer = new QTimer(this);
    m_reloadGuardTimer->setSingleShot(true);
    m_reloadGuardTimer->setInterval(300);
    connect(m_reloadGuardTimer, &QTimer::timeout, this, [this]() {
        m_isSavingConfig = false;
    });

    m_processMonitorTimer = new QTimer(this);
    m_processMonitorTimer->setInterval(2000);
    connect(m_processMonitorTimer, &QTimer::timeout, this, [this]() {
        updateProcessMonitor(false);
    });
    m_processMonitorTimer->start();

    setupConfigWatcher();

    // Shortcuts & Accessibility (P3-14)
    auto *f1Shortcut = new QShortcut(QKeySequence::HelpContents, this);
    connect(f1Shortcut, &QShortcut::activated, this, [this]() {
        if (m_rootTabWidget) m_rootTabWidget->setCurrentIndex(3);
    });

    auto *quitShortcut = new QShortcut(QKeySequence::Quit, this);
    connect(quitShortcut, &QShortcut::activated, this, &QWidget::close);

    auto *closeShortcut = new QShortcut(QKeySequence::Close, this);
    connect(closeShortcut, &QShortcut::activated, this, &QWidget::close);

    auto *refreshShortcut = new QShortcut(QKeySequence::Refresh, this);
    connect(refreshShortcut, &QShortcut::activated, this, [this]() {
        loadHistoryData();
        updateProcessMonitor(true);
    });

    setWindowTitle("Konvert");
    setWindowIcon(QIcon(":/logo.svg"));
    resize(700, 740);
}

QString MainWindow::currentLanguage() const {
    return AppTranslator::getActiveLanguage(m_config.language);
}

static QString trKey(const QString &key, const QString &lang) {
    return AppTranslator::tr(key, lang);
}

static void selectComboLanguage(QComboBox *combo, const QString &langCode) {
    if (!combo) return;
    int idx = combo->findData(langCode);
    if (idx < 0) {
        if (langCode.isEmpty() || langCode == "auto") {
            idx = 0;
        } else {
            combo->addItem(langCode, langCode);
            idx = combo->count() - 1;
        }
    }
    combo->setCurrentIndex(idx);
}

QWidget* MainWindow::createCard(const QString &title, QWidget *contentWidget) {
    auto *frame = new QFrame(this);
    frame->setObjectName("konvertCard");
    frame->setFrameShape(QFrame::StyledPanel);
    frame->setStyleSheet(
        "QFrame#konvertCard { "
        "  border: 1px solid palette(midlight); "
        "  border-radius: 8px; "
        "  background-color: palette(alternate-base); "
        "  margin-top: 4px; "
        "  margin-bottom: 8px; "
        "}"
    );

    auto *layout = new QVBoxLayout(frame);
    layout->setContentsMargins(16, 14, 16, 14);

    if (!title.isEmpty()) {
        auto *lblTitle = new QLabel(QString("<b>%1</b>").arg(title.toHtmlEscaped()), frame);
        lblTitle->setStyleSheet("font-size: 1.0em; border: none; background: transparent;");
        layout->addWidget(lblTitle);

        auto *sep = new QFrame(frame);
        sep->setFrameShape(QFrame::HLine);
        sep->setStyleSheet("background-color: palette(midlight); max-height: 1px; margin-bottom: 8px; border: none;");
        layout->addWidget(sep);
    }

    layout->addWidget(contentWidget);

    return frame;
}

QCheckBox* MainWindow::createActionCheckBox(
    const QString &id,
    const QString &label,
    const QString &tooltip,
    const QString &iconName,
    QWidget *parent
) {
    auto *cb = new QCheckBox(label, parent);
    if (!iconName.isEmpty()) {
        QIcon ic = QIcon::fromTheme(iconName);
        if (ic.isNull()) {
            QString fallback = iconName;
            fallback.remove("-symbolic");
            if (fallback.startsWith("konvert-")) {
                if (fallback == "konvert-image") fallback = "image-x-generic";
                else if (fallback == "konvert-video") fallback = "video-x-generic";
                else if (fallback == "konvert-audio") fallback = "audio-x-generic";
                else if (fallback == "konvert-document") fallback = "x-office-document";
                else if (fallback == "konvert-compress") fallback = "transform-scale";
                else if (fallback == "konvert-shield") fallback = "security-high";
                else if (fallback == "konvert-monitor") fallback = "video-display";
                else if (fallback == "konvert-globe") fallback = "network-workgroup";
                else if (fallback == "konvert-mail") fallback = "mail-attachment";
                else if (fallback == "konvert-folder") fallback = "folder-pictures";
                else if (fallback == "konvert-layers") fallback = "x-office-document";
                else if (fallback == "konvert-scaling") fallback = "transform-scale";
            }
            ic = QIcon::fromTheme(fallback);
        }
        cb->setIcon(ic);
    }
    if (!tooltip.isEmpty()) {
        cb->setToolTip(tooltip);
    }
    cb->setProperty("actionId", id);
    cb->setChecked(m_config.enabledActions.contains(id));
    cb->setStyleSheet("QCheckBox { font-size: 1.0em; padding: 4px 0; }");

    connect(cb, &QCheckBox::toggled, this, &MainWindow::onActionCheckChanged);
    m_actionCheckBoxes.append(cb);
    return cb;
}

void MainWindow::setupUi() {
    m_actionCheckBoxes.clear();
    m_videoLosslessWarnLabel = nullptr;
    m_depsStatusEdit = nullptr;
    m_lastMonitoredPid = -2;
    if (auto *oldCentral = takeCentralWidget()) {
        oldCentral->hide();
        oldCentral->deleteLater();
    }

    auto *central = new QWidget(this);
    auto *mainLayout = new QVBoxLayout(central);
    mainLayout->setContentsMargins(12, 12, 12, 12);

    m_rootTabWidget = new QTabWidget(this);
    m_rootTabWidget->setStyleSheet("QTabWidget::pane { border: 1px solid palette(midlight); border-radius: 6px; }");

    const QString lang = currentLanguage();
    m_rootTabWidget->addTab(createConvertTab(), QIcon::fromTheme("document-edit-symbolic"), trKey("tab_convert", lang));
    const int historyTabIndex = m_rootTabWidget->addTab(createHistoryTab(), QIcon::fromTheme("view-history-symbolic"), trKey("tab_history", lang));
    m_settingsTabIndex = m_rootTabWidget->addTab(createSettingsTab(), QIcon::fromTheme("configure-symbolic", QIcon::fromTheme("preferences-system-symbolic")), trKey("tab_settings", lang));
    m_rootTabWidget->addTab(createAboutTab(), QIcon::fromTheme("help-about-symbolic"), trKey("tab_about", lang));

    // CLI'den yapılan dönüşümler sekme açılınca tembel olarak yüklensin.
    connect(m_rootTabWidget, &QTabWidget::currentChanged, this, [this, historyTabIndex](int index) {
        if (index == historyTabIndex) {
            loadHistoryData();
        } else if (index == m_settingsTabIndex) {
            updateProcessMonitor(true);
        }
    });

    mainLayout->addWidget(m_rootTabWidget);
    setCentralWidget(central);
}

QWidget* MainWindow::createConvertTab() {
    const QString lang = currentLanguage();
    auto *container = new QWidget(this);
    auto *layout = new QVBoxLayout(container);
    layout->setContentsMargins(8, 8, 8, 8);
    layout->setSpacing(8);

    auto *catTabs = new QTabWidget(container);
    catTabs->setStyleSheet("QTabWidget::pane { border: 1px solid palette(midlight); border-radius: 6px; }");

    catTabs->addTab(createPhotosTab(), trKey("cat_photos", lang));
    catTabs->addTab(createVideosTab(), trKey("cat_videos", lang));
    catTabs->addTab(createMusicTab(), trKey("cat_music", lang));
    catTabs->addTab(createDocumentsTab(), trKey("cat_documents", lang));

    layout->addWidget(catTabs);
    return container;
}

QWidget* MainWindow::createPhotosTab() {
    const QString lang = currentLanguage();
    auto *scroll = new QScrollArea(this);
    scroll->setWidgetResizable(true);
    scroll->setFrameShape(QFrame::NoFrame);

    auto *container = new QWidget();
    auto *layout = new QVBoxLayout(container);
    layout->setContentsMargins(12, 12, 12, 12);

    // Card 1: Context Menu Toggles
    auto *actionsWidget = new QWidget(container);
    auto *actionsGrid = new QGridLayout(actionsWidget);
    actionsGrid->setContentsMargins(0, 0, 0, 0);
    actionsGrid->setSpacing(10);

    actionsGrid->addWidget(createActionCheckBox(
        "compress_photo",
        AppTranslator::trActionLabel("compress_photo", lang),
        AppTranslator::trActionDesc("compress_photo", lang),
        "konvert-compress",
        actionsWidget
    ), 0, 0, 1, 2);

    actionsGrid->addWidget(createActionCheckBox(
        "to_webp",
        AppTranslator::trActionLabel("to_webp", lang),
        AppTranslator::trActionDesc("to_webp", lang),
        "konvert-image",
        actionsWidget
    ), 1, 0);

    actionsGrid->addWidget(createActionCheckBox(
        "to_png",
        AppTranslator::trActionLabel("to_png", lang),
        AppTranslator::trActionDesc("to_png", lang),
        "konvert-image",
        actionsWidget
    ), 1, 1);

    actionsGrid->addWidget(createActionCheckBox(
        "to_jpg",
        AppTranslator::trActionLabel("to_jpg", lang),
        AppTranslator::trActionDesc("to_jpg", lang),
        "konvert-image",
        actionsWidget
    ), 2, 0);

    actionsGrid->addWidget(createActionCheckBox(
        "to_svg",
        AppTranslator::trActionLabel("to_svg", lang),
        AppTranslator::trActionDesc("to_svg", lang),
        "konvert-image",
        actionsWidget
    ), 2, 1);

    actionsGrid->addWidget(createActionCheckBox(
        "to_ico",
        AppTranslator::trActionLabel("to_ico", lang),
        AppTranslator::trActionDesc("to_ico", lang),
        "konvert-image",
        actionsWidget
    ), 3, 0);

    actionsGrid->addWidget(createActionCheckBox(
        "to_heic",
        AppTranslator::trActionLabel("to_heic", lang),
        AppTranslator::trActionDesc("to_heic", lang),
        "konvert-image",
        actionsWidget
    ), 3, 1);

    actionsGrid->addWidget(createActionCheckBox(
        "to_bmp",
        AppTranslator::trActionLabel("to_bmp", lang),
        AppTranslator::trActionDesc("to_bmp", lang),
        "konvert-image",
        actionsWidget
    ), 4, 0);

    actionsGrid->addWidget(createActionCheckBox(
        "to_avif",
        AppTranslator::trActionLabel("to_avif", lang),
        AppTranslator::trActionDesc("to_avif", lang),
        "konvert-image",
        actionsWidget
    ), 4, 1);

    actionsGrid->addWidget(createActionCheckBox(
        "to_jxl",
        AppTranslator::trActionLabel("to_jxl", lang),
        AppTranslator::trActionDesc("to_jxl", lang),
        "konvert-image",
        actionsWidget
    ), 5, 0);

    actionsGrid->addWidget(createActionCheckBox(
        "to_jxl_lossless",
        AppTranslator::trActionLabel("to_jxl_lossless", lang),
        AppTranslator::trActionDesc("to_jxl_lossless", lang),
        "konvert-image",
        actionsWidget
    ), 5, 1);

    actionsGrid->addWidget(createActionCheckBox(
        "merge_to_pdf",
        AppTranslator::trActionLabel("merge_to_pdf", lang),
        AppTranslator::trActionDesc("merge_to_pdf", lang),
        "konvert-layers",
        actionsWidget
    ), 6, 0);

    actionsGrid->addWidget(createActionCheckBox(
        "strip_metadata",
        AppTranslator::trActionLabel("strip_metadata", lang),
        AppTranslator::trActionDesc("strip_metadata", lang),
        "konvert-shield",
        actionsWidget
    ), 6, 1);

    actionsGrid->addWidget(createActionCheckBox(
        "resize_50",
        AppTranslator::trActionLabel("resize_50", lang),
        AppTranslator::trActionDesc("resize_50", lang),
        "konvert-scaling",
        actionsWidget
    ), 7, 0);

    actionsGrid->addWidget(createActionCheckBox(
        "resize_1080p",
        AppTranslator::trActionLabel("resize_1080p", lang),
        AppTranslator::trActionDesc("resize_1080p", lang),
        "konvert-monitor",
        actionsWidget
    ), 7, 1);

    actionsGrid->addWidget(createActionCheckBox(
        "web_opt",
        AppTranslator::trActionLabel("web_opt", lang),
        AppTranslator::trActionDesc("web_opt", lang),
        "konvert-globe",
        actionsWidget
    ), 8, 0);

    actionsGrid->addWidget(createActionCheckBox(
        "discord_8mb_photo",
        AppTranslator::trActionLabel("discord_8mb_photo", lang),
        AppTranslator::trActionDesc("discord_8mb_photo", lang),
        "konvert-mail",
        actionsWidget
    ), 8, 1);

    actionsGrid->addWidget(createActionCheckBox(
        "email_25mb_photo",
        AppTranslator::trActionLabel("email_25mb_photo", lang),
        AppTranslator::trActionDesc("email_25mb_photo", lang),
        "konvert-mail",
        actionsWidget
    ), 9, 0, 1, 2);

    layout->addWidget(createCard(trKey("card_actions", lang), actionsWidget));

    // Card 2: Quality & Compression
    auto *qualityWidget = new QWidget(container);
    auto *qualityLayout = new QVBoxLayout(qualityWidget);
    qualityLayout->setContentsMargins(0, 0, 0, 0);

    auto *sliderRow = new QHBoxLayout();
    auto *sliderLabel = new QLabel(trKey("lbl_photo_quality", lang), qualityWidget);
    sliderLabel->setToolTip(trKey("tip_photo_quality", lang));
    sliderRow->addWidget(sliderLabel);

    m_photoQualitySlider = new QSlider(Qt::Horizontal, qualityWidget);
    sliderLabel->setBuddy(m_photoQualitySlider);
    m_photoQualitySlider->setRange(1, 100);
    m_photoQualitySlider->setValue(m_config.imageQuality);
    m_photoQualitySlider->setToolTip("1-100");

    m_photoQualityLabel = new QLabel(QString("%1%").arg(m_config.imageQuality), qualityWidget);
    m_photoQualityLabel->setFixedWidth(40);

    connect(m_photoQualitySlider, &QSlider::valueChanged, this, [this](int val) {
        m_photoQualityLabel->setText(QString("%1%").arg(val));
        m_config.imageQuality = val;
        saveSettings();
    });

    sliderRow->addWidget(m_photoQualitySlider);
    sliderRow->addWidget(m_photoQualityLabel);
    qualityLayout->addLayout(sliderRow);

    m_photoLosslessCheck = new QCheckBox(trKey("lbl_lossless_photo", lang), qualityWidget);
    m_photoLosslessCheck->setToolTip(trKey("tip_lossless_photo", lang));
    m_photoLosslessCheck->setChecked(m_config.imageLossless);
    if (m_photoQualitySlider) {
        m_photoQualitySlider->setEnabled(!m_config.imageLossless);
    }
    connect(m_photoLosslessCheck, &QCheckBox::toggled, this, [this](bool checked) {
        m_config.imageLossless = checked;
        if (m_photoQualitySlider) {
            m_photoQualitySlider->setEnabled(!checked);
        }
        saveSettings();
    });
    qualityLayout->addWidget(m_photoLosslessCheck);

    layout->addWidget(createCard(trKey("card_photo_comp", lang), qualityWidget));
    layout->addStretch();

    scroll->setWidget(container);
    return scroll;
}

QWidget* MainWindow::createVideosTab() {
    const QString lang = currentLanguage();
    auto *scroll = new QScrollArea(this);
    scroll->setWidgetResizable(true);
    scroll->setFrameShape(QFrame::NoFrame);

    auto *container = new QWidget();
    auto *layout = new QVBoxLayout(container);
    layout->setContentsMargins(12, 12, 12, 12);

    // Card 1: Context Menu Toggles
    auto *actionsWidget = new QWidget(container);
    auto *actionsGrid = new QGridLayout(actionsWidget);
    actionsGrid->setContentsMargins(0, 0, 0, 0);
    actionsGrid->setSpacing(10);

    actionsGrid->addWidget(createActionCheckBox(
        "compress_video",
        AppTranslator::trActionLabel("compress_video", lang),
        AppTranslator::trActionDesc("compress_video", lang),
        "konvert-compress",
        actionsWidget
    ), 0, 0, 1, 2);

    actionsGrid->addWidget(createActionCheckBox(
        "to_mp4",
        AppTranslator::trActionLabel("to_mp4", lang),
        AppTranslator::trActionDesc("to_mp4", lang),
        "konvert-video",
        actionsWidget
    ), 1, 0);

    actionsGrid->addWidget(createActionCheckBox(
        "to_mkv",
        AppTranslator::trActionLabel("to_mkv", lang),
        AppTranslator::trActionDesc("to_mkv", lang),
        "konvert-video",
        actionsWidget
    ), 1, 1);

    actionsGrid->addWidget(createActionCheckBox(
        "to_webm",
        AppTranslator::trActionLabel("to_webm", lang),
        AppTranslator::trActionDesc("to_webm", lang),
        "konvert-video",
        actionsWidget
    ), 2, 0);

    actionsGrid->addWidget(createActionCheckBox(
        "to_av1",
        AppTranslator::trActionLabel("to_av1", lang),
        AppTranslator::trActionDesc("to_av1", lang),
        "konvert-video",
        actionsWidget
    ), 2, 1);

    actionsGrid->addWidget(createActionCheckBox(
        "to_gif",
        AppTranslator::trActionLabel("to_gif", lang),
        AppTranslator::trActionDesc("to_gif", lang),
        "konvert-video",
        actionsWidget
    ), 3, 0);

    actionsGrid->addWidget(createActionCheckBox(
        "extract_audio",
        AppTranslator::trActionLabel("extract_audio", lang),
        AppTranslator::trActionDesc("extract_audio", lang),
        "konvert-audio",
        actionsWidget
    ), 3, 1);

    actionsGrid->addWidget(createActionCheckBox(
        "discord_8mb_video",
        AppTranslator::trActionLabel("discord_8mb_video", lang),
        AppTranslator::trActionDesc("discord_8mb_video", lang),
        "konvert-mail",
        actionsWidget
    ), 4, 0);

    actionsGrid->addWidget(createActionCheckBox(
        "email_25mb_video",
        AppTranslator::trActionLabel("email_25mb_video", lang),
        AppTranslator::trActionDesc("email_25mb_video", lang),
        "konvert-mail",
        actionsWidget
    ), 4, 1);

    actionsGrid->addWidget(createActionCheckBox(
        "strip_metadata_video",
        AppTranslator::trActionLabel("strip_metadata_video", lang),
        AppTranslator::trActionDesc("strip_metadata_video", lang),
        "konvert-shield",
        actionsWidget
    ), 5, 0);

    actionsGrid->addWidget(createActionCheckBox(
        "extract_frames",
        AppTranslator::trActionLabel("extract_frames", lang),
        AppTranslator::trActionDesc("extract_frames", lang),
        "konvert-folder",
        actionsWidget
    ), 5, 1);

    layout->addWidget(createCard(trKey("card_actions", lang), actionsWidget));

    // Card 2: Video Compression
    auto *compWidget = new QWidget(container);
    auto *compLayout = new QVBoxLayout(compWidget);
    compLayout->setContentsMargins(0, 0, 0, 0);

    auto *crfRow = new QHBoxLayout();
    auto *crfLabel = new QLabel(trKey("lbl_video_crf", lang), compWidget);
    crfLabel->setToolTip(trKey("tip_video_crf", lang));
    crfRow->addWidget(crfLabel);

    m_videoCrfSlider = new QSlider(Qt::Horizontal, compWidget);
    crfLabel->setBuddy(m_videoCrfSlider);
    m_videoCrfSlider->setRange(0, 51);
    m_videoCrfSlider->setValue(m_config.videoCrf);
    m_videoCrfSlider->setToolTip("CRF (0-51)");
    m_videoCrfLabel = new QLabel(QString::number(m_config.videoCrf), compWidget);
    m_videoCrfLabel->setFixedWidth(30);

    connect(m_videoCrfSlider, &QSlider::valueChanged, this, [this](int val) {
        m_videoCrfLabel->setText(QString::number(val));
        m_config.videoCrf = val;
        saveSettings();
    });

    crfRow->addWidget(m_videoCrfSlider);
    crfRow->addWidget(m_videoCrfLabel);
    compLayout->addLayout(crfRow);

    auto *presetRow = new QHBoxLayout();
    auto *presetLabel = new QLabel(trKey("lbl_video_preset", lang), compWidget);
    presetLabel->setToolTip(trKey("tip_video_preset", lang));
    presetRow->addWidget(presetLabel);

    m_videoPresetCombo = new QComboBox(compWidget);
    presetLabel->setBuddy(m_videoPresetCombo);
    m_videoPresetCombo->addItems({"ultrafast", "superfast", "veryfast", "faster", "fast", "medium", "slow", "slower", "veryslow"});
    if (m_videoPresetCombo->findText(m_config.videoPreset) < 0 && !m_config.videoPreset.isEmpty()) {
        m_videoPresetCombo->addItem(m_config.videoPreset);
    }
    m_videoPresetCombo->setCurrentText(m_config.videoPreset);
    connect(m_videoPresetCombo, &QComboBox::currentTextChanged, this, [this](const QString &text) {
        m_config.videoPreset = text;
        saveSettings();
    });
    presetRow->addWidget(m_videoPresetCombo);
    compLayout->addLayout(presetRow);

    auto *hwaccelRow = new QHBoxLayout();
    auto *hwaccelLabel = new QLabel(trKey("lbl_video_hwaccel", lang), compWidget);
    hwaccelLabel->setToolTip(trKey("tip_video_hwaccel", lang));
    hwaccelRow->addWidget(hwaccelLabel);

    m_videoHwaccelCombo = new QComboBox(compWidget);
    hwaccelLabel->setBuddy(m_videoHwaccelCombo);
    m_videoHwaccelCombo->addItem(trKey("hwaccel_auto", lang), "auto");
    m_videoHwaccelCombo->addItem("NVIDIA NVENC", "nvenc");
    m_videoHwaccelCombo->addItem("VA-API (Intel / AMD)", "vaapi");
    m_videoHwaccelCombo->addItem(trKey("hwaccel_none", lang), "none");

    int hwIdx = 0;
    for (int i = 0; i < m_videoHwaccelCombo->count(); ++i) {
        if (m_videoHwaccelCombo->itemData(i).toString() == m_config.videoHwaccel) {
            hwIdx = i;
            break;
        }
    }
    m_videoHwaccelCombo->setCurrentIndex(hwIdx);
    connect(m_videoHwaccelCombo, QOverload<int>::of(&QComboBox::currentIndexChanged), this, [this](int idx) {
        m_config.videoHwaccel = m_videoHwaccelCombo->itemData(idx).toString();
        saveSettings();
    });
    hwaccelRow->addWidget(m_videoHwaccelCombo);
    compLayout->addLayout(hwaccelRow);

    m_videoLosslessCheck = new QCheckBox(trKey("lbl_video_lossless", lang), compWidget);
    m_videoLosslessCheck->setToolTip(trKey("tip_video_lossless", lang));
    m_videoLosslessCheck->setChecked(m_config.videoLossless);
    if (m_videoCrfSlider) {
        m_videoCrfSlider->setEnabled(!m_config.videoLossless);
    }
    m_videoLosslessWarnLabel = new QLabel(
        trKey("warn_video_lossless", lang),
        compWidget
    );
    m_videoLosslessWarnLabel->setWordWrap(true);
    m_videoLosslessWarnLabel->setStyleSheet("color: #d97706; font-size: 0.88em;");
    m_videoLosslessWarnLabel->setVisible(m_config.videoLossless);

    connect(m_videoLosslessCheck, &QCheckBox::toggled, this, [this](bool checked) {
        m_config.videoLossless = checked;
        if (m_videoCrfSlider) {
            m_videoCrfSlider->setEnabled(!checked);
        }
        if (m_videoLosslessWarnLabel) {
            m_videoLosslessWarnLabel->setVisible(checked);
        }
        saveSettings();
    });
    compLayout->addWidget(m_videoLosslessCheck);
    compLayout->addWidget(m_videoLosslessWarnLabel);

    layout->addWidget(createCard(trKey("card_video_comp", lang), compWidget));
    layout->addStretch();

    scroll->setWidget(container);
    return scroll;
}

QWidget* MainWindow::createMusicTab() {
    const QString lang = currentLanguage();
    auto *scroll = new QScrollArea(this);
    scroll->setWidgetResizable(true);
    scroll->setFrameShape(QFrame::NoFrame);

    auto *container = new QWidget();
    auto *layout = new QVBoxLayout(container);
    layout->setContentsMargins(12, 12, 12, 12);

    // Card 1: Context Menu Toggles
    auto *actionsWidget = new QWidget(container);
    auto *actionsLayout = new QVBoxLayout(actionsWidget);
    actionsLayout->setContentsMargins(0, 0, 0, 0);
    actionsLayout->setSpacing(8);

    actionsLayout->addWidget(createActionCheckBox(
        "to_mp3",
        AppTranslator::trActionLabel("to_mp3", lang),
        AppTranslator::trActionDesc("to_mp3", lang),
        "konvert-audio",
        actionsWidget
    ));
    actionsLayout->addWidget(createActionCheckBox(
        "to_wav",
        AppTranslator::trActionLabel("to_wav", lang),
        AppTranslator::trActionDesc("to_wav", lang),
        "konvert-audio",
        actionsWidget
    ));
    actionsLayout->addWidget(createActionCheckBox(
        "audio_to_video",
        AppTranslator::trActionLabel("audio_to_video", lang),
        AppTranslator::trActionDesc("audio_to_video", lang),
        "konvert-video",
        actionsWidget
    ));

    layout->addWidget(createCard(trKey("card_actions", lang), actionsWidget));

    // Card 2: Audio Settings
    auto *audioWidget = new QWidget(container);
    auto *audioLayout = new QVBoxLayout(audioWidget);
    audioLayout->setContentsMargins(0, 0, 0, 0);

    auto *bitrateRow = new QHBoxLayout();
    auto *bitrateLabel = new QLabel(trKey("lbl_audio_bitrate", lang), audioWidget);
    bitrateLabel->setToolTip(trKey("tip_audio_bitrate", lang));
    bitrateRow->addWidget(bitrateLabel);

    m_audioBitrateCombo = new QComboBox(audioWidget);
    bitrateLabel->setBuddy(m_audioBitrateCombo);
    m_audioBitrateCombo->addItems({"128k", "192k", "256k", "320k"});
    const QString configuredBitrate =
        m_config.audioBitrate.contains("320") ? "320k" : m_config.audioBitrate;
    if (m_audioBitrateCombo->findText(configuredBitrate) < 0 && !configuredBitrate.isEmpty()) {
        m_audioBitrateCombo->addItem(configuredBitrate);
    }
    m_audioBitrateCombo->setCurrentText(configuredBitrate);
    connect(m_audioBitrateCombo, &QComboBox::currentTextChanged, this, [this](const QString &text) {
        m_config.audioBitrate = text;
        saveSettings();
    });
    bitrateRow->addWidget(m_audioBitrateCombo);
    audioLayout->addLayout(bitrateRow);

    layout->addWidget(createCard(trKey("card_audio_comp", lang), audioWidget));
    layout->addStretch();

    scroll->setWidget(container);
    return scroll;
}

QWidget* MainWindow::createDocumentsTab() {
    const QString lang = currentLanguage();
    auto *scroll = new QScrollArea(this);
    scroll->setWidgetResizable(true);
    scroll->setFrameShape(QFrame::NoFrame);

    auto *container = new QWidget();
    auto *layout = new QVBoxLayout(container);
    layout->setContentsMargins(12, 12, 12, 12);

    // Card 1: Context Menu Toggles
    auto *actionsWidget = new QWidget(container);
    auto *actionsLayout = new QVBoxLayout(actionsWidget);
    actionsLayout->setContentsMargins(0, 0, 0, 0);
    actionsLayout->setSpacing(8);

    actionsLayout->addWidget(createActionCheckBox(
        "compress_doc",
        AppTranslator::trActionLabel("compress_doc", lang),
        AppTranslator::trActionDesc("compress_doc", lang),
        "konvert-compress",
        actionsWidget
    ));
    actionsLayout->addWidget(createActionCheckBox(
        "udf_to_pdf",
        AppTranslator::trActionLabel("udf_to_pdf", lang),
        AppTranslator::trActionDesc("udf_to_pdf", lang),
        "konvert-document",
        actionsWidget
    ));
    actionsLayout->addWidget(createActionCheckBox(
        "udf_to_txt",
        AppTranslator::trActionLabel("udf_to_txt", lang),
        AppTranslator::trActionDesc("udf_to_txt", lang),
        "konvert-document",
        actionsWidget
    ));
    actionsLayout->addWidget(createActionCheckBox(
        "pdf_to_images",
        AppTranslator::trActionLabel("pdf_to_images", lang),
        AppTranslator::trActionDesc("pdf_to_images", lang),
        "konvert-image",
        actionsWidget
    ));
    actionsLayout->addWidget(createActionCheckBox(
        "docx_to_pdf",
        AppTranslator::trActionLabel("docx_to_pdf", lang),
        AppTranslator::trActionDesc("docx_to_pdf", lang),
        "konvert-document",
        actionsWidget
    ));
    actionsLayout->addWidget(createActionCheckBox(
        "docx_to_txt",
        AppTranslator::trActionLabel("docx_to_txt", lang),
        AppTranslator::trActionDesc("docx_to_txt", lang),
        "konvert-document",
        actionsWidget
    ));

    layout->addWidget(createCard(trKey("card_actions", lang), actionsWidget));

    // Card 2: Document Settings
    auto *docWidget = new QWidget(container);
    auto *docLayout = new QVBoxLayout(docWidget);
    docLayout->setContentsMargins(0, 0, 0, 0);

    auto *dpiRow = new QHBoxLayout();
    auto *dpiLabel = new QLabel(trKey("lbl_pdf_dpi", lang), docWidget);
    dpiLabel->setToolTip(trKey("tip_pdf_dpi", lang));
    dpiRow->addWidget(dpiLabel);

    m_pdfDpiSpin = new QSpinBox(docWidget);
    dpiLabel->setBuddy(m_pdfDpiSpin);
    m_pdfDpiSpin->setRange(72, 1200);
    m_pdfDpiSpin->setValue(m_config.pdfDpi);
    connect(m_pdfDpiSpin, &QSpinBox::valueChanged, this, [this](int val) {
        m_config.pdfDpi = val;
        saveSettings();
    });
    dpiRow->addWidget(m_pdfDpiSpin);
    docLayout->addLayout(dpiRow);

    layout->addWidget(createCard(trKey("card_doc_comp", lang), docWidget));
    layout->addStretch();

    scroll->setWidget(container);
    return scroll;
}

QWidget* MainWindow::createHistoryTab() {
    const QString lang = currentLanguage();
    auto *container = new QWidget(this);
    auto *layout = new QVBoxLayout(container);
    layout->setContentsMargins(12, 12, 12, 12);
    layout->setSpacing(10);

    // Summary & Control Card
    auto *summaryWidget = new QWidget(container);
    auto *summaryLayout = new QHBoxLayout(summaryWidget);
    summaryLayout->setContentsMargins(0, 0, 0, 0);

    auto *statsLayout = new QVBoxLayout();
    m_totalOperationsLabel = new QLabel(trKey("lbl_total_operations_init", lang), summaryWidget);
    m_totalSavingsLabel = new QLabel(trKey("lbl_total_savings_init", lang), summaryWidget);
    statsLayout->addWidget(m_totalOperationsLabel);
    statsLayout->addWidget(m_totalSavingsLabel);
    summaryLayout->addLayout(statsLayout);

    summaryLayout->addStretch();

    auto *btnRefresh = new QPushButton(trKey("btn_refresh", lang), summaryWidget);
    btnRefresh->setIcon(QIcon::fromTheme("view-refresh-symbolic"));
    connect(btnRefresh, &QPushButton::clicked, this, &MainWindow::loadHistoryData);
    summaryLayout->addWidget(btnRefresh);

    auto *btnClear = new QPushButton(trKey("btn_clear_history", lang), summaryWidget);
    btnClear->setIcon(QIcon::fromTheme("edit-delete-symbolic"));
    connect(btnClear, &QPushButton::clicked, this, &MainWindow::clearHistoryData);
    summaryLayout->addWidget(btnClear);

    layout->addWidget(createCard(trKey("card_history_stats", lang), summaryWidget));

    // History Table
    m_historyTable = new QTableWidget(container);
    m_historyTable->setColumnCount(7);
    m_historyTable->setHorizontalHeaderLabels({
        trKey("hdr_date_time", lang),
        trKey("hdr_action", lang),
        trKey("hdr_files", lang),
        trKey("hdr_original", lang),
        trKey("hdr_output", lang),
        trKey("hdr_savings", lang),
        trKey("hdr_status", lang)
    });
    m_historyTable->horizontalHeader()->setSectionResizeMode(QHeaderView::ResizeToContents);
    m_historyTable->horizontalHeader()->setStretchLastSection(true);
    m_historyTable->setAlternatingRowColors(true);
    m_historyTable->setEditTriggers(QAbstractItemView::NoEditTriggers);
    m_historyTable->setSelectionBehavior(QAbstractItemView::SelectRows);
    m_historyTable->setSelectionMode(QAbstractItemView::SingleSelection);
    m_historyTable->verticalHeader()->setVisible(false);

    layout->addWidget(m_historyTable, 1);

    // File Preview & Location Panel
    m_previewPanel = new QWidget(container);
    auto *previewLayout = new QHBoxLayout(m_previewPanel);
    previewLayout->setContentsMargins(6, 6, 6, 6);
    previewLayout->setSpacing(14);

    m_previewThumbLabel = new QLabel(m_previewPanel);
    m_previewThumbLabel->setFixedSize(80, 80);
    m_previewThumbLabel->setAlignment(Qt::AlignCenter);
    m_previewThumbLabel->setStyleSheet("border: 1px solid palette(midlight); border-radius: 6px; background-color: palette(base);");
    previewLayout->addWidget(m_previewThumbLabel);

    auto *infoLayout = new QVBoxLayout();
    infoLayout->setSpacing(4);
    m_previewTitleLabel = new QLabel(trKey("lbl_preview_select", lang), m_previewPanel);
    m_previewTitleLabel->setWordWrap(true);
    infoLayout->addWidget(m_previewTitleLabel);

    m_previewPathLabel = new QLabel("", m_previewPanel);
    m_previewPathLabel->setStyleSheet("color: palette(placeholder-text); font-size: 0.88em;");
    m_previewPathLabel->setWordWrap(true);
    infoLayout->addWidget(m_previewPathLabel);

    auto *btnRow = new QHBoxLayout();
    btnRow->setSpacing(8);

    m_previewOpenBtn = new QPushButton(trKey("btn_open_file", lang), m_previewPanel);
    m_previewOpenBtn->setIcon(QIcon::fromTheme("document-open-symbolic"));
    m_previewOpenBtn->setEnabled(false);
    connect(m_previewOpenBtn, &QPushButton::clicked, this, [this]() {
        if (!m_selectedOutputPath.isEmpty() && QFile::exists(m_selectedOutputPath)) {
            QDesktopServices::openUrl(QUrl::fromLocalFile(m_selectedOutputPath));
        }
    });
    btnRow->addWidget(m_previewOpenBtn);

    m_previewDolphinBtn = new QPushButton(trKey("btn_show_in_fm", lang), m_previewPanel);
    m_previewDolphinBtn->setIcon(QIcon::fromTheme("system-file-manager-symbolic"));
    m_previewDolphinBtn->setEnabled(false);
    connect(m_previewDolphinBtn, &QPushButton::clicked, this, [this]() {
        if (!m_selectedOutputPath.isEmpty()) {
            if (QFile::exists(m_selectedOutputPath)) {
                if (!QProcess::startDetached("dolphin", {"--select", m_selectedOutputPath})) {
                    QDesktopServices::openUrl(QUrl::fromLocalFile(QFileInfo(m_selectedOutputPath).dir().absolutePath()));
                }
            }
        }
    });
    btnRow->addWidget(m_previewDolphinBtn);
    btnRow->addStretch();
    infoLayout->addLayout(btnRow);

    previewLayout->addLayout(infoLayout, 1);

    layout->addWidget(createCard(trKey("card_history_preview", lang), m_previewPanel));

    connect(m_historyTable, &QTableWidget::itemSelectionChanged, this, &MainWindow::onHistorySelectionChanged);

    return container;
}

void MainWindow::loadHistoryData() {
    if (!m_historyTable) return;
    const QString lang = currentLanguage();

    QString dataDir = QStandardPaths::writableLocation(QStandardPaths::GenericDataLocation);
    QString historyFile = QDir(dataDir).filePath("konvert/history.json");

    QFile file(historyFile);
    if (!file.open(QIODevice::ReadOnly)) {
        m_historyTable->setRowCount(0);
        if (m_totalOperationsLabel) m_totalOperationsLabel->setText(trKey("lbl_total_operations_init", lang));
        if (m_totalSavingsLabel) m_totalSavingsLabel->setText(trKey("lbl_total_savings_init", lang));
        return;
    }

    QByteArray data = file.readAll();
    file.close();

    QJsonDocument doc = QJsonDocument::fromJson(data);
    if (!doc.isArray()) return;

    QJsonArray array = doc.array();
    m_historyTable->setUpdatesEnabled(false);
    m_historyTable->setRowCount(array.size());

    qint64 totalSavedBytes = 0;
    int totalFiles = 0;

    auto formatBytes = [](qint64 bytes) -> QString {
        const double KB = 1024.0;
        const double MB = KB * 1024.0;
        const double GB = MB * 1024.0;
        const QString sign = bytes < 0 ? "-" : "";
        double b = std::abs(static_cast<double>(bytes));
        if (b >= GB) return sign + QString::number(b / GB, 'f', 1) + " GB";
        if (b >= MB) return sign + QString::number(b / MB, 'f', 1) + " MB";
        if (b >= KB) return sign + QString::number(b / KB, 'f', 1) + " KB";
        return QString::number(bytes) + " B";
    };

    for (int i = 0; i < array.size(); ++i) {
        QJsonObject obj = array[i].toObject();
        QString timestamp = obj["timestamp"].toString();
        QString target = obj["target"].toString();
        int fileCount = obj["file_count"].toInt();
        qint64 origSize = static_cast<qint64>(obj["original_size_bytes"].toDouble());
        qint64 outSize = static_cast<qint64>(obj["output_size_bytes"].toDouble());
        double savingsPct = obj["savings_percent"].toDouble();
        QString status = obj["status"].toString();
        const QString normalizedStatus = status.trimmed().toLower();
        const bool isCancelled = (normalizedStatus == "cancelled"
            || normalizedStatus == "canceled"
            || normalizedStatus == "iptal"
            || normalizedStatus == "iptal edildi");
        const bool isFailed = (normalizedStatus == "failed"
            || normalizedStatus == "hata"
            || normalizedStatus == "başarısız");

        totalFiles += fileCount;
        // Başarısız veya iptal edilen işlemler tasarruf toplamına katılmaz; büyüyen dönüşümler düşülür (imzalı delta).
        if (!isFailed && !isCancelled) {
            totalSavedBytes += (origSize - outSize);
        }

        QString outPath = obj["output_path"].toString();
        auto *timeItem = new QTableWidgetItem(timestamp);
        timeItem->setData(Qt::UserRole, outPath);
        m_historyTable->setItem(i, 0, timeItem);
        m_historyTable->setItem(i, 1, new QTableWidgetItem(target.toUpper()));
        m_historyTable->setItem(i, 2, new QTableWidgetItem(QString::number(fileCount)));
        m_historyTable->setItem(i, 3, new QTableWidgetItem(formatBytes(origSize)));
        m_historyTable->setItem(i, 4, new QTableWidgetItem(formatBytes(outSize)));

        QString savingsStr = QString("%1%").arg(savingsPct, 0, 'f', 1);
        if (savingsPct > 0) savingsStr = QString("+%1").arg(savingsStr);
        auto *savingsItem = new QTableWidgetItem(savingsStr);
        if (savingsPct > 0) {
            savingsItem->setForeground(QColor(46, 170, 70));
        }
        m_historyTable->setItem(i, 5, savingsItem);

        auto *statusItem = new QTableWidgetItem(status);
        if (normalizedStatus == "success" || normalizedStatus == "completed"
            || normalizedStatus == "tamamlandı" || normalizedStatus == "başarılı") {
            statusItem->setForeground(QColor(46, 170, 70));
        } else if (normalizedStatus == "partial" || normalizedStatus == "kısmi" || isCancelled) {
            statusItem->setForeground(QColor(210, 150, 40));
        } else {
            statusItem->setForeground(QColor(220, 80, 70));
        }
        m_historyTable->setItem(i, 6, statusItem);
    }
    m_historyTable->setUpdatesEnabled(true);

    if (m_totalOperationsLabel) {
        m_totalOperationsLabel->setText(trKey("lbl_total_operations_fmt", lang).arg(array.size()).arg(totalFiles));
    }
    if (m_totalSavingsLabel) {
        m_totalSavingsLabel->setText(trKey("lbl_total_savings_fmt", lang).arg(formatBytes(totalSavedBytes)));
    }

    onHistorySelectionChanged();
}

void MainWindow::onHistorySelectionChanged() {
    if (!m_historyTable) return;
    const QString lang = currentLanguage();
    int row = m_historyTable->currentRow();
    if (row < 0 || row >= m_historyTable->rowCount()) {
        m_selectedOutputPath.clear();
        if (m_previewTitleLabel) m_previewTitleLabel->setText(trKey("lbl_preview_select", lang));
        if (m_previewPathLabel) m_previewPathLabel->setText("");
        if (m_previewThumbLabel) m_previewThumbLabel->clear();
        if (m_previewOpenBtn) m_previewOpenBtn->setEnabled(false);
        if (m_previewDolphinBtn) m_previewDolphinBtn->setEnabled(false);
        return;
    }

    auto *item = m_historyTable->item(row, 0);
    if (!item) return;

    m_selectedOutputPath = item->data(Qt::UserRole).toString();

    if (m_selectedOutputPath.isEmpty() || !QFile::exists(m_selectedOutputPath)) {
        if (m_previewTitleLabel) {
            m_previewTitleLabel->setText(
                m_selectedOutputPath.isEmpty()
                    ? trKey("lbl_preview_no_path", lang)
                    : trKey("lbl_preview_missing", lang)
            );
        }
        if (m_previewPathLabel) m_previewPathLabel->setText(m_selectedOutputPath);
        if (m_previewThumbLabel) m_previewThumbLabel->clear();
        if (m_previewOpenBtn) m_previewOpenBtn->setEnabled(false);
        if (m_previewDolphinBtn) m_previewDolphinBtn->setEnabled(false);
        return;
    }

    QFileInfo fi(m_selectedOutputPath);
    QString ext = fi.suffix().toLower();
    const QString sizeText = m_historyTable->item(row, 4) ? m_historyTable->item(row, 4)->text() : "";

    if (m_previewTitleLabel) {
        m_previewTitleLabel->setText(
            QString("<b>%1</b> (%2)").arg(fi.fileName().toHtmlEscaped(), sizeText.toHtmlEscaped())
        );
    }
    if (m_previewPathLabel) {
        m_previewPathLabel->setText(m_selectedOutputPath);
    }

    const quint64 reqId = ++m_previewRequestId;
    const qreal dpr = devicePixelRatioF();

    // Küçük resmi UI iş parçacığını bloklamadan arka plan havuzunda DPR ölçekli çöz.
    if (ext == "png" || ext == "jpg" || ext == "jpeg" || ext == "webp" || ext == "avif"
        || ext == "bmp" || ext == "svg" || ext == "gif" || ext == "ico"
        || ext == "tiff" || ext == "tif")
    {
        if (m_previewThumbLabel) {
            QFileIconProvider prov;
            QPixmap placeholder = prov.icon(fi).pixmap(QSize(52, 52), dpr);
            placeholder.setDevicePixelRatio(dpr);
            m_previewThumbLabel->setPixmap(placeholder);
        }

        const QString thumbPath = m_selectedOutputPath;
        const QSize targetPx(qRound(72.0 * dpr), qRound(72.0 * dpr));
        QPointer<MainWindow> self(this);
        QThreadPool::globalInstance()->start([self, reqId, thumbPath, targetPx, dpr]() {
            QImageReader reader(thumbPath);
            reader.setAutoTransform(true);
            const QSize originalSize = reader.size();
            if (originalSize.isValid()) {
                reader.setScaledSize(originalSize.scaled(targetPx, Qt::KeepAspectRatio));
            }
            const QImage thumb = reader.read();
            if (!self) return;
            QMetaObject::invokeMethod(self, [self, reqId, thumbPath, thumb, dpr]() {
                if (!self || self->m_previewRequestId != reqId || !self->m_previewThumbLabel) {
                    return;
                }
                if (!thumb.isNull()) {
                    QPixmap pix = QPixmap::fromImage(thumb);
                    pix.setDevicePixelRatio(dpr);
                    self->m_previewThumbLabel->setPixmap(pix);
                } else {
                    QFileIconProvider prov;
                    QPixmap fallback = prov.icon(QFileInfo(thumbPath)).pixmap(QSize(52, 52), dpr);
                    fallback.setDevicePixelRatio(dpr);
                    self->m_previewThumbLabel->setPixmap(fallback);
                }
            }, Qt::QueuedConnection);
        });
    } else if (m_previewThumbLabel) {
        QFileIconProvider prov;
        QPixmap fallback = prov.icon(fi).pixmap(QSize(52, 52), dpr);
        fallback.setDevicePixelRatio(dpr);
        m_previewThumbLabel->setPixmap(fallback);
    }

    if (m_previewOpenBtn) m_previewOpenBtn->setEnabled(true);
    if (m_previewDolphinBtn) m_previewDolphinBtn->setEnabled(true);
}

void MainWindow::clearHistoryData() {
    const QString lang = currentLanguage();
    const auto answer = QMessageBox::question(
        this,
        trKey("dlg_clear_history_title", lang),
        trKey("dlg_clear_history_body", lang),
        QMessageBox::Yes | QMessageBox::No,
        QMessageBox::No);
    if (answer != QMessageBox::Yes) {
        return;
    }

    QString dataDir = QStandardPaths::writableLocation(QStandardPaths::GenericDataLocation);
    QString historyFile = QDir(dataDir).filePath("konvert/history.json");
    if (QFile::exists(historyFile) && !QFile::remove(historyFile)) {
        QMessageBox::warning(
            this,
            trKey("dlg_error_title", lang),
            trKey("dlg_clear_history_err", lang));
        return;
    }
    loadHistoryData();
    onHistorySelectionChanged();
}

QWidget* MainWindow::createAboutTab() {
    const QString lang = currentLanguage();
    auto *scroll = new QScrollArea(this);
    scroll->setWidgetResizable(true);
    scroll->setFrameShape(QFrame::NoFrame);

    auto *container = new QWidget();
    auto *layout = new QVBoxLayout(container);
    layout->setContentsMargins(14, 14, 14, 14);
    layout->setSpacing(12);

    // 1. Authentic KDE Header: Logo + App Name + Subtitle
    auto *headerWidget = new QWidget(container);
    auto *headerLayout = new QVBoxLayout(headerWidget);
    headerLayout->setAlignment(Qt::AlignCenter);
    headerLayout->setContentsMargins(0, 4, 0, 4);
    headerLayout->setSpacing(6);

    auto *iconLabel = new QLabel(headerWidget);
    const qreal dpr = devicePixelRatioF();
    QPixmap logoPix = QIcon(":/logo.svg").pixmap(QSize(72, 72), dpr);
    if (!logoPix.isNull()) {
        logoPix.setDevicePixelRatio(dpr);
        iconLabel->setPixmap(logoPix);
    }
    iconLabel->setAlignment(Qt::AlignCenter);
    headerLayout->addWidget(iconLabel);

    auto *titleLabel = new QLabel(
        QString(
            "<div style='text-align: center;'>"
            "<span style='font-size: 1.55em; font-weight: bold;'>Konvert</span> "
            "<span style='font-size: 0.95em; opacity: 0.7;'>%1</span><br>"
            "<span style='font-size: 0.92em;'>%2</span></div>"
        ).arg(QString(KONVERT_APP_VERSION).toHtmlEscaped(), trKey("about_subtitle", lang).toHtmlEscaped()),
        headerWidget
    );
    titleLabel->setAlignment(Qt::AlignCenter);
    headerLayout->addWidget(titleLabel);

    layout->addWidget(createCard("", headerWidget));

    // 2. Authentic KDE About Sub-Tabs (KAboutApplicationDialog Style)
    auto *aboutTabs = new QTabWidget(container);
    aboutTabs->setStyleSheet("QTabWidget::pane { border: 1px solid palette(midlight); border-radius: 6px; }");

    // --- SUB-TAB 1: ABOUT (HAKKINDA) ---
    auto *infoWidget = new QWidget();
    auto *infoLayout = new QVBoxLayout(infoWidget);
    infoLayout->setContentsMargins(14, 14, 14, 14);
    infoLayout->setSpacing(12);

    auto *lblDesc = new QLabel(trKey("about_description", lang), infoWidget);
    lblDesc->setWordWrap(true);
    infoLayout->addWidget(lblDesc);

    auto *specGrid = new QGridLayout();
    specGrid->setVerticalSpacing(8);
    specGrid->setHorizontalSpacing(18);

    QString desktopEnv = qgetenv("XDG_CURRENT_DESKTOP");
    if (desktopEnv.isEmpty()) desktopEnv = "KDE Plasma";

    auto addSpecRow = [&](int row, const QString &label, const QString &val) {
        auto *lblK = new QLabel(QString("<b>%1</b>").arg(label), infoWidget);
        auto *lblV = new QLabel(val, infoWidget);
        lblV->setOpenExternalLinks(true);
        specGrid->addWidget(lblK, row, 0);
        specGrid->addWidget(lblV, row, 1);
    };

    QString archStr = QSysInfo::buildCpuArchitecture();
    if (archStr.isEmpty()) archStr = "x86_64";

    addSpecRow(0, trKey("about_spec_version", lang), QString("%1 (%2-linux)").arg(KONVERT_APP_VERSION, archStr));
    addSpecRow(1, trKey("about_spec_desktop", lang), desktopEnv);
    addSpecRow(2, trKey("about_spec_gui", lang), trKey("about_spec_gui_val", lang));
    addSpecRow(3, trKey("about_spec_engine", lang), trKey("about_spec_engine_val", lang));
    addSpecRow(4, trKey("about_spec_homepage", lang), "<a href='https://github.com/enesehs/konvert'>github.com/enesehs/konvert</a>");
    addSpecRow(5, trKey("about_spec_bugs", lang), "<a href='https://github.com/enesehs/konvert/issues'>github.com/enesehs/konvert/issues</a>");

    infoLayout->addLayout(specGrid);
    infoLayout->addStretch();
    aboutTabs->addTab(infoWidget, trKey("about_tab_info", lang));

    // --- SUB-TAB 2: AUTHORS & COMMUNITY (YAZARLAR VE TOPLULUK) ---
    auto *authorsWidget = new QWidget();
    auto *authorsLayout = new QVBoxLayout(authorsWidget);
    authorsLayout->setContentsMargins(14, 14, 14, 14);
    authorsLayout->setSpacing(12);

    auto *authorBox = new QFrame(authorsWidget);
    authorBox->setFrameShape(QFrame::StyledPanel);
    authorBox->setStyleSheet("QFrame { border: 1px solid palette(midlight); border-radius: 6px; background-color: palette(base); padding: 10px; }");
    auto *authorBoxLayout = new QVBoxLayout(authorBox);
    authorBoxLayout->setSpacing(4);
    auto *lblAuthTitle = new QLabel(QString("<b>%1</b>").arg(trKey("about_author_title", lang)), authorBox);
    auto *lblAuthDetail = new QLabel(
        QString(
            "<b>enesehs</b> &lt;<a href='https://enesehs.dev'>https://enesehs.dev</a>&gt;<br>"
            "<span style='font-size: 0.92em; opacity: 0.8;'>%1</span>"
        ).arg(trKey("about_author_desc", lang)),
        authorBox
    );
    lblAuthDetail->setOpenExternalLinks(true);
    authorBoxLayout->addWidget(lblAuthTitle);
    authorBoxLayout->addWidget(lblAuthDetail);
    authorsLayout->addWidget(authorBox);

    authorsLayout->addStretch();
    aboutTabs->addTab(authorsWidget, trKey("about_tab_authors", lang));

    // --- SUB-TAB 3: COMPONENTS (BİLEŞENLER) ---
    auto *compWidget = new QWidget();
    auto *compLayout = new QVBoxLayout(compWidget);
    compLayout->setContentsMargins(14, 14, 14, 14);
    compLayout->setSpacing(10);

    auto *compGrid = new QGridLayout();
    compGrid->setVerticalSpacing(8);
    compGrid->setHorizontalSpacing(16);

    auto addCompRow = [&](int row, const QString &title, const QString &desc) {
        auto *lblT = new QLabel(QString("<b>%1</b>").arg(title), compWidget);
        auto *lblD = new QLabel(desc, compWidget);
        lblD->setWordWrap(true);
        compGrid->addWidget(lblT, row, 0, Qt::AlignTop);
        compGrid->addWidget(lblD, row, 1);
    };

    addCompRow(0, "Rust & Rayon", trKey("about_comp_rust", lang));
    addCompRow(1, "Qt 6 Widgets", trKey("about_comp_qt", lang));
    addCompRow(2, "zbus & D-Bus", trKey("about_comp_zbus", lang));
    addCompRow(3, "FFmpeg", trKey("about_comp_ffmpeg", lang));
    addCompRow(4, "Potrace & Oxipng", trKey("about_comp_potrace", lang));
    addCompRow(5, "libjxl (JPEG XL)", trKey("about_comp_jxl", lang));
    addCompRow(6, "Poppler & Ghostscript", trKey("about_comp_pdf", lang));
    addCompRow(7, "ImageMagick", trKey("about_comp_im", lang));

    compLayout->addLayout(compGrid);
    compLayout->addStretch();
    aboutTabs->addTab(compWidget, trKey("about_tab_components", lang));

    // --- SUB-TAB 4: LICENSE (LİSANS) ---
    auto *licWidget = new QWidget();
    auto *licLayout = new QVBoxLayout(licWidget);
    licLayout->setContentsMargins(14, 14, 14, 14);
    licLayout->setSpacing(8);

    auto *lblLicHeader = new QLabel("<b>GNU General Public License v3.0 (GPL-3.0-or-later)</b>", licWidget);
    licLayout->addWidget(lblLicHeader);

    auto *licBrowser = new QTextBrowser(licWidget);
    licBrowser->setReadOnly(true);
    licBrowser->setOpenExternalLinks(true);
    licBrowser->setFont(QFontDatabase::systemFont(QFontDatabase::FixedFont));
    licBrowser->setStyleSheet("QTextBrowser { border: 1px solid palette(midlight); border-radius: 6px; padding: 10px; background-color: palette(base); font-size: 0.88em; }");
    licBrowser->setPlainText(
        "GNU GENERAL PUBLIC LICENSE\n"
        "Version 3, 29 June 2007\n\n"
        "Copyright (C) 2007 Free Software Foundation, Inc. <https://fsf.org/>\n\n"
        "This program is free software: you can redistribute it and/or modify\n"
        "it under the terms of the GNU General Public License as published by\n"
        "the Free Software Foundation, either version 3 of the License, or\n"
        "(at your option) any later version.\n\n"
        "This program is distributed in the hope that it will be useful,\n"
        "but WITHOUT ANY WARRANTY; without even the implied warranty of\n"
        "MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the\n"
        "GNU General Public License for more details.\n\n"
        "You should have received a copy of the GNU General Public License\n"
        "along with this program. If not, see <https://www.gnu.org/licenses/>."
    );
    licLayout->addWidget(licBrowser);
    aboutTabs->addTab(licWidget, trKey("about_tab_license", lang));

    layout->addWidget(aboutTabs);
    layout->addStretch();

    scroll->setWidget(container);
    return scroll;
}

void MainWindow::updateProcessMonitor(bool force) {
    if (!m_processStatusLabel || !m_cancelProcessBtn) return;
    if (!force && (!isVisible() || (m_rootTabWidget && m_rootTabWidget->currentIndex() != m_settingsTabIndex))) {
        return;
    }

    const QString lang = currentLanguage();
    qint64 pid = getActiveKonvertPid();
    const bool paused = pid > 0 && isPidPaused(pid);

    if (!force && pid == m_lastMonitoredPid && paused == m_lastMonitoredPaused && lang == m_lastMonitoredLang) {
        return;
    }
    m_lastMonitoredPid = pid;
    m_lastMonitoredPaused = paused;
    m_lastMonitoredLang = lang;

    if (pid > 0) {
        m_activeProcessPaused = paused;
        if (paused) {
            m_processStatusLabel->setText(
                QString("<span style='color: #f39c12; font-weight: bold;'>||</span> %1")
                    .arg(trKey("lbl_process_paused", lang).arg(pid))
            );
            if (m_pauseResumeProcessBtn) {
                m_pauseResumeProcessBtn->setIcon(QIcon::fromTheme("media-playback-start-symbolic"));
                m_pauseResumeProcessBtn->setText(trKey("btn_resume_process", lang));
                m_pauseResumeProcessBtn->setEnabled(true);
            }
        } else {
            m_processStatusLabel->setText(
                QString("<span style='color: #27ae60; font-weight: bold;'>●</span> %1")
                    .arg(trKey("lbl_process_running", lang).arg(pid))
            );
            if (m_pauseResumeProcessBtn) {
                m_pauseResumeProcessBtn->setIcon(QIcon::fromTheme("media-playback-pause-symbolic"));
                m_pauseResumeProcessBtn->setText(trKey("btn_pause_process", lang));
                m_pauseResumeProcessBtn->setEnabled(true);
            }
        }
        m_cancelProcessBtn->setEnabled(true);
    } else {
        m_activeProcessPaused = false;
        m_processStatusLabel->setText(
            QString("<span style='color: #7f8c8d;'>○</span> %1")
                .arg(trKey("lbl_process_idle", lang))
        );
        if (m_pauseResumeProcessBtn) {
            m_pauseResumeProcessBtn->setIcon(QIcon::fromTheme("media-playback-pause-symbolic"));
            m_pauseResumeProcessBtn->setText(trKey("btn_pause_process", lang));
            m_pauseResumeProcessBtn->setEnabled(false);
        }
        m_cancelProcessBtn->setEnabled(false);
    }
}

QWidget* MainWindow::createSettingsTab() {
    const QString lang = currentLanguage();
    auto *scroll = new QScrollArea(this);
    scroll->setWidgetResizable(true);
    scroll->setFrameShape(QFrame::NoFrame);

    auto *container = new QWidget();
    auto *layout = new QVBoxLayout(container);
    layout->setContentsMargins(12, 12, 12, 12);
    layout->setSpacing(12);

    // --- Card 1: General Preferences ---
    auto *genWidget = new QWidget(container);
    auto *genLayout = new QVBoxLayout(genWidget);
    genLayout->setContentsMargins(4, 4, 4, 4);
    genLayout->setSpacing(12);

    // Language row
    auto *langRow = new QHBoxLayout();
    auto *lblLang = new QLabel(QString("<b>%1</b>").arg(trKey("lbl_language", lang)), genWidget);
    langRow->addWidget(lblLang);

    m_langCombo = new QComboBox(genWidget);
    lblLang->setBuddy(m_langCombo);
    m_langCombo->addItem(trKey("lang_auto", lang), "auto");
    m_langCombo->addItem("Türkçe", "tr");
    m_langCombo->addItem("English", "en");
    m_langCombo->addItem("Deutsch", "de");
    m_langCombo->addItem("Français", "fr");
    m_langCombo->addItem("Español", "es");
    m_langCombo->addItem("Italiano", "it");
    m_langCombo->addItem("Русский", "ru");
    m_langCombo->addItem("中文", "zh");
    m_langCombo->addItem("日本語", "ja");

    selectComboLanguage(m_langCombo, m_config.language);
    connect(m_langCombo, QOverload<int>::of(&QComboBox::currentIndexChanged), this, &MainWindow::onLanguageChanged);

    langRow->addWidget(m_langCombo);
    langRow->addStretch();
    genLayout->addLayout(langRow);

    // Notification checkbox
    m_notifyCheck = new QCheckBox(trKey("lbl_notifications", lang), genWidget);
    m_notifyCheck->setToolTip(trKey("tip_notifications", lang));
    m_notifyCheck->setChecked(m_config.notificationEnabled);
    connect(m_notifyCheck, &QCheckBox::toggled, this, [this](bool checked) {
        m_config.notificationEnabled = checked;
        saveSettings();
    });
    genLayout->addWidget(m_notifyCheck);

    // Output Directory selector (P3-10)
    auto *outDirLabel = new QLabel(QString("<b>%1</b>").arg(trKey("lbl_output_dir", lang)), genWidget);
    genLayout->addWidget(outDirLabel);

    auto *outDirRow = new QHBoxLayout();
    m_outputDirEdit = new QLineEdit(genWidget);
    outDirLabel->setBuddy(m_outputDirEdit);
    m_outputDirEdit->setReadOnly(true);
    m_outputDirEdit->setPlaceholderText(trKey("lbl_output_dir_default", lang));
    m_outputDirEdit->setText(m_config.outputDirectory);
    m_outputDirEdit->setToolTip(trKey("tip_output_dir", lang));
    outDirRow->addWidget(m_outputDirEdit);

    auto *browseBtn = new QPushButton(QIcon::fromTheme("document-open-symbolic"), trKey("btn_browse", lang), genWidget);
    connect(browseBtn, &QPushButton::clicked, this, [this]() {
        const QString lang = currentLanguage();
        QString initial = m_config.outputDirectory.isEmpty() ? QDir::homePath() : m_config.outputDirectory;
        QString dir = QFileDialog::getExistingDirectory(this, trKey("lbl_output_dir", lang), initial);
        if (!dir.isEmpty()) {
            m_config.outputDirectory = dir;
            m_outputDirEdit->setText(dir);
            saveSettings();
        }
    });
    outDirRow->addWidget(browseBtn);

    auto *resetBtn = new QPushButton(QIcon::fromTheme("edit-clear-symbolic"), trKey("btn_reset", lang), genWidget);
    connect(resetBtn, &QPushButton::clicked, this, [this]() {
        m_config.outputDirectory = "";
        m_outputDirEdit->clear();
        saveSettings();
    });
    outDirRow->addWidget(resetBtn);

    genLayout->addLayout(outDirRow);
    layout->addWidget(createCard(trKey("card_general_settings", lang), genWidget));

    // --- Card 2: Active Process Monitor (P3-11) ---
    auto *monitorWidget = new QWidget(container);
    auto *monitorLayout = new QVBoxLayout(monitorWidget);
    monitorLayout->setContentsMargins(4, 4, 4, 4);
    monitorLayout->setSpacing(10);

    auto *statusRow = new QHBoxLayout();
    m_processStatusLabel = new QLabel(monitorWidget);
    m_processStatusLabel->setStyleSheet("font-size: 1.0em;");
    statusRow->addWidget(m_processStatusLabel);
    statusRow->addStretch();

    m_pauseResumeProcessBtn = new QPushButton(
        QIcon::fromTheme("media-playback-pause-symbolic"),
        trKey("btn_pause_process", lang),
        monitorWidget
    );
    m_pauseResumeProcessBtn->setEnabled(false);
    connect(m_pauseResumeProcessBtn, &QPushButton::clicked, this, [this]() {
        const QString lang = currentLanguage();
        const QString flag = m_activeProcessPaused ? "--resume" : "--pause";
        const QString titleKey = m_activeProcessPaused ? "btn_resume_process" : "btn_pause_process";
        runKonvertCommandAsync(m_pauseResumeProcessBtn, {flag}, [this, lang, titleKey](bool ok, const QString &out) {
            if (!ok && !out.isEmpty()) {
                QMessageBox::warning(this, trKey(titleKey, lang), out);
            }
            updateProcessMonitor(true);
        });
    });
    statusRow->addWidget(m_pauseResumeProcessBtn);

    m_cancelProcessBtn = new QPushButton(QIcon::fromTheme("process-stop-symbolic"), trKey("btn_cancel_process", lang), monitorWidget);
    m_cancelProcessBtn->setEnabled(false);
    connect(m_cancelProcessBtn, &QPushButton::clicked, this, [this]() {
        const QString lang = currentLanguage();
        runKonvertCommandAsync(m_cancelProcessBtn, {"--cancel"}, [this, lang](bool ok, const QString &out) {
            if (!out.isEmpty()) {
                if (ok) {
                    QMessageBox::information(this, trKey("btn_cancel_process", lang), out);
                } else {
                    QMessageBox::warning(this, trKey("btn_cancel_process", lang), out);
                }
            }
            updateProcessMonitor(true);
            loadHistoryData();
        });
    });
    statusRow->addWidget(m_cancelProcessBtn);
    monitorLayout->addLayout(statusRow);
    layout->addWidget(createCard(trKey("card_process_monitor", lang), monitorWidget));

    // --- Card 3: Desktop Integration & System Dependencies (P3-9) ---
    auto *deskWidget = new QWidget(container);
    auto *deskLayout = new QVBoxLayout(deskWidget);
    deskLayout->setContentsMargins(4, 4, 4, 4);
    deskLayout->setSpacing(10);

    auto *deskDesc = new QLabel(trKey("msg_desk_desc", lang), deskWidget);
    deskDesc->setWordWrap(true);
    deskLayout->addWidget(deskDesc);

    auto *btnRow = new QHBoxLayout();
    auto *installBtn = new QPushButton(QIcon::fromTheme("system-software-install-symbolic"), trKey("btn_install_integration", lang), deskWidget);
    connect(installBtn, &QPushButton::clicked, this, [this, installBtn]() {
        const QString lang = currentLanguage();
        runKonvertCommandAsync(installBtn, {"--install-all"}, [this, lang](bool ok, const QString &out) {
            if (ok) {
                QString msg = out.isEmpty() ? trKey("msg_install_ok", lang) : out;
                QMessageBox::information(this, trKey("btn_install_integration", lang), msg);
            } else {
                QString msg = out.isEmpty() ? trKey("msg_install_fail", lang) : out;
                QMessageBox::warning(this, trKey("btn_install_integration", lang), msg);
            }
        });
    });
    btnRow->addWidget(installBtn);

    auto *uninstallBtn = new QPushButton(QIcon::fromTheme("edit-delete-symbolic"), trKey("btn_uninstall_integration", lang), deskWidget);
    connect(uninstallBtn, &QPushButton::clicked, this, [this, uninstallBtn]() {
        const QString lang = currentLanguage();
        auto reply = QMessageBox::question(
            this,
            trKey("btn_uninstall_integration", lang),
            trKey("msg_uninstall_confirm", lang),
            QMessageBox::Yes | QMessageBox::No
        );
        if (reply == QMessageBox::Yes) {
            runKonvertCommandAsync(uninstallBtn, {"--uninstall-all"}, [this, lang](bool ok, const QString &out) {
                if (ok) {
                    QString msg = out.isEmpty() ? trKey("msg_uninstall_ok", lang) : out;
                    QMessageBox::information(this, trKey("btn_uninstall_integration", lang), msg);
                } else {
                    QString msg = out.isEmpty() ? trKey("msg_uninstall_fail", lang) : out;
                    QMessageBox::warning(this, trKey("btn_uninstall_integration", lang), msg);
                }
            });
        }
    });
    btnRow->addWidget(uninstallBtn);

    auto *checkDepsBtn = new QPushButton(QIcon::fromTheme("system-run-symbolic"), trKey("btn_check_deps", lang), deskWidget);
    connect(checkDepsBtn, &QPushButton::clicked, this, [this, checkDepsBtn, lang]() {
        runKonvertCommandAsync(checkDepsBtn, {"--doctor"}, [this, lang](bool, const QString &out) {
            QString text = out.isEmpty() ? trKey("msg_doctor_fail", lang) : out;
            if (m_depsStatusEdit) {
                m_depsStatusEdit->setPlainText(text);
                m_depsStatusEdit->setVisible(true);
            }
        });
    });
    btnRow->addWidget(checkDepsBtn);
    btnRow->addStretch();
    deskLayout->addLayout(btnRow);

    m_depsStatusEdit = new QPlainTextEdit(deskWidget);
    m_depsStatusEdit->setReadOnly(true);
    m_depsStatusEdit->setFont(QFontDatabase::systemFont(QFontDatabase::FixedFont));
    m_depsStatusEdit->setLineWrapMode(QPlainTextEdit::WidgetWidth);
    m_depsStatusEdit->setMinimumHeight(180);
    m_depsStatusEdit->setMaximumHeight(320);
    m_depsStatusEdit->setStyleSheet("QPlainTextEdit { background: palette(base); border: 1px solid palette(midlight); border-radius: 4px; padding: 8px; }");
    m_depsStatusEdit->setVisible(false);
    deskLayout->addWidget(m_depsStatusEdit);

    layout->addWidget(createCard(trKey("card_desktop_integration", lang), deskWidget));

    layout->addStretch();
    scroll->setWidget(container);

    updateProcessMonitor(true);
    return scroll;
}

void MainWindow::onActionCheckChanged() {
    QStringList enabled;
    for (auto *cb : m_actionCheckBoxes) {
        if (cb->isChecked()) {
            enabled.append(cb->property("actionId").toString());
        }
    }
    m_config.enabledActions = enabled;
    saveSettings();
}

void MainWindow::captureScreenshots(const QString &outputDir) {
    QDir().mkpath(outputDir);
    resize(740, 780);
    show();
    QApplication::processEvents();

    if (m_rootTabWidget) {
        m_rootTabWidget->setCurrentIndex(0);
        QApplication::processEvents();
        grab().save(QDir(outputDir).filePath("screenshot_convert.png"));

        m_rootTabWidget->setCurrentIndex(1);
        if (m_historyTable && m_historyTable->rowCount() > 0) {
            m_historyTable->selectRow(0);
        }
        QApplication::processEvents();
        grab().save(QDir(outputDir).filePath("screenshot_history.png"));

        m_rootTabWidget->setCurrentIndex(2);
        QApplication::processEvents();
        grab().save(QDir(outputDir).filePath("screenshot_settings.png"));

        m_rootTabWidget->setCurrentIndex(3);
        QApplication::processEvents();
        grab().save(QDir(outputDir).filePath("screenshot_about.png"));
    }
}

void MainWindow::onLanguageChanged(int index) {
    if (!m_langCombo) return;
    QString langCode = m_langCombo->itemData(index).toString();
    if (m_config.language != langCode) {
        const int prevTab = m_rootTabWidget ? m_rootTabWidget->currentIndex() : 0;
        m_config.language = langCode;
        saveSettings();
        QTimer::singleShot(0, this, [this, prevTab]() {
            setupUi();
            if (m_rootTabWidget && prevTab >= 0 && prevTab < m_rootTabWidget->count()) {
                m_rootTabWidget->setCurrentIndex(prevTab);
            }
        });
    }
}

void MainWindow::saveSettings() {
    if (m_saveTimer) {
        m_saveTimer->start();
    } else {
        saveSettingsNow();
    }
}

void MainWindow::runKonvertCommandAsync(
    QPushButton *btn,
    const QStringList &args,
    std::function<void(bool, const QString &)> onDone
) {
    QPointer<QPushButton> btnPtr(btn);
    if (btnPtr) {
        btnPtr->setEnabled(false);
    }

    auto *proc = new QProcess(this);
    auto completed = QSharedPointer<bool>::create(false);

    connect(proc, QOverload<int, QProcess::ExitStatus>::of(&QProcess::finished),
        this, [this, proc, btnPtr, completed, cb = onDone](int exitCode, QProcess::ExitStatus exitStatus) {
            if (*completed) return;
            *completed = true;
            if (btnPtr) {
                btnPtr->setEnabled(true);
            }
            QString out = QString::fromUtf8(proc->readAllStandardOutput()).trimmed();
            const QString err = QString::fromUtf8(proc->readAllStandardError()).trimmed();
            if (out.isEmpty()) {
                out = err;
            } else if (!err.isEmpty() && (exitStatus != QProcess::NormalExit || exitCode != 0)) {
                out += "\n" + err;
            }
            const bool ok = (exitStatus == QProcess::NormalExit && exitCode == 0);
            proc->deleteLater();
            if (cb) cb(ok, out);
        });

    connect(proc, &QProcess::errorOccurred,
        this, [this, proc, btnPtr, completed, cb = onDone](QProcess::ProcessError) {
            if (*completed) return;
            *completed = true;
            if (btnPtr) {
                btnPtr->setEnabled(true);
            }
            const QString errMsg = proc->errorString();
            proc->deleteLater();
            if (cb) cb(false, errMsg);
        });

    proc->start(ConfigManager::findKonvertBinary(), args);
}

void MainWindow::saveSettingsNow() {
    m_isSavingConfig = true;
    if (m_reloadGuardTimer) {
        m_reloadGuardTimer->stop();
    }
    m_config.saveAsync(this, [this](bool ok) {
        if (!ok) {
            const QString lang = currentLanguage();
            QMessageBox::warning(
                this,
                trKey("msg_save_err_title", lang),
                trKey("msg_save_err_body", lang));
        }
        const QString configPath = ConfigManager::getConfigPath();
        if (m_configWatcher && QFile::exists(configPath) && !m_configWatcher->files().contains(configPath)) {
            m_configWatcher->addPath(configPath);
        }
        if (m_reloadGuardTimer) {
            m_reloadGuardTimer->start();
        } else {
            m_isSavingConfig = false;
        }
    });
}

void MainWindow::flushSettingsSync() {
    m_isSavingConfig = true;
    if (m_reloadGuardTimer) {
        m_reloadGuardTimer->stop();
    }
    m_config.save();
    m_isSavingConfig = false;
}

void MainWindow::setupConfigWatcher() {
    const QString configPath = ConfigManager::getConfigPath();
    const QString configDir = QFileInfo(configPath).absolutePath();
    QDir().mkpath(configDir);

    if (!m_configWatcher) {
        m_configWatcher = new QFileSystemWatcher(this);
    }
    if (!m_configWatcher->directories().contains(configDir)) {
        m_configWatcher->addPath(configDir);
    }
    if (QFile::exists(configPath) && !m_configWatcher->files().contains(configPath)) {
        m_configWatcher->addPath(configPath);
    }

    auto handleFsEvent = [this, configPath]() {
        if (m_configWatcher && QFile::exists(configPath) && !m_configWatcher->files().contains(configPath)) {
            m_configWatcher->addPath(configPath);
        }
        if (m_isSavingConfig || (m_saveTimer && m_saveTimer->isActive())) {
            return;
        }
        reloadConfigToUi();
    };

    connect(m_configWatcher, &QFileSystemWatcher::fileChanged, this, [handleFsEvent](const QString &) {
        handleFsEvent();
    });
    connect(m_configWatcher, &QFileSystemWatcher::directoryChanged, this, [handleFsEvent](const QString &) {
        handleFsEvent();
    });
}

void MainWindow::reloadConfigToUi() {
    m_config.load();

    if (m_photoQualitySlider) {
        const QSignalBlocker blocker(m_photoQualitySlider);
        m_photoQualitySlider->setValue(m_config.imageQuality);
        m_photoQualitySlider->setEnabled(!m_config.imageLossless);
    }
    if (m_photoQualityLabel) {
        m_photoQualityLabel->setText(QString("%1%").arg(m_config.imageQuality));
    }
    if (m_photoLosslessCheck) {
        const QSignalBlocker blocker(m_photoLosslessCheck);
        m_photoLosslessCheck->setChecked(m_config.imageLossless);
    }

    if (m_videoCrfSlider) {
        const QSignalBlocker blocker(m_videoCrfSlider);
        m_videoCrfSlider->setValue(m_config.videoCrf);
        m_videoCrfSlider->setEnabled(!m_config.videoLossless);
    }
    if (m_videoCrfLabel) {
        m_videoCrfLabel->setText(QString::number(m_config.videoCrf));
    }
    if (m_videoPresetCombo) {
        const QSignalBlocker blocker(m_videoPresetCombo);
        int idx = m_videoPresetCombo->findText(m_config.videoPreset);
        if (idx >= 0) m_videoPresetCombo->setCurrentIndex(idx);
    }
    if (m_videoLosslessCheck) {
        const QSignalBlocker blocker(m_videoLosslessCheck);
        m_videoLosslessCheck->setChecked(m_config.videoLossless);
    }
    if (m_videoLosslessWarnLabel) {
        m_videoLosslessWarnLabel->setVisible(m_config.videoLossless);
    }
    if (m_videoHwaccelCombo) {
        const QSignalBlocker blocker(m_videoHwaccelCombo);
        for (int i = 0; i < m_videoHwaccelCombo->count(); ++i) {
            if (m_videoHwaccelCombo->itemData(i).toString() == m_config.videoHwaccel) {
                m_videoHwaccelCombo->setCurrentIndex(i);
                break;
            }
        }
    }

    if (m_audioBitrateCombo) {
        const QSignalBlocker blocker(m_audioBitrateCombo);
        int idx = m_audioBitrateCombo->findText(m_config.audioBitrate);
        if (idx >= 0) m_audioBitrateCombo->setCurrentIndex(idx);
    }
    if (m_pdfDpiSpin) {
        const QSignalBlocker blocker(m_pdfDpiSpin);
        m_pdfDpiSpin->setValue(m_config.pdfDpi);
    }
    if (m_notifyCheck) {
        const QSignalBlocker blocker(m_notifyCheck);
        m_notifyCheck->setChecked(m_config.notificationEnabled);
    }
    if (m_outputDirEdit) {
        const QSignalBlocker blocker(m_outputDirEdit);
        m_outputDirEdit->setText(m_config.outputDirectory);
    }
    if (m_langCombo) {
        const QSignalBlocker blocker(m_langCombo);
        selectComboLanguage(m_langCombo, m_config.language);
    }

    for (auto *cb : m_actionCheckBoxes) {
        if (cb) {
            const QSignalBlocker blocker(cb);
            QString id = cb->property("actionId").toString();
            cb->setChecked(m_config.enabledActions.contains(id));
        }
    }
}

void MainWindow::closeEvent(QCloseEvent *event) {
    if (m_saveTimer && m_saveTimer->isActive()) {
        m_saveTimer->stop();
        flushSettingsSync();
    }
    QMainWindow::closeEvent(event);
}
