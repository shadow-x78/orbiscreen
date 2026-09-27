// Orbiscreen - app.js (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

const I18N = {
    en: {
        btnInputMode: "Input Mode",
        btnKeyboard: "Keyboard",
        btnLock: "Lock Session",
        btnSettings: "Settings",
        btnStats: "Statistics",
        btnHideControls: "Hide Toolbar",
        btnFullscreen: "Fullscreen",
        btnDisconnect: "Disconnect",
        btnShowControls: "Menu",
        btnCloseKeyboard: "Close",
        settingsTitle: "Settings",
        themeTitle: "Theme",
        themeDark: "Dark",
        themeLight: "Light",
        language: "Language",
        fitMode: "Scaling",
        fitContain: "Fit",
        fitCover: "Fill",
        fitNone: "100%",
        perfStats: "Telemetry",
        latency: "Latency",
        resolution: "Resolution",
        encoder: "Encoder",
        hostActions: "Host Actions",
        actionResync: "Resync",
        statusConnecting: "Connecting",
        statusConnectingSub: "Connecting...",
        statusConnectingSub: "Connecting to Linux host...",
        statusConnected: "Connected",
        statusDisconnected: "Disconnected",
        statusStreamError: "Stream Error",
        statusStreamErrorSub: "Reconnecting...",
        statusAuthFailed: "Auth Required",
        statusAuthFailedSub: "Enter session token",
        tokenPlaceholder: "Session token...",
        btnConnect: "Connect",
        btnReconnect: "Reconnect",
        vncActive: "Pointer Locked · Press <kbd>Esc</kbd> to unlock",
        cursorReleased: "Pointer unlocked",
        modeTouch: "Touch",
        modeTouchpad: "Touchpad",
        controlsHidden: "Toolbar hidden",
        toastLocked: "Locked",
        toastLockSent: "Lock sent",
        toastCadSent: "Ctrl+Alt+Del sent",
        toastResynced: "Resynced",
        toastDisconnected: "Disconnected",
        statsTitle: "Statistics",
        statsDelay: "Network delay",
        statsAge: "Current frame age",
        statsReceived: "Received",
        statsFrames: "Frames",
        statsWindow: "last 1 min",
        statusUnsupported: "Unsupported browser",
        statusUnsupportedSub: "This client needs the WebCodecs VideoDecoder API. Open the page in Chrome, Brave, Edge, or another Chromium browser."
    },
    ar: {
        btnInputMode: "وضع الإدخال",
        btnKeyboard: "لوحة المفاتيح",
        btnLock: "قفل الجلسة",
        btnSettings: "الإعدادات",
        btnHideControls: "إخفاء شريط الأدوات",
        btnFullscreen: "ملء الشاشة",
        btnDisconnect: "قطع الاتصال",
        btnRestoreToolbar: "إظهار شريط الأدوات",
        btnCloseKeyboard: "إغلاق",
        settingsTitle: "الإعدادات",
        themeTitle: "المظهر (الثيم)",
        themeDark: "داكن",
        themeLight: "فاتح",
        language: "اللغة",
        fitMode: "تنسيق العرض",
        fitContain: "احتواء",
        fitCover: "ملء الشاشة",
        fitNone: "100%",
        perfStats: "بيانات الأداء",
        latency: "التأخير",
        resolution: "الدقة",
        encoder: "المرمّز",
        hostActions: "إجراءات المضيف",
        actionResync: "إعادة المزامنة",
        statusConnecting: "جارِ الاتصال",
        statusConnectingSub: "الاتصال بمضيف لينكس...",
        statusConnected: "متصل",
        statusDisconnected: "انقطع الاتصال",
        statusStreamError: "خطأ في البث",
        statusStreamErrorSub: "جارِ إعادة المحاولة...",
        statusAuthFailed: "المصادقة مطلوبة",
        statusAuthFailedSub: "أدخل رمز الجلسة للمتابعة",
        tokenPlaceholder: "رمز الجلسة...",
        btnConnect: "اتصال",
        btnReconnect: "إعادة الاتصال",
        vncActive: "المؤشر محصور · اضغط <kbd>Esc</kbd> للتحرير",
        cursorReleased: "تم تحرير المؤشر",
        modeTouch: "اللمس",
        modeTouchpad: "لوحة اللمس",
        controlsHidden: "تم إخفاء الشريط",
        toastLocked: "تم القفل",
        toastLockSent: "تم إرسال أمر القفل",
        toastCadSent: "تم إرسال Ctrl+Alt+Del",
        toastResynced: "تمت المزامنة",
        toastDisconnected: "تم قطع الاتصال",
        statusUnsupported: "متصفح غير مدعوم",
        statusUnsupportedSub: "يحتاج هذا العميل إلى WebCodecs VideoDecoder. افتح الصفحة في Chrome أو Brave أو Edge أو متصفح Chromium آخر."
    }
};

let currentLang = localStorage.getItem("orbiscreen_web_lang") || "en";

function t(key) {
    const dict = I18N[currentLang] || I18N.en;
    return dict[key] || I18N.en[key] || key;
}

function applyTranslations() {
    document.documentElement.lang = currentLang;
    document.documentElement.dir = (currentLang === "ar") ? "rtl" : "ltr";

    document.querySelectorAll("[data-i18n]").forEach((el) => {
        const k = el.getAttribute("data-i18n");
        if (k && t(k)) el.textContent = t(k);
    });
    document.querySelectorAll("[data-i18n-title]").forEach((el) => {
        const k = el.getAttribute("data-i18n-title");
        if (k && t(k)) el.title = t(k);
    });
    document.querySelectorAll("[data-i18n-placeholder]").forEach((el) => {
        const k = el.getAttribute("data-i18n-placeholder");
        if (k && t(k)) el.placeholder = t(k);
    });
    document.querySelectorAll("[data-i18n-html]").forEach((el) => {
        const k = el.getAttribute("data-i18n-html");
        if (k && t(k)) el.innerHTML = t(k);
    });

    const lblWebLang = document.getElementById("lblWebLang");
    if (lblWebLang) {
        lblWebLang.textContent = (currentLang === "ar") ? "EN" : "عربي";
    }

    document.querySelectorAll("#webLangChips .chipBtn").forEach((btn) => {
        btn.classList.toggle("active", btn.getAttribute("data-lang") === currentLang);
    });
}

function setLanguage(lang) {
    currentLang = lang;
    localStorage.setItem("orbiscreen_web_lang", lang);
    applyTranslations();
}

let currentTheme = localStorage.getItem("orbiscreen_web_theme") || "dark";

function applyTheme(theme) {
    currentTheme = theme;
    localStorage.setItem("orbiscreen_web_theme", theme);

    if (theme === "light") {
        document.body.classList.add("theme-light");
    } else {
        document.body.classList.remove("theme-light");
    }

    const iconDark = document.getElementById("iconThemeDark");
    const iconLight = document.getElementById("iconThemeLight");
    if (iconDark && iconLight) {
        if (theme === "light") {
            iconDark.classList.add("hidden");
            iconLight.classList.remove("hidden");
        } else {
            iconLight.classList.add("hidden");
            iconDark.classList.remove("hidden");
        }
    }

    document.querySelectorAll("#webThemeChips .chipBtn").forEach((btn) => {
        btn.classList.toggle("active", btn.getAttribute("data-theme") === theme);
    });
}

const statusTitle = document.getElementById("statusTitle");
const statusSubtitle = document.getElementById("statusSubtitle");
const overlayEl = document.getElementById("overlay");
const brandLogo = document.getElementById("brandLogo");
const statusSpinner = document.getElementById("statusSpinner");
const statusIcon = document.getElementById("statusIcon");
const btnReconnect = document.getElementById("btnReconnect");
const stageEl = document.getElementById("stage");
const videoEl = document.getElementById("remoteVideo");
const touchIndicator = document.getElementById("touchIndicator");
const controlToolbar = document.getElementById("controlToolbar");
const btnShowControls = document.getElementById("btnShowControls");
const hostNameEl = document.getElementById("hostName");
const hostInfoEl = document.getElementById("hostInfo");

function setOverlayState(state, title, subtitle) {
    if (overlayEl) overlayEl.classList.remove("hidden");
    if (statusTitle && title) statusTitle.textContent = title;
    if (statusSubtitle && subtitle) statusSubtitle.textContent = subtitle;

    releaseControl();
    hideControlsChrome();
    if (keyboardDrawer) keyboardDrawer.classList.add("hidden");
    if (settingsModal) settingsModal.classList.add("hidden");

    if (state === "connecting") {
        if (brandLogo) brandLogo.classList.add("hidden");
        if (statusSpinner) statusSpinner.classList.remove("hidden");
        if (btnReconnect) btnReconnect.classList.add("hidden");
    } else if (state === "unsupported") {
        if (statusSpinner) statusSpinner.classList.add("hidden");
        if (brandLogo) brandLogo.classList.remove("hidden");
        if (btnReconnect) btnReconnect.classList.add("hidden");
    } else {
        if (statusSpinner) statusSpinner.classList.add("hidden");
        if (brandLogo) brandLogo.classList.remove("hidden");
        if (btnReconnect) btnReconnect.classList.remove("hidden");
    }
}

