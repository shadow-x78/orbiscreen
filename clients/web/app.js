// Orbiscreen - web client (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

const statusEl = document.getElementById("status");
const resolutionEl = document.getElementById("resolution");
const overlayEl = document.getElementById("overlay");
const videoEl = document.getElementById("remoteVideo");
const touchIndicator = document.getElementById("touchIndicator");

let displayWidth = 1920;
let displayHeight = 1080;

function setStatus(text) {
    statusEl.textContent = text;
}

// Protocol payloads mirror the Rust `IncomingInput` enum in
// crates/orbiscreen-transport: tagged Pointer/Key/Stylus variants.
function sendInput(payload) {
    fetch("/input", {
        method: "POST",
        headers: { "content-type": "application/json" },
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

for (let i = 0; i < 12; i += 1) KEYCODE_MAP[`F${i + 1}`] = 59 + i;
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

async function start() {
    setStatus("Connecting to stream…");
    try {
        const response = await fetch("/api/info");
        if (response.ok) {
            const info = await response.json();
            if (Number.isFinite(info?.display_width)
                && Number.isFinite(info?.display_height)) {
                displayWidth = info.display_width;
                displayHeight = info.display_height;
                resolutionEl.textContent =
                    `${displayWidth} × ${displayHeight}`;
            }
        }
    } catch (error) {
        // Keep the default resolution if host info is unavailable.
        console.warn("api/info fetch failed:", error);
    }

    videoEl.src = "/stream";
    videoEl.play().catch((error) => {
        setStatus(`Playback error: ${error.message}`);
        console.error(error);
    });
    overlayEl.classList.add("hidden");
    setStatus("Streaming");
}

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

videoEl.addEventListener("playing", () => {
    streamActive = true;
});
videoEl.addEventListener("error", () => {
    streamActive = false;
    setStatus("Stream error — check that the daemon is running");
});

start().catch((error) => {
    setStatus(`Error: ${error.message}`);
    console.error(error);
});
