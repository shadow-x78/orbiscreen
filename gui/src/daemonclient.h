// Orbiscreen - Qt GUI daemon client (GPL-3.0-or-later) https://github.com/shadow-x78/orbiscreen

#pragma once

#include <QByteArray>
#include <QImage>
#include <QObject>
#include <QString>
#include <QStringList>
#include <functional>

#include "report.h"

class QDBusInterface;
class QTimer;

class DaemonClient : public QObject {
    Q_OBJECT

public:
    explicit DaemonClient(QObject *parent = nullptr);

    void refreshStatus();
    void startService();
    void stopService();
    void restartService();
    void setResolution(int width, int height, int fps);
    void runDoctor();
    void runDoctorFix();
    void renderQr(const QString &text);

    static QString token();
    static QString sessionUrl(const DaemonStatus &status, const QString &token);
    static QString usbUrl(const DaemonStatus &status, const QString &token);
    static bool isAutostartEnabled();
    static bool setAutostart(bool enabled, QString *error = nullptr);
    static bool openUrl(const QString &url, QString *error = nullptr);

signals:
    void statusReady(const DaemonStatus &status);
    void statusFailed(const QString &error);
    void actionFinished(const QString &message);
    void actionFailed(const QString &error);
    void doctorReady(const DoctorReport &report);
    void doctorFailed(const QString &error);
    void qrReady(const QImage &image);

private:
    void fallbackStatus();
    void stopImpl(const std::function<void(bool, const QString &)> &done);
    void runProcess(
        const QString &program, const QStringList &arguments,
        const std::function<void(bool, const QByteArray &, const QString &)> &done);
    static QString configDir();
    static QStringList detectLocalIps();

    QDBusInterface *m_interface = nullptr;
    QTimer *m_pollTimer = nullptr;
    quint64 m_statusSequence = 0;
};