const btnInputMode = document.getElementById("btnInputMode");
const iconMouse = document.getElementById("iconMouse");
const iconTouch = document.getElementById("iconTouch");
const btnKeyboard = document.getElementById("btnKeyboard");
const btnLock = document.getElementById("btnLock");
const btnSettings = document.getElementById("btnSettings");
const btnStats = document.getElementById("btnStats");
const btnHideControls = document.getElementById("btnHideControls");
const btnFullscreen = document.getElementById("btnFullscreen");
const btnDisconnect = document.getElementById("btnDisconnect");
const btnThemeToggle = document.getElementById("btnThemeToggle");
const iconThemeDark = document.getElementById("iconThemeDark");
const iconThemeLight = document.getElementById("iconThemeLight");
const btnWebLang = document.getElementById("btnWebLang");
const lblWebLang = document.getElementById("lblWebLang");
const btnRestoreToolbar = document.getElementById("btnRestoreToolbar");

const keyboardDrawer = document.getElementById("keyboardDrawer");
const btnCloseKeyboard = document.getElementById("btnCloseKeyboard");
const keyboardImeInput = document.getElementById("keyboardImeInput");
const btnSendCad = document.getElementById("btnSendCad");

const settingsModal = document.getElementById("settingsModal");
const btnCloseSettings = document.getElementById("btnCloseSettings");
const statLatency = document.getElementById("statLatency");
const statRes = document.getElementById("statRes");
const statEncoder = document.getElementById("statEncoder");
const btnActionResync = document.getElementById("btnActionResync");

const vncBanner = document.getElementById("vncBanner");
const toastEl = document.getElementById("toast");
const tokenRow = document.getElementById("tokenRow");
const tokenInput = document.getElementById("tokenInput");
const btnConnect = document.getElementById("btnConnect");
const pairingPanel = document.getElementById("pairingPanel");
const pairingName = document.getElementById("pairingName");
const pairingStatus = document.getElementById("pairingStatus");
const pairingRequestId = document.getElementById("pairingRequestId");
const btnPair = document.getElementById("btnPair");
const btnCancelPair = document.getElementById("btnCancelPair");
const hostApprovalModal = document.getElementById("hostApprovalModal");
const hostAdminToken = document.getElementById("hostAdminToken");
const hostApprovalStatus = document.getElementById("hostApprovalStatus");
const hostPairingRequests = document.getElementById("hostPairingRequests");
const hostPairedClients = document.getElementById("hostPairedClients");
const btnLoadApprovals = document.getElementById("btnLoadApprovals");
let pairingAttempt = null;
let adminController = null;
let adminBusy = false;

function showPairing(message = "Choose Pair this device, then approve the matching request on the host.") {
    destroyPlayer();
    setOverlayState("unsupported", "Host approval required", "Pair this device to connect.");
    pairingPanel.classList.remove("hidden");
    pairingStatus.textContent = message;
    btnPair.disabled = window.location.protocol !== "https:" || !!pairingAttempt;
    if (window.location.protocol !== "https:") {
        pairingStatus.textContent = "Pairing requires HTTPS. Open the host's HTTPS client and verify its certificate first.";
    }
}

async function pairingFetch(path, options = {}) {
    if (window.location.protocol !== "https:") throw new Error("Pairing requires HTTPS.");
    const response = await fetch(path, {
        ...options,
        signal: AbortSignal.any([AbortSignal.timeout(15000), ...(options.signal ? [options.signal] : [])]),
        cache: "no-store",
        credentials: "omit",
        redirect: "error",
        referrerPolicy: "no-referrer",
    });
    if (!response.ok) {
        if (response.status === 401 || response.status === 403) {
            throw new Error("Access denied. Host administration requires the admin token from the host config token file.");
        }
        throw new Error(`Pairing request failed (HTTP ${response.status}). Refresh or try again.`);
    }
    return response.status === 204 ? {} : response.json();
}

function stopPairing() {
    if (pairingAttempt) {
        clearTimeout(pairingAttempt.timer);
        pairingAttempt.controller.abort();
        pairingAttempt = null;
    }
    btnPair.disabled = window.location.protocol !== "https:";
    pairingName.disabled = false;
    btnCancelPair.classList.add("hidden");
    pairingRequestId.textContent = "";
}

async function pollPairing(attempt) {
    if (pairingAttempt !== attempt) return;
    try {
        if (Date.now() >= attempt.expiresAt) throw new Error("Approval timed out. Request pairing again.");
        const data = await pairingFetch(`/api/pair/status?id=${encodeURIComponent(attempt.id)}`, {
            signal: attempt.controller.signal,
        });
        if (pairingAttempt !== attempt) return;
        if (data.status === "pending") {
            attempt.timer = setTimeout(() => pollPairing(attempt), 2000);
            return;
        }
        if (data.status === "approved" && typeof data.credential === "string" && data.credential.length > 0) {
            authToken = data.credential;
            stopPairing();
            pairingPanel.classList.add("hidden");
            tokenRow.classList.add("hidden");
            await start().catch(() => {
                setOverlayState("error", "Paired", "Approval received, but connecting failed. Reconnect to retry.");
            });
            return;
        }
        throw new Error(data.status === "unknown"
            ? "Request denied, expired or already claimed. Request pairing again."
            : "Invalid approval response. Request pairing again.");
    } catch (error) {
        if (pairingAttempt !== attempt) return;
        stopPairing();
        showPairing(error.message || "Could not check approval. Request pairing again.");
    }
}

btnPair.addEventListener("click", async () => {
    if (pairingAttempt || window.location.protocol !== "https:") return;
    const name = pairingName.value.trim();
    if (!name) {
        pairingStatus.textContent = "Enter a device name to identify this request on the host.";
        pairingName.focus();
        return;
    }
    destroyPlayer();
    const attempt = { controller: new AbortController(), timer: null, expiresAt: Date.now() + 600000 };
    pairingAttempt = attempt;
    btnPair.disabled = true;
    pairingName.disabled = true;
    btnCancelPair.classList.remove("hidden");
    pairingStatus.textContent = "Requesting host approval…";
    try {
        const data = await pairingFetch("/api/pair/request", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ name }),
            signal: attempt.controller.signal,
        });
        if (pairingAttempt !== attempt) return;
        if (data.status !== "pending" || typeof data.request_id !== "string" || !data.request_id) {
            throw new Error("Invalid pairing response. Request pairing again.");
        }
        attempt.id = data.request_id;
        pairingStatus.textContent = `Waiting for host approval for ${name}. Compare this request ID on the host; do not share it elsewhere.`;
        pairingRequestId.textContent = `Request ID: ${data.request_id}`;
        attempt.timer = setTimeout(() => pollPairing(attempt), 2000);
    } catch (error) {
        if (pairingAttempt !== attempt) return;
        stopPairing();
        showPairing(error.message || "Could not request pairing.");
    }
});

btnCancelPair.addEventListener("click", () => {
    stopPairing();
    showPairing("Stopped waiting. The host may still see the request; deny it there before trying again.");
});

function closeHostApprovals() {
    if (adminController) adminController.abort();
    adminController = null;
    adminBusy = false;
    hostAdminToken.value = "";
    hostApprovalStatus.textContent = "";
    hostPairingRequests.replaceChildren();
    hostPairedClients.replaceChildren();
    hostApprovalModal.classList.add("hidden");
    btnLoadApprovals.disabled = false;
}

function pairingMetadata(parent, label, value) {
    const line = document.createElement("p");
    line.className = "pairingIdentity";
    line.textContent = `${label}: ${typeof value === "string" ? value : "Unknown"}`;
    parent.append(line);
}

function renderPairingList(container, entries, requests) {
    container.replaceChildren();
    if (!entries.length) {
        const empty = document.createElement("p");
        empty.textContent = requests ? "No pending requests." : "No paired devices.";
        container.append(empty);
        return;
    }
    for (const entry of entries) {
        if (!entry || typeof entry !== "object") continue;
        const id = requests ? entry.request_id : entry.client_id;
        const card = document.createElement("section");
        card.className = "pairingDevice";
        pairingMetadata(card, "Device", entry.label || entry.name);
        if (requests) pairingMetadata(card, "Peer", entry.peer);
        pairingMetadata(card, requests ? "Request ID" : "Client ID", id);
        pairingMetadata(card, "Status", requests ? (entry.approved ? "approved, awaiting claim" : "pending") : entry.status);
        const actions = document.createElement("div");
        actions.className = "pairingActions";
        const allowed = requests ? (entry.approved ? [] : ["approve", "deny"]) : (entry.status === "approved" ? ["revoke"] : []);
        for (const action of allowed) {
            if (typeof id !== "string" || !id) continue;
            const name = entry.label || entry.name;
            if (action === "approve" && (typeof entry.peer !== "string" || !entry.peer || typeof name !== "string" || !name)) continue;
            const button = document.createElement("button");
            button.type = "button";
            button.className = "chipBtn";
            button.textContent = action === "approve" ? "Approve this device" : action === "deny" ? "Deny" : "Revoke";
            button.addEventListener("click", () => loadHostApprovals({ action, id }));
            actions.append(button);
        }
        card.append(actions);
        container.append(card);
    }
}

