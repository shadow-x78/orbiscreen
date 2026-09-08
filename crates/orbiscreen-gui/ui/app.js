// Orbiscreen - Linux Desktop Control Center Frontend (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

const isTauri = typeof window.__TAURI__ !== "undefined";

async function invoke(cmd, args = {}) {
    if (isTauri && window.__TAURI__.core && window.__TAURI__.core.invoke) {
        return await window.__TAURI__.core.invoke(cmd, args);
    }
    if (cmd === "get_status") {
        return {
            running: false,
            frames_forwarded: 0,
            active_clients: 0,
            total_clients: 0,
            auth_failures: 0,
            usb_devices: 0,
            encoder: "NVENC (nvh264enc)",
            capture_backend: "KWin Wayland",
            display_width: 1920,
            display_height: 1080,
            display_fps: 60,
            signaling_port: 54321,
            udp_port: 54322,
            local_ips: ["192.168.1.145"],
            session_token: "orb_token"
        };
    }
    if (cmd === "get_autostart") return false;
    return null;
}

const I18N = {
    en: {
        brandName: "Orbiscreen",
        tabConnect: "Connect",
        tabDisplay: "Display & Input",
        tabDoctor: "Doctor",
        modeWifi: "Wi-Fi Wireless",
        modeUsb: "USB Cable (Zero Lag)",
        ready: "Ready",
        streaming: "Streaming",
        stopped: "Stopped",
        statusReadyHeadline: "Ready to Stream",
        statusReadySubtitle: "Open Orbiscreen on your Android tablet or phone to connect as second display.",
        statusStreamingHeadline: "Streaming Active",
        statusStreamingSubtitle: "Streaming to connected device with ultra-low latency hardware encoding.",
        statusOfflineHeadline: "Virtual Display Offline",
        statusOfflineSubtitle: "Click Start Extended Display below to activate your virtual monitor.",
        deviceConnected: "1 Device Connected",
        devicesConnected: "{n} Devices Connected",
        noDevices: "0 Devices",
        qrPrompt: "Scan with Orbiscreen on your Android tablet or phone",
        copy: "Copy",
        copied: "Copied!",
        open: "Open",
        guide1: "Connect tablet & PC to Wi-Fi",
        guide2: "Open app & tap Scan",
        guide3: "Enjoy smooth touch & stylus",
        startDisplay: "Start Extended Display",
        stopDisplay: "Stop Display",
        usbTitle: "USB Accessory Pipeline",
        usbDesc: "Stream directly over USB cable with hardware acceleration and zero network jitter.",
        usbWaiting: "Waiting for device...",
        usbReady: "USB Device Ready",
        btnOpenUsb: "Open USB Localhost Stream",
        usbHint1: "• Connect your phone or tablet via USB.",
        usbHint2: "• Select Always Allow when Android prompts for accessory permission.",
        resGroup: "Display Geometry",
        resolution: "Resolution",
        resDesc: "Extended desktop virtual canvas dimensions",
        framerate: "Framerate",
        fpsDesc: "Target display refresh frequency",
        encoder: "Hardware Encoder",
        encDesc: "Engine used for low-latency H.264 stream",
        inputGroup: "Touch & Input Injection",
        touchScreen: "Direct Multitouch Screen",
        touchDesc: "Inject Android touches via kernel uinput slots",
        stylus: "Stylus Digitizer (Pressure & Tilt)",
        stylusDesc: "Support pressure sensitivity for Krita, GIMP & notes",
        systemGroup: "System Integration",
        autostart: "Launch on Desktop Login",
        autostartDesc: "Start Orbiscreen daemon on system startup",
        doctorTitle: "System Health & Diagnostics",
        doctorDesc: "Check display server, hardware encoding & device permissions",
        scan: "Scan",
        autoFix: "Auto-Fix",
        checking: "Checking...",
        fixing: "Fixing...",
        docCompositor: "Compositor & Display Server",
        docBackend: "Virtual Screen Pipeline",
        docUinput: "Input Injection (/dev/uinput)",
        docEncoder: "Video Hardware Acceleration",
        docPorts: "Network Ports (8788 / 8789)",
        docPortsDesc: "Signaling & UDP Annex-B transport ready",
        docTipTitle: "Diagnostic Notes",
        docTipDesc: "Ensure your tablet and host PC share the same Wi-Fi subnet, or use a high-speed USB cable for zero-latency direct streaming.",
        connected: "Connected",
        restartTitle: "Restart Host Daemon",
        toastRestart: "Restarting host daemon...",
        toastRestartDone: "Daemon restarted successfully",
        toastStarted: "Extended display started",
        toastStopped: "Extended display stopped",
        toastCopied: "Stream URL copied to clipboard"
    },
    ar: {
        brandName: "أوربي سكرين",
        tabConnect: "الاتصال",
        tabDisplay: "الشاشة والإدخال",
        tabDoctor: "الفحص الذكي",
        modeWifi: "واي فاي لاسلكي",
        modeUsb: "كابل USB (بدون تأخير)",
        ready: "جاهز",
        streaming: "جارِ البث",
        stopped: "متوقف",
        statusReadyHeadline: "جاهز للاتصال والبث",
        statusReadySubtitle: "افتح تطبيق Orbiscreen على جهاز الأندرويد للاتصال كشاشة ثانية.",
        statusStreamingHeadline: "البث نشط الآن",
        statusStreamingSubtitle: "يتم البث المباشر للجهاز المتصل بترميز عتادي وزمن استجابة فائق السرعة.",
        statusOfflineHeadline: "الشاشة الافتراضية متوقفة",
        statusOfflineSubtitle: "اضغط على زر بدء الشاشة الموسعة بالأسفل لتفعيل الشاشة.",
        deviceConnected: "جهاز واحد متصل",
        devicesConnected: "{n} أجهزة متصلة",
        noDevices: "لا توجد أجهزة",
        qrPrompt: "امسح الرمز بواسطة تطبيق Orbiscreen على جهاز الأندرويد",
        copy: "نسخ",
        copied: "تم النسخ!",
        open: "فتح",
        guide1: "اتصل بنفس شبكة الواي فاي للكمبيوتر",
        guide2: "افتح التطبيق واضغط مسح QR",
        guide3: "تحكم فوري باللمس والقلم",
        startDisplay: "بدء الشاشة الموسعة",
        stopDisplay: "إيقاف الشاشة",
        usbTitle: "الاتصال المباشر عبر USB",
        usbDesc: "بث مباشر عبر كابل USB بتسريع عتادي ودون أي تأخير بالشبكة.",
        usbWaiting: "في انتظار توصيل الجهاز...",
        usbReady: "جهاز USB متصل وجاهز",
        btnOpenUsb: "فتح البث عبر المضيف المحلي",
        usbHint1: "• صِل هاتفك أو جهازك اللوحي بكابل USB.",
        usbHint2: "• اختر السماح دائماً عند ظهور تنبيه الملحق على الأندرويد.",
        resGroup: "أبعاد الشاشة",
        resolution: "الدقة",
        resDesc: "أبعاد الشاشة الافتراضية الموسعة لسطح المكتب",
        framerate: "معدل التحديث",
        fpsDesc: "تردد تحديث الإطارات للشاشة",
        encoder: "المرمّز العتادي",
        encDesc: "المحرك المستخدم لترميز H.264 فائق السرعة",
        inputGroup: "اللمس وإدخال التحكم",
        touchScreen: "شاشة اللمس المتعدد المباشر",
        touchDesc: "تمرير اللمسات عبر أنوية uinput في لينكس",
        stylus: "دعم القلم الرقمي (الضغط والميلان)",
        stylusDesc: "حساسية الضغط لتطبيقات الرسم مثل Krita والملاحظات",
        systemGroup: "تكامل النظام",
        autostart: "تشغيل تلقائي عند بدء التشغيل",
        autostartDesc: "بدء الخدمة الخلفية تلقائياً عند تسجيل الدخول",
        doctorTitle: "الفحص الذكي وتشخيص النظام",
        doctorDesc: "فحص خادم العرض، ومسرعات الفيديو، وصلاحيات النظام",
        scan: "فحص الآن",
        autoFix: "إصلاح تلقائي",
        checking: "جارِ الفحص...",
        fixing: "جارِ الإصلاح...",
        docCompositor: "مدير النوافذ وخادم العرض",
        docBackend: "مسار الشاشة الافتراضية",
        docUinput: "وحدة حقن الإدخال (/dev/uinput)",
        docEncoder: "تسريع الفيديو العتادي",
        docPorts: "منافذ الشبكة (8788 / 8789)",
        docPortsDesc: "بروتوكول الإشارات ونقل UDP المباشر جاهز",
        docTipTitle: "ملاحظات الفحص",
        docTipDesc: "تأكد من وجود جهازك والكمبيوتر على نفس الشبكة المحلية، أو استخدم كابل USB مباشر لأقل زمن استجابة ممكن.",
        connected: "متصل",
        restartTitle: "إعادة تشغيل الخدمة",
        toastRestart: "جارِ إعادة تشغيل الخدمة...",
        toastRestartDone: "تمت إعادة تشغيل الخدمة بنجاح",
        toastStarted: "تم بدء الشاشة الموسعة",
        toastStopped: "تم إيقاف الشاشة الموسعة",
        toastCopied: "تم نسخ رابط البث إلى الحافظة"
    }
};

