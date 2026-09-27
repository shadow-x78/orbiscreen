// Orbiscreen - stats.test.js (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

const test = require("node:test");
const assert = require("node:assert/strict");
const annexb = require("./annexb.js");
const stats = require("./stats.js");

function encodeUe(codeNum) {
    const x = codeNum + 1;
    const zeros = 31 - Math.clz32(x);
    const suffix = x.toString(2).slice(1);
    return "0".repeat(zeros) + "1" + suffix;
}

function bitsToBytes(bits) {
    const padded = bits.padEnd(Math.ceil(bits.length / 8) * 8, "0");
    const out = new Uint8Array(padded.length / 8);
    for (let i = 0; i < out.length; i += 1) {
        out[i] = parseInt(padded.slice(i * 8, i * 8 + 8), 2);
    }
    return out;
}

function sliceAu(nalType, sliceType) {
    const bits = encodeUe(0) + encodeUe(sliceType);
    const payload = bitsToBytes(bits);
    const header = 0x60 | (nalType & 0x1f);
    const nal = new Uint8Array(1 + payload.length);
    nal[0] = header;
    nal.set(payload, 1);
    return annexb.withStartCode(nal);
}

test("classifies IDR as I", () => {
    const au = annexb.withStartCode(Uint8Array.of(0x65, 0x88));
    assert.equal(annexb.classifyAccessUnit(au), "I");
});

test("classifies P/B/I slices", () => {
    assert.equal(annexb.classifyAccessUnit(sliceAu(1, 0)), "P");
    assert.equal(annexb.classifyAccessUnit(sliceAu(1, 1)), "B");
    assert.equal(annexb.classifyAccessUnit(sliceAu(1, 2)), "I");
    assert.equal(annexb.classifyAccessUnit(sliceAu(1, 7)), "I");
});

test("SPS/PPS only is other; IDR AU is I", () => {
    const sps = annexb.withStartCode(Uint8Array.of(0x67, 0x64, 0x00, 0x28));
    const pps = annexb.withStartCode(Uint8Array.of(0x68, 0xee, 0x3c));
    const idr = annexb.withStartCode(Uint8Array.of(0x65, 0x88));
    const header = new Uint8Array(sps.length + pps.length);
    header.set(sps, 0);
    header.set(pps, sps.length);
    assert.equal(annexb.classifyAccessUnit(header), "other");
    const au = new Uint8Array(header.length + idr.length);
    au.set(header, 0);
    au.set(idr, header.length);
    assert.equal(annexb.classifyAccessUnit(au), "I");
});

test("bytes land in the current bucket", () => {
    const s = new stats.StreamStats({ windowMs: 60000, bucketMs: 1000 });
    s.noteBytes(400, 10);
    s.noteBytes(600, 20);
    const snap = s.snapshot(50);
    assert.equal(snap.bytesSeries[snap.bytesSeries.length - 1], 1000);
});

test("buckets rotate across the last minute window", () => {
    const s = new stats.StreamStats({ windowMs: 4000, bucketMs: 1000 });
    s.noteBytes(100, 0);
    s.noteBytes(200, 1000);
    s.noteBytes(300, 2000);
    const snap = s.snapshot(2100);
    assert.equal(snap.bytesSeries.length, 4);
    assert.equal(snap.bytesSeries[1], 100);
    assert.equal(snap.bytesSeries[2], 200);
    assert.equal(snap.bytesSeries[3], 300);
});

test("stacked frames count by kind", () => {
    const s = new stats.StreamStats({ windowMs: 4000, bucketMs: 1000 });
    s.noteFrame("I", 0);
    s.noteFrame("P", 10);
    s.noteFrame("P", 20);
    s.noteFrame("B", 30);
    const snap = s.snapshot(40);
    assert.equal(snap.frameI[snap.frameI.length - 1], 1);
    assert.equal(snap.frameP[snap.frameP.length - 1], 2);
    assert.equal(snap.frameB[snap.frameB.length - 1], 1);
    assert.equal(snap.lastMinuteI, 1);
    assert.equal(snap.lastMinuteP, 2);
    assert.equal(snap.lastMinuteB, 1);
});

