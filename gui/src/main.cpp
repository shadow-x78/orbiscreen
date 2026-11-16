// Orbiscreen - Qt GUI entry point (GPL-3.0-or-later) https://github.com/shadow-x78/orbiscreen

#include "i18n.h"
#include "mainwindow.h"
#include "theme.h"

#include <QApplication>

int main(int argc, char *argv[])
{
    QApplication app(argc, argv);
    QApplication::setApplicationVersion(QStringLiteral(ORBISCREEN_VERSION));
    Theme::apply();
    I18n::instance().load();

    auto *window = new MainWindow;
    window->setAttribute(Qt::WA_DeleteOnClose);
    window->show();
    return app.exec();
}
