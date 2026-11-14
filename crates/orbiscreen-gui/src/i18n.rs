// Orbiscreen - Localization (GPL-3.0-or-later) https://github.com/shadow-x78/orbiscreen

use std::collections::HashMap;
use std::path::PathBuf;

use gtk::glib;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Lang {
    #[default]
    En,
    Ar,
}

impl Lang {
    pub fn other(self) -> Self {
        match self {
            Lang::En => Lang::Ar,
            Lang::Ar => Lang::En,
        }
    }

    pub fn code(self) -> &'static str {
        match self {
            Lang::En => "en",
            Lang::Ar => "ar",
        }
    }

    fn from_code(code: &str) -> Self {
        match code.trim().to_ascii_lowercase().as_str() {
            "ar" => Lang::Ar,
            _ => Lang::En,
        }
    }
}

static EN: std::sync::LazyLock<HashMap<&'static str, &'static str>> = std::sync::LazyLock::new(
    || {
        HashMap::from([
        ("tabConnect", "Connect"),
        ("tabDisplay", "Display & Input"),
        ("tabDoctor", "Doctor"),
        ("modeWifi", "Wi-Fi Wireless"),
        ("ready", "Ready"),
        ("streaming", "Streaming"),
        ("stopped", "Stopped"),
        ("statusReadyHeadline", "Ready to Stream"),
        (
            "statusReadySubtitle",
            "Open Orbiscreen on your Android tablet or phone to connect as second display.",
        ),
        ("statusStreamingHeadline", "Streaming Active"),
        (
            "statusStreamingSubtitle",
            "Streaming to connected device with ultra-low latency hardware encoding.",
        ),
        ("statusOfflineHeadline", "Virtual Display Offline"),
        (
            "statusOfflineSubtitle",
            "Click Start Extended Display below to activate your virtual monitor.",
        ),
        ("deviceConnected", "1 Device Connected"),
        ("devicesConnected", "{n} Devices Connected"),
        ("noDevices", "0 Devices"),
        (
            "qrPrompt",
            "Scan with Orbiscreen on your Android tablet or phone",
        ),
        ("copy", "Copy"),
        ("copied", "Copied!"),
        ("open", "Open"),
        ("guide1", "Connect tablet &amp; PC to Wi-Fi"),
        ("guide2", "Open app &amp; tap Scan"),
        ("guide3", "Enjoy smooth touch &amp; stylus"),
        ("startDisplay", "Start Extended Display"),
        ("stopDisplay", "Stop Display"),
        ("usbTitle", "USB Accessory Pipeline"),
        (
            "usbDesc",
            "Stream directly over USB cable with hardware acceleration and zero network jitter.",
        ),
        ("usbWaiting", "Waiting for device..."),
        ("usbReady", "USB Device Ready"),
        ("btnOpenUsb", "Open USB Localhost Stream"),
        ("usbHint1", "Connect your phone or tablet via USB."),
        ("usbHint2", "Select Always Allow when Android prompts for accessory permission."),
        ("resGroup", "Display Geometry"),
        ("resolution", "Resolution"),
        ("resDesc", "Extended desktop virtual canvas dimensions"),
        ("framerate", "Framerate"),
        ("fpsDesc", "Target display refresh frequency"),
        ("encoder", "Hardware Encoder"),
        ("inputGroup", "Touch &amp; Input Injection"),
        ("touchScreen", "Direct Multitouch Screen"),
        ("touchDesc", "Inject Android touches via kernel uinput slots"),
        ("stylus", "Stylus Digitizer (Pressure &amp; Tilt)"),
        ("stylusDesc", "Support pressure sensitivity for Krita, GIMP &amp; notes"),
        ("systemGroup", "System Integration"),
        ("autostart", "Launch on Desktop Login"),
        ("autostartDesc", "Start Orbiscreen daemon on system startup"),
        ("doctorTitle", "System Health &amp; Diagnostics"),
        (
            "doctorDesc",
            "Check display server, hardware encoding &amp; device permissions",
        ),
        ("scan", "Scan"),
        ("autoFix", "Auto-Fix"),
        ("checking", "Checking..."),
        ("fixing", "Fixing..."),
        ("docCompositor", "Compositor & Display Server"),
        ("docBackend", "Virtual Screen Pipeline"),
        ("docUinput", "Input Injection (/dev/uinput)"),
        ("docEncoder", "Video Hardware Acceleration"),
        ("portsTitle", "Network Ports"),
        ("docUsb", "USB Accessory Bridge"),
        ("permVerified", "Permissions Verified"),
        ("permDenied", "No write access to /dev/uinput"),
        ("available", "Available"),
        ("notDetected", "Not detected"),
        ("usbNoDevices", "No USB devices"),
        ("usbPermissionDenied", "USB permission denied (check udev rules)"),
        (
            "docPortsDesc",
            "Signaling &amp; UDP Annex-B transport ready",
        ),
        ("docTipTitle", "Diagnostic Notes"),
        (
            "docTipDesc",
            "Ensure your tablet and host PC share the same Wi-Fi subnet, or use a high-speed USB cable for zero-latency direct streaming.",
        ),
        ("connected", "Connected"),
        ("restartTitle", "Restart Host Daemon"),
        ("devices", "Devices"),
        ("about", "About"),
        ("quit", "Quit"),
        ("toastRestart", "Restarting host daemon..."),
        ("toastRestartDone", "Daemon restarted successfully"),
        ("toastStarted", "Extended display started"),
        ("toastStopped", "Extended display stopped"),
        ("toastCopied", "Stream URL copied to clipboard"),
        ("toastDoctorFailed", "Doctor check failed"),
        ("toastFixDone", "Auto-fix routine finished"),
        ("toastFixFailed", "Auto-fix failed"),
        ("toastSetFailed", "Failed to apply display settings"),
    ])
    },
);

