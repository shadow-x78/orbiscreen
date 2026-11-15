// Orbiscreen - Qt GUI daemon client (GPL-3.0-or-later) https://github.com/shadow-x78/orbiscreen

#include "daemonclient.h"

#include <QAbstractSocket>
#include <QDBusConnection>
#include <QDBusConnectionInterface>
#include <QDBusInterface>
#include <QDBusPendingCallWatcher>
#include <QDBusPendingReply>
#include <QDBusReply>
#include <QDesktopServices>
#include <QDir>
#include <QFile>
#include <QFileInfo>
#include <QHostAddress>
#include <QJsonDocument>
#include <QJsonObject>
#include <QNetworkInterface>
#include <QProcess>
#include <QRegularExpression>
#include <QSharedPointer>
#include <QStandardPaths>
#include <QUdpSocket>
#include <QTimer>
#include <QUrl>

namespace {

QString configRoot()
{
    return QStandardPaths::writableLocation(QStandardPaths::GenericConfigLocation);
}

QString autostartPath()
{
    return configRoot() + QStringLiteral("/autostart/orbiscreen.desktop");
}

const char *desktopEntry =
    "[Desktop Entry]\n"
    "Type=Application\n"
    "Name=Orbiscreen\n"
    "Exec=orbiscreen start --daemon\n"
    "Hidden=false\n"
    "NoDisplay=false\n"
    "X-GNOME-Autostart-enabled=true\n";

}

DaemonClient::DaemonClient(QObject *parent)
    : QObject(parent)
{
    m_interface = new QDBusInterface(QStringLiteral("com.orbiscreen.Daemon"),
                                     QStringLiteral("/com/orbiscreen/Daemon"),
                                     QStringLiteral("com.orbiscreen.Daemon"),
                                     QDBusConnection::sessionBus(), this);
    m_interface->setTimeout(500);

    m_pollTimer = new QTimer(this);
    m_pollTimer->setInterval(3000);
    connect(m_pollTimer, &QTimer::timeout, this, &DaemonClient::refreshStatus);
    m_pollTimer->start();
}

void DaemonClient::refreshStatus()
{
    if (!m_interface) {
        fallbackStatus();
        return;
    }

    const quint64 sequence = ++m_statusSequence;
    auto *watcher = new QDBusPendingCallWatcher(
        m_interface->asyncCall(QStringLiteral("GetStatus")), this);
    connect(watcher, &QDBusPendingCallWatcher::finished, this,
            [this, sequence](QDBusPendingCallWatcher *call) {
                if (sequence != m_statusSequence) {
                    call->deleteLater();
                    return;
                }
                QDBusPendingReply<QString> reply = *call;
                if (reply.isError()) {
                    fallbackStatus();
                    call->deleteLater();
                    return;
                }
                QJsonParseError error{};
                const QJsonDocument document =
                    QJsonDocument::fromJson(reply.value().toUtf8(), &error);
                if (error.error != QJsonParseError::NoError || !document.isObject()) {
                    emit statusFailed(error.errorString());
                    call->deleteLater();
                    return;
                }
                bool ok = false;
                const StatusReport report = StatusReport::fromJson(document.object(), &ok);
                if (!ok) {
                    emit statusFailed(QStringLiteral("invalid status report"));
                    call->deleteLater();
                    return;
                }
                DaemonStatus status;
                status.report = report;
                status.udpPort = static_cast<quint16>(report.signalingPort + 1);
                status.localIps = detectLocalIps();
                emit statusReady(status);
                call->deleteLater();
            });
}

