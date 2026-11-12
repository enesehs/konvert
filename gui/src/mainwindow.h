#pragma once

#include <QMainWindow>
#include <QTabWidget>
#include <QListWidget>
#include <QSlider>
#include <QCheckBox>
#include <QComboBox>
#include <QSpinBox>
#include <QLabel>
#include <QPushButton>
#include <QLineEdit>
#include <QTableWidget>
#include <QHeaderView>
#include <QFileSystemWatcher>
#include "configmanager.h"

#include <QTimer>

class MainWindow : public QMainWindow {
    Q_OBJECT

public:
    explicit MainWindow(QWidget *parent = nullptr);
    ~MainWindow() override = default;
    void captureScreenshots(const QString &outputDir);

private slots:
    void onActionCheckChanged();
    void onLanguageChanged(int index);
    void onHistorySelectionChanged();
    void saveSettings();
    void saveSettingsNow();

protected:
    void closeEvent(QCloseEvent *event) override;

private:
    void setupUi();
    QString currentLanguage() const;
    QWidget* createConvertTab();
    QWidget* createPhotosTab();
    QWidget* createVideosTab();
    QWidget* createMusicTab();
    QWidget* createDocumentsTab();
    QWidget* createHistoryTab();
    QWidget* createSettingsTab();
    QWidget* createAboutTab();

    void loadHistoryData();
    void clearHistoryData();
    void updateProcessMonitor();
    void setupConfigWatcher();
    void reloadConfigToUi();

    QWidget* createCard(const QString &title, QWidget *contentWidget);
    QCheckBox* createActionCheckBox(const QString &id, const QString &label, const QString &tooltip, const QString &iconName, QWidget *parent);

    ConfigManager m_config;
    QList<QCheckBox*> m_actionCheckBoxes;

    // Photos
    QSlider *m_photoQualitySlider = nullptr;
    QLabel *m_photoQualityLabel = nullptr;
    QCheckBox *m_photoLosslessCheck = nullptr;

    // Videos
    QSlider *m_videoCrfSlider = nullptr;
    QLabel *m_videoCrfLabel = nullptr;
    QComboBox *m_videoPresetCombo = nullptr;
    QCheckBox *m_videoLosslessCheck = nullptr;

    // Music
    QComboBox *m_audioBitrateCombo = nullptr;

    // Documents
    QSpinBox *m_pdfDpiSpin = nullptr;

    // History
    QTableWidget *m_historyTable = nullptr;
    QLabel *m_totalOperationsLabel = nullptr;
    QLabel *m_totalSavingsLabel = nullptr;
    QWidget *m_previewPanel = nullptr;
    QLabel *m_previewThumbLabel = nullptr;
    QLabel *m_previewTitleLabel = nullptr;
    QLabel *m_previewPathLabel = nullptr;
    QPushButton *m_previewOpenBtn = nullptr;
    QPushButton *m_previewDolphinBtn = nullptr;
    QString m_selectedOutputPath;

    // General & Settings
    QCheckBox *m_notifyCheck = nullptr;
    QComboBox *m_langCombo = nullptr;
    QLineEdit *m_outputDirEdit = nullptr;
    QLabel *m_processStatusLabel = nullptr;
    QPushButton *m_cancelProcessBtn = nullptr;
    QLabel *m_depsStatusLabel = nullptr;
    QTimer *m_processMonitorTimer = nullptr;
    QTabWidget *m_rootTabWidget = nullptr;
    QTimer *m_saveTimer = nullptr;
    QFileSystemWatcher *m_configWatcher = nullptr;
    bool m_isSavingConfig = false;
};