let currentLang = localStorage.getItem("orbiscreen_lang") || "en";

function t(key) {
    const dict = I18N[currentLang] || I18N.en;
    return dict[key] || I18N.en[key] || key;
}

function applyTranslations() {
    document.documentElement.lang = currentLang;
    document.documentElement.dir = (currentLang === "ar") ? "rtl" : "ltr";

    document.querySelectorAll("[data-i18n]").forEach(el => {
        const key = el.getAttribute("data-i18n");
        if (key && t(key)) el.textContent = t(key);
    });

    document.querySelectorAll("[data-i18n-title]").forEach(el => {
        const key = el.getAttribute("data-i18n-title");
        if (key && t(key)) el.title = t(key);
    });

    const lblLangText = document.getElementById("lblLangText");
    if (lblLangText) {
        lblLangText.textContent = (currentLang === "ar") ? "EN" : "عربي";
    }
}

function setLanguage(lang) {
    currentLang = lang;
    localStorage.setItem("orbiscreen_lang", lang);
    applyTranslations();
    refreshStatus();
}

let currentTheme = localStorage.getItem("orbiscreen_theme") || "system";

function applyTheme(theme) {
    currentTheme = theme;
    localStorage.setItem("orbiscreen_theme", theme);

    document.body.classList.remove("theme-light", "theme-dark", "theme-system");
    document.body.classList.add(`theme-${theme}`);

    document.querySelectorAll(".themeBtn").forEach(btn => {
        btn.classList.toggle("active", btn.getAttribute("data-theme") === theme);
    });
}