void DaemonClient::fallbackStatus()
{
    StatusReport report;
    const QDBusConnection bus = QDBusConnection::sessionBus();
    const QDBusConnectionInterface *iface = bus.interface();
    const QDBusReply<bool> registered = iface
        ? iface->isServiceRegistered(QStringLiteral("com.orbiscreen.Daemon"))
        : QDBusReply<bool>();
    report.running = registered.isValid() && registered.value();

    QFile file(configDir() + QStringLiteral("/orbiscreen.toml"));
    if (file.open(QIODevice::ReadOnly)) {
        const QString text = QString::fromUtf8(file.readAll());
        const auto readInt = [&text](const char *key, int fallback) {
            const QRegularExpression expression(
                QStringLiteral("^\\s*%1\\s*=\\s*(\\d+)").arg(QString::fromLatin1(key)),
                QRegularExpression::MultilineOption);
            const QRegularExpressionMatch match = expression.match(text);
            return match.hasMatch() ? match.captured(1).toInt() : fallback;
        };
        report.displayWidth = readInt("width", 1920);
        report.displayHeight = readInt("height", 1080);
        report.displayFps = readInt("refresh_rate_hz", 60);
    }

    DaemonStatus status;
    status.report = report;
    status.udpPort = static_cast<quint16>(report.signalingPort + 1);
    status.localIps = detectLocalIps();
    emit statusReady(status);
}

void DaemonClient::startService()
{
    runProcess(QStringLiteral("systemctl"),
               {QStringLiteral("--user"), QStringLiteral("start"), QStringLiteral("orbiscreen")},
               [this](bool ok, const QByteArray &, const QString &error) {
                   if (ok) {
                       emit actionFinished(QStringLiteral("Started orbiscreen systemd service"));
                       refreshStatus();
                       return;
                   }
                   if (QProcess::startDetached(QStringLiteral("orbiscreen"),
                                               {QStringLiteral("start"),
                                                QStringLiteral("--daemon")})) {
                       emit actionFinished(QStringLiteral("Spawned background orbiscreen daemon"));
                       QTimer::singleShot(500, this, &DaemonClient::refreshStatus);
                   } else {
                       emit actionFailed(error.isEmpty()
                                             ? QStringLiteral("Failed to start daemon")
                                             : error);
                   }
               });
}

void DaemonClient::stopImpl(const std::function<void(bool, const QString &)> &done)
{
    const auto fallback = [this, done]() {
        runProcess(QStringLiteral("systemctl"),
                   {QStringLiteral("--user"), QStringLiteral("stop"),
                    QStringLiteral("orbiscreen")},
                   [this, done](bool ok, const QByteArray &, const QString &error) {
                       if (ok) {
                           done(true, QStringLiteral("Stopped orbiscreen systemd service"));
                           return;
                       }
                       runProcess(QStringLiteral("orbiscreen"), {QStringLiteral("stop")},
                                  [done](bool stopped, const QByteArray &,
                                         const QString &stopError) {
                                      done(stopped,
                                           stopped ? QStringLiteral("Stopped orbiscreen daemon")
                                                   : stopError);
                                  });
                       Q_UNUSED(error);
                   });
    };

    if (!m_interface || !m_interface->isValid()) {
        fallback();
        return;
    }

    auto *watcher =
        new QDBusPendingCallWatcher(m_interface->asyncCall(QStringLiteral("Stop")), this);
    connect(watcher, &QDBusPendingCallWatcher::finished, this,
            [fallback, done](QDBusPendingCallWatcher *call) {
                QDBusPendingReply<QString> reply = *call;
                if (reply.isError()) {
                    fallback();
                } else {
                    done(true, reply.value());
                }
                call->deleteLater();
            });
}

void DaemonClient::stopService()
{
    stopImpl([this](bool ok, const QString &message) {
        if (ok) {
            emit actionFinished(message);
            refreshStatus();
        } else {
            emit actionFailed(message);
        }
    });
}

void DaemonClient::restartService()
{
    runProcess(QStringLiteral("systemctl"),
               {QStringLiteral("--user"), QStringLiteral("restart"),
                QStringLiteral("orbiscreen")},
               [this](bool ok, const QByteArray &, const QString &) {
                   if (ok) {
                       emit actionFinished(QStringLiteral("Restarted orbiscreen systemd service"));
                       refreshStatus();
                       return;
                   }
                   stopImpl([this](bool stopped, const QString &error) {
                       if (!stopped) {
                           emit actionFailed(error);
                           return;
                       }
                       QTimer::singleShot(250, this, &DaemonClient::startService);
                   });
               });
}

