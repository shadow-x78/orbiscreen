// Orbiscreen - Qt GUI tests (GPL-3.0-or-later) https://github.com/shadow-x78/orbiscreen

#include <QtTest>
#include <QJsonDocument>
#include <QJsonObject>
#include <QStringList>

#include "daemonclient.h"
#include "i18n.h"
#include "report.h"

class GuiTests : public QObject {
    Q_OBJECT

private slots:
    void parsesStatusReport();
    void parsesDoctorReport();
    void rejectsMalformedReports();
    void formatsSessionUrls();
    void formatsUsbUrls();
    void i18nDictionariesHaveParity();
    void formatsDeviceCount();
    void mapsBackendHealth();
};

void GuiTests::parsesStatusReport()
{
    const QByteArray json = R"json({
        "running": true,
        "frames_forwarded": 42,
        "active_clients": 2,
        "total_clients": 3,
        "auth_failures": 0,
        "usb_devices": 1,
        "usb_connected_devices": ["Tablet"],
        "usb_aoa_ready": true,
        "encoder": "vaapi",
        "capture_backend": "kwin-virtual",
        "display_width": 2560,
        "display_height": 1440,
        "display_fps": 120,
        "signaling_port": 8788,
        "version": "0.41.0"
    })json";
    const QJsonObject object = QJsonDocument::fromJson(json).object();
    bool ok = false;
    const StatusReport report = StatusReport::fromJson(object, &ok);
    QVERIFY(ok);
    QVERIFY(report.running);
    QCOMPARE(report.activeClients, 2);
    QCOMPARE(report.usbDevices, 1);
    QCOMPARE(report.usbConnectedDevices, QStringList{QStringLiteral("Tablet")});
    QVERIFY(report.usbAoaReady);
    QCOMPARE(report.encoder, QStringLiteral("vaapi"));
    QCOMPARE(report.displayWidth, 2560);
    QCOMPARE(report.displayHeight, 1440);
    QCOMPARE(report.displayFps, 120);
    QCOMPARE(report.signalingPort, 8788);
}

void GuiTests::parsesDoctorReport()
{
    const QByteArray json = R"json({
        "session": "wayland",
        "compositor": "KDE Plasma",
        "current_desktop": "KDE",
        "capture_plan": ["kwin-virtual", "portal"],
        "display_backend": "Native (KWin Virtual)",
        "display_backend_kind": "native_kwin",
        "input_backend": "Wayland",
        "uinput_writable": true,
        "portal_on_session_bus": true,
        "screencast_saved_token": false,
        "remote_desktop_saved_token": false,
        "swaymsg": false,
        "hyprctl": false,
        "cosmic_comp": false,
        "cosmic_randr": false,
        "wlroots_virtual_output_ipc": null,
        "usb": {"devices": [], "aoa_ready": false, "permission_denied": false}
    })json";
    const QJsonObject object = QJsonDocument::fromJson(json).object();
    bool ok = false;
    const DoctorReport report = DoctorReport::fromJson(object, &ok);
    QVERIFY(ok);
    QCOMPARE(report.session, QStringLiteral("wayland"));
    QCOMPARE(report.compositor, QStringLiteral("KDE Plasma"));
    QCOMPARE(report.displayBackendKind, QStringLiteral("native_kwin"));
    QVERIFY(report.backendHealthy());
    QVERIFY(report.uinputWritable);
    QVERIFY(report.hasPortalOnSessionBus);
    QVERIFY(report.portalOnSessionBus);
    QVERIFY(report.usb.devices.isEmpty());
}

void GuiTests::rejectsMalformedReports()
{
    bool ok = true;
    StatusReport::fromJson(QJsonObject{}, &ok);
    QVERIFY(!ok);
    ok = true;
    DoctorReport::fromJson(QJsonObject{}, &ok);
    QVERIFY(!ok);
}

void GuiTests::formatsSessionUrls()
{
    DaemonStatus status;
    status.report.signalingPort = 8788;
    status.localIps = {QStringLiteral("192.168.1.5")};
    QCOMPARE(DaemonClient::sessionUrl(status, QString()),
             QStringLiteral("http://192.168.1.5:8788/client/"));
    QCOMPARE(DaemonClient::sessionUrl(status, QStringLiteral("abc123")),
             QStringLiteral("http://192.168.1.5:8788/client/#token=abc123"));
}

void GuiTests::formatsUsbUrls()
{
    DaemonStatus status;
    status.report.signalingPort = 9000;
    status.localIps = {QStringLiteral("192.168.1.5")};
    QCOMPARE(DaemonClient::usbUrl(status, QStringLiteral("abc123")),
             QStringLiteral("http://127.0.0.1:9000/client/#token=abc123"));
}

void GuiTests::i18nDictionariesHaveParity()
{
    I18n &i18n = I18n::instance();
    for (const QString &key : i18n.keys()) {
        QVERIFY2(i18n.has(Lang::Ar, key),
                 qPrintable(QStringLiteral("Arabic dictionary misses %1").arg(key)));
        QVERIFY2(i18n.has(Lang::En, key),
                 qPrintable(QStringLiteral("English dictionary misses %1").arg(key)));
    }
}

void GuiTests::formatsDeviceCount()
{
    I18n &i18n = I18n::instance();
    const Lang previous = i18n.lang();
    i18n.setLang(Lang::En);
    QCOMPARE(i18n.format(QStringLiteral("devicesConnected"), 3),
             QStringLiteral("3 Devices Connected"));
    i18n.setLang(previous);
}

void GuiTests::mapsBackendHealth()
{
    DoctorReport report;
    report.displayBackendKind = QStringLiteral("native_kwin");
    QVERIFY(report.backendHealthy());
    report.displayBackendKind = QStringLiteral("native_wlroots");
    QVERIFY(report.backendHealthy());
    report.displayBackendKind = QStringLiteral("evdi_compatible");
    QVERIFY(report.backendHealthy());
    report.displayBackendKind = QStringLiteral("evdi_missing");
    QVERIFY(!report.backendHealthy());
}

QTEST_GUILESS_MAIN(GuiTests)

#include "tst_gui.moc"
