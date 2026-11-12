#include <QApplication>
#include <QDir>
#include <QLockFile>
#include <QMessageBox>
#include <QStandardPaths>
#include "mainwindow.h"

int main(int argc, char *argv[]) {
    QApplication app(argc, argv);
    app.setApplicationName("Konvert");
    app.setApplicationDisplayName("Konvert");
    app.setOrganizationName("Konvert");

    // Tek örnek: iki GUI aynı config.toml'u ezmesin.
    const QString lockDir = QStandardPaths::writableLocation(QStandardPaths::RuntimeLocation);
    QLockFile lock(QDir(lockDir).filePath("konvert-settings.lock"));
    lock.setStaleLockTime(0);
    if (!lock.tryLock(100)) {
        QMessageBox::information(
            nullptr,
            "Konvert",
            "Konvert settings is already running.");
        return 0;
    }

    MainWindow window;

#ifdef KONVERT_DEBUG
    for (int i = 1; i < argc; ++i) {
        if (QString(argv[i]) == "--screenshot-all" && i + 1 < argc) {
            window.captureScreenshots(argv[i + 1]);
            return 0;
        }
    }
#endif

    window.show();

    return app.exec();
}