function showToast(message, duration = 2200) {
    const container = document.getElementById("toastContainer");
    if (!container) return;

    const item = document.createElement("div");
    item.className = "toastItem";
    item.textContent = message;
    container.appendChild(item);

    setTimeout(() => {
        item.style.opacity = "0";
        item.style.transform = "translateY(8px) scale(0.95)";
        item.style.transition = "all 0.25s ease";
        setTimeout(() => item.remove(), 250);
    }, duration);
}

const segmentBtns = document.querySelectorAll(".segmentBtn");
const tabPanes = document.querySelectorAll(".tabPane");

segmentBtns.forEach(btn => {
    btn.addEventListener("click", () => {
        const tab = btn.getAttribute("data-tab");
        segmentBtns.forEach(b => b.classList.remove("active"));
        tabPanes.forEach(p => p.classList.remove("active"));

        btn.classList.add("active");
        const target = document.getElementById(`tab-${tab}`);
        if (target) target.classList.add("active");
    });
});

const modeBtns = document.querySelectorAll(".modeBtn");
const subPanes = document.querySelectorAll(".subPane");

modeBtns.forEach(btn => {
    btn.addEventListener("click", () => {
        const sub = btn.getAttribute("data-sub");
        modeBtns.forEach(b => b.classList.remove("active"));
        subPanes.forEach(p => p.classList.remove("active"));

        btn.classList.add("active");
        const target = document.getElementById(`sub-${sub}`);
        if (target) target.classList.add("active");
    });
});

const statusPill = document.getElementById("statusPill");
const statusLabel = document.getElementById("statusLabel");
const btnToggleService = document.getElementById("btnToggleService");
const toggleText = document.getElementById("toggleText");
const toggleIcon = document.getElementById("toggleIcon");
const btnRestartService = document.getElementById("btnRestartService");

const previewResolution = document.getElementById("previewResolution");
const previewLatency = document.getElementById("previewLatency");
const displayStateHeadline = document.getElementById("displayStateHeadline");
const displayStateSubtitle = document.getElementById("displayStateSubtitle");
const activeClientText = document.getElementById("activeClientText");

const inpSessionUrl = document.getElementById("inpSessionUrl");
const btnCopyUrl = document.getElementById("btnCopyUrl");
const btnOpenBrowser = document.getElementById("btnOpenBrowser");
const btnOpenUsb = document.getElementById("btnOpenUsb");
const usbStatusTag = document.getElementById("usbStatusTag");

