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

#ifndef KONVERT_APP_VERSION
#define KONVERT_APP_VERSION "v0.1-beta"
#endif

static QString findKonvertBinary() {
    QString program = "konvert";
    const QFileInfo sibling(QDir(QCoreApplication::applicationDirPath()).filePath("konvert"));
    if (sibling.exists() && sibling.isExecutable()) {
        return sibling.absoluteFilePath();
    }
    const QFileInfo cargoSibling(QDir(QCoreApplication::applicationDirPath()).filePath("../../target/release/konvert"));
    if (cargoSibling.exists() && cargoSibling.isExecutable()) {
        return cargoSibling.canonicalFilePath();
    }
    const QFileInfo cargoDebugSibling(QDir(QCoreApplication::applicationDirPath()).filePath("../../target/debug/konvert"));
    if (cargoDebugSibling.exists() && cargoDebugSibling.isExecutable()) {
        return cargoDebugSibling.canonicalFilePath();
    }
    return program;
}

static QString getPidFilePath() {
    QString cacheDir = QStandardPaths::writableLocation(QStandardPaths::GenericCacheLocation);
    return QDir(cacheDir).filePath("konvert/running.pid");
}

static qint64 getActiveKonvertPid() {
    QString pidPath = getPidFilePath();
    QFile file(pidPath);
    if (!file.exists() || !file.open(QIODevice::ReadOnly | QIODevice::Text)) {
        return 0;
    }
    QTextStream in(&file);
    QString pidStr = in.readLine().trimmed();
    bool ok = false;
    qint64 pid = pidStr.toLongLong(&ok);
    if (!ok || pid <= 0) {
        return 0;
    }

    QFile commFile(QString("/proc/%1/comm").arg(pid));
    if (commFile.open(QIODevice::ReadOnly | QIODevice::Text)) {
        QString comm = QTextStream(&commFile).readLine().trimmed();
        if (comm == "konvert") {
            return pid;
        }
    }
    QFileInfo exeInfo(QString("/proc/%1/exe").arg(pid));
    if (exeInfo.exists()) {
        QString symlink = exeInfo.symLinkTarget();
        if (symlink.contains("konvert") && !symlink.contains("konvert-settings")) {
            return pid;
        }
    }
    return 0;
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

    m_processMonitorTimer = new QTimer(this);
    m_processMonitorTimer->setInterval(2000);
    connect(m_processMonitorTimer, &QTimer::timeout, this, &MainWindow::updateProcessMonitor);
    m_processMonitorTimer->start();

    setupConfigWatcher();

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

QWidget* MainWindow::createCard(const QString &title, QWidget *contentWidget) {
    auto *frame = new QFrame(this);
    frame->setFrameShape(QFrame::StyledPanel);
    frame->setStyleSheet(
        "QFrame { "
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
        auto *lblTitle = new QLabel(QString("<b>%1</b>").arg(title), frame);
        lblTitle->setStyleSheet("font-size: 13px; border: none; background: transparent;");
        layout->addWidget(lblTitle);

        auto *sep = new QFrame(frame);
        sep->setFrameShape(QFrame::HLine);
        sep->setStyleSheet("background-color: palette(midlight); max-height: 1px; margin-bottom: 8px; border: none;");
        layout->addWidget(sep);
    }

    contentWidget->setStyleSheet("border: none; background: transparent;");
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
    cb->setStyleSheet("QCheckBox { font-size: 13px; padding: 4px 0; }");

    connect(cb, &QCheckBox::toggled, this, &MainWindow::onActionCheckChanged);
    m_actionCheckBoxes.append(cb);
    return cb;
}

void MainWindow::setupUi() {
    auto *oldCentral = centralWidget();
    if (oldCentral) {
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
    const int settingsTabIndex = m_rootTabWidget->addTab(createSettingsTab(), QIcon::fromTheme("configure-symbolic", QIcon::fromTheme("preferences-system-symbolic")), trKey("tab_settings", lang));
    m_rootTabWidget->addTab(createAboutTab(), QIcon::fromTheme("help-about-symbolic"), trKey("tab_about", lang));

    // CLI'den yapılan dönüşümler sekme açılınca görünsün.
    connect(m_rootTabWidget, &QTabWidget::currentChanged, this, [this, historyTabIndex, settingsTabIndex](int index) {
        if (index == historyTabIndex) {
            loadHistoryData();
        } else if (index == settingsTabIndex) {
            updateProcessMonitor();
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
        lang == "tr" ? "Görseli Sıkıştır" : "Compress Image",
        lang == "tr" ? "Görseli insan gözünün fark edemeyeceği maksimum görsel kalite ve en küçük dosya boyutu dengesiyle WebP formatında sıkıştırır." : "Compresses image to WebP with optimal balance of maximum visual quality and minimum file size.",
        "konvert-compress",
        actionsWidget
    ), 0, 0, 1, 2);

    actionsGrid->addWidget(createActionCheckBox(
        "to_webp",
        lang == "tr" ? "WebP'ye Dönüştür" : "Convert to WebP",
        lang == "tr" ? "Modern WebP formatına dönüştürür. JPEG ve PNG'ye kıyasla yüksek oranda boyut tasarrufu sağlar." : "Converts to modern WebP format, providing significant size reduction compared to JPEG and PNG.",
        "konvert-image",
        actionsWidget
    ), 1, 0);

    actionsGrid->addWidget(createActionCheckBox(
        "to_png",
        lang == "tr" ? "PNG'ye Dönüştür" : "Convert to PNG",
        lang == "tr" ? "Kayıpsız ve şeffaflık (alpha) kanalını destekleyen standart PNG formatına dönüştürür." : "Converts to lossless standard PNG with transparency support.",
        "konvert-image",
        actionsWidget
    ), 1, 1);

    actionsGrid->addWidget(createActionCheckBox(
        "to_jpg",
        lang == "tr" ? "JPG'ye Dönüştür" : "Convert to JPG",
        lang == "tr" ? "Evrensel uyumluluğa sahip standart JPEG formatına dönüştürür." : "Converts to universal standard JPEG image.",
        "konvert-image",
        actionsWidget
    ), 2, 0);

    actionsGrid->addWidget(createActionCheckBox(
        "to_svg",
        lang == "tr" ? "SVG'ye Dönüştür" : "Convert to SVG",
        lang == "tr" ? "Görseli ölçeklenebilir vektör grafiği formatına dönüştürür." : "Converts image to scalable vector graphic format.",
        "konvert-image",
        actionsWidget
    ), 2, 1);

    actionsGrid->addWidget(createActionCheckBox(
        "to_ico",
        lang == "tr" ? "ICO'ya Dönüştür" : "Convert to ICO",
        lang == "tr" ? "Web sitesi favicon veya uygulama simgesi olarak kullanılan ICO formatına dönüştürür." : "Converts to standard Windows and favicon ICO format.",
        "konvert-image",
        actionsWidget
    ), 3, 0);

    actionsGrid->addWidget(createActionCheckBox(
        "to_heic",
        lang == "tr" ? "HEIC'e Dönüştür" : "Convert to HEIC",
        lang == "tr" ? "Apple ve modern cihazlarda kullanılan yüksek verimli HEIC formatına dönüştürür." : "Converts to high-efficiency HEIC format.",
        "konvert-image",
        actionsWidget
    ), 3, 1);

    actionsGrid->addWidget(createActionCheckBox(
        "to_bmp",
        lang == "tr" ? "BMP'ye Dönüştür" : "Convert to BMP",
        lang == "tr" ? "Sıkıştırmasız standart Windows Bitmap formatına dönüştürür." : "Converts to uncompressed standard Windows Bitmap format.",
        "konvert-image",
        actionsWidget
    ), 4, 0);

    actionsGrid->addWidget(createActionCheckBox(
        "to_avif",
        lang == "tr" ? "AVIF'e Dönüştür" : "Convert to AVIF",
        lang == "tr" ? "AV1 video kodlayıcı tabanlı, yüksek sıkıştırma verimliliği sunan yeni nesil görsel formatı." : "Next-generation image format based on AV1, providing superior compression.",
        "konvert-image",
        actionsWidget
    ), 4, 1);

    actionsGrid->addWidget(createActionCheckBox(
        "to_jxl",
        lang == "tr" ? "JXL'e Dönüştür" : "Convert to JXL",
        lang == "tr" ? "Yeni nesil JPEG XL formatına dönüştürür. Geniş renk gamutu ve üstün sıkıştırma oranı sunar." : "Converts to next-generation JPEG XL format with wide color gamut and high compression.",
        "konvert-image",
        actionsWidget
    ), 5, 0);

    actionsGrid->addWidget(createActionCheckBox(
        "to_jxl_lossless",
        lang == "tr" ? "Kayıpsız JXL'e Dönüştür" : "Convert to JXL (Lossless)",
        lang == "tr" ? "Piksel kaybı olmaksızın matematiksel olarak tam kayıpsız JPEG XL sıkıştırması uygular." : "Mathematically lossless JPEG XL compression without pixel loss.",
        "konvert-image",
        actionsWidget
    ), 5, 1);

    actionsGrid->addWidget(createActionCheckBox(
        "merge_to_pdf",
        lang == "tr" ? "Tek PDF Olarak Birleştir" : "Merge to Single PDF",
        lang == "tr" ? "Seçilen tüm görselleri tek bir PDF belgesi içerisinde sayfalar halinde birleştirir." : "Merges all selected images into a single multi-page PDF document.",
        "konvert-layers",
        actionsWidget
    ), 6, 0);

    actionsGrid->addWidget(createActionCheckBox(
        "strip_metadata",
        lang == "tr" ? "EXIF ve Metadata Temizle" : "Strip Metadata & EXIF",
        lang == "tr" ? "Görseldeki GPS konumu, çekim tarihi, kamera modeli ve etiketleri temizleyerek gizliliği korur." : "Cleans GPS coordinates, camera model, shooting date, and personal tags for privacy.",
        "konvert-shield",
        actionsWidget
    ), 6, 1);

    actionsGrid->addWidget(createActionCheckBox(
        "resize_50",
        lang == "tr" ? "Görseli %50 Küçült" : "Resize by 50%",
        lang == "tr" ? "Görselin genişlik ve yükseklik çözünürlüğünü yarıya indirerek dosya boyutunu büyük oranda azaltır." : "Reduces image dimensions by half (50%) to shrink file size.",
        "konvert-scaling",
        actionsWidget
    ), 7, 0);

    actionsGrid->addWidget(createActionCheckBox(
        "resize_1080p",
        lang == "tr" ? "Full HD'ye Ölçekle" : "Resize to Full HD",
        lang == "tr" ? "Görselin en-boy oranını koruyarak 1920x1080 Full HD sınırlarına sığacak şekilde ölçekler." : "Scales image proportionally to fit within 1920x1080 Full HD bounds.",
        "konvert-monitor",
        actionsWidget
    ), 7, 1);

    actionsGrid->addWidget(createActionCheckBox(
        "web_opt",
        lang == "tr" ? "Web İçin Optimize Et (WebP)" : "Optimize for Web (WebP)",
        lang == "tr" ? "WebP formatında %82 kalite ve sıfır EXIF ile web için ideal boyuta getirir." : "Converts to WebP with 82% quality and stripped EXIF for fast web delivery.",
        "konvert-globe",
        actionsWidget
    ), 8, 0);

    actionsGrid->addWidget(createActionCheckBox(
        "discord_8mb_photo",
        lang == "tr" ? "Discord İçin Sıkıştır (8 MB)" : "Compress for Discord (8 MB)",
        lang == "tr" ? "Görseli 8 MB sınırına sığacak şekilde optimize eder." : "Optimizes image to guarantee size under 8 MB.",
        "konvert-mail",
        actionsWidget
    ), 8, 1);

    actionsGrid->addWidget(createActionCheckBox(
        "email_25mb_photo",
        lang == "tr" ? "E-Posta İçin Sıkıştır (25 MB)" : "Compress for Email (25 MB)",
        lang == "tr" ? "Görseli 25 MB e-posta ek sınırına uyacak şekilde optimize eder." : "Optimizes image to fit standard 25 MB email attachment limits.",
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
    sliderLabel->setToolTip(lang == "tr" ? "Hedef görsel kalitesi (1-100). 85 önerilir." : "Target image quality percentage (1-100). 85 is recommended.");
    sliderRow->addWidget(sliderLabel);

    m_photoQualitySlider = new QSlider(Qt::Horizontal, qualityWidget);
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
    m_photoLosslessCheck->setToolTip(lang == "tr" ? "WebP, PNG ve JXL dönüşümlerinde tam kayıpsız modu zorlar." : "Forces mathematically lossless mode in WebP, PNG, and JXL conversions.");
    m_photoLosslessCheck->setChecked(m_config.imageLossless);
    connect(m_photoLosslessCheck, &QCheckBox::toggled, this, [this](bool checked) {
        m_config.imageLossless = checked;
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
        lang == "tr" ? "Videoyu Sıkıştır" : "Compress Video",
        lang == "tr" ? "Videoyu H.264 CRF 28 ve slow kodlama önayarı ile görsel kalitesini koruyarak %80'e varan oranda sıkıştırır." : "Compresses video with H.264 CRF 28 and slow preset preserving visual quality.",
        "konvert-compress",
        actionsWidget
    ), 0, 0, 1, 2);

    actionsGrid->addWidget(createActionCheckBox(
        "to_mp4",
        lang == "tr" ? "MP4'e Dönüştür" : "Convert to MP4",
        lang == "tr" ? "Evrensel cihaz ve tarayıcı uyumluluğu için standart H.264/AAC MP4 formatına dönüştürür." : "Converts to standard H.264/AAC MP4 for universal device and browser compatibility.",
        "konvert-video",
        actionsWidget
    ), 1, 0);

    actionsGrid->addWidget(createActionCheckBox(
        "to_mkv",
        lang == "tr" ? "MKV'ye Dönüştür" : "Convert to MKV",
        lang == "tr" ? "Çoklu ses ve altyazı izlerini destekleyen esnek Matroska video kapsayıcısına dönüştürür." : "Converts to versatile Matroska video container supporting multiple audio and subtitle streams.",
        "konvert-video",
        actionsWidget
    ), 1, 1);

    actionsGrid->addWidget(createActionCheckBox(
        "to_webm",
        lang == "tr" ? "WebM'ye Dönüştür" : "Convert to WebM",
        lang == "tr" ? "Web ve HTML5 uyumlu açık kaynaklı VP9 video formatına dönüştürür." : "Converts to open-source VP9 WebM format for web and HTML5 streaming.",
        "konvert-video",
        actionsWidget
    ), 2, 0);

    actionsGrid->addWidget(createActionCheckBox(
        "to_gif",
        lang == "tr" ? "GIF'e Dönüştür" : "Convert to GIF",
        lang == "tr" ? "İki geçişli renk paleti analiziyle videonun ilk 15 saniyesini yüksek kaliteli animasyonlu GIF'e dönüştürür." : "Converts the first 15 seconds of video to a high-quality animated GIF using two-pass palette optimization.",
        "konvert-video",
        actionsWidget
    ), 2, 1);

    actionsGrid->addWidget(createActionCheckBox(
        "extract_frames",
        lang == "tr" ? "Kareleri Klasöre Aktar" : "Export Frames to Folder",
        lang == "tr" ? "Videonun her video karesini PNG resimleri olarak yeni bir klasöre döker." : "Exports all video frames as individual PNG images to a dedicated folder.",
        "konvert-folder",
        actionsWidget
    ), 3, 0);

    actionsGrid->addWidget(createActionCheckBox(
        "extract_audio",
        lang == "tr" ? "Videodan Sesi Ayıkla" : "Extract Audio",
        lang == "tr" ? "Videonun ses izini doğrudan ayrı bir MP3 ses dosyası olarak dışa aktarır." : "Demuxes and extracts the audio stream directly into an MP3 file.",
        "konvert-audio",
        actionsWidget
    ), 3, 1);

    actionsGrid->addWidget(createActionCheckBox(
        "discord_8mb_video",
        lang == "tr" ? "Discord İçin Sıkıştır (8 MB)" : "Compress for Discord (8 MB)",
        lang == "tr" ? "Videonun süresine göre bit hızını otomatik ayarlayarak 8 MB boyutuna sığdırır." : "Automatically computes video bitrate according to length to fit within 8 MB.",
        "konvert-mail",
        actionsWidget
    ), 4, 0);

    actionsGrid->addWidget(createActionCheckBox(
        "email_25mb_video",
        lang == "tr" ? "E-Posta İçin Sıkıştır (25 MB)" : "Compress for Email (25 MB)",
        lang == "tr" ? "Videonun süresine göre bit hızını otomatik ayarlayarak 25 MB e-posta ek sınırına sığdırır." : "Automatically calculates bitrate to fit within 25 MB email attachment limit.",
        "konvert-mail",
        actionsWidget
    ), 4, 1);

    layout->addWidget(createCard(trKey("card_actions", lang), actionsWidget));

    // Card 2: Video Compression
    auto *compWidget = new QWidget(container);
    auto *compLayout = new QVBoxLayout(compWidget);
    compLayout->setContentsMargins(0, 0, 0, 0);

    auto *crfRow = new QHBoxLayout();
    auto *crfLabel = new QLabel(trKey("lbl_video_crf", lang), compWidget);
    crfLabel->setToolTip(lang == "tr" ? "Sabit Oran Faktörü (0-51): 0 kayıpsız, 18-23 dengeli yüksek kalite, 28+ yüksek sıkıştırma." : "Constant Rate Factor (0-51): 0 is lossless, 18-23 is balanced high quality, 28+ is high compression.");
    crfRow->addWidget(crfLabel);

    m_videoCrfSlider = new QSlider(Qt::Horizontal, compWidget);
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
    presetLabel->setToolTip(lang == "tr" ? "Kodlayıcı hız önayarı: 'slow' veya 'medium' en iyi sıkıştırma/süre dengesini sunar." : "Encoder speed preset: 'slow' or 'medium' offers the best compression-to-time ratio.");
    presetRow->addWidget(presetLabel);

    m_videoPresetCombo = new QComboBox(compWidget);
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

    m_videoLosslessCheck = new QCheckBox(trKey("lbl_video_lossless", lang), compWidget);
    m_videoLosslessCheck->setToolTip(lang == "tr" ? "CRF 0 ile x264 RGB kayıpsız kodlama yapar." : "Encodes using x264 RGB with CRF 0 without lossy compression artifacts.");
    m_videoLosslessCheck->setChecked(m_config.videoLossless);
    connect(m_videoLosslessCheck, &QCheckBox::toggled, this, [this](bool checked) {
        m_config.videoLossless = checked;
        saveSettings();
    });
    compLayout->addWidget(m_videoLosslessCheck);

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
        lang == "tr" ? "MP3'e Dönüştür" : "Convert to MP3",
        lang == "tr" ? "En yaygın evrensel ses formatı olan MP3'e dönüştürür." : "Converts to universal standard MP3 audio.",
        "konvert-audio",
        actionsWidget
    ));
    actionsLayout->addWidget(createActionCheckBox(
        "to_wav",
        lang == "tr" ? "WAV'a Dönüştür" : "Convert to WAV",
        lang == "tr" ? "Stüdyo kalitesinde, tam kayıpsız ve sıkıştırmasız PCM WAV ses dosyası üretir." : "Converts to uncompressed studio-quality PCM WAV audio.",
        "konvert-audio",
        actionsWidget
    ));
    actionsLayout->addWidget(createActionCheckBox(
        "audio_to_video",
        lang == "tr" ? "Videoya Dönüştür" : "Convert to Video",
        lang == "tr" ? "Ses dosyası için arka planda KDE mavisi ses dalga formu üreterek paylaşılabilir bir MP4 videosu oluşturur." : "Generates an aesthetic KDE-blue waveform visualization video in MP4 format from audio.",
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
    bitrateLabel->setToolTip(lang == "tr" ? "MP3 ve AAC kodlama için hedef ses bit hızı. 320k şeffaf stüdyo kalitesi sunar." : "Target audio bitrate for MP3 and AAC encoding. 320k provides transparent CD quality.");
    bitrateRow->addWidget(bitrateLabel);

    m_audioBitrateCombo = new QComboBox(audioWidget);
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
        lang == "tr" ? "PDF Sıkıştır" : "Compress PDF",
        lang == "tr" ? "PDF belgesinin içerisindeki yüksek çözünürlüklü görselleri ve veri akışlarını sıkıştırarak dosya boyutunu düşürür." : "Compresses high-resolution embedded images and streams in PDF files to minimize file size.",
        "konvert-compress",
        actionsWidget
    ));
    actionsLayout->addWidget(createActionCheckBox(
        "udf_to_pdf",
        lang == "tr" ? "UDF'i PDF'e Dönüştür" : "Convert UDF to PDF",
        lang == "tr" ? "Ulusal Yargı Ağı Bilişim Sistemi (.udf) belgelerini doğrudan görüntülenebilir PDF belgesine dönüştürür." : "Converts Turkish National Judicial Network (.udf) legal documents directly into standard PDF.",
        "konvert-document",
        actionsWidget
    ));
    actionsLayout->addWidget(createActionCheckBox(
        "udf_to_txt",
        lang == "tr" ? "UDF'i Metne Dönüştür" : "Convert UDF to Text",
        lang == "tr" ? "UYAP .udf belgesinin içindeki saf metin içeriğini ayrıştırarak .txt dosyası olarak kaydeder." : "Extracts clean text content from .udf documents into a standard text file.",
        "konvert-document",
        actionsWidget
    ));
    actionsLayout->addWidget(createActionCheckBox(
        "pdf_to_images",
        lang == "tr" ? "PDF Sayfalarını PNG'ye Çıkar" : "Extract PDF Pages to PNG",
        lang == "tr" ? "PDF belgesinin her bir sayfasını yüksek çözünürlüklü PNG resimleri olarak ayrı ayrı klasöre aktarır." : "Renders all pages of a PDF document into high-resolution PNG image files.",
        "konvert-image",
        actionsWidget
    ));

    layout->addWidget(createCard(trKey("card_actions", lang), actionsWidget));

    // Card 2: Document Settings
    auto *docWidget = new QWidget(container);
    auto *docLayout = new QVBoxLayout(docWidget);
    docLayout->setContentsMargins(0, 0, 0, 0);

    auto *dpiRow = new QHBoxLayout();
    auto *dpiLabel = new QLabel(trKey("lbl_pdf_dpi", lang), docWidget);
    dpiLabel->setToolTip(lang == "tr" ? "PDF sayfaları görsele aktarılırken kullanılacak DPI çözünürlüğü. 300 DPI standart baskı kalitesidir." : "DPI resolution when rendering PDF pages to images. 300 DPI is standard print quality.");
    dpiRow->addWidget(dpiLabel);

    m_pdfDpiSpin = new QSpinBox(docWidget);
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
    m_totalOperationsLabel = new QLabel(lang == "tr" ? "<b>Toplam İşlem:</b> 0" : "<b>Total Conversions:</b> 0", summaryWidget);
    m_totalSavingsLabel = new QLabel(lang == "tr" ? "<b>Toplam Tasarruf:</b> 0 MB" : "<b>Total Saved:</b> 0 MB", summaryWidget);
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

    layout->addWidget(createCard(lang == "tr" ? "Dönüştürme İstatistikleri" : "Conversion Statistics", summaryWidget));

    // History Table
    m_historyTable = new QTableWidget(container);
    m_historyTable->setColumnCount(7);
    if (lang == "tr") {
        m_historyTable->setHorizontalHeaderLabels({"Tarih / Saat", "İşlem", "Dosya", "Orijinal", "Çıktı", "Tasarruf", "Durum"});
    } else {
        m_historyTable->setHorizontalHeaderLabels({"Date / Time", "Action", "Files", "Original", "Output", "Savings", "Status"});
    }
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
    m_previewTitleLabel = new QLabel(lang == "tr" ? "Önizleme için tablodan bir satır seçin." : "Select a row from the table to preview file.", m_previewPanel);
    m_previewTitleLabel->setWordWrap(true);
    infoLayout->addWidget(m_previewTitleLabel);

    m_previewPathLabel = new QLabel("", m_previewPanel);
    m_previewPathLabel->setStyleSheet("color: palette(mid); font-size: 11px;");
    m_previewPathLabel->setWordWrap(true);
    infoLayout->addWidget(m_previewPathLabel);

    auto *btnRow = new QHBoxLayout();
    btnRow->setSpacing(8);

    m_previewOpenBtn = new QPushButton(lang == "tr" ? "Dosyayı Aç" : "Open File", m_previewPanel);
    m_previewOpenBtn->setIcon(QIcon::fromTheme("document-open-symbolic"));
    m_previewOpenBtn->setEnabled(false);
    connect(m_previewOpenBtn, &QPushButton::clicked, this, [this]() {
        if (!m_selectedOutputPath.isEmpty() && QFile::exists(m_selectedOutputPath)) {
            QDesktopServices::openUrl(QUrl::fromLocalFile(m_selectedOutputPath));
        }
    });
    btnRow->addWidget(m_previewOpenBtn);

    m_previewDolphinBtn = new QPushButton(lang == "tr" ? "Dolphin'de Göster" : "Show in File Manager", m_previewPanel);
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

    layout->addWidget(createCard(lang == "tr" ? "Dosya Önizleme ve Konum" : "File Preview & Location", m_previewPanel));

    connect(m_historyTable, &QTableWidget::itemSelectionChanged, this, &MainWindow::onHistorySelectionChanged);

    loadHistoryData();

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
        if (m_totalOperationsLabel) m_totalOperationsLabel->setText(lang == "tr" ? "<b>Toplam İşlem:</b> 0" : "<b>Total Conversions:</b> 0");
        if (m_totalSavingsLabel) m_totalSavingsLabel->setText(lang == "tr" ? "<b>Toplam Tasarruf:</b> 0 MB" : "<b>Total Saved:</b> 0 MB");
        return;
    }

    QByteArray data = file.readAll();
    file.close();

    QJsonDocument doc = QJsonDocument::fromJson(data);
    if (!doc.isArray()) return;

    QJsonArray array = doc.array();
    m_historyTable->setRowCount(array.size());

    qint64 totalSavedBytes = 0;
    int totalFiles = 0;

    auto formatBytes = [](qint64 bytes) -> QString {
        const double KB = 1024.0;
        const double MB = KB * 1024.0;
        const double GB = MB * 1024.0;
        double b = static_cast<double>(bytes);
        if (b >= GB) return QString::number(b / GB, 'f', 1) + " GB";
        if (b >= MB) return QString::number(b / MB, 'f', 1) + " MB";
        if (b >= KB) return QString::number(b / KB, 'f', 1) + " KB";
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

        totalFiles += fileCount;
        // Büyüyen dönüşümler toplamdan düşülür (imzalı delta).
        totalSavedBytes += (origSize - outSize);

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
        const QString normalizedStatus = status.trimmed().toLower();
        if (normalizedStatus == "success" || normalizedStatus == "completed"
            || normalizedStatus == "tamamlandı" || normalizedStatus == "başarılı") {
            statusItem->setForeground(QColor(46, 170, 70));
        } else if (normalizedStatus == "partial" || normalizedStatus == "kısmi") {
            statusItem->setForeground(QColor(210, 150, 40));
        } else {
            statusItem->setForeground(QColor(220, 80, 70));
        }
        m_historyTable->setItem(i, 6, statusItem);
    }

    if (m_totalOperationsLabel) {
        if (lang == "tr") {
            m_totalOperationsLabel->setText(QString("<b>Toplam İşlem:</b> %1 (%2 dosya)").arg(array.size()).arg(totalFiles));
        } else {
            m_totalOperationsLabel->setText(QString("<b>Total Conversions:</b> %1 (%2 files)").arg(array.size()).arg(totalFiles));
        }
    }
    if (m_totalSavingsLabel) {
        if (lang == "tr") {
            m_totalSavingsLabel->setText(QString("<b>Toplam Tasarruf:</b> %1").arg(formatBytes(totalSavedBytes)));
        } else {
            m_totalSavingsLabel->setText(QString("<b>Total Saved:</b> %1").arg(formatBytes(totalSavedBytes)));
        }
    }

    onHistorySelectionChanged();
}

void MainWindow::onHistorySelectionChanged() {
    if (!m_historyTable) return;
    const QString lang = currentLanguage();
    int row = m_historyTable->currentRow();
    if (row < 0 || row >= m_historyTable->rowCount()) {
        m_selectedOutputPath.clear();
        if (m_previewTitleLabel) m_previewTitleLabel->setText(lang == "tr" ? "Önizleme için tablodan bir satır seçin." : "Select a row from the table to preview file.");
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
                    ? (lang == "tr" ? "Bu işlem için dosya yolu kaydedilmemiş." : "No file path recorded for this operation.")
                    : (lang == "tr" ? "Dosya diskte bulunamadı (taşınmış veya silinmiş olabilir)." : "File not found on disk (may have been moved or deleted).")
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

    if (m_previewTitleLabel) {
        m_previewTitleLabel->setText(QString("<b>%1</b> (%2)").arg(fi.fileName()).arg(m_historyTable->item(row, 4) ? m_historyTable->item(row, 4)->text() : ""));
    }
    if (m_previewPathLabel) {
        m_previewPathLabel->setText(m_selectedOutputPath);
    }

    // Thumbnail Preview
    if (ext == "png" || ext == "jpg" || ext == "jpeg" || ext == "webp" || ext == "avif" || ext == "bmp" || ext == "svg") {
        // Tam boy görseli UI thread'inde çözmek yerine hedef boyutta oku.
        QImageReader reader(m_selectedOutputPath);
        reader.setAutoTransform(true);
        const QSize originalSize = reader.size();
        if (originalSize.isValid()) {
            reader.setScaledSize(originalSize.scaled(QSize(72, 72), Qt::KeepAspectRatio));
        }
        const QImage thumb = reader.read();
        if (!thumb.isNull()) {
            m_previewThumbLabel->setPixmap(QPixmap::fromImage(thumb));
        } else {
            QFileIconProvider prov;
            m_previewThumbLabel->setPixmap(prov.icon(fi).pixmap(52, 52));
        }
    } else {
        QFileIconProvider prov;
        m_previewThumbLabel->setPixmap(prov.icon(fi).pixmap(52, 52));
    }

    if (m_previewOpenBtn) m_previewOpenBtn->setEnabled(true);
    if (m_previewDolphinBtn) m_previewDolphinBtn->setEnabled(true);
}

void MainWindow::clearHistoryData() {
    const QString lang = currentLanguage();
    const auto answer = QMessageBox::question(
        this,
        lang == "tr" ? "Geçmişi temizle" : "Clear history",
        lang == "tr" ? "Tüm dönüşüm geçmişi silinecek. Emin misiniz?"
                     : "All conversion history will be deleted. Continue?",
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
            lang == "tr" ? "Hata" : "Error",
            lang == "tr" ? "Geçmiş dosyası silinemedi." : "Could not delete the history file.");
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
    QPixmap logoPix(":/logo.svg");
    if (!logoPix.isNull()) {
        iconLabel->setPixmap(logoPix.scaled(72, 72, Qt::KeepAspectRatio, Qt::SmoothTransformation));
    }
    iconLabel->setAlignment(Qt::AlignCenter);
    headerLayout->addWidget(iconLabel);

    auto *titleLabel = new QLabel(
        QString(
            "<div style='text-align: center;'>"
            "<span style='font-size: 22px; font-weight: bold;'>Konvert</span> "
            "<span style='font-size: 13px; opacity: 0.7;'>%1</span><br>"
            "<span style='font-size: 12px;'>%2</span></div>"
        ).arg(KONVERT_APP_VERSION, lang == "tr" ? "Linux ve KDE Plasma için yerel dosya dönüştürücü" : "Native file converter for Linux and KDE Plasma"),
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

    auto *lblDesc = new QLabel(
        lang == "tr"
            ? "Konvert; Linux masaüstü ortamları ve KDE Plasma için sıfır telemetri ve tam gizlilik esasıyla geliştirilmiş yerel bir dosya işleme aracıdır. Dosyalarınız cihazınızdan asla ayrılmaz; tüm dönüştürme ve optimizasyon işlemleri yerel donanım kaynaklarıyla gerçekleştirilir."
            : "Konvert is a fast, offline, and native file conversion utility designed for Linux and KDE Plasma. All processing runs entirely on your local machine with strict zero-telemetry privacy.",
        infoWidget
    );
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

    addSpecRow(0, lang == "tr" ? "Sürüm:" : "Version:", QString("%1 (%2-linux)").arg(KONVERT_APP_VERSION, archStr));
    addSpecRow(1, lang == "tr" ? "Masaüstü Ortamı:" : "Desktop Environment:", desktopEnv);
    addSpecRow(2, lang == "tr" ? "Grafik Arayüzü:" : "GUI Framework:", lang == "tr" ? "Qt 6 (KDE Breeze Uyumlu)" : "Qt 6 (KDE Breeze Compliant)");
    addSpecRow(3, lang == "tr" ? "Çekirdek Motor:" : "Core Engine:", lang == "tr" ? "Rust 2024 & Rayon Çok Çekirdekli Motor" : "Rust 2024 & Rayon Multi-Core Engine");
    addSpecRow(4, lang == "tr" ? "Web Sitesi:" : "Homepage:", "<a href='https://github.com/enesehs/konvert'>github.com/enesehs/konvert</a>");
    addSpecRow(5, lang == "tr" ? "Hata Bildirimi:" : "Bug Tracker:", "<a href='https://github.com/enesehs/konvert/issues'>github.com/enesehs/konvert/issues</a>");

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
    auto *lblAuthTitle = new QLabel(QString("<b>%1</b>").arg(lang == "tr" ? "Geliştirici" : "Author"), authorBox);
    auto *lblAuthDetail = new QLabel(
        QString(
            "<b>enesehs</b> &lt;<a href='mailto:enesehs@protonmail.com'>enesehs@protonmail.com</a>&gt;<br>"
            "<span style='font-size: 12px; opacity: 0.8;'>%1</span>"
        ).arg(lang == "tr" ? "Çekirdek mimari, Rust dönüştürme motoru ve Qt6 arayüz tasarımı." : "Core architecture, Rust conversion engine, and Qt6 GUI design."),
        authorBox
    );
    lblAuthDetail->setOpenExternalLinks(true);
    authorBoxLayout->addWidget(lblAuthTitle);
    authorBoxLayout->addWidget(lblAuthDetail);
    authorsLayout->addWidget(authorBox);

    auto *commBox = new QFrame(authorsWidget);
    commBox->setFrameShape(QFrame::StyledPanel);
    commBox->setStyleSheet("QFrame { border: 1px solid palette(midlight); border-radius: 6px; background-color: palette(base); padding: 10px; }");
    auto *commBoxLayout = new QVBoxLayout(commBox);
    commBoxLayout->setSpacing(4);
    auto *lblCommTitle = new QLabel(QString("<b>%1</b>").arg(lang == "tr" ? "Açık Kaynak Topluluğu" : "Open Source Community"), commBox);
    auto *lblCommDetail = new QLabel(
        lang == "tr"
            ? "Bu proje özgür yazılım dünyasının dayanışma ruhu sayesinde hayata geçirilmiştir. KDE Topluluğu'na, Linux çekirdeğine, Rust ve Qt açık kaynak ekosistemine emek veren tüm geliştiricilere teşekkür ederiz."
            : "Konvert is built upon the collective spirit of the free software ecosystem. Heartfelt thanks to the KDE Community, Linux kernel contributors, and the Rust and Qt open source projects.",
        commBox
    );
    lblCommDetail->setWordWrap(true);
    commBoxLayout->addWidget(lblCommTitle);
    commBoxLayout->addWidget(lblCommDetail);
    authorsLayout->addWidget(commBox);

    authorsLayout->addStretch();
    aboutTabs->addTab(authorsWidget, trKey("about_tab_community", lang));

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

    addCompRow(0, "Rust & Rayon", lang == "tr" ? "Paralel çok çekirdekli sistem motoru ve güvenli bellek yönetimi." : "Multi-threaded parallel systems engine and memory safety.");
    addCompRow(1, "Qt 6 Widgets", lang == "tr" ? "KDE Breeze ile kusursuz uyumlu yerel grafik kullanıcı arayüzü." : "Native KDE Breeze compliant graphical user interface.");
    addCompRow(2, "FFmpeg", lang == "tr" ? "Gelişmiş video ve ses kodlama altyapısı, dalga formu üretimi." : "High performance video & audio transcoding framework.");
    addCompRow(3, "libjxl (JPEG XL)", lang == "tr" ? "Yeni nesil yüksek verimli görsel sıkıştırma ve kod çözücü." : "Next-gen high efficiency image coding and compression.");
    addCompRow(4, "Poppler & Ghostscript", lang == "tr" ? "PDF sayfa ayrıştırma, metin çıkarma ve belge sıkıştırma." : "PDF page rasterization, text extraction, and document compression.");
    addCompRow(5, "ImageMagick", lang == "tr" ? "Görsel format dönüşümleri, PDF birleştirme ve metadata temizleme." : "Image manipulation, PDF merging, and metadata stripping.");

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
    licBrowser->setStyleSheet("QTextBrowser { border: 1px solid palette(midlight); border-radius: 6px; padding: 10px; background-color: palette(base); font-size: 11px; }");
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

void MainWindow::updateProcessMonitor() {
    if (!m_processStatusLabel || !m_cancelProcessBtn) return;
    const QString lang = currentLanguage();
    qint64 pid = getActiveKonvertPid();
    if (pid > 0) {
        m_processStatusLabel->setText(
            QString("<span style='color: #27ae60; font-weight: bold;'>●</span> %1")
                .arg(trKey("lbl_process_running", lang).arg(pid))
        );
        m_cancelProcessBtn->setEnabled(true);
    } else {
        m_processStatusLabel->setText(
            QString("<span style='color: #7f8c8d;'>○</span> %1")
                .arg(trKey("lbl_process_idle", lang))
        );
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
    m_langCombo->addItem(lang == "tr" ? "Otomatik Algıla (Sistem Dili)" : "Auto Detect (System)", "auto");
    m_langCombo->addItem("Türkçe", "tr");
    m_langCombo->addItem("English", "en");
    m_langCombo->addItem("Deutsch", "de");
    m_langCombo->addItem("Français", "fr");
    m_langCombo->addItem("Español", "es");

    int curIdx = 0;
    for (int i = 0; i < m_langCombo->count(); ++i) {
        if (m_langCombo->itemData(i).toString() == m_config.language) {
            curIdx = i;
            break;
        }
    }
    m_langCombo->setCurrentIndex(curIdx);
    connect(m_langCombo, QOverload<int>::of(&QComboBox::currentIndexChanged), this, &MainWindow::onLanguageChanged);

    langRow->addWidget(m_langCombo);
    langRow->addStretch();
    genLayout->addLayout(langRow);

    // Notification checkbox
    m_notifyCheck = new QCheckBox(trKey("lbl_notifications", lang), genWidget);
    m_notifyCheck->setToolTip(lang == "tr" ? "Dönüştürme esnasında yerel ilerleme çubuğu ve D-Bus bildirimlerini gösterir." : "Displays real-time D-Bus notification with progress bar during conversion.");
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
    m_outputDirEdit->setReadOnly(true);
    m_outputDirEdit->setPlaceholderText(trKey("lbl_output_dir_default", lang));
    m_outputDirEdit->setText(m_config.outputDirectory);
    m_outputDirEdit->setToolTip(lang == "tr" ? "Dönüştürülen tüm dosyaların kaydedileceği varsayılan klasör. Boş bırakıldığında girdi dosyasıyla aynı klasöre kaydedilir."
                                            : "Default directory where converted files will be saved. If left empty, files are saved in the same folder as the input file.");
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
    m_processStatusLabel->setStyleSheet("font-size: 13px;");
    statusRow->addWidget(m_processStatusLabel);
    statusRow->addStretch();

    m_cancelProcessBtn = new QPushButton(QIcon::fromTheme("process-stop-symbolic"), trKey("btn_cancel_process", lang), monitorWidget);
    m_cancelProcessBtn->setEnabled(false);
    connect(m_cancelProcessBtn, &QPushButton::clicked, this, [this]() {
        const QString lang = currentLanguage();
        QProcess proc;
        proc.start(findKonvertBinary(), {"--cancel"});
        if (proc.waitForFinished(3000)) {
            QString out = QString::fromUtf8(proc.readAllStandardOutput()).trimmed();
            if (out.isEmpty()) {
                out = QString::fromUtf8(proc.readAllStandardError()).trimmed();
            }
            if (!out.isEmpty()) {
                QMessageBox::information(this, trKey("btn_cancel_process", lang), out);
            }
        }
        updateProcessMonitor();
        loadHistoryData();
    });
    statusRow->addWidget(m_cancelProcessBtn);
    monitorLayout->addLayout(statusRow);
    layout->addWidget(createCard(trKey("card_process_monitor", lang), monitorWidget));

    // --- Card 3: Desktop Integration & System Dependencies (P3-9) ---
    auto *deskWidget = new QWidget(container);
    auto *deskLayout = new QVBoxLayout(deskWidget);
    deskLayout->setContentsMargins(4, 4, 4, 4);
    deskLayout->setSpacing(10);

    auto *deskDesc = new QLabel(
        lang == "tr" ? "KDE Dolphin, GNOME Nautilus, Cinnamon Nemo ve XFCE Thunar dosya yöneticisi entegrasyonlarını kurabilir veya kaldırabilirsiniz."
                     : "Install or remove context menu integrations for KDE Dolphin, GNOME Nautilus, Cinnamon Nemo, and XFCE Thunar.",
        deskWidget
    );
    deskDesc->setWordWrap(true);
    deskLayout->addWidget(deskDesc);

    auto *btnRow = new QHBoxLayout();
    auto *installBtn = new QPushButton(QIcon::fromTheme("system-software-install-symbolic"), trKey("btn_install_integration", lang), deskWidget);
    connect(installBtn, &QPushButton::clicked, this, [this]() {
        const QString lang = currentLanguage();
        QProcess proc;
        proc.start(findKonvertBinary(), {"--install-all"});
        if (proc.waitForFinished(5000)) {
            QString out = QString::fromUtf8(proc.readAllStandardOutput()).trimmed();
            if (out.isEmpty()) {
                out = QString::fromUtf8(proc.readAllStandardError()).trimmed();
            }
            if (out.isEmpty()) {
                out = lang == "tr" ? "Entegrasyonlar başarıyla kuruldu." : "Integrations installed successfully.";
            }
            QMessageBox::information(this, trKey("btn_install_integration", lang), out);
        } else {
            QMessageBox::warning(this, trKey("btn_install_integration", lang),
                lang == "tr" ? "İşlem zaman aşımına uğradı veya başlatılamadı." : "Operation timed out or failed to start.");
        }
    });
    btnRow->addWidget(installBtn);

    auto *uninstallBtn = new QPushButton(QIcon::fromTheme("edit-delete-symbolic"), trKey("btn_uninstall_integration", lang), deskWidget);
    connect(uninstallBtn, &QPushButton::clicked, this, [this]() {
        const QString lang = currentLanguage();
        auto reply = QMessageBox::question(
            this,
            trKey("btn_uninstall_integration", lang),
            lang == "tr" ? "Konvert sağ tık menüsü eylemlerini tüm dosya yöneticilerinden kaldırmak istediğinize emin misiniz?"
                         : "Are you sure you want to remove Konvert context menu actions from all file managers?",
            QMessageBox::Yes | QMessageBox::No
        );
        if (reply == QMessageBox::Yes) {
            QProcess proc;
            proc.start(findKonvertBinary(), {"--uninstall-all"});
            if (proc.waitForFinished(5000)) {
                QString out = QString::fromUtf8(proc.readAllStandardOutput()).trimmed();
                if (out.isEmpty()) {
                    out = QString::fromUtf8(proc.readAllStandardError()).trimmed();
                }
                if (out.isEmpty()) {
                    out = lang == "tr" ? "Entegrasyonlar kaldırıldı." : "Integrations uninstalled.";
                }
                QMessageBox::information(this, trKey("btn_uninstall_integration", lang), out);
            }
        }
    });
    btnRow->addWidget(uninstallBtn);

    auto *checkDepsBtn = new QPushButton(QIcon::fromTheme("system-run-symbolic"), trKey("btn_check_deps", lang), deskWidget);
    connect(checkDepsBtn, &QPushButton::clicked, this, [this]() {
        const QString lang = currentLanguage();
        QProcess proc;
        proc.start(findKonvertBinary(), {"--check-deps"});
        if (proc.waitForFinished(5000)) {
            QString out = QString::fromUtf8(proc.readAllStandardOutput()).trimmed();
            if (out.isEmpty()) {
                out = QString::fromUtf8(proc.readAllStandardError()).trimmed();
            }
            if (m_depsStatusLabel) {
                m_depsStatusLabel->setText(QString("<pre style='margin:0; font-size:11px;'>%1</pre>").arg(out.toHtmlEscaped()));
                m_depsStatusLabel->setVisible(true);
            }
        }
    });
    btnRow->addWidget(checkDepsBtn);
    btnRow->addStretch();
    deskLayout->addLayout(btnRow);

    m_depsStatusLabel = new QLabel(deskWidget);
    m_depsStatusLabel->setTextFormat(Qt::RichText);
    m_depsStatusLabel->setStyleSheet("background: palette(base); border: 1px solid palette(midlight); border-radius: 4px; padding: 8px;");
    m_depsStatusLabel->setVisible(false);
    deskLayout->addWidget(m_depsStatusLabel);

    layout->addWidget(createCard(trKey("card_desktop_integration", lang), deskWidget));

    layout->addStretch();
    scroll->setWidget(container);

    updateProcessMonitor();
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
        m_config.language = langCode;
        saveSettings();
        m_actionCheckBoxes.clear();
        setupUi();
    }
}

void MainWindow::saveSettings() {
    if (m_saveTimer) {
        m_saveTimer->start();
    } else {
        saveSettingsNow();
    }
}

void MainWindow::saveSettingsNow() {
    m_isSavingConfig = true;
    if (!m_config.save()) {
        const QString lang = currentLanguage();
        QMessageBox::warning(
            this,
            lang == "tr" ? "Ayarlar kaydedilemedi" : "Could not save settings",
            lang == "tr" ? "Ayar dosyası yazılamadı. Disk izinlerini kontrol edin."
                         : "The settings file could not be written. Check file permissions.");
    }
    if (m_configWatcher) {
        m_configWatcher->addPath(ConfigManager::getConfigPath());
    }
    QTimer::singleShot(250, this, [this]() {
        m_isSavingConfig = false;
    });
}

void MainWindow::setupConfigWatcher() {
    QString configPath = ConfigManager::getConfigPath();
    if (!QFile::exists(configPath)) {
        return;
    }
    if (!m_configWatcher) {
        m_configWatcher = new QFileSystemWatcher(this);
    }
    m_configWatcher->addPath(configPath);
    connect(m_configWatcher, &QFileSystemWatcher::fileChanged, this, [this](const QString &path) {
        if (m_configWatcher && QFile::exists(path)) {
            m_configWatcher->addPath(path);
        }
        if (m_isSavingConfig) {
            return;
        }
        reloadConfigToUi();
    });
}

void MainWindow::reloadConfigToUi() {
    m_config.load();
    if (m_photoQualitySlider) m_photoQualitySlider->setValue(m_config.imageQuality);
    if (m_photoLosslessCheck) m_photoLosslessCheck->setChecked(m_config.imageLossless);
    if (m_videoCrfSlider) m_videoCrfSlider->setValue(m_config.videoCrf);
    if (m_videoPresetCombo) {
        int idx = m_videoPresetCombo->findText(m_config.videoPreset);
        if (idx >= 0) m_videoPresetCombo->setCurrentIndex(idx);
    }
    if (m_videoLosslessCheck) m_videoLosslessCheck->setChecked(m_config.videoLossless);
    if (m_audioBitrateCombo) {
        int idx = m_audioBitrateCombo->findText(m_config.audioBitrate);
        if (idx >= 0) m_audioBitrateCombo->setCurrentIndex(idx);
    }
    if (m_pdfDpiSpin) m_pdfDpiSpin->setValue(m_config.pdfDpi);
    if (m_notifyCheck) m_notifyCheck->setChecked(m_config.notificationEnabled);
    if (m_outputDirEdit) m_outputDirEdit->setText(m_config.outputDirectory);

    for (auto *cb : m_actionCheckBoxes) {
        if (cb) {
            QString id = cb->property("actionId").toString();
            cb->blockSignals(true);
            cb->setChecked(m_config.enabledActions.contains(id));
            cb->blockSignals(false);
        }
    }
}

void MainWindow::closeEvent(QCloseEvent *event) {
    if (m_saveTimer && m_saveTimer->isActive()) {
        m_saveTimer->stop();
        saveSettingsNow();
    }
    QMainWindow::closeEvent(event);
}