async function loadHostApprovals(action = null) {
    if (adminBusy) return;
    const token = hostAdminToken.value.trim();
    if (!token) {
        hostApprovalStatus.textContent = "Paste the host admin token first. Device credentials cannot administer pairing.";
        hostAdminToken.focus();
        return;
    }
    adminBusy = true;
    const controller = new AbortController();
    adminController = controller;
    btnLoadApprovals.disabled = true;
    hostApprovalModal.querySelectorAll(".pairingDevice button").forEach((button) => { button.disabled = true; });
    hostApprovalStatus.textContent = action ? "Updating device…" : "Loading devices…";
    try {
        const options = { headers: { Authorization: `Bearer ${token}` }, signal: controller.signal };
        if (action) {
            await pairingFetch("/api/pair", {
                ...options,
                method: "POST",
                headers: { ...options.headers, "Content-Type": "application/json" },
                body: JSON.stringify(action),
            });
        }
        const data = await pairingFetch("/api/pair", options);
        if (adminController !== controller) return;
        if (!Array.isArray(data.requests) || !Array.isArray(data.clients)) throw new Error("Invalid device list.");
        renderPairingList(hostPairingRequests, data.requests, true);
        renderPairingList(hostPairedClients, data.clients, false);
        hostApprovalStatus.textContent = action ? "Device updated. List refreshed." : "Compare each request with the requesting device before approving.";
    } catch (error) {
        if (adminController !== controller) return;
        hostPairingRequests.replaceChildren();
        hostPairedClients.replaceChildren();
        hostApprovalStatus.textContent = error.message || "Could not load devices.";
    } finally {
        if (adminController === controller) {
            adminController = null;
            adminBusy = false;
            btnLoadApprovals.disabled = false;
        }
    }
}

function openHostApprovals() {
    releaseControl();
    settingsModal.classList.add("hidden");
    hostApprovalModal.classList.remove("hidden");
    hostApprovalStatus.textContent = window.location.protocol === "https:" ? "" : "Host approvals require HTTPS.";
    btnLoadApprovals.disabled = window.location.protocol !== "https:";
    hostAdminToken.disabled = window.location.protocol !== "https:";
    hostAdminToken.focus();
}
document.getElementById("btnHostApprovals").addEventListener("click", openHostApprovals);
document.getElementById("btnSettingsHostApprovals").addEventListener("click", openHostApprovals);
document.getElementById("btnCloseApprovals").addEventListener("click", closeHostApprovals);
btnLoadApprovals.addEventListener("click", () => loadHostApprovals());
hostAdminToken.addEventListener("input", () => {
    if (adminController) adminController.abort();
    adminController = null;
    adminBusy = false;
    btnLoadApprovals.disabled = window.location.protocol !== "https:";
    hostPairingRequests.replaceChildren();
    hostPairedClients.replaceChildren();
    hostApprovalStatus.textContent = "";
});
window.addEventListener("pagehide", () => {
    stopPairing();
    closeHostApprovals();
});

let displayWidth = 1920;
let displayHeight = 1080;
let encoderName = "NVENC";
let videoTransport = "";
let authToken = "";
let wtConfig = null;
let wtTransport = null;
let wtWriter = null;
let videoDecoder = null;
let reconnectTimer = null;
let reconnectDelay = 1000;
let userDisconnected = false;
let auReader = null;
const MAX_RECONNECT_DELAY = 10000;
let clockOffsetNs = 0n;
let lastFrameAt = 0;
let streamActive = false;
let isVncFocused = false;
let isTouchMode = true;
let toastTimer = null;
let vncBannerTimer = null;
let latencyWatchdog = null;
let holeWatchdog = null;
let lastDelayMs = null;
let infoTick = null;
const streamStats = (typeof OrbiStats !== "undefined") ? new OrbiStats.StreamStats() : null;
const pendingPresent = [];
let statsVisible = false;
let statsRaf = 0;
const statsWidget = document.getElementById("statsWidget");
const statsDelayEl = document.getElementById("statsDelay");
const statsAgeEl = document.getElementById("statsAge");
const statsBytesEl = document.getElementById("statsBytes");
const statsFpsEl = document.getElementById("statsFps");
const statsBytesGraph = document.getElementById("statsBytesGraph");
const statsFramesGraph = document.getElementById("statsFramesGraph");
const statsLegI = document.getElementById("statsLegI");
const statsLegP = document.getElementById("statsLegP");
const statsLegD = document.getElementById("statsLegD");
let controlsVisible = false;
let controlsLockedHidden = false;
let controlsHideTimer = null;
const heldKeys = new Set();
const pressedButtons = new Set();
const activeTouches = new Map();
let pendingMove = null;
let moveRaf = null;

const urlParams = new URLSearchParams(window.location.search);
if (urlParams.has("token")) {
    authToken = urlParams.get("token");
} else if (window.location.hash && window.location.hash.length > 1) {
    const hashParams = new URLSearchParams(window.location.hash.substring(1));
    if (hashParams.has("token")) {
        authToken = hashParams.get("token");
    }
}

function showToast(text, duration = 2200) {
    if (!toastEl) return;
    toastEl.textContent = text;
    toastEl.classList.remove("hidden");
    clearTimeout(toastTimer);
    toastTimer = setTimeout(() => {
        toastEl.classList.add("hidden");
    }, duration);
}

window.addEventListener("resize", () => updateInfoDisplay());

function updateInfoDisplay() {
    const ageMs = (streamStats && typeof streamStats.snapshot === "function")
        ? streamStats.snapshot().ageMs
        : lastDelayMs;
    const delayText = (typeof OrbiStats !== "undefined" && OrbiStats.formatToolbarDelay)
        ? OrbiStats.formatToolbarDelay(ageMs)
        : ((ageMs != null && ageMs >= 0) ? `delay ${ageMs}ms` : "delay —");
    const narrow = window.matchMedia("(orientation: portrait), (max-width: 720px)").matches;
    const encoderLabel = videoTransport ? `${encoderName} · ${videoTransport}` : encoderName;
    const infoStr = narrow
        ? delayText
        : `${displayWidth}×${displayHeight}  ${encoderLabel}  ${delayText}`;
    if (hostInfoEl) hostInfoEl.textContent = infoStr;
    if (statRes) statRes.textContent = `${displayWidth} × ${displayHeight}`;
    if (statEncoder) statEncoder.textContent = encoderLabel;
    const statsTitleEl = document.getElementById("statsTitle");
    if (statsTitleEl) {
        statsTitleEl.textContent = videoTransport ? `Statistics · ${videoTransport}` : "Statistics";
    }
    if (statLatency) {
        statLatency.textContent = (lastDelayMs != null && lastDelayMs >= 0)
            ? `${lastDelayMs} ms`
            : "-- ms";
    }
}

function hideControlsChrome() {
    clearTimeout(controlsHideTimer);
    controlsHideTimer = null;
    controlsVisible = false;
    if (controlToolbar) controlToolbar.classList.add("hidden");
    if (btnShowControls) btnShowControls.classList.add("hidden");
}

function setControlsVisible(visible) {
    clearTimeout(controlsHideTimer);
    controlsHideTimer = null;
    if (!streamActive) {
        hideControlsChrome();
        return;
    }
    if (controlsLockedHidden) {
        if (controlToolbar) controlToolbar.classList.add("hidden");
        if (btnShowControls) btnShowControls.classList.add("hidden");
        controlsVisible = false;
        return;
    }
    controlsVisible = visible;
    if (visible) {
        if (controlToolbar) controlToolbar.classList.remove("hidden");
        if (btnShowControls) btnShowControls.classList.add("hidden");
        controlsHideTimer = setTimeout(() => setControlsVisible(false), 12_000);
    } else {
        if (controlToolbar) controlToolbar.classList.add("hidden");
        if (btnShowControls) btnShowControls.classList.remove("hidden");
    }
}

function setVncFocus(focused) {
    if (focused && !streamActive) return;
    if (focused === isVncFocused) return;
    isVncFocused = focused;
    if (focused) {
        stageEl.classList.add("vncFocused");
        stageEl.focus();
        showVncBanner();
    } else {
        releaseControl();
        showToast("Cursor released", 1400);
    }
}

function showVncBanner() {
    if (!vncBanner) return;
    vncBanner.classList.remove("hidden");
    clearTimeout(vncBannerTimer);
    vncBannerTimer = setTimeout(() => {
        vncBanner.classList.add("hidden");
    }, 2400);
}