test("fps counts presented frames, not received", () => {
    const s = new stats.StreamStats({ windowMs: 4000, bucketMs: 1000 });
    s.noteFrame("I", 0);
    s.noteFrame("P", 10);
    s.noteFrame("P", 20);
    s.notePresented(1n, 30);
    s.notePresented(2n, 40);
    const snap = s.snapshot(1100);
    assert.equal(snap.fps, 2);
    assert.equal(snap.lastMinuteI, 1);
    assert.equal(snap.lastMinuteP, 2);
});

test("dropped frames stack in the red series", () => {
    const s = new stats.StreamStats({ windowMs: 4000, bucketMs: 1000 });
    s.noteFrame("P", 0);
    s.noteDropped(2, 10);
    s.noteDropped(1, 1000);
    const snap = s.snapshot(1100);
    assert.equal(snap.frameDropped[snap.frameDropped.length - 2], 2);
    assert.equal(snap.frameDropped[snap.frameDropped.length - 1], 1);
    assert.equal(snap.lastMinuteDropped, 3);
});

test("age uses clock-adjusted sent time", () => {
    const s = new stats.StreamStats();
    s.noteClockOffset(0n);
    s.notePresented(1_010_000_000n, 1010);
    assert.equal(s.snapshot(1025).ageMs, 15);
});

test("age falls back to delay plus on-screen time", () => {
    const s = new stats.StreamStats();
    s.noteDelay(8);
    s.notePresented(null, 1000);
    assert.equal(s.snapshot(1012).ageMs, 20);
});

test("toolbar delay keeps a fixed width for 1- and 2-digit ages", () => {
    const one = stats.formatToolbarDelay(2);
    const two = stats.formatToolbarDelay(10);
    const three = stats.formatToolbarDelay(123);
    const missing = stats.formatToolbarDelay(null);
    assert.equal(one.length, two.length);
    assert.equal(one.length, three.length);
    assert.equal(one.length, missing.length);
    assert.match(one, /2/);
    assert.match(two, /10/);
    assert.match(one, /^delay/);
    assert.match(one, /ms$/);
});

test("toolbar delay caps at four digits", () => {
    assert.equal(stats.formatToolbarDelay(2).length, stats.formatToolbarDelay(12345).length);
    assert.match(stats.formatToolbarDelay(12345), /9999/);
});

test("overlay layout matches the web CSS metrics", () => {
    assert.equal(stats.STATS_LAYOUT.widthPx, 236);
    assert.equal(stats.STATS_LAYOUT.padHPx, 12);
    assert.equal(stats.STATS_LAYOUT.padVPx, 10);
    assert.equal(stats.STATS_LAYOUT.rowGapPx, 2);
    assert.equal(stats.STATS_LAYOUT.titleBottomPx, 6);
    assert.equal(stats.STATS_LAYOUT.graphHeightPx, 40);
    assert.equal(stats.STATS_LAYOUT.graphMarginTopPx, 4);
    assert.equal(stats.STATS_LAYOUT.graphMarginBottomPx, 8);
    assert.equal(stats.STATS_LAYOUT.rowLineHeightPx, 14);
    assert.equal(stats.STATS_LAYOUT.footerHeightPx, 14);
});

test("overlay height fits title, rows, graphs, and footer", () => {
    const L = stats.STATS_LAYOUT;
    const title = L.rowLineHeightPx + L.titleBottomPx;
    const row = L.rowLineHeightPx + L.rowGapPx;
    const graph = L.graphMarginTopPx + L.graphHeightPx + L.graphMarginBottomPx;
    const content = title + 4 * row + 2 * graph + L.footerHeightPx;
    assert.equal(L.contentHeightPx, content);
    assert.equal(L.heightPx, L.padVPx * 2 + content);
    assert.equal(L.heightPx, 222);
});
