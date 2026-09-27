// Orbiscreen - annexb.test.js (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

const test = require("node:test");
const assert = require("node:assert/strict");
const annexb = require("./annexb.js");

test("spsDimensions reads High@4.0 1280x800", () => {
    const sps = Buffer.from(
        "0000000127640028ac11128050065bff0001000110000003001000000789da08042e",
        "hex",
    );
    assert.deepEqual(annexb.spsDimensions(sps), { width: 1280, height: 800 });
});

test("codec string from High@4.0 SPS", () => {
    const sps = annexb.withStartCode(Uint8Array.of(0x67, 0x64, 0x00, 0x28, 0xac));
    assert.equal(annexb.codecStringFromSps(sps), "avc1.640028");
});

test("extracts SPS/PPS from an IDR AU", () => {
    const sps = annexb.withStartCode(Uint8Array.of(0x67, 0x64, 0x00, 0x28, 0xac));
    const pps = annexb.withStartCode(Uint8Array.of(0x68, 0xee, 0x3c, 0x80));
    const slice = annexb.withStartCode(Uint8Array.of(0x65, 0x88));
    const au = new Uint8Array(sps.length + pps.length + slice.length);
    au.set(sps, 0);
    au.set(pps, sps.length);
    au.set(slice, sps.length + pps.length);
    const found = annexb.extractSpsPps(au);
    assert.deepEqual(Array.from(found.sps), Array.from(sps));
    assert.deepEqual(Array.from(found.pps), Array.from(pps));
});

test("hello frame encodes a complete length-prefixed body", () => {
    const frame = annexb.encodeHello("tok", "sess-1");
    const split = annexb.splitFrame(frame);
    assert.ok(split);
    assert.equal(split.used, frame.length);
    assert.equal(split.body[0], annexb.TYPE_HELLO);
});

test("frame reader reassembles split chunks", () => {
    const frame = annexb.encodeCtrl(annexb.TYPE_IDR);
    const reader = new annexb.FrameReader();
    reader.push(frame.subarray(0, 3));
    assert.equal(reader.pop(), null);
    reader.push(frame.subarray(3));
    const msg = reader.pop();
    assert.equal(msg.type, "idr");
    assert.equal(reader.pop(), null);
});

test("video message decode", () => {
    const au = Uint8Array.of(0, 0, 0, 1, 0x65, 9);
    const body = new Uint8Array(1 + 1 + 8 + 8 + au.length);
    body[0] = annexb.TYPE_VIDEO;
    body[1] = 1;
    body[2] = 42;
    body.set(au, 18);
    const frame = annexb.encodeFrame(body);
    const msg = annexb.decodeMessage(annexb.splitFrame(frame).body);
    assert.equal(msg.type, "video");
    assert.equal(msg.key, true);
    assert.equal(msg.ptsNs, 42n);
    assert.deepEqual(Array.from(msg.au), Array.from(au));
});

test("classifies IDR as I and P-slice as P", () => {
    const idr = annexb.withStartCode(Uint8Array.of(0x65, 0x88));
    assert.equal(annexb.classifyAccessUnit(idr), "I");
});

test("pickWtHost prefers the page host when it is not loopback", () => {
    assert.equal(annexb.pickWtHost({ wt_hosts: ["10.0.0.5"] }, "192.168.1.8"), "192.168.1.8");
    assert.equal(annexb.pickWtHost({ wt_hosts: ["10.0.0.5", "127.0.0.1"] }, "127.0.0.1"), "10.0.0.5");
    assert.equal(annexb.pickWtHost({ wt_hosts: ["127.0.0.1"] }, "localhost"), "127.0.0.1");
});

test("datagram assembler rebuilds a split AU and reports a gap after the hole wait", () => {
    const au = Uint8Array.from({ length: 40 }, (_, i) => i);
    const a = annexb.encodeVideoDatagram(4, 0, 2, true, 1n, 2n, au.subarray(0, 20));
    const b = annexb.encodeVideoDatagram(4, 1, 2, true, 1n, 2n, au.subarray(20));
    const asm = new annexb.DatagramAssembler();
    assert.deepEqual(asm.push(a, 0), []);
    const msg = asm.push(b, 0);
    assert.equal(msg.length, 1);
    assert.equal(msg[0].type, "video");
    assert.equal(msg[0].key, true);
    assert.deepEqual(Array.from(msg[0].au), Array.from(au));

    const next = annexb.encodeVideoDatagram(6, 0, 1, false, 3n, 4n, au.subarray(0, 8));
    assert.deepEqual(asm.push(next, 0), []);
    assert.deepEqual(asm.expire(annexb.HOLE_WAIT_MS - 1), []);
    const gap = asm.expire(annexb.HOLE_WAIT_MS);
    assert.equal(gap.length, 1);
    assert.equal(gap[0].type, "gap");
    assert.equal(gap[0].dropped, 1);
    assert.deepEqual(asm.expire(annexb.HOLE_WAIT_MS * 2), []);
});

