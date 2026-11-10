#include <QApplication>
#include "mainwindow.h"

int main(int argc, char *argv[]) {
    QApplication app(argc, argv);
    app.setApplicationName("Konvert");
    app.setApplicationDisplayName("Konvert");
    app.setOrganizationName("Konvert");

    MainWindow window;

    for (int i = 1; i < argc; ++i) {
        if (QString(argv[i]) == "--screenshot-all" && i + 1 < argc) {
            window.captureScreenshots(argv[i + 1]);
            return 0;
        }
    }

    window.show();

    return app.exec();
}
