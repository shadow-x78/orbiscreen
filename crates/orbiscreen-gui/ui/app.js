// Orbiscreen - Linux Desktop Control Center Frontend (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

const isTauri = typeof window.__TAURI__ !== "undefined";

async function invoke(cmd, args = {}) {
    if (isTauri && window.__TAURI__.core && window.__TAURI__.core.invoke) {
        return await window.__TAURI__.core.invoke(cmd, args);
    }
    console.log(`[Mock/Web] invoke: ${cmd}`, args);
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
        confine: "Confine Pointer to Display",
        confineDesc: "Trap mouse cursor within virtual display area",
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
        docPorts: "Network Ports (54321 / 54322)",
        docPortsDesc: "Signaling & UDP Annex-B transport ready",
        docTipTitle: "Diagnostic Notes",
        docTipDesc: "Ensure your tablet and host PC share the same Wi-Fi subnet, or use a high-speed USB cable for zero-latency direct streaming.",
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
        statusStreamingSubtitle: "يتم البث للجهاز المتصل بترميز عتادي فائق السرعة وبدون تأخير.",
        statusOfflineHeadline: "الشاشة الافتراضية متوقفة",
        statusOfflineSubtitle: "انقر على زر بدء الشاشة الممتدة لتفعيل مساحة العرض الإضافية.",
        deviceConnected: "جهاز متصل واحد",
        devicesConnected: "{n} أجهزة متصلة",
        noDevices: "لا أجهزة متصلة",
        qrPrompt: "امسح الرمز بواسطة Orbiscreen على هاتفك أو جهازك اللوحي",
        copy: "نسخ",
        copied: "تم النسخ!",
        open: "فتح",
        guide1: "اتصل بنفس شبكة الواي فاي للكمبيوتر والتابلت",
        guide2: "افتح التطبيق واضغط على مسح الرمز",
        guide3: "استمتع باللمس السلس ودعم القلم والضغط",
        startDisplay: "بدء الشاشة الممتدة",
        stopDisplay: "إيقاف الشاشة",
        usbTitle: "بث USB فائق السرعة",
        usbDesc: "بث مباشر عبر كابل USB بتسريع عتادي ودون أي تأثر بجودة الشبكة اللاسلكية.",
        usbWaiting: "في انتظار توصيل الجهاز...",
        usbReady: "جهاز USB متصل وجاهز",
        btnOpenUsb: "فتح بث USB المحلي",
        usbHint1: "• قم بتوصيل هاتفك أو جهازك اللوحي عبر كابل USB.",
        usbHint2: "• اختر السماح دائماً عند ظهور نافذة إذن ملحق USB في أندرويد.",
        resGroup: "أبعاد ودقة العرض",
        resolution: "دقة الشاشة",
        resDesc: "أبعاد مساحة سطح المكتب الافتراضية الإضافية",
        framerate: "معدل التحديث (الإطارات)",
        fpsDesc: "تردد تحديث الشاشة الافتراضية المستهدف",
        encoder: "مرمّز الفيديو العتادي",
        encDesc: "المحرك العتادي المستخدم لضغط وبث الفيديو H.264",
        inputGroup: "اللمس وإدخال البيانات",
        touchScreen: "شاشة لمس متعددة مباشرة",
        touchDesc: "تمرير لمسات أندرويد لنواة لينكس عبر uinput مباشرة",
        stylus: "دعم القلم الذكي (الضغط والميلان)",
        stylusDesc: "دعم مستويات الضغط لتطبيقات الرسم مثل Krita والملاحظات",
        confine: "حصر المؤشر داخل الشاشة",
        confineDesc: "منع مؤشر الفأرة من الخروج من مساحة الشاشة الافتراضية",
        systemGroup: "تكامل النظام",
        autostart: "التشغيل مع بدء تشغيل النظام",
        autostartDesc: "بدء تشغيل خدمة Orbiscreen تلقائياً عند تسجيل الدخول",
        doctorTitle: "فحص وتشخيص صحة النظام",
        doctorDesc: "فحص خادم العرض والترميز العتادي وصلاحيات الإدخال",
        scan: "فحص",
        autoFix: "إصلاح تلقائي",
        checking: "جارِ الفحص...",
        fixing: "جارِ الإصلاح...",
        docCompositor: "مدير النوافذ وخادم العرض",
        docBackend: "خط أنابيب الشاشة الافتراضية",
        docUinput: "حقن الإدخال (/dev/uinput)",
        docEncoder: "تسريع الفيديو العتادي",
        docPorts: "منافذ الشبكة (54321 / 54322)",
        docPortsDesc: "منافذ الإشارات ونقل بيانات UDP جاهزة",
        docTipTitle: "إرشادات التشخيص",
        docTipDesc: "تأكد من اتصال الجهاز والكمبيوتر بنفس الشبكة المحلية أو استخدم كابل USB لتجربة فائقة السرعة.",
        restartTitle: "إعادة تشغيل الخدمة",
        toastRestart: "جارِ إعادة تشغيل الخدمة...",
        toastRestartDone: "تمت إعادة تشغيل الخدمة بنجاح",
        toastStarted: "تم تشغيل الشاشة الممتدة",
        toastStopped: "تم إيقاف الشاشة الممتدة",
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
        if (key) el.textContent = t(key);
    });

    document.querySelectorAll("[data-i18n-title]").forEach(el => {
        const key = el.getAttribute("data-i18n-title");
        if (key) el.title = t(key);
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

const subTabBtns = document.querySelectorAll(".modeBtn, .subTabBtn");
const subTabBtns = document.querySelectorAll(".modeBtn");
const subPanes = document.querySelectorAll(".subPane");

subTabBtns.forEach(btn => {
    btn.addEventListener("click", () => {
        const sub = btn.getAttribute("data-sub");
        subTabBtns.forEach(b => b.classList.remove("active"));
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
const activeClientPill = document.getElementById("activeClientPill");
const activeClientText = document.getElementById("activeClientText");

const inpSessionUrl = document.getElementById("inpSessionUrl");
const btnCopyUrl = document.getElementById("btnCopyUrl");
const btnOpenBrowser = document.getElementById("btnOpenBrowser");
const btnOpenUsb = document.getElementById("btnOpenUsb");
const usbStatusTag = document.getElementById("usbStatusTag");

const lblHostIp = document.getElementById("lblHostIp");
const lblBackend = document.getElementById("lblBackend");
const lblEncoder = document.getElementById("lblEncoder");

const chkAutostart = document.getElementById("chkAutostart");
const btnRunDoctor = document.getElementById("btnRunDoctor");
const btnFixDoctor = document.getElementById("btnFixDoctor");
const btnLangToggle = document.getElementById("btnLangToggle");

let isRunning = false;
let currentUrl = "http://127.0.0.1:54321";

function renderQrCode(text) {
    const container = document.getElementById("qrContainer");
    if (!container || typeof qrcode === "undefined") return;
    try {
        const qr = qrcode(0, "M");
        qr.addData(text);
        qr.make();
        container.innerHTML = qr.createSvgTag(4, 2);
        container.innerHTML = qr.createSvgTag({ scalable: true, margin: 2, cellSize: 4 });
    } catch (e) {
        console.warn("QR code render failed:", e);
    }
}

async function refreshStatus() {
    try {
        const status = await invoke("get_status");
        if (!status) return;

        isRunning = status.running;

        const ip = (status.local_ips && status.local_ips.length > 0) ? status.local_ips[0] : "127.0.0.1";
        const port = status.signaling_port || 54321;
        currentUrl = `http://${ip}:${port}`;
        inpSessionUrl.value = currentUrl;
        if (inpSessionUrl) inpSessionUrl.value = currentUrl;

        if (isRunning) {
            if (status.active_clients > 0) {
                statusPill.className = "statusPill online";
                statusLabel.textContent = "Streaming";
                if (displayStateHeadline) displayStateHeadline.textContent = "Extended Screen Streaming";
                if (displayStateSubtitle) displayStateSubtitle.textContent = `Streaming to ${status.active_clients} connected device with ultra-low latency hardware encoding.`;
                if (activeClientPill) activeClientPill.classList.remove("hidden");
                if (activeClientText) activeClientText.textContent = `${status.active_clients} Device Connected (Android Tablet)`;
                if (statusPill) statusPill.className = "statusPill online";
                if (statusLabel) statusLabel.textContent = t("streaming");
                if (displayStateHeadline) displayStateHeadline.textContent = t("statusStreamingHeadline");
                if (displayStateSubtitle) displayStateSubtitle.textContent = t("statusStreamingSubtitle");
                if (activeClientText) {
                    activeClientText.textContent = (status.active_clients === 1)
                        ? t("deviceConnected")
                        : t("devicesConnected").replace("{n}", status.active_clients);
                }
            } else {
                statusPill.className = "statusPill ready";
                statusLabel.textContent = "Ready";
                if (displayStateHeadline) displayStateHeadline.textContent = "Ready to Stream";
                if (displayStateSubtitle) displayStateSubtitle.textContent = "Open Orbiscreen on your Android tablet or phone to connect as second display.";
                if (activeClientPill) activeClientPill.classList.add("hidden");
                if (statusPill) statusPill.className = "statusPill ready";
                if (statusLabel) statusLabel.textContent = t("ready");
                if (displayStateHeadline) displayStateHeadline.textContent = t("statusReadyHeadline");
                if (displayStateSubtitle) displayStateSubtitle.textContent = t("statusReadySubtitle");
                if (activeClientText) activeClientText.textContent = t("noDevices");
            }
            if (btnToggleService) {
                btnToggleService.className = "btnMasterAction danger";
                if (toggleText) toggleText.textContent = "Stop Display";
                if (toggleText) toggleText.textContent = t("stopDisplay");
                if (toggleIcon) toggleIcon.innerHTML = `<path d="M6 6h12v12H6z"/>`;
            }
        } else {
            statusPill.className = "statusPill offline";
            statusLabel.textContent = "Stopped";
            if (displayStateHeadline) displayStateHeadline.textContent = "Virtual Display Offline";
            if (displayStateSubtitle) displayStateSubtitle.textContent = "Click Start Display above to activate your extended desktop workspace.";
            if (activeClientPill) activeClientPill.classList.add("hidden");
            if (statusPill) statusPill.className = "statusPill offline";
            if (statusLabel) statusLabel.textContent = t("stopped");
            if (displayStateHeadline) displayStateHeadline.textContent = t("statusOfflineHeadline");
            if (displayStateSubtitle) displayStateSubtitle.textContent = t("statusOfflineSubtitle");
            if (activeClientText) activeClientText.textContent = t("noDevices");
            if (btnToggleService) {
                btnToggleService.className = "btnMasterAction";
                if (toggleText) toggleText.textContent = "Start Display";
                if (toggleText) toggleText.textContent = t("startDisplay");
                if (toggleIcon) toggleIcon.innerHTML = `<path d="M8 5v14l11-7z"/>`;
            }
        }

        const width = status.display_width || 1920;
        const height = status.display_height || 1080;
        const fps = status.display_fps || 60;
        if (previewResolution) previewResolution.textContent = `${width} × ${height} @ ${fps}Hz`;
        if (previewLatency) previewLatency.textContent = (status.udp_port) ? "UDP ~3ms" : "HTTP Transport";

        if (lblHostIp) lblHostIp.textContent = `Host: ${ip}`;
        if (lblBackend) lblBackend.textContent = status.capture_backend || "KWin Wayland";
        if (lblEncoder) lblEncoder.textContent = status.encoder || "NVENC";

        if (usbStatusTag) {
            if (status.usb_devices > 0) {
                usbStatusTag.className = "usbTag ready";
                usbStatusTag.textContent = `${status.usb_devices} USB Device Ready`;
                usbStatusTag.className = "usbStatusBadge ready";
                usbStatusTag.textContent = t("usbReady");
            } else {
                usbStatusTag.className = "usbTag";
                usbStatusTag.textContent = "Waiting for device...";
                usbStatusTag.className = "usbStatusBadge";
                usbStatusTag.textContent = t("usbWaiting");
            }
        }

        renderQrCode(currentUrl);

    } catch (e) {
        console.warn("Status refresh error:", e);
    }
}

if (btnToggleService) {
    btnToggleService.addEventListener("click", async () => {
        btnToggleService.disabled = true;
        try {
            if (isRunning) {
                await invoke("stop_service");
                showToast(t("toastStopped"));
            } else {
                await invoke("start_service");
                showToast(t("toastStarted"));
            }
        } catch (e) {
            console.error("Service toggle failed:", e);
        }
        setTimeout(async () => {
            await refreshStatus();
            btnToggleService.disabled = false;
        }, 800);
    });
}

if (btnRestartService) {
    btnRestartService.addEventListener("click", async () => {
        btnRestartService.disabled = true;
        showToast(t("toastRestart"));
        try {
            await invoke("restart_service");
            showToast(t("toastRestartDone"));
        } catch (e) {
            console.error("Service restart failed:", e);
        }
        setTimeout(async () => {
            await refreshStatus();
            btnRestartService.disabled = false;
        }, 1000);
    });
}

if (btnCopyUrl) {
    btnCopyUrl.addEventListener("click", async () => {
        try {
            await navigator.clipboard.writeText(currentUrl);
            btnCopyUrl.textContent = "Copied!";
            setTimeout(() => { btnCopyUrl.textContent = "Copy"; }, 1500);
            btnCopyUrl.textContent = t("copied");
            showToast(t("toastCopied"));
            setTimeout(() => { btnCopyUrl.textContent = t("copy"); }, 1500);
        } catch (e) {
            inpSessionUrl.select();
            document.execCommand("copy");
            btnCopyUrl.textContent = "Copied!";
            setTimeout(() => { btnCopyUrl.textContent = "Copy"; }, 1500);
            if (inpSessionUrl) {
                inpSessionUrl.select();
                document.execCommand("copy");
            }
            btnCopyUrl.textContent = t("copied");
            showToast(t("toastCopied"));
            setTimeout(() => { btnCopyUrl.textContent = t("copy"); }, 1500);
        }
    });
}

if (btnOpenBrowser) {
    btnOpenBrowser.addEventListener("click", async () => {
        await invoke("open_browser", { url: currentUrl });
    });
}

if (btnOpenUsb) {
    btnOpenUsb.addEventListener("click", async () => {
        await invoke("open_browser", { url: "http://localhost:54321" });
    });
}

if (btnLangToggle) {
    btnLangToggle.addEventListener("click", () => {
        setLanguage((currentLang === "en") ? "ar" : "en");
    });
}

document.querySelectorAll(".themeBtn").forEach(btn => {
    btn.addEventListener("click", () => {
        const theme = btn.getAttribute("data-theme");
        if (theme) applyTheme(theme);
    });
});

if (chkAutostart) {
    (async () => {
        const enabled = await invoke("get_autostart");
        chkAutostart.checked = !!enabled;
    })();

    chkAutostart.addEventListener("change", async () => {
        await invoke("set_autostart", { enabled: chkAutostart.checked });
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

document.querySelectorAll("#encoderChips .chip").forEach(chip => {
    chip.addEventListener("click", () => {
        document.querySelectorAll("#encoderChips .chip").forEach(c => c.classList.remove("active"));
        chip.classList.add("active");
    });
});

if (btnRunDoctor) {
    btnRunDoctor.addEventListener("click", async () => {
        btnRunDoctor.disabled = true;
        btnRunDoctor.textContent = "Checking...";
        btnRunDoctor.textContent = t("checking");
        try {
            const jsonStr = await invoke("run_doctor_check");
            if (jsonStr) {
                const data = JSON.parse(jsonStr);
                const docCompositor = document.getElementById("docCompositor");
                const docBackend = document.getElementById("docBackend");
                const docUinput = document.getElementById("docUinput");

                if (docCompositor && data.compositor) {
                    docCompositor.textContent = `${data.compositor} (${data.session || "Wayland"})`;
                }
                if (docBackend && data.display_backend) {
                    docBackend.textContent = data.display_backend;
                }
                if (docUinput) {
                    docUinput.textContent = data.uinput_writable ? "/dev/uinput writable (evdev ready)" : "Permission denied (needs udev fix)";
                }
            }
        } catch (e) {
            console.warn("Doctor check error:", e);
        }
        btnRunDoctor.disabled = false;
        btnRunDoctor.textContent = "Scan";
        btnRunDoctor.textContent = t("scan");
    });
}

if (btnFixDoctor) {
    btnFixDoctor.addEventListener("click", async () => {
        btnFixDoctor.disabled = true;
        btnFixDoctor.textContent = "Fixing...";
        btnFixDoctor.textContent = t("fixing");
        try {
            await invoke("run_doctor_fix");
        } catch (e) {
            console.warn("Doctor fix error:", e);
        }
        setTimeout(() => {
            btnFixDoctor.disabled = false;
            btnFixDoctor.textContent = "Auto-Fix";
            btnFixDoctor.textContent = t("autoFix");
            if (btnRunDoctor) btnRunDoctor.click();
        }, 1200);
    });
}

applyTheme(currentTheme);
applyTranslations();
refreshStatus();
setInterval(refreshStatus, 2500);

if (window.location.hash) {
    const hashTab = window.location.hash.replace("#", "");
    const targetBtn = document.querySelector(`.segmentBtn[data-tab="${hashTab}"]`);
    if (targetBtn) targetBtn.click();
}

if (window.location.hash === "#usb") {
    const usbBtn = document.querySelector(`.modeBtn[data-sub="usb"], .subTabBtn[data-sub="usb"]`);
    const usbBtn = document.querySelector(`.modeBtn[data-sub="usb"]`);
    if (usbBtn) usbBtn.click();
}