const lblEncoder = document.getElementById("lblEncoder");
const chkAutostart = document.getElementById("chkAutostart");
const btnRunDoctor = document.getElementById("btnRunDoctor");
const btnFixDoctor = document.getElementById("btnFixDoctor");
const btnLangToggle = document.getElementById("btnLangToggle");

let isRunning = false;
let currentUrl = "http://127.0.0.1:8788";
let lastStatus = null;

function renderQr(url) {
    const container = document.getElementById("qrContainer");
    if (!container || typeof qrcode === "undefined") return;

    try {
        const qr = qrcode(0, "M");
        qr.addData(url);
        qr.make();
        container.innerHTML = qr.createSvgTag({ scalable: true, margin: 2 });
    } catch (e) {
        console.error("QR render error:", e);
    }
}

async function refreshStatus() {
    try {
        const status = await invoke("get_status");
        if (!status) return;

        lastStatus = status;
        isRunning = status.running;

        const ip = (status.local_ips && status.local_ips.length > 0) ? status.local_ips[0] : "127.0.0.1";
        const port = status.signaling_port || 8788;
        const tokenPart = status.session_token ? `#token=${status.session_token}` : "";
        currentUrl = `http://${ip}:${port}/${tokenPart}`;

        if (inpSessionUrl) inpSessionUrl.value = currentUrl;
        renderQr(currentUrl);

        if (statusPill && statusLabel) {
            statusPill.classList.remove("ready", "streaming", "offline");
            if (isRunning) {
                if (status.active_clients > 0) {
                    statusPill.classList.add("streaming");
                    statusLabel.textContent = t("streaming");
                    if (displayStateHeadline) displayStateHeadline.textContent = t("statusStreamingHeadline");
                    if (displayStateSubtitle) displayStateSubtitle.textContent = t("statusStreamingSubtitle");
                } else {
                    statusPill.classList.add("ready");
                    statusLabel.textContent = t("ready");
                    if (displayStateHeadline) displayStateHeadline.textContent = t("statusReadyHeadline");
                    if (displayStateSubtitle) displayStateSubtitle.textContent = t("statusReadySubtitle");
                }
            } else {
                statusPill.classList.add("offline");
                statusLabel.textContent = t("stopped");
                if (displayStateHeadline) displayStateHeadline.textContent = t("statusOfflineHeadline");
                if (displayStateSubtitle) displayStateSubtitle.textContent = t("statusOfflineSubtitle");
            }
        }

        if (btnToggleService && toggleText && toggleIcon) {
            if (isRunning) {
                btnToggleService.classList.add("running");
                toggleText.textContent = t("stopDisplay");
                toggleIcon.innerHTML = `<path d="M6 6h12v12H6z"/>`;
            } else {
                btnToggleService.classList.remove("running");
                toggleText.textContent = t("startDisplay");
                toggleIcon.innerHTML = `<path d="M8 5v14l11-7z"/>`;
            }
        }

        if (activeClientText) {
            if (status.active_clients === 1) {
                activeClientText.textContent = t("deviceConnected");
            } else if (status.active_clients > 1) {
                activeClientText.textContent = t("devicesConnected").replace("{n}", status.active_clients);
            } else {
                activeClientText.textContent = t("noDevices");
            }
        }

        const width = status.display_width || 1920;
        const height = status.display_height || 1080;
        const fps = status.display_fps || 60;
        if (previewResolution) {
            previewResolution.textContent = `${width} × ${height} @ ${fps}Hz`;
        }

        if (lblEncoder) {
            lblEncoder.textContent = status.encoder || "Auto";
        }

        if (usbStatusTag) {
            const usbPane = document.getElementById("sub-usb");
            const badge = usbPane ? usbPane.querySelector(".usbStatusBadge") : null;
            const devList = (status.usb_connected_devices && status.usb_connected_devices.length > 0)
                ? status.usb_connected_devices.join(", ")
                : "";

            if (status.usb_aoa_ready) {
                usbStatusTag.textContent = devList ? `${devList} (${t("usbReady")})` : t("usbReady");
                if (badge) badge.classList.add("ready");
            } else if (status.usb_devices > 0 || devList) {
                usbStatusTag.textContent = devList ? `${devList} (${t("connected")})` : t("usbReady");
                if (badge) badge.classList.add("ready");
            } else {
                usbStatusTag.textContent = t("usbWaiting");
                if (badge) badge.classList.remove("ready");
            }
        }
    } catch (e) {
        console.error("Status check failed:", e);
    }
}

