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

        if (isRunning) {
            if (status.active_clients > 0) {
                statusPill.className = "statusPill online";
                statusLabel.textContent = "Streaming";
                if (displayStateHeadline) displayStateHeadline.textContent = "Extended Screen Streaming";
                if (displayStateSubtitle) displayStateSubtitle.textContent = `Streaming to ${status.active_clients} connected device with ultra-low latency hardware encoding.`;
                if (activeClientPill) activeClientPill.classList.remove("hidden");
                if (activeClientText) activeClientText.textContent = `${status.active_clients} Device Connected (Android Tablet)`;
            } else {
                statusPill.className = "statusPill ready";
                statusLabel.textContent = "Ready";
                if (displayStateHeadline) displayStateHeadline.textContent = "Ready to Stream";
                if (displayStateSubtitle) displayStateSubtitle.textContent = "Open Orbiscreen on your Android tablet or phone to connect as second display.";
                if (activeClientPill) activeClientPill.classList.add("hidden");
            }
            if (btnToggleService) {
                btnToggleService.className = "btnMasterAction danger";
                if (toggleText) toggleText.textContent = "Stop Display";
                if (toggleIcon) toggleIcon.innerHTML = `<path d="M6 6h12v12H6z"/>`;
            }
        } else {
            statusPill.className = "statusPill offline";
            statusLabel.textContent = "Stopped";
            if (displayStateHeadline) displayStateHeadline.textContent = "Virtual Display Offline";
            if (displayStateSubtitle) displayStateSubtitle.textContent = "Click Start Display above to activate your extended desktop workspace.";
            if (activeClientPill) activeClientPill.classList.add("hidden");
            if (btnToggleService) {
                btnToggleService.className = "btnMasterAction";
                if (toggleText) toggleText.textContent = "Start Display";
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
            } else {
                usbStatusTag.className = "usbTag";
                usbStatusTag.textContent = "Waiting for device...";
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
            } else {
                await invoke("start_service");
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
        try {
            await invoke("restart_service");
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
        } catch (e) {
            inpSessionUrl.select();
            document.execCommand("copy");
            btnCopyUrl.textContent = "Copied!";
            setTimeout(() => { btnCopyUrl.textContent = "Copy"; }, 1500);
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
    });
}

if (btnFixDoctor) {
    btnFixDoctor.addEventListener("click", async () => {
        btnFixDoctor.disabled = true;
        btnFixDoctor.textContent = "Fixing...";
        try {
            await invoke("run_doctor_fix");
        } catch (e) {
            console.warn("Doctor fix error:", e);
        }
        setTimeout(() => {
            btnFixDoctor.disabled = false;
            btnFixDoctor.textContent = "Auto-Fix";
            if (btnRunDoctor) btnRunDoctor.click();
        }, 1200);
    });
}

refreshStatus();
setInterval(refreshStatus, 2500);

if (window.location.hash) {
    const hashTab = window.location.hash.replace("#", "");
    const targetBtn = document.querySelector(`.segmentBtn[data-tab="${hashTab}"]`);
    if (targetBtn) targetBtn.click();
}

if (window.location.hash === "#usb") {
    const usbBtn = document.querySelector(`.modeBtn[data-sub="usb"], .subTabBtn[data-sub="usb"]`);
    if (usbBtn) usbBtn.click();
}