function releaseControl() {
    isVncFocused = false;
    if (stageEl) stageEl.classList.remove("vncFocused");
    if (vncBanner) vncBanner.classList.add("hidden");
    releaseAllKeys();
    releaseAllButtons();
}

function releaseAllKeys() {
    for (const code of heldKeys) {
        sendInput({ Key: { code, pressed: false } });
    }
    heldKeys.clear();
    const modifiers = [29, 97, 42, 54, 56, 100, 125, 126];
    for (const code of modifiers) {
        sendInput({ Key: { code, pressed: false } });
    }
}

function releaseAllButtons() {
    for (const button of pressedButtons) {
        sendInput({ Pointer: { Button: { button, pressed: false } } });
    }
    pressedButtons.clear();
    releaseAllTouches();
    hideTouch();
}

window.addEventListener("blur", () => {
    releaseControl();
});

document.addEventListener("visibilitychange", () => {
    if (document.hidden) {
        releaseControl();
    }
});

if (overlayEl) {
    overlayEl.addEventListener("pointerdown", (e) => {
        e.stopPropagation();
    });
}

function usesTouchInput(event) {
    if (event.pointerType === "pen") return false;
    return isTouchMode;
}

function sendTouchAt(pointerId, x, y, pressed) {
    let rec = activeTouches.get(pointerId);
    if (!rec) {
        if (!pressed) return;
        const used = new Set();
        for (const t of activeTouches.values()) used.add(t.slot);
        let slot = 0;
        while (used.has(slot) && slot < 9) slot += 1;
        rec = { slot, id: pointerId & 0x7fffffff };
        activeTouches.set(pointerId, rec);
    }
    sendInput({ Touch: { slot: rec.slot, id: rec.id, x, y, pressed } });
    if (!pressed) activeTouches.delete(pointerId);
}

function releaseAllTouches() {
    for (const rec of activeTouches.values()) {
        sendInput({ Touch: { slot: rec.slot, id: rec.id, x: 0, y: 0, pressed: false } });
    }
    activeTouches.clear();
}

function applyInputModeIcons() {
    if (!iconMouse || !iconTouch) return;
    if (isTouchMode) {
        iconMouse.classList.add("hidden");
        iconTouch.classList.remove("hidden");
    } else {
        iconMouse.classList.remove("hidden");
        iconTouch.classList.add("hidden");
    }
}

stageEl.addEventListener("pointerdown", (event) => {
    if (!streamActive) return;
    if (!isVncFocused) {
        setVncFocus(true);
    }
    event.preventDefault();
    try {
        event.currentTarget.setPointerCapture(event.pointerId);
    } catch (_) {  }
    const { x, y } = mapPointer(event);
    if (event.pointerType === "pen") {
        sendStylus(x, y, event.pressure, event.tiltX, event.tiltY);
        showTouch(event.clientX, event.clientY);
        return;
    }
    if (usesTouchInput(event)) {
        sendTouchAt(event.pointerId, x, y, true);
        showTouch(event.clientX, event.clientY);
        return;
    }
    sendPointerMove(x, y);
    sendPointerButton(event.button + 1, true);
    showTouch(event.clientX, event.clientY);
});

stageEl.addEventListener("pointermove", (event) => {
    if (!streamActive) return;
    const touching = usesTouchInput(event) || activeTouches.has(event.pointerId);
    if (!isVncFocused && !touching) return;
    event.preventDefault();
    const { x, y } = mapPointer(event);
    if (event.pointerType === "pen") {
        if (event.buttons > 0) {
            sendStylus(x, y, event.pressure, event.tiltX, event.tiltY);
            showTouch(event.clientX, event.clientY);
        }
        return;
    }
    if (touching) {
        if (event.buttons > 0 || activeTouches.has(event.pointerId)) {
            sendTouchAt(event.pointerId, x, y, true);
            showTouch(event.clientX, event.clientY);
        }
        return;
    }
    if (!isVncFocused) return;
    if (event.buttons > 0) {
        showTouch(event.clientX, event.clientY);
        sendPointerMove(x, y);
    } else {
        queuePointerMove(x, y);
    }
});

stageEl.addEventListener("pointerup", (event) => {
    if (!streamActive) return;
    event.preventDefault();
    try {
        event.currentTarget.releasePointerCapture(event.pointerId);
    } catch (_) {  }
    const { x, y } = mapPointer(event);
    if (event.pointerType === "pen") {
        sendStylus(x, y, 0, event.tiltX, event.tiltY);
        hideTouch();
        return;
    }
    if (usesTouchInput(event) || activeTouches.has(event.pointerId)) {
        sendTouchAt(event.pointerId, x, y, false);
        if (activeTouches.size === 0) hideTouch();
        return;
    }
    if (!isVncFocused) return;
    sendPointerButton(event.button + 1, false);
    hideTouch();
});

stageEl.addEventListener("pointerleave", (event) => {
    if (activeTouches.has(event.pointerId)) {
        const { x, y } = mapPointer(event);
        sendTouchAt(event.pointerId, x, y, false);
        if (activeTouches.size === 0) hideTouch();
        return;
    }
    if (isVncFocused && !isTouchMode) {
        releaseAllButtons();
    }
});

stageEl.addEventListener("pointercancel", (event) => {
    if (activeTouches.has(event.pointerId)) {
        const { x, y } = mapPointer(event);
        sendTouchAt(event.pointerId, x, y, false);
        if (activeTouches.size === 0) hideTouch();
        return;
    }
    if (isVncFocused && !isTouchMode) {
        releaseAllButtons();
    }
});

stageEl.addEventListener("wheel", (event) => {
    if (!streamActive || !isVncFocused) return;
    event.preventDefault();
    sendWheel(normalizeWheel(event));
}, { passive: false });

window.addEventListener("keydown", (event) => {
    if (event.code === "Escape") {
        event.preventDefault();
        if (isVncFocused) {
            setVncFocus(false);
        }
        keyboardDrawer.classList.add("hidden");
        settingsModal.classList.add("hidden");
        closeHostApprovals();
        unlatchAll();
        return;
    }
    if (event.code === "F11") {
        event.preventDefault();
        toggleFullscreen();
        return;
    }
    if (!isVncFocused || !streamActive || event.repeat) return;
    if (event.target === keyboardImeInput) return;
    event.preventDefault();
    sendKey(event.code, true);
});

window.addEventListener("keyup", (event) => {
    if (event.code === "Escape" || event.code === "F11") return;
    if (!isVncFocused || !streamActive) return;
    if (event.target === keyboardImeInput) return;
    event.preventDefault();
    sendKey(event.code, false);
});

applyInputModeIcons();

if (btnInputMode) {
    btnInputMode.addEventListener("click", (e) => {
        e.stopPropagation();
        isTouchMode = !isTouchMode;
        applyInputModeIcons();
        showToast(isTouchMode ? t("modeTouch") : t("modeTouchpad"));
    });
}

if (btnKeyboard) {
    btnKeyboard.addEventListener("click", (e) => {
        e.stopPropagation();
        keyboardDrawer.classList.toggle("hidden");
        if (!keyboardDrawer.classList.contains("hidden")) {
            if (keyboardImeInput) {
                keyboardImeInput.focus();
            }
            if (!isVncFocused) {
                setVncFocus(true);
            }
        } else {
            unlatchAll();
        }
    });
}

if (btnCloseKeyboard) {
    btnCloseKeyboard.addEventListener("click", (e) => {
        e.stopPropagation();
        keyboardDrawer.classList.add("hidden");
        unlatchAll();
    });
}

if (btnSettings) {
    btnSettings.addEventListener("click", (e) => {
        e.stopPropagation();
        settingsModal.classList.remove("hidden");
    });
}

function setStatsVisible(visible) {
    statsVisible = !!visible;
    try { localStorage.setItem("orbiscreen.statsVisible", statsVisible ? "1" : "0"); } catch (_) {}
    if (statsWidget) {
        statsWidget.classList.toggle("hidden", !statsVisible || !streamActive);
        statsWidget.setAttribute("aria-hidden", (!statsVisible || !streamActive) ? "true" : "false");
    }
    if (btnStats) btnStats.classList.toggle("active", statsVisible);
    if (statsVisible && streamActive) startStatsPaint();
}

