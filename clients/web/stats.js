// Orbiscreen - stats.js (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

(function (root, factory) {
    if (typeof module === "object" && module.exports) {
        module.exports = factory();
    } else {
        root.OrbiStats = factory();
    }
}(typeof self !== "undefined" ? self : this, function () {
    const KIND_I = "I";
    const KIND_P = "P";
    const KIND_B = "B";
    const KIND_OTHER = "other";

    class StreamStats {
        constructor({ windowMs = 60000, bucketMs = 1000 } = {}) {
            this.windowMs = windowMs;
            this.bucketMs = bucketMs;
            this.n = Math.max(1, Math.floor(windowMs / bucketMs));
            this.reset();
        }

        reset() {
            this.bytes = new Array(this.n).fill(0);
            this.framesI = new Array(this.n).fill(0);
            this.framesP = new Array(this.n).fill(0);
            this.framesB = new Array(this.n).fill(0);
            this.framesOther = new Array(this.n).fill(0);
            this.framesDropped = new Array(this.n).fill(0);
            this.presented = new Array(this.n).fill(0);
            this.head = 0;
            this.cursorBucket = null;
            this.delayMs = null;
            this.presentedSentNs = null;
            this.presentedAtMs = 0;
            this.clockOffsetNs = 0n;
        }

        noteBytes(count, nowMs = Date.now()) {
            if (!(count > 0)) return;
            this._rotate(nowMs);
            this.bytes[this.head] += count;
        }

        noteFrame(kind, nowMs = Date.now()) {
            this._rotate(nowMs);
            if (kind === KIND_I) this.framesI[this.head] += 1;
            else if (kind === KIND_P) this.framesP[this.head] += 1;
            else if (kind === KIND_B) this.framesB[this.head] += 1;
            else this.framesOther[this.head] += 1;
        }

        noteDropped(count = 1, nowMs = Date.now()) {
            if (!(count > 0)) return;
            this._rotate(nowMs);
            this.framesDropped[this.head] += count;
        }

        noteDelay(ms) {
            if (ms >= 0 && ms <= 30000) this.delayMs = ms;
        }

        notePresented(sentNs, nowMs = Date.now()) {
            this.presentedSentNs = sentNs;
            this.presentedAtMs = nowMs;
            this._rotate(nowMs);
            this.presented[this.head] += 1;
        }

        noteClockOffset(offsetNs) {
            this.clockOffsetNs = offsetNs;
        }

        snapshot(nowMs = Date.now()) {
            this._rotate(nowMs);
            const bytesSeries = new Array(this.n);
            const frameI = new Array(this.n);
            const frameP = new Array(this.n);
            const frameB = new Array(this.n);
            const frameOther = new Array(this.n);
            const frameDropped = new Array(this.n);
            let lastMinuteI = 0;
            let lastMinuteP = 0;
            let lastMinuteB = 0;
            let lastMinuteOther = 0;
            let lastMinuteDropped = 0;
            for (let k = 0; k < this.n; k += 1) {
                const idx = (this.head + 1 + k) % this.n;
                bytesSeries[k] = this.bytes[idx];
                frameI[k] = this.framesI[idx];
                frameP[k] = this.framesP[idx];
                frameB[k] = this.framesB[idx];
                frameOther[k] = this.framesOther[idx];
                frameDropped[k] = this.framesDropped[idx];
                lastMinuteI += frameI[k];
                lastMinuteP += frameP[k];
                lastMinuteB += frameB[k];
                lastMinuteOther += frameOther[k];
                lastMinuteDropped += frameDropped[k];
            }
            const intoBucket = nowMs - this.cursorBucket * this.bucketMs;
            const elapsed = Math.min(this.bucketMs, Math.max(1, intoBucket));
            const prevIx = (this.head - 1 + this.n) % this.n;
            const bytesPerSec = intoBucket < 200
                ? this.bytes[prevIx]
                : Math.round(this.bytes[this.head] * this.bucketMs / elapsed);
            const prevFps = this.presented[prevIx];
            const curFps = this.presented[this.head];
            const fps = intoBucket < 200
                ? prevFps
                : Math.round(curFps * this.bucketMs / elapsed);
            return {
                delayMs: this.delayMs,
                ageMs: this._age(nowMs),
                bytesPerSec,
                bytesSeries,
                frameI,
                frameP,
                frameB,
                frameOther,
                frameDropped,
                fps,
                lastMinuteI,
                lastMinuteP,
                lastMinuteB,
                lastMinuteOther,
                lastMinuteDropped,
            };
        }

        _age(nowMs) {
            if (this.presentedSentNs != null) {
                const nowNs = BigInt(nowMs) * 1000000n;
                const age = Number((nowNs + this.clockOffsetNs - this.presentedSentNs) / 1000000n);
                if (age >= 0 && age <= 30000) return age;
            }
            if (this.presentedAtMs > 0 && this.delayMs != null) {
                return this.delayMs + Math.max(0, nowMs - this.presentedAtMs);
            }
            return null;
        }

        _rotate(nowMs) {
            const bucket = Math.floor(nowMs / this.bucketMs);
            if (this.cursorBucket == null) {
                this.cursorBucket = bucket;
                return;
            }
            const delta = bucket - this.cursorBucket;
            if (delta <= 0) return;
            if (delta >= this.n) {
                this.bytes.fill(0);
                this.framesI.fill(0);
                this.framesP.fill(0);
                this.framesB.fill(0);
                this.framesOther.fill(0);
                this.framesDropped.fill(0);
                this.presented.fill(0);
            } else {
                for (let step = 1; step <= delta; step += 1) {
                    const idx = (this.head + step) % this.n;
                    this.bytes[idx] = 0;
                    this.framesI[idx] = 0;
                    this.framesP[idx] = 0;
                    this.framesB[idx] = 0;
                    this.framesOther[idx] = 0;
                    this.framesDropped[idx] = 0;
                    this.presented[idx] = 0;
                }
            }
            this.head = (this.head + delta) % this.n;
            this.cursorBucket = bucket;
        }
    }

    function formatRate(bytesPerSec) {
        const n = Math.max(0, bytesPerSec || 0);
        if (n >= 1e6) return `${(n / 1e6).toFixed(1)} MB/s`;
        if (n >= 1e3) return `${Math.round(n / 1e3)} KB/s`;
        return `${n} B/s`;
    }

    function formatMs(ms) {
        return (ms == null || ms < 0) ? "—" : `${ms} ms`;
    }

    function sizeCanvas(canvas) {
        const dpr = window.devicePixelRatio || 1;
        const w = canvas.clientWidth || canvas.width;
        const h = canvas.clientHeight || canvas.height;
        const pw = Math.max(1, Math.round(w * dpr));
        const ph = Math.max(1, Math.round(h * dpr));
        if (canvas.width !== pw || canvas.height !== ph) {
            canvas.width = pw;
            canvas.height = ph;
        }
        const ctx = canvas.getContext("2d");
        ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
        return { ctx, w, h };
    }

    function drawBytesGraph(canvas, series) {
        const { ctx, w, h } = sizeCanvas(canvas);
        ctx.clearRect(0, 0, w, h);
        if (!series || !series.length) return;
        let max = 1;
        for (let i = 0; i < series.length; i += 1) {
            if (series[i] > max) max = series[i];
        }
        ctx.beginPath();
        ctx.moveTo(0, h);
        for (let i = 0; i < series.length; i += 1) {
            const x = series.length === 1 ? 0 : (i / (series.length - 1)) * w;
            const y = h - (series[i] / max) * (h - 2);
            ctx.lineTo(x, y);
        }
        ctx.lineTo(w, h);
        ctx.closePath();
        ctx.fillStyle = "rgba(116, 199, 236, 0.32)";
        ctx.fill();
        ctx.strokeStyle = "#74c7ec";
        ctx.lineWidth = 1.25;
        ctx.stroke();
    }

    function drawFramesGraph(canvas, snap) {
        const { ctx, w, h } = sizeCanvas(canvas);
        ctx.clearRect(0, 0, w, h);
        const n = snap.frameI.length;
        if (!n) return;
        let max = 1;
        for (let i = 0; i < n; i += 1) {
            const t = snap.frameI[i] + snap.frameP[i] + snap.frameOther[i] + (snap.frameDropped[i] || 0);
            if (t > max) max = t;
        }
        const barW = w / n;
        for (let i = 0; i < n; i += 1) {
            let y = h;
            const stack = [
                [snap.frameI[i], "#a6e3a1"],
                [snap.frameP[i], "#89b4fa"],
                [snap.frameOther[i], "#6c7086"],
                [snap.frameDropped[i] || 0, "#f38ba8"],
            ];
            for (let s = 0; s < stack.length; s += 1) {
                const v = stack[s][0];
                if (!v) continue;
                const bh = (v / max) * h;
                y -= bh;
                ctx.fillStyle = stack[s][1];
                ctx.fillRect(i * barW, y, Math.max(1, barW - 0.4), bh);
            }
        }
    }

    const STATS_LAYOUT = {
        widthPx: 236,
        padHPx: 12,
        padVPx: 10,
        rowGapPx: 2,
        titleBottomPx: 6,
        graphHeightPx: 40,
        graphMarginTopPx: 4,
        graphMarginBottomPx: 8,
        rowLineHeightPx: 14,
        footerHeightPx: 14,
    };
    STATS_LAYOUT.contentHeightPx =
        (STATS_LAYOUT.rowLineHeightPx + STATS_LAYOUT.titleBottomPx) +
        4 * (STATS_LAYOUT.rowLineHeightPx + STATS_LAYOUT.rowGapPx) +
        2 * (STATS_LAYOUT.graphMarginTopPx + STATS_LAYOUT.graphHeightPx +
            STATS_LAYOUT.graphMarginBottomPx) +
        STATS_LAYOUT.footerHeightPx;
    STATS_LAYOUT.heightPx = STATS_LAYOUT.padVPx * 2 + STATS_LAYOUT.contentHeightPx;

    const TOOLBAR_DELAY_DIGITS = 4;

    function formatToolbarDelay(ageMs) {
        const body = (ageMs != null && ageMs >= 0)
            ? String(Math.min(ageMs, 9999))
            : "—";
        return `delay ${body.padStart(TOOLBAR_DELAY_DIGITS)}ms`;
    }

    return {
        KIND_I, KIND_P, KIND_B, KIND_OTHER,
        StreamStats, formatRate, formatMs, formatToolbarDelay,
        drawBytesGraph, drawFramesGraph, STATS_LAYOUT,
    };
}));