test("datagram assembler ignores a late AU and does not rewind lastSeq", () => {
    const payload = Uint8Array.of(1, 2, 3, 4);
    const asm = new annexb.DatagramAssembler();
    const first = asm.push(annexb.encodeVideoDatagram(10, 0, 1, true, 1n, 2n, payload), 0);
    assert.equal(first[0].type, "video");
    assert.equal(first[0].key, true);

    const late = annexb.encodeVideoDatagram(8, 0, 1, false, 3n, 4n, payload);
    assert.deepEqual(asm.push(late, 0), []);
    assert.deepEqual(asm.push(annexb.encodeVideoDatagram(10, 0, 1, true, 1n, 2n, payload), 0), []);

    const inOrder = asm.push(annexb.encodeVideoDatagram(11, 0, 1, false, 5n, 6n, payload), 0);
    assert.equal(inOrder[0].type, "video");
    assert.equal(inOrder[0].key, false);
});

test("datagram assembler treats wrap from 65535 to 0 as in-order", () => {
    const payload = Uint8Array.of(9);
    const asm = new annexb.DatagramAssembler();
    assert.equal(asm.push(annexb.encodeVideoDatagram(65535, 0, 1, true, 1n, 2n, payload), 0)[0].type, "video");
    const wrapped = asm.push(annexb.encodeVideoDatagram(0, 0, 1, false, 3n, 4n, payload), 0);
    assert.equal(wrapped[0].type, "video");
    assert.equal(wrapped[0].key, false);
});

test("datagram assembler plays 11 then 12 when 12 arrives first", () => {
    const payload = Uint8Array.of(1, 2, 3, 4);
    const asm = new annexb.DatagramAssembler();
    assert.equal(asm.push(annexb.encodeVideoDatagram(10, 0, 1, true, 1n, 2n, payload), 0)[0].seq, 10);
    assert.deepEqual(asm.push(annexb.encodeVideoDatagram(12, 0, 1, false, 3n, 4n, payload), 0), []);
    const filled = asm.push(annexb.encodeVideoDatagram(11, 0, 1, false, 5n, 6n, payload), 10);
    assert.deepEqual(filled.map((m) => m.seq), [11, 12]);
    assert.equal(filled.every((m) => m.type === "video"), true);
    assert.deepEqual(asm.expire(10 + annexb.HOLE_WAIT_MS), []);
});

test("datagram assembler emits gap if the hole never fills", () => {
    const payload = Uint8Array.of(7);
    const asm = new annexb.DatagramAssembler();
    asm.push(annexb.encodeVideoDatagram(10, 0, 1, true, 1n, 2n, payload), 0);
    assert.deepEqual(asm.push(annexb.encodeVideoDatagram(12, 0, 1, false, 3n, 4n, payload), 0), []);
    const gap = asm.expire(annexb.HOLE_WAIT_MS);
    assert.equal(gap.length, 1);
    assert.equal(gap[0].type, "gap");
    const late = asm.push(annexb.encodeVideoDatagram(11, 0, 1, false, 5n, 6n, payload), annexb.HOLE_WAIT_MS + 1);
    assert.equal(late[0].type, "video");
    assert.equal(late[0].seq, 11);
});

test("after a reliable IDR, the next P-frame plays even if lastSeq had a hole", () => {
    const payload = Uint8Array.of(1);
    const asm = new annexb.DatagramAssembler();
    asm.push(annexb.encodeVideoDatagram(10, 0, 1, true, 1n, 2n, payload), 0);
    assert.deepEqual(asm.push(annexb.encodeVideoDatagram(12, 0, 1, false, 3n, 4n, payload), 0), []);
    const gap = asm.expire(annexb.HOLE_WAIT_MS);
    assert.equal(gap[0].type, "gap");

    asm.onReliableKeyframe();
    const next = asm.push(annexb.encodeVideoDatagram(40, 0, 1, false, 5n, 6n, payload), annexb.HOLE_WAIT_MS + 10);
    assert.equal(next.length, 1);
    assert.equal(next[0].type, "video");
    assert.equal(next[0].seq, 40);
    assert.equal(next[0].key, false);
    assert.deepEqual(asm.expire(annexb.HOLE_WAIT_MS * 2), []);
});

test("hashFromBase64 yields 32 bytes", () => {
    const raw = Uint8Array.from({ length: 32 }, (_, i) => i);
    const b64 = Buffer.from(raw).toString("base64");
    assert.deepEqual(Array.from(annexb.hashFromBase64(b64)), Array.from(raw));
});

test("252 data shards recover two erasures", () => {
    const k = 252;
    const src = Array.from({ length: k }, (_, i) => Uint8Array.from([i & 255, 1, 2, 3]));
    const parity = annexb.encodeFec(src, 4);
    const data = src.map((row) => row);
    data[1] = null;
    data[250] = null;
    assert.equal(annexb.recoverFec(data, parity), true);
    assert.deepEqual(Array.from(data[1]), Array.from(src[1]));
    assert.deepEqual(Array.from(data[250]), Array.from(src[250]));
});

