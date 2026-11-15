// Orbiscreen - Qt GUI data models (GPL-3.0-or-later) https://github.com/shadow-x78/orbiscreen

#pragma once

#include <QJsonObject>
#include <QtGlobal>
#include <QString>
#include <QStringList>

struct UsbReport {
    QStringList devices;
    bool aoaReady = false;
    bool permissionDenied = false;
};

struct StatusReport {
    bool running = false;
    quint64 framesForwarded = 0;
    int activeClients = 0;
    quint64 totalClients = 0;
    quint64 authFailures = 0;
    int usbDevices = 0;
    QStringList usbConnectedDevices;
    bool usbAoaReady = false;
    QString encoder = QStringLiteral("auto");
    QString captureBackend;
    int displayWidth = 1920;
    int displayHeight = 1080;
    int displayFps = 60;
    int signalingPort = 8788;
    QString version;

    static StatusReport fromJson(const QJsonObject &object, bool *ok = nullptr);
};

struct DoctorReport {
    QString session;
    QString compositor;
    QString currentDesktop;
    QStringList capturePlan;
    QString displayBackend;
    QString displayBackendKind = QStringLiteral("unknown");
    QString inputBackend;
    bool uinputWritable = false;
    bool hasPortalOnSessionBus = false;
    bool portalOnSessionBus = false;
    bool screencastSavedToken = false;
    bool remoteDesktopSavedToken = false;
    bool swaymsg = false;
    bool hyprctl = false;
    bool cosmicComp = false;
    bool cosmicRandr = false;
    QString wlrootsVirtualOutputIpc;
    UsbReport usb;

    static DoctorReport fromJson(const QJsonObject &object, bool *ok = nullptr);
    bool backendHealthy() const;
};

struct DaemonStatus {
    StatusReport report;
    quint16 udpPort = 8789;
    QStringList localIps;

    QString primaryIp() const;
};
