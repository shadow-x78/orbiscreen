// Orbiscreen - app.js (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

const I18N = {
    en: {
        btnInputMode: "Input Mode",
        btnKeyboard: "Keyboard",
        btnLock: "Lock Session",
        btnSettings: "Settings",
        btnHideControls: "Hide Toolbar",
        btnFullscreen: "Fullscreen",
        btnDisconnect: "Disconnect",
        btnRestoreToolbar: "Show Toolbar",
        btnCloseKeyboard: "Close",
        settingsTitle: "Settings",
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
        actionBlank: "Blank Display",
        actionResync: "Resync",
        actionTurnOn: "Display On",
        actionTurnOff: "Display Off",
        statusConnecting: "Connecting",
        statusConnectingSub: "Connecting...",
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
        toastBlanked: "Blanked",
        toastUnblanked: "Active",
        toastCadSent: "Ctrl+Alt+Del sent",
        toastResynced: "Resynced",
        toastDisconnected: "Disconnected"
    },
    ar: {
        btnInputMode: "نمط الإدخال",
        btnKeyboard: "لوحة المفاتيح",
        btnLock: "قفل الجلسة",
        btnSettings: "الإعدادات",
        btnHideControls: "إخفاء الشريط",
        btnFullscreen: "ملء الشاشة",
        btnDisconnect: "قطع الاتصال",
        btnRestoreToolbar: "إظهار الشريط",
        btnCloseKeyboard: "إغلاق",
        settingsTitle: "الإعدادات",
        language: "اللغة",
        fitMode: "التحجيم",
        fitContain: "ملاءمة",
        fitCover: "ملء",
        fitNone: "100%",
        perfStats: "الإحصائيات",
        latency: "الاستجابة",
        resolution: "الدقة",
        encoder: "المُرَمِّز",
        hostActions: "إجراءات المضيف",
        actionBlank: "تعتيم",
        actionResync: "مزامنة",
        actionTurnOn: "تشغيل الشاشة",
        actionTurnOff: "تعتيم الشاشة",
        statusConnecting: "جارٍ الاتصال",
        statusConnectingSub: "جارٍ الاتصال...",
        statusConnected: "متصل",
        statusDisconnected: "غير متصل",
        statusStreamError: "خطأ في البث",
        statusStreamErrorSub: "جارٍ إعادة المحاولة...",
        statusAuthFailed: "المصادقة مطلوبة",
        statusAuthFailedSub: "أدخل رمز الجلسة",
        tokenPlaceholder: "رمز الجلسة...",
        btnConnect: "اتصال",
        btnReconnect: "إعادة الاتصال",
        vncActive: "التحكم نشط · اضغط <kbd>Esc</kbd> للتحرير",
        cursorReleased: "تم التحرير",
        modeTouch: "لمس",
        modeTouchpad: "لوحة اللمس",
        controlsHidden: "تم إخفاء الشريط",
        toastLocked: "تم القفل",
        toastLockSent: "تم إرسال الأمر",
        toastBlanked: "تم التعتيم",
        toastUnblanked: "الشاشة نشطة",
        toastCadSent: "تم إرسال الإشارة",
        toastResynced: "تمت المزامنة",
        toastDisconnected: "تم قطع الاتصال"
    }
};

let currentLang = localStorage.getItem("orbiscreen_lang") || (navigator.language && navigator.language.startsWith("ar") ? "ar" : "en");

function t(key) {
    const dict = I18N[currentLang] || I18N.en;
    return dict[key] || I18N.en[key] || key;
}

function applyTranslations() {
    document.documentElement.lang = currentLang;
    document.documentElement.dir = currentLang === "ar" ? "rtl" : "ltr";

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

    const btnEn = document.getElementById("btnLangEn");
    const btnAr = document.getElementById("btnLangAr");
    if (btnEn) btnEn.classList.toggle("active", currentLang === "en");
    if (btnAr) btnAr.classList.toggle("active", currentLang === "ar");
}

