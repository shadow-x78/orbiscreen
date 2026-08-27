// Orbiscreen - web client (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen
const statusEl = document.getElementById("status");
const resolutionEl = document.getElementById("resolution");
const overlayEl = document.getElementById("overlay");
const videoEl = document.getElementById("remoteVideo");
const touchIndicator = document.getElementById("touchIndicator");

let displayWidth = 1920;
let displayHeight = 1080;
let authToken = "";
let mpegtsPlayer = null;
let reconnectTimer = null;
let reconnectDelay = 1000;
const MAX_RECONNECT_DELAY = 10000;
let streamActive = false;
const pressedButtons = new Set();

function setStatus(text) {
    statusEl.textContent = text;
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

function sendPointerButton(button, pressed) {
    if (pressed) {
        pressedButtons.add(button);
    } else {
        pressedButtons.delete(button);
    }
    sendInput({ Pointer: { Button: { button, pressed } } });
}

function releaseAllButtons() {
    for (const button of pressedButtons) {
        sendInput({ Pointer: { Button: { button, pressed: false } } });
    }
    pressedButtons.clear();
    hideTouch();
}

function sendWheel(deltaY) {
    sendInput({ Pointer: { Wheel: { delta_y: deltaY } } });
}

function normalizeWheel(event) {
    const vw = videoEl.videoWidth || displayWidth;
    const vh = videoEl.videoHeight || displayHeight;
    let pixels;
    if (event.deltaMode === 1) {
        pixels = event.deltaY * 16;
    } else if (event.deltaMode === 2) {
        pixels = event.deltaY * vh;
    } else {
        pixels = event.deltaY;
    }
    const steps = pixels / 100;
    return Math.max(-12, Math.min(12, steps));
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

let controlActive = false;
let controlFallback = false;
let virtualX = 0;
let virtualY = 0;

function videoDimensions() {
    return {
        w: videoEl.videoWidth || displayWidth,
        h: videoEl.videoHeight || displayHeight,
    };
}

function hasPointerLockSupport() {
    return typeof videoEl.requestPointerLock === "function";
}

function enterControl(event) {
    if (hasPointerLockSupport() && event.pointerType === "mouse") {
        const maybePromise = videoEl.requestPointerLock();
        if (maybePromise && typeof maybePromise.catch === "function") {
            maybePromise.catch(() => {
                controlFallback = true;
                controlActive = true;
                setControlHint();
            });
        }
        return false;
    }
    controlFallback = true;
    controlActive = true;
    setControlHint();
    return true;
}

function exitControl() {
    controlActive = false;
    controlFallback = false;
    releaseAllButtons();
    setControlHint();
}

function relativeMove(event) {
    const { w, h } = videoDimensions();
    virtualX = Math.max(0, Math.min(w - 1, virtualX + event.movementX));
    virtualY = Math.max(0, Math.min(h - 1, virtualY + event.movementY));
    return { x: virtualX, y: virtualY };
}

function showTouch(x, y) {
    touchIndicator.style.left = `${x}px`;
    touchIndicator.style.top = `${y}px`;
    touchIndicator.classList.remove("hidden");
}

function hideTouch() {
    touchIndicator.classList.add("hidden");
}

function canPlayMpegTs() {
    return typeof mpegts !== "undefined"
        && mpegts.getFeatureList().mseLivePlayback;
}

function destroyPlayer() {
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
    exitControl();
}

async function fetchClientConfig() {
    try {
        const cfg = await fetch("/client/config.json");
        if (cfg.ok) {
            return await cfg.json();
        }
    } catch (error) {
        console.warn("config.json fetch failed:", error);
    }
    return null;
}

async function refreshToken() {
    const info = await fetchClientConfig();
    if (info && typeof info.token === "string" && info.token.length > 0) {
        authToken = info.token;
    }
}

function scheduleReconnect(reason) {
    if (reconnectTimer) return;
    destroyPlayer();
    overlayEl.classList.remove("hidden");
    setStatus(`Stream lost (${reason}) — retrying in ${reconnectDelay / 1000}s…`);
    reconnectTimer = setTimeout(() => {
        reconnectTimer = null;
        (async () => {
            await refreshToken();
            startStream();
        })().catch((error) => {
            console.warn("reconnect attempt failed:", error);
            scheduleReconnect("reconnect error");
        });
    }, reconnectDelay);
    reconnectDelay = Math.min(reconnectDelay * 2, MAX_RECONNECT_DELAY);
}

function startStream() {
    if (!canPlayMpegTs()) {
        overlayEl.classList.remove("hidden");
        setStatus("This browser does not support MSE playback (Chrome/Firefox/Edge required)");
        return;
    }
    destroyPlayer();
    setStatus("Connecting to stream…");

    const streamUrl = authToken
        ? `/stream?token=${encodeURIComponent(authToken)}`
        : "/stream";
    mpegtsPlayer = mpegts.createPlayer({
        type: "mpegts",
        isLive: true,
        url: streamUrl,
    }, {
        autoCleanupSourceBuffer: true,
        enableStashBuffer: false,
        lazyLoad: false,
        lazyLoadMaxDuration: 0,
        seekType: "range",
        liveBufferLatencyChasing: true,
        liveBufferLatencyMaxLatency: 1.0,
        liveBufferLatencyMinRemain: 0.2,
        liveSync: true,
    });

    mpegtsPlayer.attachMediaElement(videoEl);

    mpegtsPlayer.on(mpegts.Events.ERROR, (errorType, errorDetail, errorInfo) => {
        console.error("mpegts error:", errorType, errorDetail, errorInfo);
        scheduleReconnect(errorDetail || errorType || "unknown error");
    });
    mpegtsPlayer.on(mpegts.Events.MEDIA_INFO, (mediaInfo) => {
        if (mediaInfo && mediaInfo.width && mediaInfo.height) {
            displayWidth = mediaInfo.width;
            displayHeight = mediaInfo.height;
            resolutionEl.textContent = `${displayWidth} × ${displayHeight}`;
        }
    });

    mpegtsPlayer.load();
    mpegtsPlayer.play().catch((error) => {
        console.error("play() failed:", error);
        scheduleReconnect("play() rejected");
    });
}

async function start() {
    setStatus("Reading host info…");

    const info = await fetchClientConfig();
    if (info) {
        if (typeof info.token === "string" && info.token.length > 0) {
            authToken = info.token;
        }
        if (Number.isFinite(info.display_width)) displayWidth = info.display_width;
        if (Number.isFinite(info.display_height)) displayHeight = info.display_height;
    }
    try {
        const response = await fetch("/api/info");
        if (response.ok) {
            const apiInfo = await response.json();
            if (Number.isFinite(apiInfo?.display_width)
                && Number.isFinite(apiInfo?.display_height)) {
                displayWidth = apiInfo.display_width;
                displayHeight = apiInfo.display_height;
            }
        }
    } catch (error) {
        console.warn("api/info fetch failed:", error);
    }
    resolutionEl.textContent = `${displayWidth} × ${displayHeight}`;

    overlayEl.classList.add("hidden");
    startStream();
}

videoEl.addEventListener("playing", () => {
    streamActive = true;
    reconnectDelay = 1000;
    overlayEl.classList.add("hidden");
    setStatus("Streaming");
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

function setControlHint() {
    const hint = document.getElementById("controlHint");
    if (!hint) return;
    if (controlActive) {
        hint.textContent = controlFallback
            ? "Control active - tap outside the video to release"
            : "Control active - press Esc to release";
    } else {
        hint.textContent = "Click to control";
    }
}

function applyPointerDown(event) {
    const { x, y } = mapPointer(event);
    if (controlFallback) {
        const { w, h } = videoDimensions();
        virtualX = Math.max(0, Math.min(w - 1, x));
        virtualY = Math.max(0, Math.min(h - 1, y));
    }
    sendPointerMove(x, y);
    sendPointerButton(event.button + 1, true);
    if (event.pointerType === "pen") {
        sendStylus(x, y, event.pressure, event.tiltX, event.tiltY);
    }
    showTouch(event.clientX, event.clientY);
}

videoEl.addEventListener("pointerdown", (event) => {
    event.preventDefault();
    try {
        videoEl.setPointerCapture(event.pointerId);
    } catch (_) { }
    if (!controlActive) {
        const enteredFallback = enterControl(event);
        if (!enteredFallback) return;
    }
    applyPointerDown(event);
});

document.addEventListener("pointerlockchange", () => {
    const locked = document.pointerLockElement === videoEl;
    if (locked) {
        controlActive = true;
        controlFallback = false;
        const { w, h } = videoDimensions();
        virtualX = Math.floor(w / 2);
        virtualY = Math.floor(h / 2);
        setControlHint();
    } else if (controlActive && !controlFallback) {
        exitControl();
    }
});

videoEl.addEventListener("pointermove", (event) => {
    if (event.pointerType === "pen") {
        const { x, y } = mapPointer(event);
        sendStylus(x, y, event.pressure, event.tiltX, event.tiltY);
        return;
    }
    if (!controlActive) return;
    if (controlFallback) {
        const { x, y } = mapPointer(event);
        sendPointerMove(x, y);
    } else {
        const { x, y } = relativeMove(event);
        sendPointerMove(x, y);
    }
});

videoEl.addEventListener("pointerup", (event) => {
    if (!controlActive) return;
    sendPointerButton(event.button + 1, false);
    hideTouch();
});

videoEl.addEventListener("pointercancel", () => {
    releaseAllButtons();
});

videoEl.addEventListener("pointerleave", () => {
    releaseAllButtons();
});

videoEl.addEventListener("wheel", (event) => {
    if (!controlActive) return;
    event.preventDefault();
    sendWheel(normalizeWheel(event));
}, { passive: false });

window.addEventListener("keydown", (event) => {
    if (!streamActive || !controlActive || event.repeat) return;
    event.preventDefault();
    sendKey(event.code, true);
});

window.addEventListener("keyup", (event) => {
    if (!streamActive || !controlActive) return;
    event.preventDefault();
    sendKey(event.code, false);
});

document.addEventListener("pointerdown", (event) => {
    if (controlActive && controlFallback && event.target !== videoEl) {
        exitControl();
    }
});

start().catch((error) => {
    setStatus(`Error: ${error.message}`);
    console.error(error);
});
