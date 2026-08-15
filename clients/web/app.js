// Orbiscreen - web client (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen
//
// Playback uses MSE via the locally-vendored mpegts.js (no CDN): browsers
// cannot feed a raw MPEG-TS HTTP stream to <video src>, so we demux it here
// and push H.264 into a MediaSource. On any failure we tear the player down
// and reconnect with exponential backoff.

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
    sendInput({ Pointer: { Button: { button, pressed } } });
}

function sendWheel(deltaY) {
    sendInput({ Pointer: { Wheel: { delta_y: deltaY } } });
}

// Linux input event codes (see linux/input-event-codes.h).
const KEYCODE_MAP = {
    Escape: 1, Enter: 28, Backspace: 14, Tab: 15, Space: 57,
    ArrowUp: 103, ArrowDown: 108, ArrowLeft: 105, ArrowRight: 106,
    ShiftLeft: 42, ShiftRight: 54, ControlLeft: 29, ControlRight: 97,
    AltLeft: 56, AltRight: 100, MetaLeft: 125, MetaRight: 126,
    Delete: 111, Insert: 110, Home: 102, End: 107,
    PageUp: 104, PageDown: 109, CapsLock: 58, NumLock: 69,
    ScrollLock: 70, PrintScreen: 99, Pause: 119, ContextMenu: 127,
};

// F1..F10 start at evdev code 59, then jump: F11=87, F12=88.
for (let i = 0; i < 10; i += 1) KEYCODE_MAP[`F${i + 1}`] = 59 + i;
KEYCODE_MAP.F11 = 87;
KEYCODE_MAP.F12 = 88;
for (let i = 0; i < 10; i += 1) {
    KEYCODE_MAP[`Digit${(i + 1) % 10}`] = 2 + i;
    KEYCODE_MAP[`Numpad${i}`] = i === 0 ? 82 : 71 + (i - 1);
}
for (let i = 0; i < 26; i += 1) {
    KEYCODE_MAP[`Key${String.fromCharCode(65 + i)}`] = [16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
        30, 31, 32, 33, 34, 35, 36, 37, 38, 44, 45, 46, 47, 48, 49, 50][i];
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
    if (code === undefined) return; // untranslatable key; nothing the host can do
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
    // The video is letterboxed by object-fit: contain — map coordinates
    // within the visible content box back to the stream's intrinsic size.
    const scale = Math.min(rect.width / vw, rect.height / vh);
    const offsetX = (rect.width - vw * scale) / 2;
    const offsetY = (rect.height - vh * scale) / 2;
    const clamp = (v, max) => Math.max(0, Math.min(max, v));
    return {
        x: clamp((event.clientX - rect.left - offsetX) / scale, vw),
        y: clamp((event.clientY - rect.top - offsetY) / scale, vh),
    };
}

function showTouch(x, y) {
    touchIndicator.hidden = false;
    touchIndicator.style.left = `${x}px`;
    touchIndicator.style.top = `${y}px`;
    touchIndicator.classList.remove("hidden");
}

function hideTouch() {
    touchIndicator.classList.add("hidden");
}

// ---------------------------------------------------------------------------
// Stream playback (MSE)
// ---------------------------------------------------------------------------

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
}

function scheduleReconnect(reason) {
    if (reconnectTimer) return;
    destroyPlayer();
    setStatus(`Stream lost (${reason}) — retrying in ${reconnectDelay / 1000}s…`);
    reconnectTimer = setTimeout(() => {
        reconnectTimer = null;
        startStream();
    }, reconnectDelay);
    reconnectDelay = Math.min(reconnectDelay * 2, MAX_RECONNECT_DELAY);
}

function startStream() {
    if (!canPlayMpegTs()) {
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
        // Live tuning: chase the live edge instead of buffering up.
        autoCleanupSourceBuffer: true,
        liveBufferLatencyChasing: true,
        lazyLoad: false,
        lazyLoadMaxDuration: 0,
        seekType: "range",
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
    });
}

async function start() {
    setStatus("Reading host info…");

    // Bootstrap: resolution + session token (served by the daemon itself).
    try {
        const cfg = await fetch("/client/config.json");
        if (cfg.ok) {
            const info = await cfg.json();
            if (typeof info.token === "string" && info.token.length > 0) {
                authToken = info.token;
            }
            if (Number.isFinite(info.display_width)) displayWidth = info.display_width;
            if (Number.isFinite(info.display_height)) displayHeight = info.display_height;
        }
    } catch (error) {
        console.warn("config.json fetch failed:", error);
    }
    try {
        const response = await fetch("/api/info");
        if (response.ok) {
            const info = await response.json();
            if (Number.isFinite(info?.display_width)
                && Number.isFinite(info?.display_height)) {
                displayWidth = info.display_width;
                displayHeight = info.display_height;
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
    reconnectDelay = 1000; // reset backoff after successful playback
    overlayEl.classList.add("hidden");
    setStatus("Streaming");
});

// If the live connection stalls (daemon restart, network drop) the stream
// element errors out — kick off the reconnect loop.
videoEl.addEventListener("error", () => {
    if (streamActive || mpegtsPlayer) {
        scheduleReconnect("media error");
    }
});

// Detect the daemon closing our HTTP body (reached end of stream).
videoEl.addEventListener("ended", () => {
    if (streamActive || mpegtsPlayer) {
        scheduleReconnect("stream ended");
    }
});

videoEl.addEventListener("pointerdown", (event) => {
    event.preventDefault();
    try {
        videoEl.setPointerCapture(event.pointerId);
    } catch (_) { /* pointer already released */ }
    const { x, y } = mapPointer(event);
    // Keep the pointer in sync before the button so drags start where expected.
    sendPointerMove(x, y);
    // DOM button 0/1/2 maps to Linux buttons 1/2/3.
    sendPointerButton(event.button + 1, true);
    if (event.pointerType === "pen") {
        sendStylus(x, y, event.pressure, event.tiltX, event.tiltY);
    }
    showTouch(event.clientX, event.clientY);
});

videoEl.addEventListener("pointermove", (event) => {
    const { x, y } = mapPointer(event);
    if (event.pointerType === "pen") {
        sendStylus(x, y, event.pressure, event.tiltX, event.tiltY);
    } else {
        sendPointerMove(x, y);
    }
});

videoEl.addEventListener("pointerup", (event) => {
    sendPointerButton(event.button + 1, false);
    hideTouch();
});

// Pointer cancelled (e.g. touch interrupted) — release every button or the
// host is left with a stuck pressed button.
videoEl.addEventListener("pointercancel", (event) => {
    if (event.buttons > 0) {
        sendPointerButton(1, false);
    }
    hideTouch();
});

videoEl.addEventListener("wheel", (event) => {
    event.preventDefault();
    sendWheel(event.deltaY);
}, { passive: false });

// Only swallow keys once the stream is up — before that, F5/DevTools etc.
// must still reach the browser.
let streamActive = false;

window.addEventListener("keydown", (event) => {
    if (streamActive) event.preventDefault();
    sendKey(event.code, true);
});

window.addEventListener("keyup", (event) => {
    if (streamActive) event.preventDefault();
    sendKey(event.code, false);
});

start().catch((error) => {
    setStatus(`Error: ${error.message}`);
    console.error(error);
});