function startStatsPaint() {
    if (statsRaf || !streamStats || typeof OrbiStats === "undefined") return;
    const tick = () => {
        statsRaf = 0;
        if (!statsVisible || !streamActive) return;
        const snap = streamStats.snapshot();
        if (statsDelayEl) statsDelayEl.textContent = OrbiStats.formatMs(snap.delayMs);
        if (statsAgeEl) statsAgeEl.textContent = OrbiStats.formatMs(snap.ageMs);
        if (statsBytesEl) statsBytesEl.textContent = OrbiStats.formatRate(snap.bytesPerSec);
        if (statsFpsEl) statsFpsEl.textContent = `${snap.fps} fps`;
        if (statsLegI) statsLegI.textContent = `I ${snap.lastMinuteI}`;
        if (statsLegP) statsLegP.textContent = `P ${snap.lastMinuteP}`;
        if (statsLegD) statsLegD.textContent = `D ${snap.lastMinuteDropped || 0}`;
        if (statsBytesGraph) OrbiStats.drawBytesGraph(statsBytesGraph, snap.bytesSeries);
        if (statsFramesGraph) OrbiStats.drawFramesGraph(statsFramesGraph, snap);
        updateInfoDisplay();
        statsRaf = requestAnimationFrame(tick);
    };
    statsRaf = requestAnimationFrame(tick);
}

if (btnStats) {
    btnStats.addEventListener("click", (e) => {
        e.stopPropagation();
        setStatsVisible(!statsVisible);
    });
}

try {
    if (localStorage.getItem("orbiscreen.statsVisible") === "1") {
        statsVisible = true;
        if (btnStats) btnStats.classList.add("active");
    }
} catch (_) {}

if (btnCloseSettings) {
    btnCloseSettings.addEventListener("click", () => {
        settingsModal.classList.add("hidden");
    });
}

if (settingsModal) {
    settingsModal.addEventListener("click", (e) => {
        if (e.target === settingsModal) {
            settingsModal.classList.add("hidden");
        }
    });
}

document.querySelectorAll(".chipBtn[data-fit]").forEach((btn) => {
    btn.addEventListener("click", () => {
        document.querySelectorAll(".chipBtn[data-fit]").forEach((b) => b.classList.remove("active"));
        btn.classList.add("active");
        if (videoEl) videoEl.style.objectFit = btn.dataset.fit;
        showToast(`Fit: ${btn.textContent}`);
    });
});

if (btnThemeToggle) {
    btnThemeToggle.addEventListener("click", (e) => {
        e.stopPropagation();
        applyTheme(currentTheme === "dark" ? "light" : "dark");
    });
}

if (btnWebLang) {
    btnWebLang.addEventListener("click", (e) => {
        e.stopPropagation();
        setLanguage(currentLang === "en" ? "ar" : "en");
    });
}

document.querySelectorAll("#webThemeChips .chipBtn").forEach((btn) => {
    btn.addEventListener("click", () => {
        const theme = btn.getAttribute("data-theme");
        if (theme) applyTheme(theme);
    });
});

document.querySelectorAll("#webLangChips .chipBtn").forEach((btn) => {
    btn.addEventListener("click", () => {
        const lang = btn.getAttribute("data-lang");
        if (lang) setLanguage(lang);
    });
});

if (btnHideControls) {
    btnHideControls.addEventListener("click", (e) => {
        e.stopPropagation();
        controlsLockedHidden = true;
        setControlsVisible(false);
        showToast(t("controlsHidden"), 1800);
    });
}

if (btnShowControls) {
    btnShowControls.addEventListener("click", (e) => {
        e.stopPropagation();
        setControlsVisible(true);
    });
}

stageEl.addEventListener("dblclick", (e) => {
    if (!streamActive || !controlsLockedHidden) return;
    e.preventDefault();
    controlsLockedHidden = false;
    setControlsVisible(true);
});

if (btnFullscreen) {
    btnFullscreen.addEventListener("click", (e) => {
        e.stopPropagation();
        if (!document.fullscreenElement) {
            document.documentElement.requestFullscreen().catch(() => {});
        } else {
            document.exitFullscreen().catch(() => {});
        }
    });
}

async function sendHostAction(action, extra = {}) {
    const headers = { "content-type": "application/json" };
    if (authToken) headers.authorization = `Bearer ${authToken}`;
    try {
        const res = await fetch("/api/control", {
            method: "POST",
            headers,
            body: JSON.stringify({ action, ...extra }),
        });
        return res.ok;
    } catch (err) {
        console.warn(`Host action ${action} failed:`, err);
        return false;
    }
}

const IDR_DEBOUNCE_MS = 250;
let lastIdrAt = 0;
let waitingForKeyframe = false;

function noteKeyframe() {
    waitingForKeyframe = false;
}

function requestIdr(opts = {}) {
    const now = Date.now();
    if (now - lastIdrAt < IDR_DEBOUNCE_MS) return false;
    lastIdrAt = now;
    if (!opts.keepDecoding) waitingForKeyframe = true;
    if (wtWriter) {
        try { wtWriter.write(OrbiAnnexB.encodeCtrl(OrbiAnnexB.TYPE_IDR)); } catch (_) {}
    }
    sendHostAction("idr", displaySessionId ? { session: displaySessionId } : {});
    return true;
}

if (btnLock) {
    btnLock.addEventListener("click", async (e) => {
        e.stopPropagation();
        const ok = await sendHostAction("lock");
        showToast(ok ? t("toastLocked") : t("toastLockSent"));
    });
}

if (btnSendCad) {
    btnSendCad.addEventListener("click", async (e) => {
        e.stopPropagation();
        await sendHostAction("ctrl_alt_del");
        showToast(t("toastCadSent"));
    });
}

if (btnActionResync) {
    btnActionResync.addEventListener("click", (e) => {
        e.stopPropagation();
        settingsModal.classList.add("hidden");
        requestIdr();
        startStream();
        showToast(t("toastResynced"));
    });
}

if (btnDisconnect) {
    btnDisconnect.addEventListener("click", (e) => {
        e.stopPropagation();
        userDisconnected = true;
        destroyPlayer();
        setOverlayState("disconnected", t("statusDisconnected"), t("statusDisconnected"));
        showToast(t("toastDisconnected"));
    });
}

if (btnReconnect) {
    btnReconnect.addEventListener("click", (e) => {
        e.stopPropagation();
        userDisconnected = false;
        if (pairingAttempt) return;
        start().catch(() => showPairing("Could not reconnect. Try pairing again."));
    });
}

if (btnConnect && tokenInput) {
    btnConnect.addEventListener("click", () => {
        const val = tokenInput.value.trim();
        if (val) {
            if (window.location.protocol !== "https:") return;
            stopPairing();
            authToken = val;
            tokenInput.value = "";
            tokenRow.classList.add("hidden");
            start().catch(() => showPairing("Could not connect. Try pairing again."));
        }
    });
}

function sendInput(payload) {
    if (!authToken || !displaySessionId || window.location.protocol !== "https:") return;
    const headers = { "content-type": "application/json" };
    if (authToken) headers.authorization = `Bearer ${authToken}`;
    if (displaySessionId) headers["x-orbiscreen-session"] = displaySessionId;
    fetch("/input", {
        method: "POST",
        headers,
        body: JSON.stringify(payload),
    }).catch((err) => console.warn("sendInput failed:", err));
}

function sendPointerMove(x, y) {
    sendInput({ Pointer: { Move: { x, y } } });
}

function queuePointerMove(x, y) {
    pendingMove = { x, y };
    if (!moveRaf) {
        moveRaf = requestAnimationFrame(() => {
            if (pendingMove) {
                sendPointerMove(pendingMove.x, pendingMove.y);
                pendingMove = null;
            }
            moveRaf = null;
        });
    }
}

function sendPointerButton(button, pressed) {
    if (pressed) {
        pressedButtons.add(button);
    } else {
        pressedButtons.delete(button);
    }
    sendInput({ Pointer: { Button: { button, pressed } } });
}

function sendWheel(deltaY) {
    sendInput({ Pointer: { Wheel: { delta_y: deltaY } } });
}

function normalizeWheel(event) {
    const vh = (videoEl && (videoEl.videoHeight || videoEl.height)) || displayHeight;
    let pixels = event.deltaY;
    if (event.deltaMode === 1) pixels *= 16;
    else if (event.deltaMode === 2) pixels *= vh;
    return Math.max(-12, Math.min(12, pixels / 100));
}

const KEYCODE_MAP = {
    Escape: 1, Enter: 28, Backspace: 14, Tab: 15, Space: 57,
    ArrowUp: 103, ArrowDown: 108, ArrowLeft: 105, ArrowRight: 106,
    ShiftLeft: 42, ShiftRight: 54, ControlLeft: 29, ControlRight: 97,
    AltLeft: 56, AltRight: 100, MetaLeft: 125, MetaRight: 126,
    Delete: 111, Insert: 110, Home: 102, End: 107,
    PageUp: 104, PageDown: 109, CapsLock: 58, NumLock: 69,
    ScrollLock: 70, PrintScreen: 99, Pause: 119, ContextMenu: 127,
};
for (let i = 0; i < 10; i += 1) KEYCODE_MAP[`F${i + 1}`] = 59 + i;
KEYCODE_MAP.F11 = 87;
KEYCODE_MAP.F12 = 88;
for (let i = 0; i < 10; i += 1) {
    KEYCODE_MAP[`Digit${(i + 1) % 10}`] = 2 + i;
    KEYCODE_MAP[`Numpad${i}`] = i === 0 ? 82 : 71 + (i - 1);
}
const LETTER_KEYCODES = {
    A: 30, B: 48, C: 46, D: 32, E: 18, F: 33, G: 34, H: 35, I: 23,
    J: 36, K: 37, L: 38, M: 50, N: 49, O: 24, P: 25, Q: 16, R: 19,
    S: 31, T: 20, U: 22, V: 47, W: 17, X: 45, Y: 21, Z: 44,
};
for (const [letter, code] of Object.entries(LETTER_KEYCODES)) {
    KEYCODE_MAP[`Key${letter}`] = code;
}
Object.assign(KEYCODE_MAP, {
    Minus: 12, Equal: 13, BracketLeft: 26, BracketRight: 27,
    Backslash: 43, Semicolon: 39, Quote: 40, Backquote: 41,
    Comma: 51, Period: 52, Slash: 53, IntlBackslash: 86,
    NumpadAdd: 78, NumpadSubtract: 74, NumpadMultiply: 55,
    NumpadDivide: 98, NumpadEnter: 96, NumpadDecimal: 83, NumpadComma: 121,
});

