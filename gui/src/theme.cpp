// Orbiscreen - Qt GUI theme helpers (GPL-3.0-or-later) https://github.com/shadow-x78/orbiscreen

#include "theme.h"

#include <QApplication>
#include <QGuiApplication>
#include <QIcon>
#include <QProcess>
#include <QStyle>
#include <QStyleFactory>
#include <QTimer>

#include <memory>

namespace {

bool themeAlreadyKnown()
{
    const QString name = QIcon::themeName();
    return !name.isEmpty() && name != QStringLiteral("hicolor");
}

QString unquote(QString name)
{
    if (name.startsWith(QLatin1Char('\'')) && name.endsWith(QLatin1Char('\'')) && name.size() > 2) {
        name = name.mid(1, name.size() - 2);
    }
    return name;
}

}

namespace Theme {

void apply()
{
    QApplication::setApplicationName(QStringLiteral("orbiscreen-gui"));
    QApplication::setApplicationDisplayName(QStringLiteral("Orbiscreen"));
    QApplication::setDesktopFileName(QStringLiteral("orbiscreen.gui"));
    QIcon::setFallbackThemeName(QStringLiteral("hicolor"));

    if (QApplication::style()->objectName().isEmpty()) {
        if (QStyle *fusion = QStyleFactory::create(QStringLiteral("Fusion"))) {
            QApplication::setStyle(fusion);
        }
    }

    QApplication::setWindowIcon(
        icon({QStringLiteral("orbiscreen-gui"), QStringLiteral("orbiscreen"),
              QStringLiteral("video-display")},
             QStyle::SP_ComputerIcon));
}

void resolveSystemIconTheme(std::function<void()> ready)
{
    if (themeAlreadyKnown()) {
        ready();
        return;
    }
    auto *process = new QProcess;
    auto *timer = new QTimer;
    timer->setSingleShot(true);
    auto done = std::make_shared<bool>(false);
    auto finish = [process, timer, done, ready]() {
        if (*done) {
            return;
        }
        *done = true;
        if (process->state() != QProcess::NotRunning) {
            process->kill();
        } else if (process->exitStatus() == QProcess::NormalExit && process->exitCode() == 0) {
            const QString name =
                unquote(QString::fromUtf8(process->readAllStandardOutput()).trimmed());
            if (!name.isEmpty()) {
                QIcon::setThemeName(name);
            }
        }
        process->deleteLater();
        timer->deleteLater();
        ready();
    };
    QObject::connect(process, QOverload<int, QProcess::ExitStatus>::of(&QProcess::finished),
                     process, [finish](int, QProcess::ExitStatus) { finish(); });
    QObject::connect(timer, &QTimer::timeout, process, finish);
    process->start(QStringLiteral("gsettings"),
                   {QStringLiteral("get"), QStringLiteral("org.gnome.desktop.interface"),
                    QStringLiteral("icon-theme")});
    timer->start(250);
}

QIcon icon(const QStringList &themeNames, QStyle::StandardPixmap fallback)
{
    for (const QString &name : themeNames) {
        const QIcon icon = QIcon::fromTheme(name);
        if (!icon.isNull()) {
            return icon;
        }
    }
    return QApplication::style()->standardIcon(fallback);
}

QString iconName(const QString &kind)
{
    if (kind == QStringLiteral("ok")) {
        return QStringLiteral("emblem-ok-symbolic");
    }
    if (kind == QStringLiteral("warn")) {
        return QStringLiteral("dialog-warning-symbolic");
    }
    return QStringLiteral("dialog-error-symbolic");
}

QIcon statusIcon(const QString &kind)
{
    if (kind == QStringLiteral("ok")) {
        return icon({iconName(kind), QStringLiteral("dialog-ok")},
                    QStyle::SP_DialogApplyButton);
    }
    if (kind == QStringLiteral("warn")) {
        return icon({iconName(kind)}, QStyle::SP_MessageBoxWarning);
    }
    return icon({iconName(kind)}, QStyle::SP_MessageBoxCritical);
}

}