void DaemonClient::setResolution(int width, int height, int fps)
{
    if (!m_interface) {
        emit actionFailed(QStringLiteral("D-Bus unavailable"));
        return;
    }
    auto *watcher = new QDBusPendingCallWatcher(
        m_interface->asyncCall(QStringLiteral("SetResolution"), static_cast<uint>(width),
                               static_cast<uint>(height), static_cast<uint>(fps)),
        this);
    connect(watcher, &QDBusPendingCallWatcher::finished, this,
            [this](QDBusPendingCallWatcher *call) {
                QDBusPendingReply<QString> reply = *call;
                if (reply.isError()) {
                    emit actionFailed(reply.error().message());
                } else {
                    emit actionFinished(reply.value());
                    refreshStatus();
                }
                call->deleteLater();
            });
}

void DaemonClient::runDoctor()
{
    runProcess(QStringLiteral("orbiscreen"),
               {QStringLiteral("doctor"), QStringLiteral("--json")},
               [this](bool ok, const QByteArray &output, const QString &error) {
                   if (!ok) {
                       emit doctorFailed(error.isEmpty()
                                             ? QStringLiteral("Doctor check failed")
                                             : error);
                       return;
                   }
                   QJsonParseError parseError{};
                   const QJsonDocument document =
                       QJsonDocument::fromJson(output, &parseError);
                   if (parseError.error != QJsonParseError::NoError
                       || !document.isObject()) {
                       emit doctorFailed(parseError.errorString());
                       return;
                   }
                   bool parsed = false;
                   const DoctorReport report =
                       DoctorReport::fromJson(document.object(), &parsed);
                   if (!parsed) {
                       emit doctorFailed(QStringLiteral("invalid doctor report"));
                       return;
                   }
                   emit doctorReady(report);
               });
}

void DaemonClient::runDoctorFix()
{
    runProcess(QStringLiteral("orbiscreen"),
               {QStringLiteral("doctor"), QStringLiteral("--fix"), QStringLiteral("--yes")},
               [this](bool ok, const QByteArray &output, const QString &error) {
                   if (!ok) {
                       emit doctorFailed(error.isEmpty()
                                             ? QStringLiteral("Auto-fix failed")
                                             : error);
                       return;
                   }
                   emit actionFinished(
                       QString::fromUtf8(output).trimmed().isEmpty()
                           ? QStringLiteral("Auto-fix routine finished")
                           : QString::fromUtf8(output).trimmed());
               });
}

void DaemonClient::renderQr(const QString &text)
{
    if (!m_interface) {
        emit qrReady(QImage());
        return;
    }
    auto *watcher = new QDBusPendingCallWatcher(
        m_interface->asyncCall(QStringLiteral("RenderQr"), text), this);
    connect(watcher, &QDBusPendingCallWatcher::finished, this,
            [this](QDBusPendingCallWatcher *call) {
                QDBusPendingReply<QString> reply = *call;
                if (reply.isError()) {
                    emit qrReady(QImage());
                    call->deleteLater();
                    return;
                }
                const QString value = reply.value();
                const int separator = value.indexOf(QLatin1Char(':'));
                bool ok = false;
                const int width = value.left(separator).toInt(&ok);
                if (separator <= 0 || !ok || width <= 0) {
                    emit qrReady(QImage());
                    call->deleteLater();
                    return;
                }
                const QByteArray bytes =
                    QByteArray::fromBase64(value.mid(separator + 1).toLatin1());
                if (bytes.size() != width * width) {
                    emit qrReady(QImage());
                    call->deleteLater();
                    return;
                }
                const QImage image(reinterpret_cast<const uchar *>(bytes.constData()), width,
                                   width, width, QImage::Format_Grayscale8);
                emit qrReady(image.copy());
                call->deleteLater();
            });
}

