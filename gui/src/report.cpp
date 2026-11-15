// Orbiscreen - Qt GUI data models (GPL-3.0-or-later) https://github.com/shadow-x78/orbiscreen

#include "report.h"

#include <QJsonArray>
#include <QJsonValue>

namespace {

QStringList stringList(const QJsonValue &value)
{
    QStringList result;
    for (const QJsonValue &entry : value.toArray()) {
        if (entry.isString()) {
            result.append(entry.toString());
        }
    }
    return result;
}

quint64 unsignedValue(const QJsonValue &value)
{
    return static_cast<quint64>(value.toDouble());
}

}

StatusReport StatusReport::fromJson(const QJsonObject &object, bool *ok)
{
    StatusReport report;
    report.running = object.value(QStringLiteral("running")).toBool();
    report.framesForwarded = unsignedValue(object.value(QStringLiteral("frames_forwarded")));
    report.activeClients = object.value(QStringLiteral("active_clients")).toInt();
    report.totalClients = unsignedValue(object.value(QStringLiteral("total_clients")));
    report.authFailures = unsignedValue(object.value(QStringLiteral("auth_failures")));
    report.usbDevices = object.value(QStringLiteral("usb_devices")).toInt();
    report.usbConnectedDevices =
        stringList(object.value(QStringLiteral("usb_connected_devices")));
    report.usbAoaReady = object.value(QStringLiteral("usb_aoa_ready")).toBool();
    report.encoder = object.value(QStringLiteral("encoder")).toString(QStringLiteral("auto"));
    report.captureBackend = object.value(QStringLiteral("capture_backend")).toString();
    report.displayWidth = object.value(QStringLiteral("display_width")).toInt(1920);
    report.displayHeight = object.value(QStringLiteral("display_height")).toInt(1080);
    report.displayFps = object.value(QStringLiteral("display_fps")).toInt(60);
    report.signalingPort = object.value(QStringLiteral("signaling_port")).toInt(8788);
    report.version = object.value(QStringLiteral("version")).toString();

    if (ok) {
        *ok = !object.isEmpty();
    }
    return report;
}

DoctorReport DoctorReport::fromJson(const QJsonObject &object, bool *ok)
{
    DoctorReport report;
    report.session = object.value(QStringLiteral("session")).toString();
    report.compositor = object.value(QStringLiteral("compositor")).toString();
    report.currentDesktop = object.value(QStringLiteral("current_desktop")).toString();
    report.capturePlan = stringList(object.value(QStringLiteral("capture_plan")));
    report.displayBackend = object.value(QStringLiteral("display_backend")).toString();
    report.displayBackendKind =
        object.value(QStringLiteral("display_backend_kind")).toString(QStringLiteral("unknown"));
    report.inputBackend = object.value(QStringLiteral("input_backend")).toString();
    report.uinputWritable = object.value(QStringLiteral("uinput_writable")).toBool();
    const QJsonValue portal = object.value(QStringLiteral("portal_on_session_bus"));
    if (!portal.isNull() && !portal.isUndefined()) {
        report.hasPortalOnSessionBus = true;
        report.portalOnSessionBus = portal.toBool();
    }
    report.screencastSavedToken =
        object.value(QStringLiteral("screencast_saved_token")).toBool();
    report.remoteDesktopSavedToken =
        object.value(QStringLiteral("remote_desktop_saved_token")).toBool();
    report.swaymsg = object.value(QStringLiteral("swaymsg")).toBool();
    report.hyprctl = object.value(QStringLiteral("hyprctl")).toBool();
    report.cosmicComp = object.value(QStringLiteral("cosmic_comp")).toBool();
    report.cosmicRandr = object.value(QStringLiteral("cosmic_randr")).toBool();
    report.wlrootsVirtualOutputIpc =
        object.value(QStringLiteral("wlroots_virtual_output_ipc")).toString();

    const QJsonObject usb = object.value(QStringLiteral("usb")).toObject();
    report.usb.devices = stringList(usb.value(QStringLiteral("devices")));
    report.usb.aoaReady = usb.value(QStringLiteral("aoa_ready")).toBool();
    report.usb.permissionDenied = usb.value(QStringLiteral("permission_denied")).toBool();

    if (ok) {
        *ok = !object.isEmpty();
    }
    return report;
}

bool DoctorReport::backendHealthy() const
{
    return displayBackendKind == QStringLiteral("native_kwin")
        || displayBackendKind == QStringLiteral("native_wlroots")
        || displayBackendKind == QStringLiteral("evdi_compatible");
}

QString DaemonStatus::primaryIp() const
{
    for (const QString &ip : localIps) {
        if (!ip.isEmpty() && ip != QStringLiteral("127.0.0.1")) {
            return ip;
        }
    }
    if (!localIps.isEmpty()) {
        return localIps.first();
    }
    return QStringLiteral("127.0.0.1");
}
