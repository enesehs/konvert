#include <QApplication>
#include <QDir>
#include <QGuiApplication>
#include <QLocalServer>
#include <QLocalSocket>
#include <QLockFile>
#include <QStandardPaths>
#include "mainwindow.h"

int main(int argc, char *argv[]) {
    QApplication app(argc, argv);
    app.setApplicationName("Konvert");
    app.setApplicationDisplayName("Konvert");
    app.setOrganizationName("Konvert");
    QGuiApplication::setDesktopFileName("io.github.enesehs.konvert");

    // Tek örnek: ikinci açılışta mevcut pencereyi öne getir.
    QString lockDir = QStandardPaths::writableLocation(QStandardPaths::RuntimeLocation);
    if (lockDir.isEmpty()) {
        lockDir = QStandardPaths::writableLocation(QStandardPaths::TempLocation);
    }
    QDir().mkpath(lockDir);

    const QString lockPath = QDir(lockDir).filePath("konvert-settings.lock");
    const QString socketName = QDir(lockDir).filePath("konvert-settings.sock");

    QLockFile lock(lockPath);
    lock.setStaleLockTime(0);
    if (!lock.tryLock(100)) {
        QLocalSocket socket;
        socket.connectToServer(socketName);
        if (socket.waitForConnected(500)) {
            socket.write("ACTIVATE\n");
            socket.flush();
            socket.waitForBytesWritten(500);
            socket.disconnectFromServer();
        }
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

    QLocalServer::removeServer(socketName);
    QLocalServer server;
    if (server.listen(socketName)) {
        QObject::connect(&server, &QLocalServer::newConnection, &window, [&server, &window]() {
            while (QLocalSocket *client = server.nextPendingConnection()) {
                client->readAll();
                client->close();
                client->deleteLater();
            }
            window.setWindowState((window.windowState() & ~Qt::WindowMinimized) | Qt::WindowActive);
            window.show();
            window.raise();
            window.activateWindow();
        });
    }

    window.show();

    return app.exec();
}