void DaemonClient::runProcess(
    const QString &program, const QStringList &arguments,
    const std::function<void(bool, const QByteArray &, const QString &)> &done)
{
    auto *process = new QProcess(this);
    const auto handled = QSharedPointer<bool>::create(false);
    const auto finish = [process, handled, done](bool ok, const QByteArray &output,
                                                 const QString &error) {
        if (*handled) {
            return;
        }
        *handled = true;
        done(ok, output, error);
        process->deleteLater();
    };

    connect(process, QOverload<int, QProcess::ExitStatus>::of(&QProcess::finished), this,
            [process, finish](int exitCode, QProcess::ExitStatus status) {
                finish(exitCode == 0 && status == QProcess::NormalExit,
                       process->readAllStandardOutput(),
                       QString::fromLocal8Bit(process->readAllStandardError()).trimmed());
            });
    connect(process, &QProcess::errorOccurred, this,
            [process, finish](QProcess::ProcessError) {
                finish(false, QByteArray(), process->errorString());
            });

    process->start(program, arguments);
}

QString DaemonClient::token()
{
    QFile file(configDir() + QStringLiteral("/token"));
    if (!file.open(QIODevice::ReadOnly)) {
        return QString();
    }
    return QString::fromUtf8(file.readAll()).trimmed();
}

QString DaemonClient::sessionUrl(const DaemonStatus &status, const QString &token)
{
    const int port = status.report.signalingPort > 0 ? status.report.signalingPort : 8788;
    QString url = QStringLiteral("http://%1:%2/client/").arg(status.primaryIp()).arg(port);
    if (!token.isEmpty()) {
        url += QStringLiteral("#token=") + token;
    }
    return url;
}

QString DaemonClient::usbUrl(const DaemonStatus &status, const QString &token)
{
    const int port = status.report.signalingPort > 0 ? status.report.signalingPort : 8788;
    QString url = QStringLiteral("http://127.0.0.1:%1/client/").arg(port);
    if (!token.isEmpty()) {
        url += QStringLiteral("#token=") + token;
    }
    return url;
}

bool DaemonClient::isAutostartEnabled()
{
    return QFile::exists(autostartPath());
}

bool DaemonClient::setAutostart(bool enabled, QString *error)
{
    const QString path = autostartPath();
    if (!enabled) {
        if (QFile::exists(path) && !QFile::remove(path)) {
            if (error) {
                *error = QStringLiteral("Failed to remove %1").arg(path);
            }
            return false;
        }
        return true;
    }

    if (!QDir().mkpath(QFileInfo(path).absolutePath())) {
        if (error) {
            *error = QStringLiteral("Failed to create %1")
                         .arg(QFileInfo(path).absolutePath());
        }
        return false;
    }
    QFile file(path);
    if (!file.open(QIODevice::WriteOnly | QIODevice::Truncate)) {
        if (error) {
            *error = file.errorString();
        }
        return false;
    }
    if (file.write(desktopEntry) < 0) {
        if (error) {
            *error = file.errorString();
        }
        return false;
    }
    return true;
}

bool DaemonClient::openUrl(const QString &value, QString *error)
{
    const QUrl url(value);
    if (!url.isValid() || (url.scheme() != QStringLiteral("http")
                           && url.scheme() != QStringLiteral("https"))) {
        if (error) {
            *error = QStringLiteral("only http and https URLs can be opened");
        }
        return false;
    }
    if (!QDesktopServices::openUrl(url)) {
        if (error) {
            *error = QStringLiteral("failed to launch a browser");
        }
        return false;
    }
    return true;
}

QString DaemonClient::configDir()
{
    return configRoot() + QStringLiteral("/orbiscreen");
}

QStringList DaemonClient::detectLocalIps()
{
    QStringList addresses;
    QUdpSocket probe;
    probe.connectToHost(QHostAddress(QStringLiteral("8.8.8.8")), 80);
    if (probe.waitForConnected(100)) {
        const QHostAddress local = probe.localAddress();
        if (!local.isNull() && !local.isLoopback()) {
            addresses.append(local.toString());
        }
    }
    for (const QHostAddress &address : QNetworkInterface::allAddresses()) {
        if (address.protocol() == QAbstractSocket::IPv4Protocol && !address.isLoopback()) {
            addresses.append(address.toString());
        }
    }
    if (addresses.isEmpty()) {
        addresses.append(QStringLiteral("127.0.0.1"));
    }
    return addresses;
}