test("an access unit past 252 shards splits and survives one loss", () => {
    const au = new Uint8Array(2013).fill(7);
    const blocks = annexb.shardAuBlocks(au, 8);
    assert.ok(blocks.length > 1);
    assert.ok(blocks.every((block) => block.data.length <= 252 && block.count === blocks.length));
    const asm = new annexb.DatagramAssembler();
    let out = [];
    for (const block of blocks) {
        const dropFirst = block.index === 0 && block.parity.length > 0;
        for (let i = 0; i < block.data.length; i += 1) {
            if (dropFirst && i === 0) continue;
            out = out.concat(asm.push(annexb.encodeBlockedDatagram(
                9, i, block.data.length, true, 1n, 2n, block.data[i], block.index, block.count,
            ), 0));
        }
        if (dropFirst) {
            out = out.concat(asm.push(annexb.encodeBlockedDatagram(
                9, block.data.length, block.data.length, true, 1n, 2n, block.parity[0], block.index, block.count,
            ), 0));
        }
    }
    const video = out.filter((item) => item.type === "video");
    assert.equal(video.length, 1);
    assert.deepEqual(Array.from(video[0].au), Array.from(au));
});

test("parity ladder matches the host", () => {
    assert.equal(annexb.parityCount(1), 0);
    assert.equal(annexb.parityCount(3), 0);
    assert.equal(annexb.parityCount(4), 2);
    assert.equal(annexb.parityCount(16), 2);
    assert.equal(annexb.parityCount(17), 3);
    assert.equal(annexb.parityCount(64), 3);
    assert.equal(annexb.parityCount(65), 4);
});

test("JS Reed-Solomon recovers two missing data shards", () => {
    const k = 4;
    const width = 16;
    const data = Array.from({ length: k }, (_, i) =>
        Uint8Array.from({ length: width }, (_, b) => (i * 31 + b) & 0xff),
    );
    const parity = annexb.encodeFec(data, 2);
    const slots = data.map((d) => new Uint8Array(d));
    slots[1] = null;
    slots[3] = null;
    assert.equal(annexb.recoverFec(slots, parity), true);
    for (let i = 0; i < k; i += 1) {
        assert.deepEqual(Array.from(slots[i]), Array.from(data[i]));
    }
});

test("assembler rebuilds an AU after two data datagrams are lost", () => {
    const k = 4;
    const width = 8;
    const auLen = k * width - 4;
    const prefix = new Uint8Array(4);
    prefix[0] = auLen & 0xff;
    prefix[1] = (auLen >> 8) & 0xff;
    const body = Uint8Array.from({ length: auLen }, (_, i) => i & 0xff);
    const blob = new Uint8Array(4 + auLen);
    blob.set(prefix, 0);
    blob.set(body, 4);
    const parts = [];
    for (let i = 0; i < k; i += 1) parts.push(blob.subarray(i * width, (i + 1) * width));
    const parity = annexb.encodeFec(parts, 2);
    const asm = new annexb.DatagramAssembler();
    const pkts = [];
    for (let i = 0; i < k; i += 1) {
        if (i === 0 || i === 2) continue;
        pkts.push(annexb.encodeVideoDatagram(3, i, k, false, 1n, 2n, parts[i]));
    }
    pkts.push(annexb.encodeVideoDatagram(3, k, k, false, 1n, 2n, parity[0]));
    pkts.push(annexb.encodeVideoDatagram(3, k + 1, k, false, 1n, 2n, parity[1]));
    let out = [];
    for (const p of pkts) out = out.concat(asm.push(p, 0));
    assert.equal(out.length, 1);
    assert.equal(out[0].type, "video");
    assert.deepEqual(Array.from(out[0].au), Array.from(body));
});

test("parity first does not open a pending AU; all data still emits", () => {
    const k = 4;
    const width = 8;
    const auLen = k * width - 4;
    const prefix = new Uint8Array(4);
    prefix[0] = auLen & 0xff;
    const body = Uint8Array.from({ length: auLen }, (_, i) => (i + 3) & 0xff);
    const blob = new Uint8Array(4 + auLen);
    blob.set(prefix, 0);
    blob.set(body, 4);
    const parts = [];
    for (let i = 0; i < k; i += 1) parts.push(blob.subarray(i * width, (i + 1) * width));
    const parity = annexb.encodeFec(parts, 2);
    const asm = new annexb.DatagramAssembler();
    assert.deepEqual(asm.push(annexb.encodeVideoDatagram(4, k, k, false, 1n, 2n, parity[0]), 0), []);
    let out = [];
    for (let i = 0; i < k; i += 1) {
        out = out.concat(asm.push(annexb.encodeVideoDatagram(4, i, k, false, 1n, 2n, parts[i]), 0));
    }
    assert.equal(out.length, 1);
    assert.deepEqual(Array.from(out[0].au), Array.from(body));
    assert.deepEqual(asm.push(annexb.encodeVideoDatagram(4, k + 1, k, false, 1n, 2n, parity[1]), 1), []);
});