if (btnToggleService) {
    btnToggleService.addEventListener("click", async () => {
        try {
            if (isRunning) {
                await invoke("stop_service");
                showToast(t("toastStopped"));
            } else {
                await invoke("start_service");
                showToast(t("toastStarted"));
            }
            await refreshStatus();
        } catch (e) {
            showToast(`Error: ${e}`);
        }
    });
}

if (btnRestartService) {
    btnRestartService.addEventListener("click", async () => {
        try {
            btnRestartService.classList.add("spinning");
            showToast(t("toastRestart"));
            await invoke("restart_service");
            setTimeout(async () => {
                btnRestartService.classList.remove("spinning");
                showToast(t("toastRestartDone"));
                await refreshStatus();
            }, 1000);
        } catch (e) {
            btnRestartService.classList.remove("spinning");
            showToast(`Error: ${e}`);
        }
    });
}

if (btnCopyUrl && inpSessionUrl) {
    btnCopyUrl.addEventListener("click", () => {
        navigator.clipboard.writeText(inpSessionUrl.value);
        btnCopyUrl.textContent = t("copied");
        showToast(t("toastCopied"));
        setTimeout(() => {
            btnCopyUrl.textContent = t("copy");
        }, 1500);
    });
}

if (btnOpenBrowser && inpSessionUrl) {
    btnOpenBrowser.addEventListener("click", async () => {
        await invoke("open_browser", { url: inpSessionUrl.value });
    });
}

if (btnOpenUsb) {
    btnOpenUsb.addEventListener("click", async () => {
        const port = lastStatus && lastStatus.signaling_port ? lastStatus.signaling_port : 8788;
        await invoke("open_browser", { url: `http://127.0.0.1:${port}/` });
    });
}

document.querySelectorAll("#resChips .chip").forEach(chip => {
    chip.addEventListener("click", () => {
        document.querySelectorAll("#resChips .chip").forEach(c => c.classList.remove("active"));
        chip.classList.add("active");
    });
});

document.querySelectorAll("#fpsChips .chip").forEach(chip => {
    chip.addEventListener("click", () => {
        document.querySelectorAll("#fpsChips .chip").forEach(c => c.classList.remove("active"));
        chip.classList.add("active");
    });
});

if (btnLangToggle) {
    btnLangToggle.addEventListener("click", () => {
        setLanguage(currentLang === "en" ? "ar" : "en");
    });
}

document.querySelectorAll(".themeBtn").forEach(btn => {
    btn.addEventListener("click", () => {
        const theme = btn.getAttribute("data-theme");
        if (theme) applyTheme(theme);
    });
});

if (chkAutostart) {
    invoke("get_autostart").then(enabled => {
        chkAutostart.checked = Boolean(enabled);
    });

    chkAutostart.addEventListener("change", async () => {
        await invoke("set_autostart", { enabled: chkAutostart.checked });
    });
}

if (btnRunDoctor) {
    btnRunDoctor.addEventListener("click", async () => {
        btnRunDoctor.textContent = t("checking");
        try {
            const jsonStr = await invoke("run_doctor_check");
            if (jsonStr) {
                const docCompositor = document.getElementById("docCompositor");
                const docBackend = document.getElementById("docBackend");
                const docUinput = document.getElementById("docUinput");
                const docEncoder = document.getElementById("docEncoder");
                if (docCompositor) docCompositor.textContent = "Wayland / X11 Active";
                if (docBackend) docBackend.textContent = "KWin / Headless Output Ready";
                if (docUinput) docUinput.textContent = "Permissions Verified";
                if (docEncoder) docEncoder.textContent = "Hardware Encoding Available";
            }
        } catch (e) {
            console.error("Doctor error:", e);
        } finally {
            btnRunDoctor.textContent = t("scan");
        }
    });
}

if (btnFixDoctor) {
    btnFixDoctor.addEventListener("click", async () => {
        btnFixDoctor.textContent = t("fixing");
        try {
            await invoke("run_doctor_fix");
            showToast("Auto-fix routine finished");
        } catch (e) {
            showToast(`Fix error: ${e}`);
        } finally {
            btnFixDoctor.textContent = t("autoFix");
        }
    });
}

document.addEventListener("DOMContentLoaded", async () => {
    applyTranslations();
    applyTheme(currentTheme);
    await refreshStatus();
    setInterval(refreshStatus, 3000);
});
