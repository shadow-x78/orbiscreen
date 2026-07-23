// Orbiscreen - web client (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen


const statusEl = document.getElementById("status");
const resolutionEl = document.getElementById("resolution");
const overlayEl = document.getElementById("overlay");
const videoEl = document.getElementById("remoteVideo");
const touchIndicator = document.getElementById("touchIndicator");

let peerConnection = null;
let dataChannel = null;
let displayWidth = 1920;
let displayHeight = 1080;

function setStatus(text) {
    statusEl.textContent = text;
}

function sendInput(event) {
    if (dataChannel && dataChannel.readyState === "open") {
        try {
            dataChannel.send(JSON.stringify(event));
            return;
        } catch (error) {
            console.warn("sendInput via dataChannel failed:", error);
        }
    }
    fetch("/input", {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify(event),
    }).catch((err) => console.warn("sendInput HTTP fallback failed:", err));
}

async function exchangeSdp(offer) {
    const response = await fetch("/sdp", {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify(offer),
    });
    if (!response.ok) throw new Error(`signaling failed: ${response.status}`);
    return response.json();
}

function mapPointer(event) {
    const rect = videoEl.getBoundingClientRect();
    const cx = (event.clientX - rect.left) / rect.width;
    const cy = (event.clientY - rect.top) / rect.height;
    return {
        x: Math.max(0, Math.min(1, cx)) * displayWidth,
        y: Math.max(0, Math.min(1, cy)) * displayHeight,
    };
}

function waitForIceComplete(pc) {
    if (pc.iceGatheringState === "complete") return Promise.resolve();
    return new Promise((resolve) => {
        const onChange = () => {
            if (pc.iceGatheringState === "complete") {
                pc.removeEventListener("icegatheringstatechange", onChange);
                resolve();
            }
        };
        pc.addEventListener("icegatheringstatechange", onChange);
        setTimeout(resolve, 1500);
    });
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
    setStatus("Opening WebRTC connection…");
    peerConnection = new RTCPeerConnection({
        iceServers: [{ urls: "stun:stun.l.google.com:19302" }],
    });

    peerConnection.addEventListener("track", (event) => {
        const [stream] = event.streams;
        videoEl.srcObject = stream;
        const settings = stream?.getVideoTracks?.()?.[0]?.getSettings?.();
        if (settings?.width && settings?.height) {
            displayWidth = settings.width;
            displayHeight = settings.height;
            resolutionEl.textContent = `${displayWidth} × ${displayHeight}`;
        }
        overlayEl.classList.add("hidden");
        setStatus("Streaming");
    });

    peerConnection.addEventListener("connectionstatechange", () => {
        setStatus(`Peer: ${peerConnection.connectionState}`);
        if (peerConnection.connectionState === "failed"
            || peerConnection.connectionState === "disconnected") {
            overlayEl.classList.remove("hidden");
        }
    });

    dataChannel = peerConnection.createDataChannel("input", { ordered: false });
    dataChannel.addEventListener("open", () => setStatus("Input channel open"));
    dataChannel.addEventListener("close", () => setStatus("Input channel closed"));

    const offer = await peerConnection.createOffer({
        offerToReceiveVideo: true,
        offerToReceiveAudio: false,
    });
    await peerConnection.setLocalDescription(offer);
    await waitForIceComplete(peerConnection);

    try {
        const answer = await exchangeSdp({
            type: "answer",
            sdp: peerConnection.localDescription.sdp,
        });
        if (answer?.sdp && answer?.type) {
            await peerConnection.setRemoteDescription(
                new RTCSessionDescription({ type: answer.type, sdp: answer.sdp }),
            );
        } else {
            setStatus("Connecting to direct stream...");
            videoEl.src = "/stream";
            videoEl.play().catch(() => {});
            overlayEl.classList.add("hidden");
        }
    } catch (error) {
        console.error(error);
        setStatus("Connecting to direct stream fallback...");
        videoEl.src = "/stream";
        videoEl.play().catch(() => {});
        overlayEl.classList.add("hidden");
    }
}

videoEl.addEventListener("pointerdown", (event) => {
    const { x, y } = mapPointer(event);
    const button = event.button + 1;
    sendInput({ kind: "button", x, y, button, pressed: true });
    if (event.pointerType === "pen") {
        sendInput({
            kind: "stylus",
            stylusKind: "pressure",
            x,
            y,
            pressure: event.pressure,
            tiltX: 0,
            tiltY: 0,
        });
    }
    showTouch(event.clientX, event.clientY);
});

videoEl.addEventListener("pointermove", (event) => {
    const { x, y } = mapPointer(event);
    if (event.pointerType === "pen" && event.buttons > 0) {
        sendInput({
            kind: "stylus",
            stylusKind: "tilt",
            x,
            y,
            pressure: event.pressure,
            tiltX: event.tiltX,
            tiltY: event.tiltY,
        });
    } else {
        sendInput({ kind: "move", x, y });
    }
});

videoEl.addEventListener("pointerup", (event) => {
    const { x, y } = mapPointer(event);
    const button = event.button + 1;
    sendInput({ kind: "button", x, y, button, pressed: false });
    hideTouch();
});

videoEl.addEventListener("wheel", (event) => {
    event.preventDefault();
    sendInput({ kind: "wheel", deltaY: event.deltaY });
}, { passive: false });

window.addEventListener("keydown", (event) => {
    sendInput({ kind: "key", code: event.keyCode || 0, pressed: true });
});

window.addEventListener("keyup", (event) => {
    sendInput({ kind: "key", code: event.keyCode || 0, pressed: false });
});

start().catch((error) => {
    setStatus(`Error: ${error.message}`);
    console.error(error);
});