function sendKey(domCode, pressed) {
    const code = KEYCODE_MAP[domCode];
    if (code === undefined) return;
    if (pressed) {
        heldKeys.add(code);
    } else {
        heldKeys.delete(code);
    }
    sendInput({ Key: { code, pressed } });
}

function sendStylus(x, y, pressure, tiltX, tiltY) {
    sendInput({
        Stylus: {
            Tilt: {
                x, y, pressure,
                tilt_x_deg: tiltX,
                tilt_y_deg: tiltY,
            },
        },
    });
}

const latchedModifiers = {
    ControlLeft: false,
    AltLeft: false,
    ShiftLeft: false,
    MetaLeft: false,
};

function sendRawCode(code, pressed) {
    if (pressed) {
        heldKeys.add(code);
    } else {
        heldKeys.delete(code);
    }
    sendInput({ Key: { code, pressed } });
}

function unlatchAll() {
    const latches = [
        { key: "ControlLeft", code: 29 },
        { key: "AltLeft", code: 56 },
        { key: "ShiftLeft", code: 42 },
        { key: "MetaLeft", code: 125 },
    ];
    for (const { key, code } of latches) {
        if (latchedModifiers[key]) {
            latchedModifiers[key] = false;
            sendRawCode(code, false);
            const btn = document.querySelector(`.keyBtn[data-latch="${key}"]`);
            if (btn) btn.classList.remove("active");
        }
    }
}

function keyCodeFor(c) {
    const upper = c.toUpperCase();
    if (LETTER_KEYCODES[upper]) return LETTER_KEYCODES[upper];
    const map = {
        '0': 11, '1': 2, '2': 3, '3': 4, '4': 5,
        '5': 6, '6': 7, '7': 8, '8': 9, '9': 10,
        ' ': 57, '\n': 28, '\t': 15,
        '-': 12, '=': 13, '[': 26, ']': 27,
        ';': 39, "'": 40, '`': 41, '\\': 43,
        ',': 51, '.': 52, '/': 53,
    };
    return map[c] || 0;
}

function sendChar(c) {
    const shiftedChars = {
        '!': 2, '@': 3, '#': 4, '$': 5, '%': 6,
        '^': 7, '&': 8, '*': 9, '(': 10, ')': 11,
        '_': 12, '+': 13, '{': 26, '}': 27, ':': 39,
        '"': 40, '~': 41, '|': 43, '<': 51, '>': 52, '?': 53,
    };
    if (c >= 'A' && c <= 'Z') {
        const code = keyCodeFor(c);
        if (code) {
            sendRawCode(42, true);
            sendRawCode(code, true);
            sendRawCode(code, false);
            sendRawCode(42, false);
        }
    } else if (shiftedChars[c]) {
        const code = shiftedChars[c];
        sendRawCode(42, true);
        sendRawCode(code, true);
        sendRawCode(code, false);
        sendRawCode(42, false);
    } else {
        const code = keyCodeFor(c);
        if (code) {
            sendRawCode(code, true);
            sendRawCode(code, false);
        }
    }
}

document.querySelectorAll(".keyBtn").forEach((btn) => {
    btn.addEventListener("pointerdown", (e) => {
        e.preventDefault();
    });
});

document.querySelectorAll(".keyBtn[data-code]").forEach((btn) => {
    btn.addEventListener("click", (e) => {
        e.stopPropagation();
        const code = btn.dataset.code;
        sendKey(code, true);
        setTimeout(() => {
            sendKey(code, false);
            unlatchAll();
        }, 50);
    });
});

document.querySelectorAll(".keyBtn[data-latch]").forEach((btn) => {
    btn.addEventListener("click", (e) => {
        e.stopPropagation();
        const latchKey = btn.dataset.latch;
        const rawCode = parseInt(btn.dataset.raw, 10);
        latchedModifiers[latchKey] = !latchedModifiers[latchKey];
        const active = latchedModifiers[latchKey];
        btn.classList.toggle("active", active);
        sendRawCode(rawCode, active);
    });
});

document.querySelectorAll(".keyBtn[data-action]").forEach((btn) => {
    btn.addEventListener("click", (e) => {
        e.stopPropagation();
        const action = btn.dataset.action;
        const keyMap = { undo: 44, copy: 46, paste: 47 };
        const key = keyMap[action];
        if (key) {
            sendRawCode(29, true);
            sendRawCode(key, true);
            setTimeout(() => {
                sendRawCode(key, false);
                sendRawCode(29, false);
                unlatchAll();
            }, 50);
        }
    });
});

if (keyboardImeInput) {
    const DUMMY = "   ";
    keyboardImeInput.value = DUMMY;

    keyboardImeInput.addEventListener("keydown", (e) => {
        if (e.key === "Backspace") {
            e.preventDefault();
            sendRawCode(14, true);
            setTimeout(() => sendRawCode(14, false), 40);
        } else if (e.key === "Enter") {
            e.preventDefault();
            sendRawCode(28, true);
            setTimeout(() => {
                sendRawCode(28, false);
                unlatchAll();
            }, 40);
        } else if (e.key === "Tab") {
            e.preventDefault();
            sendRawCode(15, true);
            setTimeout(() => sendRawCode(15, false), 40);
        } else if (e.key === "Escape") {
            e.preventDefault();
            keyboardDrawer.classList.add("hidden");
            unlatchAll();
        }
    });

    keyboardImeInput.addEventListener("input", () => {
        const val = keyboardImeInput.value;
        if (val.length < DUMMY.length || !val) {
            sendRawCode(14, true);
            setTimeout(() => sendRawCode(14, false), 40);
        } else if (val.length > DUMMY.length) {
            const added = val.substring(DUMMY.length);
            for (const ch of added) {
                sendChar(ch);
            }
            unlatchAll();
        }
        keyboardImeInput.value = DUMMY;
    });
}

function mapPointer(event) {
    const target = videoEl;
    const rect = target.getBoundingClientRect();
    const vw = target.videoWidth || target.width || displayWidth;
    const vh = target.videoHeight || target.height || displayHeight;
    const scale = Math.min(rect.width / vw, rect.height / vh);
    const offsetX = (rect.width - vw * scale) / 2;
    const offsetY = (rect.height - vh * scale) / 2;
    const clamp = (v, max) => Math.max(0, Math.min(max, v));
    return {
        x: clamp((event.clientX - rect.left - offsetX) / scale, vw - 1),
        y: clamp((event.clientY - rect.top - offsetY) / scale, vh - 1),
    };
}

function showTouch(x, y) {
    if (!touchIndicator) return;
    touchIndicator.style.left = `${x}px`;
    touchIndicator.style.top = `${y}px`;
    touchIndicator.classList.remove("hidden");
}

function hideTouch() {
    if (touchIndicator) touchIndicator.classList.add("hidden");
}

function playbackSupport() {
    return {
        webTransport: typeof WebTransport === "function",
        videoDecoder: typeof VideoDecoder === "function",
        annexb: typeof OrbiAnnexB !== "undefined",
    };
}

function canPlayWebTransport() {
    const s = playbackSupport();
    return s.webTransport && s.annexb && s.videoDecoder;
}

function unsupportedPlayback() {
    setOverlayState("unsupported", t("statusUnsupported"), t("statusUnsupportedSub"));
}

function closeDecoder() {
    if (videoDecoder) {
        try { videoDecoder.close(); } catch (_) {}
        videoDecoder = null;
    }
}

function destroyPlayer() {
    if (reconnectTimer) {
        clearTimeout(reconnectTimer);
        reconnectTimer = null;
    }
    if (auReader) {
        try { auReader.cancel(); } catch (_) {}
        auReader = null;
    }
    if (wtWriter) {
        try { wtWriter.close(); } catch (_) {}
        wtWriter = null;
    }
    if (wtTransport) {
        try { wtTransport.close(); } catch (_) {}
        wtTransport = null;
    }
    closeDecoder();
    streamActive = false;
    if (infoTick) {
        clearInterval(infoTick);
        infoTick = null;
    }
    if (statsWidget) {
        statsWidget.classList.add("hidden");
        statsWidget.setAttribute("aria-hidden", "true");
    }
    clearInterval(latencyWatchdog);
    clearInterval(holeWatchdog);
    holeWatchdog = null;
    releaseControl();
}

