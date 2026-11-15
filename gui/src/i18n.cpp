// Orbiscreen - Qt GUI localization (GPL-3.0-or-later) https://github.com/shadow-x78/orbiscreen

#include "i18n.h"

#include <QDir>
#include <QFile>
#include <QFileInfo>
#include <QJsonDocument>
#include <QJsonObject>
#include <QStandardPaths>

namespace {

const QHash<QString, QString> &english()
{
    static const QHash<QString, QString> map = {
        {QStringLiteral("tabConnect"), QStringLiteral("Connect")},
        {QStringLiteral("tabDisplay"), QStringLiteral("Display & Input")},
        {QStringLiteral("tabDoctor"), QStringLiteral("Doctor")},
        {QStringLiteral("modeWifi"), QStringLiteral("Wi-Fi Wireless")},
        {QStringLiteral("ready"), QStringLiteral("Ready")},
        {QStringLiteral("streaming"), QStringLiteral("Streaming")},
        {QStringLiteral("stopped"), QStringLiteral("Stopped")},
        {QStringLiteral("statusReadyHeadline"), QStringLiteral("Ready to Stream")},
        {QStringLiteral("statusReadySubtitle"),
         QStringLiteral("Open Orbiscreen on your Android tablet or phone to connect as a second display.")},
        {QStringLiteral("statusStreamingHeadline"), QStringLiteral("Streaming Active")},
        {QStringLiteral("statusStreamingSubtitle"),
         QStringLiteral("Streaming to the connected device with ultra-low-latency hardware encoding.")},
        {QStringLiteral("statusOfflineHeadline"), QStringLiteral("Virtual Display Offline")},
        {QStringLiteral("statusOfflineSubtitle"),
         QStringLiteral("Click Start Extended Display below to activate your virtual monitor.")},
        {QStringLiteral("deviceConnected"), QStringLiteral("1 Device Connected")},
        {QStringLiteral("devicesConnected"), QStringLiteral("{n} Devices Connected")},
        {QStringLiteral("noDevices"), QStringLiteral("0 Devices")},
        {QStringLiteral("qrPrompt"),
         QStringLiteral("Scan with Orbiscreen on your Android tablet or phone")},
        {QStringLiteral("copy"), QStringLiteral("Copy")},
        {QStringLiteral("copied"), QStringLiteral("Copied!")},
        {QStringLiteral("open"), QStringLiteral("Open")},
        {QStringLiteral("guide1"), QStringLiteral("Connect tablet & PC to Wi-Fi")},
        {QStringLiteral("guide2"), QStringLiteral("Open the app & tap Scan")},
        {QStringLiteral("guide3"), QStringLiteral("Enjoy smooth touch & stylus")},
        {QStringLiteral("startDisplay"), QStringLiteral("Start Extended Display")},
        {QStringLiteral("stopDisplay"), QStringLiteral("Stop Display")},
        {QStringLiteral("usbTitle"), QStringLiteral("USB Accessory Pipeline")},
        {QStringLiteral("usbDesc"),
         QStringLiteral("Stream directly over a USB cable with hardware acceleration and zero network jitter.")},
        {QStringLiteral("usbWaiting"), QStringLiteral("Waiting for device...")},
        {QStringLiteral("usbReady"), QStringLiteral("USB Device Ready")},
        {QStringLiteral("btnOpenUsb"), QStringLiteral("Open USB Localhost Stream")},
        {QStringLiteral("usbHint1"), QStringLiteral("Connect your phone or tablet via USB.")},
        {QStringLiteral("usbHint2"),
         QStringLiteral("Select Always Allow when Android prompts for accessory permission.")},
        {QStringLiteral("resGroup"), QStringLiteral("Display Geometry")},
        {QStringLiteral("resolution"), QStringLiteral("Resolution")},
        {QStringLiteral("resDesc"), QStringLiteral("Extended desktop virtual canvas dimensions")},
        {QStringLiteral("framerate"), QStringLiteral("Framerate")},
        {QStringLiteral("fpsDesc"), QStringLiteral("Target display refresh frequency")},
        {QStringLiteral("encoder"), QStringLiteral("Hardware Encoder")},
        {QStringLiteral("inputGroup"), QStringLiteral("Touch & Input Injection")},
        {QStringLiteral("touchScreen"), QStringLiteral("Direct Multitouch Screen")},
        {QStringLiteral("touchDesc"), QStringLiteral("Inject Android touches via kernel uinput slots")},
        {QStringLiteral("stylus"), QStringLiteral("Stylus Digitizer (Pressure & Tilt)")},
        {QStringLiteral("stylusDesc"),
         QStringLiteral("Support pressure sensitivity for Krita, GIMP & notes")},
        {QStringLiteral("systemGroup"), QStringLiteral("System Integration")},
        {QStringLiteral("autostart"), QStringLiteral("Launch on Desktop Login")},
        {QStringLiteral("autostartDesc"), QStringLiteral("Start the Orbiscreen daemon at login")},
        {QStringLiteral("doctorTitle"), QStringLiteral("System Health & Diagnostics")},
        {QStringLiteral("doctorDesc"),
         QStringLiteral("Check the display server, hardware encoding & device permissions")},
        {QStringLiteral("scan"), QStringLiteral("Scan")},
        {QStringLiteral("autoFix"), QStringLiteral("Auto-Fix")},
        {QStringLiteral("checking"), QStringLiteral("Checking...")},
        {QStringLiteral("fixing"), QStringLiteral("Fixing...")},
        {QStringLiteral("docCompositor"), QStringLiteral("Compositor & Display Server")},
        {QStringLiteral("docBackend"), QStringLiteral("Virtual Screen Pipeline")},
        {QStringLiteral("docUinput"), QStringLiteral("Input Injection (/dev/uinput)")},
        {QStringLiteral("docEncoder"), QStringLiteral("Video Hardware Acceleration")},
        {QStringLiteral("portsTitle"), QStringLiteral("Network Ports")},
        {QStringLiteral("docUsb"), QStringLiteral("USB Accessory Bridge")},
        {QStringLiteral("permVerified"), QStringLiteral("Permissions Verified")},
        {QStringLiteral("permDenied"), QStringLiteral("No write access to /dev/uinput")},
        {QStringLiteral("available"), QStringLiteral("Available")},
        {QStringLiteral("notDetected"), QStringLiteral("Not detected")},
        {QStringLiteral("usbNoDevices"), QStringLiteral("No USB devices")},
        {QStringLiteral("usbPermissionDenied"),
         QStringLiteral("USB permission denied (check udev rules)")},
        {QStringLiteral("docPortsDesc"),
         QStringLiteral("Signaling & UDP Annex-B transport ready")},
        {QStringLiteral("docTipTitle"), QStringLiteral("Diagnostic Notes")},
        {QStringLiteral("docTipDesc"),
         QStringLiteral("Ensure the tablet and host share the same Wi-Fi subnet, or use a high-speed USB cable for zero-latency streaming.")},
        {QStringLiteral("connected"), QStringLiteral("Connected")},
        {QStringLiteral("restartTitle"), QStringLiteral("Restart Host Daemon")},
        {QStringLiteral("devices"), QStringLiteral("Devices")},
        {QStringLiteral("about"), QStringLiteral("About")},
        {QStringLiteral("quit"), QStringLiteral("Quit")},
        {QStringLiteral("gettingStarted"), QStringLiteral("Getting Started")},
        {QStringLiteral("menuService"), QStringLiteral("Service")},
        {QStringLiteral("menuView"), QStringLiteral("View")},
        {QStringLiteral("menuLanguage"), QStringLiteral("Language")},
        {QStringLiteral("menuHelp"), QStringLiteral("Help")},
        {QStringLiteral("actionCheckUpdates"), QStringLiteral("Check for Updates")},
        {QStringLiteral("updateAvailable"), QStringLiteral("Update %1 available (installed %2)")},
        {QStringLiteral("actionRefresh"), QStringLiteral("Refresh")},
        {QStringLiteral("actionOpenDashboard"), QStringLiteral("Open Dashboard")},
        {QStringLiteral("resolutionApply"), QStringLiteral("Apply")},
        {QStringLiteral("doctorNoReport"), QStringLiteral("Run a scan to populate diagnostics.")},
        {QStringLiteral("toastRestart"), QStringLiteral("Restarting host daemon...")},
        {QStringLiteral("toastRestartDone"), QStringLiteral("Daemon restarted successfully")},
        {QStringLiteral("toastStarted"), QStringLiteral("Extended display started")},
        {QStringLiteral("toastStopped"), QStringLiteral("Extended display stopped")},
        {QStringLiteral("toastCopied"), QStringLiteral("Stream URL copied to clipboard")},
        {QStringLiteral("toastDoctorFailed"), QStringLiteral("Doctor check failed")},
        {QStringLiteral("toastFixDone"), QStringLiteral("Auto-fix routine finished")},
        {QStringLiteral("toastFixFailed"), QStringLiteral("Auto-fix failed")},
        {QStringLiteral("toastSetFailed"), QStringLiteral("Failed to apply display settings")},
        {QStringLiteral("toastAutostartFailed"), QStringLiteral("Failed to update autostart")},
    };
    return map;
}

const QHash<QString, QString> &arabic()
{
    static const QHash<QString, QString> map = {
        {QStringLiteral("tabConnect"), QStringLiteral("الاتصال")},
        {QStringLiteral("tabDisplay"), QStringLiteral("الشاشة والإدخال")},
        {QStringLiteral("tabDoctor"), QStringLiteral("الفحص الذكي")},
        {QStringLiteral("modeWifi"), QStringLiteral("واي فاي لاسلكي")},
        {QStringLiteral("ready"), QStringLiteral("جاهز")},
        {QStringLiteral("streaming"), QStringLiteral("جارِ البث")},
        {QStringLiteral("stopped"), QStringLiteral("متوقف")},
        {QStringLiteral("statusReadyHeadline"), QStringLiteral("جاهز للاتصال والبث")},
        {QStringLiteral("statusReadySubtitle"),
         QStringLiteral("افتح تطبيق Orbiscreen على جهاز الأندرويد للاتصال كشاشة ثانية.")},
        {QStringLiteral("statusStreamingHeadline"), QStringLiteral("البث نشط الآن")},
        {QStringLiteral("statusStreamingSubtitle"),
         QStringLiteral("يتم البث المباشر للجهاز المتصل بترميز عتادي وزمن استجابة فائق السرعة.")},
        {QStringLiteral("statusOfflineHeadline"), QStringLiteral("الشاشة الافتراضية متوقفة")},
        {QStringLiteral("statusOfflineSubtitle"),
         QStringLiteral("اضغط على زر بدء الشاشة الموسعة بالأسفل لتفعيل الشاشة.")},
        {QStringLiteral("deviceConnected"), QStringLiteral("جهاز واحد متصل")},
        {QStringLiteral("devicesConnected"), QStringLiteral("{n} أجهزة متصلة")},
        {QStringLiteral("noDevices"), QStringLiteral("لا توجد أجهزة")},
        {QStringLiteral("qrPrompt"),
         QStringLiteral("امسح الرمز بواسطة تطبيق Orbiscreen على جهاز الأندرويد")},
        {QStringLiteral("copy"), QStringLiteral("نسخ")},
        {QStringLiteral("copied"), QStringLiteral("تم النسخ!")},
        {QStringLiteral("open"), QStringLiteral("فتح")},
        {QStringLiteral("guide1"), QStringLiteral("اتصل بنفس شبكة الواي فاي للكمبيوتر")},
        {QStringLiteral("guide2"), QStringLiteral("افتح التطبيق واضغط مسح QR")},
        {QStringLiteral("guide3"), QStringLiteral("تحكم فوري باللمس والقلم")},
        {QStringLiteral("startDisplay"), QStringLiteral("بدء الشاشة الموسعة")},
        {QStringLiteral("stopDisplay"), QStringLiteral("إيقاف الشاشة")},
        {QStringLiteral("usbTitle"), QStringLiteral("الاتصال المباشر عبر USB")},
        {QStringLiteral("usbDesc"),
         QStringLiteral("بث مباشر عبر كابل USB بتسريع عتادي ودون أي تأخير بالشبكة.")},
        {QStringLiteral("usbWaiting"), QStringLiteral("في انتظار توصيل الجهاز...")},
        {QStringLiteral("usbReady"), QStringLiteral("جهاز USB متصل وجاهز")},
        {QStringLiteral("btnOpenUsb"), QStringLiteral("فتح البث عبر المضيف المحلي")},
        {QStringLiteral("usbHint1"), QStringLiteral("صِل هاتفك أو جهازك اللوحي بكابل USB.")},
        {QStringLiteral("usbHint2"),
         QStringLiteral("اختر السماح دائماً عند ظهور تنبيه الملحق على الأندرويد.")},
        {QStringLiteral("resGroup"), QStringLiteral("أبعاد الشاشة")},
        {QStringLiteral("resolution"), QStringLiteral("الدقة")},
        {QStringLiteral("resDesc"), QStringLiteral("أبعاد الشاشة الافتراضية الموسعة لسطح المكتب")},
        {QStringLiteral("framerate"), QStringLiteral("معدل التحديث")},
        {QStringLiteral("fpsDesc"), QStringLiteral("تردد تحديث الإطارات للشاشة")},
        {QStringLiteral("encoder"), QStringLiteral("المرمّز العتادي")},
        {QStringLiteral("inputGroup"), QStringLiteral("اللمس وإدخال التحكم")},
        {QStringLiteral("touchScreen"), QStringLiteral("شاشة اللمس المتعدد المباشر")},
        {QStringLiteral("touchDesc"), QStringLiteral("تمرير اللمسات عبر أنوية uinput في لينكس")},
        {QStringLiteral("stylus"), QStringLiteral("دعم القلم الرقمي (الضغط والميلان)")},
        {QStringLiteral("stylusDesc"),
         QStringLiteral("حساسية الضغط لتطبيقات الرسم مثل Krita والملاحظات")},
        {QStringLiteral("systemGroup"), QStringLiteral("تكامل النظام")},
        {QStringLiteral("autostart"), QStringLiteral("تشغيل تلقائي عند بدء التشغيل")},
        {QStringLiteral("autostartDesc"), QStringLiteral("بدء الخدمة الخلفية تلقائياً عند تسجيل الدخول")},
        {QStringLiteral("doctorTitle"), QStringLiteral("الفحص الذكي وتشخيص النظام")},
        {QStringLiteral("doctorDesc"),
         QStringLiteral("فحص خادم العرض، ومسرعات الفيديو، وصلاحيات النظام")},
        {QStringLiteral("scan"), QStringLiteral("فحص الآن")},
        {QStringLiteral("autoFix"), QStringLiteral("إصلاح تلقائي")},
        {QStringLiteral("checking"), QStringLiteral("جارِ الفحص...")},
        {QStringLiteral("fixing"), QStringLiteral("جارِ الإصلاح...")},
        {QStringLiteral("docCompositor"), QStringLiteral("مدير النوافذ وخادم العرض")},
        {QStringLiteral("docBackend"), QStringLiteral("مسار الشاشة الافتراضية")},
        {QStringLiteral("docUinput"), QStringLiteral("وحدة حقن الإدخال (/dev/uinput)")},
        {QStringLiteral("docEncoder"), QStringLiteral("تسريع الفيديو العتادي")},
        {QStringLiteral("portsTitle"), QStringLiteral("منافذ الشبكة")},
        {QStringLiteral("docUsb"), QStringLiteral("جسر ملحقات USB")},
        {QStringLiteral("permVerified"), QStringLiteral("الصلاحيات مؤكدة")},
        {QStringLiteral("permDenied"), QStringLiteral("لا توجد صلاحية كتابة إلى /dev/uinput")},
        {QStringLiteral("available"), QStringLiteral("متوفر")},
        {QStringLiteral("notDetected"), QStringLiteral("غير مكتشف")},
        {QStringLiteral("usbNoDevices"), QStringLiteral("لا توجد أجهزة USB")},
        {QStringLiteral("usbPermissionDenied"), QStringLiteral("تم رفض صلاحية USB (راجع قواعد udev)")},
        {QStringLiteral("docPortsDesc"), QStringLiteral("بروتوكول الإشارات ونقل UDP المباشر جاهز")},
        {QStringLiteral("docTipTitle"), QStringLiteral("ملاحظات الفحص")},
        {QStringLiteral("docTipDesc"),
         QStringLiteral("تأكد من وجود جهازك والكمبيوتر على نفس الشبكة المحلية، أو استخدم كابل USB مباشر لأقل زمن استجابة ممكن.")},
        {QStringLiteral("connected"), QStringLiteral("متصل")},
        {QStringLiteral("restartTitle"), QStringLiteral("إعادة تشغيل الخدمة")},
        {QStringLiteral("devices"), QStringLiteral("الأجهزة")},
        {QStringLiteral("about"), QStringLiteral("حول")},
        {QStringLiteral("quit"), QStringLiteral("خروج")},
        {QStringLiteral("gettingStarted"), QStringLiteral("البدء السريع")},
        {QStringLiteral("menuService"), QStringLiteral("الخدمة")},
        {QStringLiteral("menuView"), QStringLiteral("عرض")},
        {QStringLiteral("menuLanguage"), QStringLiteral("اللغة")},
        {QStringLiteral("menuHelp"), QStringLiteral("مساعدة")},
        {QStringLiteral("actionCheckUpdates"), QStringLiteral("التحقق من التحديثات")},
        {QStringLiteral("updateAvailable"), QStringLiteral("يتوفر تحديث %1 (المثبت %2)")},
        {QStringLiteral("actionRefresh"), QStringLiteral("تحديث")},
        {QStringLiteral("actionOpenDashboard"), QStringLiteral("فتح لوحة التحكم")},
        {QStringLiteral("resolutionApply"), QStringLiteral("تطبيق")},
        {QStringLiteral("doctorNoReport"), QStringLiteral("شغّل الفحص لعرض النتائج.")},
        {QStringLiteral("toastRestart"), QStringLiteral("جارِ إعادة تشغيل الخدمة...")},
        {QStringLiteral("toastRestartDone"), QStringLiteral("تمت إعادة تشغيل الخدمة بنجاح")},
        {QStringLiteral("toastStarted"), QStringLiteral("تم بدء الشاشة الموسعة")},
        {QStringLiteral("toastStopped"), QStringLiteral("تم إيقاف الشاشة الموسعة")},
        {QStringLiteral("toastCopied"), QStringLiteral("تم نسخ رابط البث إلى الحافظة")},
        {QStringLiteral("toastDoctorFailed"), QStringLiteral("فشل تشغيل الفحص")},
        {QStringLiteral("toastFixDone"), QStringLiteral("انتهى الإصلاح التلقائي")},
        {QStringLiteral("toastFixFailed"), QStringLiteral("فشل الإصلاح التلقائي")},
        {QStringLiteral("toastSetFailed"), QStringLiteral("فشل تطبيق إعدادات الشاشة")},
        {QStringLiteral("toastAutostartFailed"), QStringLiteral("فشل تحديث التشغيل التلقائي")},
    };
    return map;
}

QString preferencesPath()
{
    return QStandardPaths::writableLocation(QStandardPaths::GenericConfigLocation)
        + QStringLiteral("/orbiscreen/gui.json");
}

}