static AR: std::sync::LazyLock<HashMap<&'static str, &'static str>> = std::sync::LazyLock::new(
    || {
        HashMap::from([
        ("tabConnect", "الاتصال"),
        ("tabDisplay", "الشاشة والإدخال"),
        ("tabDoctor", "الفحص الذكي"),
        ("modeWifi", "واي فاي لاسلكي"),
        ("ready", "جاهز"),
        ("streaming", "جارِ البث"),
        ("stopped", "متوقف"),
        ("statusReadyHeadline", "جاهز للاتصال والبث"),
        (
            "statusReadySubtitle",
            "افتح تطبيق Orbiscreen على جهاز الأندرويد للاتصال كشاشة ثانية.",
        ),
        ("statusStreamingHeadline", "البث نشط الآن"),
        (
            "statusStreamingSubtitle",
            "يتم البث المباشر للجهاز المتصل بترميز عتادي وزمن استجابة فائق السرعة.",
        ),
        ("statusOfflineHeadline", "الشاشة الافتراضية متوقفة"),
        (
            "statusOfflineSubtitle",
            "اضغط على زر بدء الشاشة الموسعة بالأسفل لتفعيل الشاشة.",
        ),
        ("deviceConnected", "جهاز واحد متصل"),
        ("devicesConnected", "{n} أجهزة متصلة"),
        ("noDevices", "لا توجد أجهزة"),
        (
            "qrPrompt",
            "امسح الرمز بواسطة تطبيق Orbiscreen على جهاز الأندرويد",
        ),
        ("copy", "نسخ"),
        ("copied", "تم النسخ!"),
        ("open", "فتح"),
        ("guide1", "اتصل بنفس شبكة الواي فاي للكمبيوتر"),
        ("guide2", "افتح التطبيق واضغط مسح QR"),
        ("guide3", "تحكم فوري باللمس والقلم"),
        ("startDisplay", "بدء الشاشة الموسعة"),
        ("stopDisplay", "إيقاف الشاشة"),
        ("usbTitle", "الاتصال المباشر عبر USB"),
        (
            "usbDesc",
            "بث مباشر عبر كابل USB بتسريع عتادي ودون أي تأخير بالشبكة.",
        ),
        ("usbWaiting", "في انتظار توصيل الجهاز..."),
        ("usbReady", "جهاز USB متصل وجاهز"),
        ("btnOpenUsb", "فتح البث عبر المضيف المحلي"),
        ("usbHint1", "صِل هاتفك أو جهازك اللوحي بكابل USB."),
        ("usbHint2", "اختر السماح دائماً عند ظهور تنبيه الملحق على الأندرويد."),
        ("resGroup", "أبعاد الشاشة"),
        ("resolution", "الدقة"),
        ("resDesc", "أبعاد الشاشة الافتراضية الموسعة لسطح المكتب"),
        ("framerate", "معدل التحديث"),
        ("fpsDesc", "تردد تحديث الإطارات للشاشة"),
        ("encoder", "المرمّز العتادي"),
        ("inputGroup", "اللمس وإدخال التحكم"),
        ("touchScreen", "شاشة اللمس المتعدد المباشر"),
        ("touchDesc", "تمرير اللمسات عبر أنوية uinput في لينكس"),
        ("stylus", "دعم القلم الرقمي (الضغط والميلان)"),
        ("stylusDesc", "حساسية الضغط لتطبيقات الرسم مثل Krita والملاحظات"),
        ("systemGroup", "تكامل النظام"),
        ("autostart", "تشغيل تلقائي عند بدء التشغيل"),
        ("autostartDesc", "بدء الخدمة الخلفية تلقائياً عند تسجيل الدخول"),
        ("doctorTitle", "الفحص الذكي وتشخيص النظام"),
        (
            "doctorDesc",
            "فحص خادم العرض، ومسرعات الفيديو، وصلاحيات النظام",
        ),
        ("scan", "فحص الآن"),
        ("autoFix", "إصلاح تلقائي"),
        ("checking", "جارِ الفحص..."),
        ("fixing", "جارِ الإصلاح..."),
        ("docCompositor", "مدير النوافذ وخادم العرض"),
        ("docBackend", "مسار الشاشة الافتراضية"),
        ("docUinput", "وحدة حقن الإدخال (/dev/uinput)"),
        ("docEncoder", "تسريع الفيديو العتادي"),
        ("portsTitle", "منافذ الشبكة"),
        ("docUsb", "جسر ملحقات USB"),
        ("permVerified", "الصلاحيات مؤكدة"),
        ("permDenied", "لا توجد صلاحية كتابة إلى /dev/uinput"),
        ("available", "متوفر"),
        ("notDetected", "غير مكتشف"),
        ("usbNoDevices", "لا توجد أجهزة USB"),
        ("usbPermissionDenied", "تم رفض صلاحية USB (راجع قواعد udev)"),
        (
            "docPortsDesc",
            "بروتوكول الإشارات ونقل UDP المباشر جاهز",
        ),
        ("docTipTitle", "ملاحظات الفحص"),
        (
            "docTipDesc",
            "تأكد من وجود جهازك والكمبيوتر على نفس الشبكة المحلية، أو استخدم كابل USB مباشر لأقل زمن استجابة ممكن.",
        ),
        ("connected", "متصل"),
        ("restartTitle", "إعادة تشغيل الخدمة"),
        ("devices", "الأجهزة"),
        ("about", "حول"),
        ("quit", "خروج"),
        ("toastRestart", "جارِ إعادة تشغيل الخدمة..."),
        ("toastRestartDone", "تمت إعادة تشغيل الخدمة بنجاح"),
        ("toastStarted", "تم بدء الشاشة الموسعة"),
        ("toastStopped", "تم إيقاف الشاشة الموسعة"),
        ("toastCopied", "تم نسخ رابط البث إلى الحافظة"),
        ("toastDoctorFailed", "فشل تشغيل الفحص"),
        ("toastFixDone", "انتهى الإصلاح التلقائي"),
        ("toastFixFailed", "فشل الإصلاح التلقائي"),
        ("toastSetFailed", "فشل تطبيق إعدادات الشاشة"),
    ])
    },
);