async function fetchClientConfig() {
    try {
        const cfg = await fetch("/client/config.json", {
            headers: authToken ? { Authorization: `Bearer ${authToken}` } : {},
            cache: "no-store",
            redirect: "error",
        });
        if (cfg.ok) return await cfg.json();
    } catch (error) {
        console.warn("config.json fetch failed:", error);
    }
    return null;
}

async function refreshToken() {
    if (authToken) return;
    const info = await fetchClientConfig();
    if (info && typeof info.token === "string" && info.token.length > 0) {
        authToken = info.token;
    }
}

function scheduleReconnect(reason) {
    if (reconnectTimer || pairingAttempt || !authToken) return;
    if (userDisconnected || reconnectTimer || pairingAttempt || !authToken) return;
    destroyPlayer();
    setOverlayState("connecting", "Connecting", `Reconnecting (${reason})…`);
    reconnectTimer = setTimeout(() => {
        reconnectTimer = null;
        (async () => {
            await refreshToken();
            const sessionGone = !displaySessionId
                || /close|attach|session|no display/i.test(String(reason || ""));
            if (sessionGone) {
                displaySessionId = "";
                await openDisplaySession();
            }
            startStream();
        })().catch((error) => {
            console.warn("reconnect attempt failed:", error);
            displaySessionId = "";
            scheduleReconnect("retry error");
        });
    }, reconnectDelay);
    reconnectDelay = Math.min(reconnectDelay * 2, MAX_RECONNECT_DELAY);
}

function markPlaying() {
    if (!streamActive) {
        streamActive = true;
        reconnectDelay = 1000;
        controlsLockedHidden = false;
        if (overlayEl) overlayEl.classList.add("hidden");
        setControlsVisible(false);
        if (statsVisible) setStatsVisible(true);
        if (!infoTick) {
            infoTick = setInterval(updateInfoDisplay, 100);
        }
    }
    noteKeyframe();
    lastFrameAt = Date.now();
}

function drawVideoFrame(frame) {
    try {
        if (videoEl.width !== frame.displayWidth || videoEl.height !== frame.displayHeight) {
            videoEl.width = frame.displayWidth;
            videoEl.height = frame.displayHeight;
            displayWidth = frame.displayWidth;
            displayHeight = frame.displayHeight;
            updateInfoDisplay();
        }
        const ctx = videoEl.getContext("2d");
        ctx.drawImage(frame, 0, 0);
        if (streamStats && pendingPresent.length) {
            streamStats.notePresented(pendingPresent.shift());
        }
        markPlaying();
    } finally {
        frame.close();
    }
}

function ensureDecoder(codec) {
    if (videoDecoder && videoDecoder.state !== "closed") {
        return videoDecoder;
    }
    closeDecoder();
    videoDecoder = new VideoDecoder({
        output: drawVideoFrame,
        error: (err) => {
            console.error("VideoDecoder error:", err);
            requestIdr();
            scheduleReconnect("decoder");
        },
    });
    videoDecoder.configure({
        codec,
        optimizeForLatency: true,
        hardwareAcceleration: "prefer-hardware",
    });
    return videoDecoder;
}

function nowNs() {
    return BigInt(Date.now()) * 1000000n;
}

function feedAccessUnit(msg) {
    if (typeof VideoDecoder !== "function") {
        unsupportedPlayback();
        return;
    }
    if (msg.key) {
        const found = OrbiAnnexB.extractSpsPps(msg.au);
        const codec = OrbiAnnexB.codecStringFromSps(found.sps);
        if (codec) {
            try {
                ensureDecoder(codec);
            } catch (err) {
                console.error("VideoDecoder configure failed:", err);
                scheduleReconnect("codec");
                return;
            }
        }
        noteKeyframe();
    }
    if (!videoDecoder || videoDecoder.state !== "configured") {
        requestIdr();
        return;
    }
    if (waitingForKeyframe && !msg.key) {
        if (streamStats) streamStats.noteDropped(1);
        requestIdr();
        return;
    }
    if (streamStats) {
        streamStats.noteFrame(OrbiAnnexB.classifyAccessUnit(msg.au));
        pendingPresent.push(msg.sentNs);
        if (pendingPresent.length > 120) pendingPresent.shift();
    }
    // Real-time timestamps keep VideoDecoder in low-latency mode.
    // Session-relative pts_ns looks like a VOD timeline and some
    // implementations then hold ~200 ms before the first output.
    const chunk = new EncodedVideoChunk({
        type: msg.key ? "key" : "delta",
        timestamp: Math.round(performance.now() * 1000),
        data: msg.au,
    });
    try {
        videoDecoder.decode(chunk);
    } catch (err) {
        console.warn("decode failed:", err);
        closeDecoder();
        requestIdr();
    }
    if (msg.sentNs > 0n) {
        const glass = Number((nowNs() + clockOffsetNs - msg.sentNs) / 1000000n);
        if (glass >= 0 && glass <= 5000) {
            lastDelayMs = glass;
            if (streamStats) streamStats.noteDelay(glass);
            updateInfoDisplay();
        }
    }
}

function setVideoTransport(name) {
    videoTransport = name || "";
    updateInfoDisplay();
}

async function startAuStream() {
    setVideoTransport("HTTPS /au");
    const params = new URLSearchParams();
    if (displaySessionId) params.set("session", displaySessionId);
    const qs = params.toString();
    const res = await fetch(qs ? `/au?${qs}` : "/au", {
        headers: authToken ? { authorization: `Bearer ${authToken}` } : {},
    });
    if (res.status === 401 || res.status === 403) {
        authToken = "";
        displaySessionId = "";
        showPairing("Credential rejected or revoked. Request host approval again.");
        return;
    }
    if (!res.ok || !res.body) {
        throw new Error(`au stream ${res.status}`);
    }
    const reader = res.body.getReader();
    auReader = reader;
    const frames = new OrbiAnnexB.FrameReader();
    (async () => {
        try {
            while (true) {
                const { value, done } = await reader.read();
                if (done) break;
                if (value && streamStats) streamStats.noteBytes(value.byteLength);
                frames.push(value);
                let msg;
                while ((msg = frames.pop())) {
                    if (msg.type === "video") feedAccessUnit(msg);
                }
            }
            scheduleReconnect("au ended");
            if (!userDisconnected) scheduleReconnect("au ended");
        } catch (err) {
            scheduleReconnect(err.message || "au");
            if (!userDisconnected) scheduleReconnect(err.message || "au");
        }
    })();
    waitingForKeyframe = true;
    requestIdr();
    if (latencyWatchdog) clearInterval(latencyWatchdog);
    latencyWatchdog = setInterval(() => {
        if (waitingForKeyframe) requestIdr();
        if (!streamActive) return;
        if (lastFrameAt && Date.now() - lastFrameAt > 750) {
            requestIdr({ keepDecoding: true });
            if (Date.now() - lastFrameAt > 3000) startStream({ quiet: true });
        }
    }, 250);
}