I18n::I18n() = default;

I18n &I18n::instance()
{
    static I18n instance;
    return instance;
}

void I18n::setLang(Lang lang)
{
    m_lang = lang;
    save();
}

QString I18n::t(const QString &key) const
{
    const QHash<QString, QString> &map = m_lang == Lang::Ar ? arabic() : english();
    auto it = map.constFind(key);
    if (it != map.constEnd()) {
        return it.value();
    }
    auto fallback = english().constFind(key);
    return fallback != english().constEnd() ? fallback.value() : key;
}

QString I18n::format(const QString &key, int count) const
{
    return t(key).replace(QStringLiteral("{n}"), QString::number(count));
}

QStringList I18n::keys() const
{
    QStringList result;
    const QList<QString> keys = english().keys();
    for (const QString &key : keys) {
        result.append(key);
    }
    return result;
}

bool I18n::has(Lang lang, const QString &key) const
{
    const QHash<QString, QString> &map = lang == Lang::Ar ? arabic() : english();
    return map.contains(key);
}

void I18n::load()
{
    QFile file(preferencesPath());
    if (!file.open(QIODevice::ReadOnly)) {
        return;
    }
    const QJsonObject object = QJsonDocument::fromJson(file.readAll()).object();
    if (object.value(QStringLiteral("language")).toString() == QStringLiteral("ar")) {
        m_lang = Lang::Ar;
    }
}

void I18n::save() const
{
    const QString path = preferencesPath();
    QDir().mkpath(QFileInfo(path).absolutePath());
    QFile file(path);
    if (!file.open(QIODevice::WriteOnly | QIODevice::Truncate)) {
        return;
    }
    QJsonObject object;
    object.insert(QStringLiteral("language"),
                  m_lang == Lang::Ar ? QStringLiteral("ar") : QStringLiteral("en"));
    file.write(QJsonDocument(object).toJson(QJsonDocument::Indented));
}