pub fn tr(lang: Lang, key: &str) -> &str {
    let dict = match lang {
        Lang::En => &*EN,
        Lang::Ar => &*AR,
    };
    dict.get(key)
        .copied()
        .or_else(|| EN.get(key).copied())
        .unwrap_or(key)
}

pub fn format(lang: Lang, key: &str, n: u32) -> String {
    tr(lang, key).replace("{n}", &n.to_string())
}

#[derive(serde::Serialize, serde::Deserialize, Default, Debug)]
struct Prefs {
    language: Option<String>,
}

fn prefs_path() -> PathBuf {
    glib::user_config_dir().join("orbiscreen").join("gui.json")
}

pub fn load_lang() -> Lang {
    std::fs::read_to_string(prefs_path())
        .ok()
        .and_then(|s| serde_json::from_str::<Prefs>(&s).ok())
        .and_then(|p| p.language.map(|l| Lang::from_code(&l)))
        .unwrap_or_default()
}

pub fn save_lang(lang: Lang) {
    let path = prefs_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let prefs = Prefs {
        language: Some(lang.code().to_string()),
    };
    if let Ok(json) = serde_json::to_string_pretty(&prefs) {
        let _ = std::fs::write(path, json);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn translates_known_keys_in_both_languages() {
        assert_eq!(tr(Lang::En, "startDisplay"), "Start Extended Display");
        assert_eq!(tr(Lang::Ar, "startDisplay"), "بدء الشاشة الموسعة");
    }

    #[test]
    fn dictionaries_cover_the_same_keys() {
        for key in EN.keys() {
            assert!(AR.contains_key(key), "arabic dictionary misses {key}");
        }
        for key in AR.keys() {
            assert!(EN.contains_key(key), "english dictionary misses {key}");
        }
    }

    #[test]
    fn unknown_keys_return_the_key() {
        assert_eq!(tr(Lang::En, "definitely_missing"), "definitely_missing");
    }

    #[test]
    fn formats_device_counts() {
        assert_eq!(
            format(Lang::En, "devicesConnected", 3),
            "3 Devices Connected"
        );
        assert_eq!(format(Lang::Ar, "devicesConnected", 3), "3 أجهزة متصلة");
    }
}