async function startStream(opts = {}) {
    if (pairingAttempt) return;
    if (!authToken) {
        showPairing();
        return;
    }
    if (window.location.protocol !== "https:") {
        const host = (wtConfig && OrbiAnnexB.pickWtHost(wtConfig, window.location.hostname))
            || window.location.hostname
            || "host";
        const port = (wtConfig && wtConfig.wt_port) || 8790;
        setOverlayState(
            "error",
            "Open the HTTPS client",
            `WebTransport needs a secure page. Open https://${host}:${port}/client/ and accept the certificate.`,
        );
        return;
    }
    if (typeof VideoDecoder !== "function") {
        unsupportedPlayback();
        return;
    }
    if (!canPlayWebTransport()) {
        const s = playbackSupport();
        const missing = [
            !s.webTransport && "WebTransport",
            !s.annexb && "protocol helpers",
        ].filter(Boolean).join(" and ");
        setOverlayState("error", "Playback Error", `This browser is missing ${missing}`);
        return;
    }
    destroyPlayer();
    lastDelayMs = null;
    clockOffsetNs = 0n;
    lastFrameAt = 0;
    if (streamStats) streamStats.reset();
    pendingPresent.length = 0;
    if (!opts.quiet) {
        setOverlayState("connecting", "Connecting", "Connecting to Linux virtual display…");
    }

    const cfg = wtConfig || {};
    const port = cfg.wt_port;
    const path = cfg.wt_path || "/orbiscreen";
    const hashB64 = cfg.cert_sha256;
    if (!port || !hashB64) {
        setOverlayState("error", "Playback Error", "Host did not advertise WebTransport");
        return;
    }

    const host = OrbiAnnexB.pickWtHost(cfg, window.location.hostname);
    const url = `https://${host}:${port}${path}`;
    waitingForKeyframe = true;

    try {
        const hash = OrbiAnnexB.hashFromBase64(hashB64);
        let transport;
        let lastErr;
        for (const opts of [
            { serverCertificateHashes: [{ algorithm: "sha-256", value: hash }] },
            {},
        ]) {
            let candidate;
            try {
                candidate = new WebTransport(url, opts);
                await Promise.race([
                    candidate.ready,
                    new Promise((_, reject) => {
                        setTimeout(() => reject(new Error("webtransport ready timeout")), 4000);
                    }),
                ]);
                transport = candidate;
                lastErr = null;
                break;
            } catch (err) {
                lastErr = err;
                try { if (candidate) candidate.close(); } catch (_) {}
            }
        }
        if (!transport) {
            console.warn("webtransport failed, using HTTPS AU stream:", lastErr);
            await startAuStream();
            return;
        }
        wtTransport = transport;
        setVideoTransport("WebTransport");
        const bidi = await transport.createBidirectionalStream();
        const writer = bidi.writable.getWriter();
        wtWriter = writer;
        await writer.write(OrbiAnnexB.encodeHello(authToken, displaySessionId || ""));

        const reader = bidi.readable.getReader();
        const frames = new OrbiAnnexB.FrameReader();
        const assembler = new OrbiAnnexB.DatagramAssembler();
        (async () => {
            try {
                while (wtTransport === transport) {
                    const { value, done } = await reader.read();
                    if (done) break;
                    if (value && streamStats) streamStats.noteBytes(value.byteLength);
                    frames.push(value);
                    let msg;
                    while ((msg = frames.pop())) {
                        if (msg.type === "helloAck") {
                            if (msg.width) displayWidth = msg.width;
                            if (msg.height) displayHeight = msg.height;
                            updateInfoDisplay();
                            requestIdr();
                        } else if (msg.type === "video") {
                            if (msg.key) assembler.onReliableKeyframe();
                            feedAccessUnit(msg);
                        } else if (msg.type === "pong") {
                            const now = nowNs();
                            const rtt = now - msg.t0Ns;
                            clockOffsetNs = msg.hostNs + rtt / 2n - now;
                            if (streamStats) streamStats.noteClockOffset(clockOffsetNs);
                        }
                    }
                }
            } catch (err) {
                if (wtTransport === transport) {
                    console.error("webtransport read:", err);
                    displaySessionId = "";
                    scheduleReconnect(err.message || "transport");
                }
            }
        })();

        const takeAssembler = (msgs) => {
            for (const msg of msgs) {
                if (msg.type === "gap") {
                    waitingForKeyframe = true;
                    if (streamStats) streamStats.noteDropped(msg.dropped || 1);
                    requestIdr();
                } else if (msg.type === "video") {
                    feedAccessUnit(msg);
                }
            }
        };
        if (holeWatchdog) clearInterval(holeWatchdog);
        holeWatchdog = setInterval(() => {
            if (wtTransport !== transport) return;
            takeAssembler(assembler.expire());
        }, 16);
        let dgramReader = null;
        try {
            dgramReader = transport.datagrams.readable.getReader();
        } catch (err) {
            console.warn("webtransport datagrams unavailable:", err);
        }
        if (dgramReader) {
            (async () => {
                try {
                    while (wtTransport === transport) {
                        const { value, done } = await dgramReader.read();
                        if (done) break;
                        if (value && streamStats) streamStats.noteBytes(value.byteLength);
                        takeAssembler(assembler.push(value));
                    }
                } catch (err) {
                    console.warn("webtransport datagram:", err);
                }
            })();
        }
    } catch (err) {
        console.error("webtransport connect:", err);
        if (!authToken && tokenRow) tokenRow.classList.remove("hidden");
        scheduleReconnect(err.message || "connect");
        return;
    }

    const wtStartedAt = Date.now();
    let auFallback = false;
    latencyWatchdog = setInterval(() => {
        if (waitingForKeyframe) requestIdr();
        if (wtWriter) {
            try { wtWriter.write(OrbiAnnexB.encodePing(nowNs())); } catch (_) {}
        }
        if (!auFallback && !streamActive && Date.now() - wtStartedAt > 3500) {
            auFallback = true;
            console.warn("webtransport produced no picture; falling back to /au");
            try { if (wtWriter) wtWriter.close(); } catch (_) {}
            try { if (wtTransport) wtTransport.close(); } catch (_) {}
            wtWriter = null;
            wtTransport = null;
            startAuStream().catch((err) => {
                scheduleReconnect(err.message || "au");
            });
            return;
        }
        if (!streamActive) return;
        if (lastFrameAt && Date.now() - lastFrameAt > 750) {
            requestIdr({ keepDecoding: true });
            if (Date.now() - lastFrameAt > 3000) {
                startStream({ quiet: true });
            }
        }
    }, 250);
}

let displaySessionId = "";

function webDeviceKey() {
    const store = "orbiscreen.deviceKey";
    try {
        const existing = localStorage.getItem(store);
        if (existing && /^[0-9a-f]{8}$/.test(existing)) {
            return existing;
        }
        const bytes = new Uint8Array(4);
        crypto.getRandomValues(bytes);
        const key = Array.from(bytes, (b) => b.toString(16).padStart(2, "0")).join("");
        localStorage.setItem(store, key);
        return key;
    } catch (_) {
        return "web";
    }
}

function nativeScreenSize() {
    const dpr = (typeof window !== "undefined" && window.devicePixelRatio) || 1;
    const cssW = (typeof window !== "undefined" && window.screen && window.screen.width) || 1920;
    const cssH = (typeof window !== "undefined" && window.screen && window.screen.height) || 1080;
    return {
        width: Math.max(1, Math.round(cssW * dpr)),
        height: Math.max(1, Math.round(cssH * dpr)),
    };
}

async function openDisplaySession() {
    if (!authToken) return;
    try {
        const body = {
            name: (typeof navigator !== "undefined" && navigator.userAgent)
                ? navigator.userAgent.split(/[()]/)[0].trim() || "web"
                : "web",
            key: webDeviceKey(),
            ...nativeScreenSize(),
        };
        const response = await fetch("/api/session", {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
                Authorization: `Bearer ${authToken}`,
            },
            body: JSON.stringify(body),
        });
        if (response.status === 401 || response.status === 403) {
            authToken = "";
            displaySessionId = "";
            showPairing("Credential rejected or revoked. Request host approval again.");
            return;
        }
        if (!response.ok) return;
        const data = await response.json();
        if (data && data.id) {
            displaySessionId = data.id;
            if (Number.isFinite(data.width) && Number.isFinite(data.height)) {
                displayWidth = data.width;
                displayHeight = data.height;
            }
            if (typeof data.encoder === "string") {
                encoderName = data.encoder.toUpperCase();
            }
        }
    } catch (error) {
        console.warn("session open failed:", error);
    }
}

async function start() {
    userDisconnected = false;
    applyTheme(currentTheme);
    applyTranslations();
    if (pairingAttempt) return;
    if (window.location.protocol !== "https:") {
        showPairing();
        return;
    }
    if (typeof VideoDecoder !== "function") {
        unsupportedPlayback();
        return;
    }
    const info = await fetchClientConfig();
    if (info) {
        wtConfig = info;
        if (!authToken && typeof info.token === "string" && info.token.length > 0) {
            authToken = info.token;
        }
        if (Number.isFinite(info.display_width) && Number.isFinite(info.display_height)) {
            displayWidth = info.display_width;
            displayHeight = info.display_height;
        }
    }
    try {
        const response = await fetch("/api/info", {
            headers: authToken ? { Authorization: `Bearer ${authToken}` } : {},
            cache: "no-store",
            redirect: "error",
        });
        if (response.ok) {
            const apiInfo = await response.json();
            if (Number.isFinite(apiInfo?.display_width) && Number.isFinite(apiInfo?.display_height)) {
                displayWidth = apiInfo.display_width;
                displayHeight = apiInfo.display_height;
            }
            if (typeof apiInfo?.encoder === "string") {
                encoderName = apiInfo.encoder.toUpperCase();
            }
            wtConfig = Object.assign({}, wtConfig || {}, apiInfo);
        }
    } catch (error) {
        console.warn("api/info fetch failed:", error);
    }

    if (!authToken) {
        showPairing();
        return;
    }
    pairingPanel.classList.add("hidden");
    displaySessionId = "";
    await openDisplaySession();
    if (!authToken) return;
    if (!displaySessionId) {
        setOverlayState("error", "Session unavailable", "Could not open a display session. Reconnect to retry.");
        return;
    }
    updateInfoDisplay();
    startStream();
}

start().catch((error) => {
    setOverlayState("error", "Initialization Failed", error.message);
    console.error(error);
});