function setLanguage(lang) {
    if (!I18N[lang]) return;
    currentLang = lang;
    localStorage.setItem("orbiscreen_lang", lang);
    applyTranslations();
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
const miniPill = document.getElementById("miniPill");
const hostNameEl = document.getElementById("hostName");
const hostInfoEl = document.getElementById("hostInfo");
const miniInfoEl = document.getElementById("miniInfo");

function setOverlayState(state, title, subtitle) {
    if (overlayEl) overlayEl.classList.remove("hidden");
    if (statusTitle && title) statusTitle.textContent = title;
    if (statusSubtitle && subtitle) statusSubtitle.textContent = subtitle;

    releaseControl();
    if (controlToolbar) controlToolbar.classList.add("hidden");
    if (miniPill) miniPill.classList.add("hidden");
    if (keyboardDrawer) keyboardDrawer.classList.add("hidden");
    if (settingsModal) settingsModal.classList.add("hidden");

    if (state === "connecting") {
        if (brandLogo) brandLogo.classList.add("hidden");
        if (statusSpinner) statusSpinner.classList.remove("hidden");
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
const btnHideControls = document.getElementById("btnHideControls");
const btnFullscreen = document.getElementById("btnFullscreen");
const btnDisconnect = document.getElementById("btnDisconnect");
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
const btnActionBlank = document.getElementById("btnActionBlank");
const btnActionResync = document.getElementById("btnActionResync");

const vncBanner = document.getElementById("vncBanner");
const toastEl = document.getElementById("toast");
const tokenRow = document.getElementById("tokenRow");
const tokenInput = document.getElementById("tokenInput");
const btnConnect = document.getElementById("btnConnect");

let displayWidth = 1920;
let displayHeight = 1080;
let encoderName = "NVENC";
let authToken = "";
let mpegtsPlayer = null;
let reconnectTimer = null;
let reconnectDelay = 1000;
const MAX_RECONNECT_DELAY = 10000;
let streamActive = false;
let isVncFocused = false;
let isTouchMode = true;
let isDpmsOff = false;
let toastTimer = null;
let vncBannerTimer = null;
let latencyWatchdog = null;
const heldKeys = new Set();
const pressedButtons = new Set();
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

function updateInfoDisplay() {
    const infoStr = `${displayWidth}×${displayHeight}  ${encoderName}`;
    if (hostInfoEl) hostInfoEl.textContent = infoStr;
    if (miniInfoEl) miniInfoEl.textContent = `${displayWidth}×${displayHeight}`;
    if (statRes) statRes.textContent = `${displayWidth} × ${displayHeight}`;
    if (statEncoder) statEncoder.textContent = encoderName;
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

stageEl.addEventListener("pointerdown", (event) => {
    if (!streamActive) return;
    if (!isVncFocused) {
        setVncFocus(true);
    }
    event.preventDefault();
    const { x, y } = mapPointer(event);
    sendPointerMove(x, y);
    sendPointerButton(event.button + 1, true);
    if (event.pointerType === "pen") {
        sendStylus(x, y, event.pressure, event.tiltX, event.tiltY);
    }
    showTouch(event.clientX, event.clientY);
});

stageEl.addEventListener("pointermove", (event) => {
    if (!streamActive || !isVncFocused) return;
    event.preventDefault();
    const { x, y } = mapPointer(event);
    if (event.buttons > 0) {
        showTouch(event.clientX, event.clientY);
        if (event.pointerType === "pen") {
            sendStylus(x, y, event.pressure, event.tiltX, event.tiltY);
        }
        sendPointerMove(x, y);
    } else {
        queuePointerMove(x, y);
    }
});

stageEl.addEventListener("pointerup", (event) => {
    if (!streamActive || !isVncFocused) return;
    event.preventDefault();
    sendPointerButton(event.button + 1, false);
    hideTouch();
});

stageEl.addEventListener("pointerleave", () => {
    if (isVncFocused) {
        releaseAllButtons();
    }
});

stageEl.addEventListener("pointercancel", () => {
    if (isVncFocused) {
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

if (btnInputMode) {
    btnInputMode.addEventListener("click", (e) => {
        e.stopPropagation();
        isTouchMode = !isTouchMode;
        if (isTouchMode) {
            iconMouse.classList.add("hidden");
            iconTouch.classList.remove("hidden");
            showToast(t("modeTouch"));
        } else {
            iconMouse.classList.remove("hidden");
            iconTouch.classList.add("hidden");
            showToast(t("modeTouchpad"));
        }
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
        videoEl.style.objectFit = btn.dataset.fit;
        showToast(`Fit: ${btn.textContent}`);
    });
});

if (btnHideControls) {
    btnHideControls.addEventListener("click", (e) => {
        e.stopPropagation();
        controlToolbar.classList.add("hidden");
        miniPill.classList.remove("hidden");
        showToast(t("controlsHidden"), 1800);
    });
}

if (btnRestoreToolbar) {
    btnRestoreToolbar.addEventListener("click", (e) => {
        e.stopPropagation();
        miniPill.classList.add("hidden");
        controlToolbar.classList.remove("hidden");
    });
}

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

const btnLangEn = document.getElementById("btnLangEn");
const btnLangAr = document.getElementById("btnLangAr");
if (btnLangEn) {
    btnLangEn.addEventListener("click", (e) => {
        e.stopPropagation();
        setLanguage("en");
    });
}
if (btnLangAr) {
    btnLangAr.addEventListener("click", (e) => {
        e.stopPropagation();
        setLanguage("ar");
    });
}

if (btnLock) {
    btnLock.addEventListener("click", async (e) => {
        e.stopPropagation();
        const ok = await sendHostAction("lock");
        showToast(ok ? t("toastLocked") : t("toastLockSent"));
    });
}

if (btnActionBlank) {
    btnActionBlank.addEventListener("click", async (e) => {
        e.stopPropagation();
        isDpmsOff = !isDpmsOff;
        await sendHostAction(isDpmsOff ? "blank" : "unblank");
        btnActionBlank.textContent = isDpmsOff ? t("actionTurnOn") : t("actionTurnOff");
        showToast(isDpmsOff ? t("toastBlanked") : t("toastUnblanked"));
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
        startStream();
        showToast(t("toastResynced"));
    });
}

if (btnDisconnect) {
    btnDisconnect.addEventListener("click", (e) => {
        e.stopPropagation();
        destroyPlayer();
        setOverlayState("disconnected", t("statusDisconnected"), t("statusDisconnected"));
        showToast(t("toastDisconnected"));
    });
}

if (btnReconnect) {
    btnReconnect.addEventListener("click", (e) => {
        e.stopPropagation();
        setOverlayState("connecting", t("statusConnecting"), t("statusConnectingSub"));
        startStream();
    });
}

if (btnConnect && tokenInput) {
    btnConnect.addEventListener("click", () => {
        const val = tokenInput.value.trim();
        if (val) {
            authToken = val;
            tokenRow.classList.add("hidden");
            startStream();
        }
    });
}

function sendInput(payload) {
    const headers = { "content-type": "application/json" };
    if (authToken) headers.authorization = `Bearer ${authToken}`;
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
    const vh = videoEl.videoHeight || displayHeight;
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
    const rect = videoEl.getBoundingClientRect();
    const vw = videoEl.videoWidth || displayWidth;
    const vh = videoEl.videoHeight || displayHeight;
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

function canPlayMpegTs() {
    return typeof mpegts !== "undefined"
        && mpegts.getFeatureList().mseLivePlayback;
}

function destroyPlayer() {
    if (reconnectTimer) {
        clearTimeout(reconnectTimer);
        reconnectTimer = null;
    }
    if (mpegtsPlayer) {
        mpegtsPlayer.destroy();
        mpegtsPlayer = null;
    }
    if (videoEl.src) {
        videoEl.pause();
        videoEl.removeAttribute("src");
        videoEl.load();
    }
    streamActive = false;
    clearInterval(latencyWatchdog);
    releaseControl();
}

async function fetchClientConfig() {
    try {
        const cfg = await fetch("/client/config.json");
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
    if (reconnectTimer) return;
    destroyPlayer();
    setOverlayState("connecting", "Connecting", `Reconnecting (${reason})…`);
    reconnectTimer = setTimeout(() => {
        reconnectTimer = null;
        (async () => {
            await refreshToken();
            startStream();
        })().catch((error) => {
            console.warn("reconnect attempt failed:", error);
            scheduleReconnect("retry error");
        });
    }, reconnectDelay);
    reconnectDelay = Math.min(reconnectDelay * 2, MAX_RECONNECT_DELAY);
}

function startStream() {
    if (!canPlayMpegTs()) {
        setOverlayState("error", "Playback Error", "Browser lacks MSE live video playback support");
        return;
    }
    destroyPlayer();
    setOverlayState("connecting", "Connecting", "Connecting to Linux virtual display…");

    const streamUrl = authToken
        ? `/stream?token=${encodeURIComponent(authToken)}`
        : "/stream";

    mpegtsPlayer = mpegts.createPlayer({
        type: "mpegts",
        isLive: true,
        url: streamUrl,
    }, {
        enableStashBuffer: false,
        stashInitialSize: 128,
        autoCleanupSourceBuffer: true,
        autoCleanupMaxBackwardDuration: 2,
        autoCleanupMinBackwardDuration: 1,
        lazyLoad: false,
        lazyLoadMaxDuration: 0,
        seekType: "range",
        liveBufferLatencyChasing: true,
        liveBufferLatencyMaxLatency: 0.08,
        liveBufferLatencyMinRemain: 0.016,
        liveSync: true,
        liveSyncTargetLatency: 0.03,
    });

    videoEl.muted = true;
    mpegtsPlayer.attachMediaElement(videoEl);

    mpegtsPlayer.on(mpegts.Events.ERROR, (errorType, errorDetail, errorInfo) => {
        console.error("mpegts error:", errorType, errorDetail, errorInfo);
        sendHostAction("idr");
        if (errorDetail === "NetworkError" && !authToken && tokenRow) {
            tokenRow.classList.remove("hidden");
        }
        scheduleReconnect(errorDetail || errorType || "network");
    });

    mpegtsPlayer.on(mpegts.Events.MEDIA_INFO, (mediaInfo) => {
        if (mediaInfo && mediaInfo.width && mediaInfo.height) {
            displayWidth = mediaInfo.width;
            displayHeight = mediaInfo.height;
            updateInfoDisplay();
        }
    });

    mpegtsPlayer.load();
    const playPromise = mpegtsPlayer.play();
    if (playPromise && typeof playPromise.catch === "function") {
        playPromise.catch((error) => {
            console.warn("autoplay blocked or rejected:", error);
            setOverlayState("connecting", "Ready to Stream", "Click anywhere on the screen to begin");
            const onFirstClick = () => {
                window.removeEventListener("pointerdown", onFirstClick);
                mpegtsPlayer.play().catch(() => {});
            };
            window.addEventListener("pointerdown", onFirstClick);
        });
    }

    latencyWatchdog = setInterval(() => {
        if (!streamActive || !videoEl || !videoEl.buffered || videoEl.buffered.length === 0) return;
        try {
            const end = videoEl.buffered.end(videoEl.buffered.length - 1);
            const delay = end - videoEl.currentTime;
            const ms = Math.max(0, Math.round(delay * 1000));
            if (statLatency) statLatency.textContent = `${ms} ms`;
            if (delay > 0.08) {
                videoEl.currentTime = end - 0.02;
            }
        } catch (_) {}
    }, 250);
}

async function start() {
    applyTranslations();
    const info = await fetchClientConfig();
    if (info) {
        if (!authToken && typeof info.token === "string" && info.token.length > 0) {
            authToken = info.token;
        }
        if (Number.isFinite(info.display_width) && Number.isFinite(info.display_height)) {
            displayWidth = info.display_width;
            displayHeight = info.display_height;
        }
    }
    try {
        const response = await fetch("/api/info");
        if (response.ok) {
            const apiInfo = await response.json();
            if (Number.isFinite(apiInfo?.display_width) && Number.isFinite(apiInfo?.display_height)) {
                displayWidth = apiInfo.display_width;
                displayHeight = apiInfo.display_height;
            }
            if (typeof apiInfo?.encoder === "string") {
                encoderName = apiInfo.encoder.toUpperCase();
            }
        }
    } catch (error) {
        console.warn("api/info fetch failed:", error);
    }

    updateInfoDisplay();
    startStream();
}

videoEl.addEventListener("playing", () => {
    streamActive = true;
    reconnectDelay = 1000;
    if (overlayEl) overlayEl.classList.add("hidden");
    if (controlToolbar) controlToolbar.classList.remove("hidden");
    if (miniPill) miniPill.classList.add("hidden");
});

videoEl.addEventListener("error", () => {
    if (streamActive || mpegtsPlayer) {
        scheduleReconnect("media error");
    }
});

videoEl.addEventListener("ended", () => {
    if (streamActive || mpegtsPlayer) {
        scheduleReconnect("stream ended");
    }
});

start().catch((error) => {
    setOverlayState("error", "Initialization Failed", error.message);
    console.error(error);
});
