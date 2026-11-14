(function (root, factory) {
    if (typeof module === "object" && module.exports) {
        module.exports = factory(require("./protocol.js"));
    } else {
        root.OrbiAnnexB = factory(root.OrbiProtocol);
    }
}(typeof self !== "undefined" ? self : this, function (protocol) {
    const {
        TYPE_VIDEO, TYPE_HELLO, TYPE_HELLO_ACK, TYPE_PING, TYPE_PONG, TYPE_IDR, TYPE_BYE,
        MAX_FRAME, VIDEO_PREFIX, DATAGRAM_HEADER, FEC_BLOCK_FLAG, PARITY_LADDER,
    } = protocol;
    const VIDEO_META_LEN = VIDEO_PREFIX - 1;

    function startCodeLen(data, i) {
        if (i + 3 < data.length && data[i] === 0 && data[i + 1] === 0 && data[i + 2] === 1) {
            return 3;
        }
        if (i + 4 < data.length && data[i] === 0 && data[i + 1] === 0
            && data[i + 2] === 0 && data[i + 3] === 1) {
            return 4;
        }
        return 0;
    }

    function withStartCode(nal) {
        const out = new Uint8Array(4 + nal.length);
        out[3] = 1;
        out.set(nal, 4);
        return out;
    }

    function unescapeRbsp(nal) {
        if (!nal || nal.length <= 1) return new Uint8Array(0);
        const out = [];
        let i = 1;
        while (i < nal.length) {
            if (i + 2 < nal.length && nal[i] === 0 && nal[i + 1] === 0 && nal[i + 2] === 3) {
                out.push(0, 0);
                i += 3;
            } else {
                out.push(nal[i]);
                i += 1;
            }
        }
        return Uint8Array.from(out);
    }

    function readUe(data) {
        let bitPos = 0;
        function bit() {
            const byteIx = (bitPos / 8) | 0;
            if (byteIx >= data.length) return -1;
            const v = (data[byteIx] >> (7 - (bitPos % 8))) & 1;
            bitPos += 1;
            return v;
        }
        function ue() {
            let zeros = 0;
            while (true) {
                const b = bit();
                if (b < 0) return -1;
                if (b === 1) break;
                zeros += 1;
                if (zeros > 31) return -1;
            }
            let suffix = 0;
            for (let i = 0; i < zeros; i += 1) {
                const b = bit();
                if (b < 0) return -1;
                suffix = (suffix << 1) | b;
            }
            return (1 << zeros) - 1 + suffix;
        }
        return { ue };
    }

    function sliceKind(nal) {
        if (!nal || !nal.length) return "other";
        const typ = nal[0] & 0x1f;
        if (typ === 5) return "I";
        if (typ !== 1 && typ !== 2) return "other";
        const rbsp = unescapeRbsp(nal);
        if (!rbsp.length) return "other";
        const bits = readUe(rbsp);
        if (bits.ue() < 0) return "other";
        const sliceType = bits.ue();
        if (sliceType < 0) return "other";
        switch (sliceType % 5) {
            case 0:
            case 3:
                return "P";
            case 1:
                return "B";
            case 2:
            case 4:
                return "I";
            default:
                return "other";
        }
    }

    function classifyAccessUnit(au) {
        let sawI = false;
        let sawB = false;
        let sawP = false;
        let i = 0;
        while (i < au.length) {
            const sc = startCodeLen(au, i);
            if (!sc) break;
            const nalStart = i + sc;
            let next = nalStart;
            while (next < au.length) {
                if (startCodeLen(au, next) && next > nalStart) break;
                next += 1;
            }
            if (nalStart < next) {
                const kind = sliceKind(au.subarray(nalStart, next));
                if (kind === "I") sawI = true;
                else if (kind === "B") sawB = true;
                else if (kind === "P") sawP = true;
            }
            i = next;
        }
        if (sawI) return "I";
        if (sawB) return "B";
        if (sawP) return "P";
        return "other";
    }

    function extractSpsPps(au) {
        let sps = null;
        let pps = null;
        let i = 0;
        while (i < au.length) {
            const sc = startCodeLen(au, i);
            if (!sc) break;
            const nalStart = i + sc;
            let next = nalStart;
            while (next < au.length) {
                if (startCodeLen(au, next) && next > nalStart) break;
                next += 1;
            }
            if (nalStart < next) {
                const nal = au.subarray(nalStart, next);
                const typ = nal[0] & 0x1f;
                if (typ === 7) sps = withStartCode(nal);
                if (typ === 8) pps = withStartCode(nal);
            }
            i = next;
        }
        return { sps, pps };
    }

    function codecStringFromSps(sps) {
        if (!sps) return null;
        const sc = startCodeLen(sps, 0);
        if (!sc || sps.length < sc + 4) return null;
        if ((sps[sc] & 0x1f) !== 7) return null;
        const hex = (n) => n.toString(16).toUpperCase().padStart(2, "0");
        return `avc1.${hex(sps[sc + 1])}${hex(sps[sc + 2])}${hex(sps[sc + 3])}`;
    }

    function concatBytes(parts) {
        let n = 0;
        for (const p of parts) n += p.length;
        const out = new Uint8Array(n);
        let o = 0;
        for (const p of parts) {
            out.set(p, o);
            o += p.length;
        }
        return out;
    }

    function u16le(n) {
        return new Uint8Array([n & 0xff, (n >> 8) & 0xff]);
    }

    function u64le(n) {
        const out = new Uint8Array(8);
        let x = BigInt(n);
        for (let i = 0; i < 8; i += 1) {
            out[i] = Number(x & 0xffn);
            x >>= 8n;
        }
        return out;
    }

    function readU16le(buf, o) {
        return buf[o] | (buf[o + 1] << 8);
    }

    function readU64le(buf, o) {
        let x = 0n;
        for (let i = 0; i < 8; i += 1) {
            x |= BigInt(buf[o + i]) << BigInt(8 * i);
        }
        return x;
    }

    function encodeFrame(body) {
        if (body.length > MAX_FRAME) throw new Error("frame too large");
        const out = new Uint8Array(4 + body.length);
        const len = body.length;
        out[0] = (len >>> 24) & 0xff;
        out[1] = (len >>> 16) & 0xff;
        out[2] = (len >>> 8) & 0xff;
        out[3] = len & 0xff;
        out.set(body, 4);
        return out;
    }

    function splitFrame(buf) {
        if (buf.length < 4) return null;
        const len = ((buf[0] << 24) | (buf[1] << 16) | (buf[2] << 8) | buf[3]) >>> 0;
        if (len > MAX_FRAME) throw new Error("frame too large");
        if (buf.length < 4 + len) return null;
        return { body: buf.subarray(4, 4 + len), used: 4 + len };
    }

    function encodeHello(token, session) {
        const t = new TextEncoder().encode(token || "");
        const s = new TextEncoder().encode(session || "");
        return encodeFrame(concatBytes([
            Uint8Array.of(TYPE_HELLO),
            u16le(t.length),
            t,
            u16le(s.length),
            s,
        ]));
    }

    function encodeCtrl(kind) {
        return encodeFrame(Uint8Array.of(kind));
    }

    function encodePing(t0Ns) {
        return encodeFrame(concatBytes([Uint8Array.of(TYPE_PING), u64le(t0Ns)]));
    }

    function decodeMessage(body) {
        if (!body.length) throw new Error("empty");
        const kind = body[0];
        const rest = body.subarray(1);
        if (kind === TYPE_HELLO_ACK) {
            if (rest.length < 4) throw new Error("truncated");
            return { type: "helloAck", width: readU16le(rest, 0), height: readU16le(rest, 2) };
        }
        if (kind === TYPE_VIDEO) {
            if (rest.length < VIDEO_META_LEN) throw new Error("truncated");
            return {
                type: "video",
                key: rest[0] !== 0,
                ptsNs: readU64le(rest, 1),
                sentNs: readU64le(rest, 9),
                au: rest.subarray(VIDEO_META_LEN),
            };
        }
        if (kind === TYPE_PONG) {
            if (rest.length < 16) throw new Error("truncated");
            return { type: "pong", t0Ns: readU64le(rest, 0), hostNs: readU64le(rest, 8) };
        }
        if (kind === TYPE_IDR) return { type: "idr" };
        if (kind === TYPE_BYE) return { type: "bye" };
        if (kind === TYPE_PING) {
            if (rest.length < 8) throw new Error("truncated");
            return { type: "ping", t0Ns: readU64le(rest, 0) };
        }
        if (kind === TYPE_HELLO) return { type: "hello" };
        throw new Error(`unknown type ${kind}`);
    }

    function hashFromBase64(b64) {
        const bin = atob(b64);
        const out = new Uint8Array(bin.length);
        for (let i = 0; i < bin.length; i += 1) out[i] = bin.charCodeAt(i);
        return out;
    }

    function pickWtHost(cfg, locationHost) {
        const h = locationHost || "";
        if (h && h !== "localhost" && h !== "127.0.0.1" && h !== "[::1]") {
            return h;
        }
        const hosts = cfg && Array.isArray(cfg.wt_hosts) ? cfg.wt_hosts : [];
        const lan = hosts.find((x) => x && x !== "127.0.0.1" && x !== "::1");
        return lan || hosts[0] || h || "127.0.0.1";
    }

    function parseVideoDatagram(buf) {
        if (!buf || buf.length < DATAGRAM_HEADER || buf[0] !== TYPE_VIDEO) return null;
        const flags = buf[1];
        let payload = buf.subarray(DATAGRAM_HEADER);
        let blockIndex = 0;
        let blockCount = 1;
        if ((flags & FEC_BLOCK_FLAG) !== 0) {
            if (payload.length < 2) return null;
            blockIndex = payload[0];
            blockCount = payload[1];
            if (blockCount === 0 || blockIndex >= blockCount) return null;
            payload = payload.subarray(2);
        }
        return {
            type: "video",
            key: (flags & 1) !== 0,
            seq: readU16le(buf, 2),
            frag: readU16le(buf, 4),
            frags: readU16le(buf, 6),
            ptsNs: readU64le(buf, 8),
            sentNs: readU64le(buf, 16),
            payload,
            blockIndex,
            blockCount,
        };
    }

    function seqDelta(cur, prev) {
        return (cur - prev) & 0xffff;
    }

    const HOLE_WAIT_MS = 48;
    const MAX_HELD = 4;
    const MAX_PARTIAL = 8;

    function parityCount(k) {
        if (k <= 3) return PARITY_LADDER[0];
        if (k <= 16) return PARITY_LADDER[1];
        if (k <= 64) return PARITY_LADDER[2];
        return PARITY_LADDER[3];
    }

    const GF = (() => {
        const exp = new Uint8Array(512);
        const log = new Uint8Array(256);
        let x = 1;
        for (let i = 0; i < 255; i += 1) {
            exp[i] = x;
            log[x] = i;
            x <<= 1;
            if (x & 0x100) x ^= 0x11d;
        }
        for (let i = 255; i < 512; i += 1) exp[i] = exp[i - 255];
        return { exp, log };
    })();

    function gfMul(a, b) {
        if (a === 0 || b === 0) return 0;
        return GF.exp[GF.log[a] + GF.log[b]];
    }
    function gfInv(a) {
        return GF.exp[255 - GF.log[a]];
    }
    function cauchy(p, d, m) {
        const denom = (p ^ ((m + d) & 255)) & 255;
        if (denom === 0) return 0;
        return gfInv(denom);
    }

    function invertMatrix(a) {
        const n = a.length;
        const m = a.map((row, i) => {
            const r = new Uint8Array(n * 2);
            r.set(row, 0);
            r[n + i] = 1;
            return r;
        });
        for (let col = 0; col < n; col += 1) {
            let piv = col;
            while (piv < n && m[piv][col] === 0) piv += 1;
            if (piv === n) return null;
            const tmp = m[col];
            m[col] = m[piv];
            m[piv] = tmp;
            const inv = gfInv(m[col][col]);
            for (let j = 0; j < n * 2; j += 1) m[col][j] = gfMul(m[col][j], inv);
            for (let row = 0; row < n; row += 1) {
                if (row === col) continue;
                const f = m[row][col];
                if (f === 0) continue;
                for (let j = 0; j < n * 2; j += 1) {
                    m[row][j] ^= gfMul(f, m[col][j]);
                }
            }
        }
        return m.map((row) => row.subarray(n));
    }

    function recoverFec(data, parity) {
        const k = data.length;
        if (k > 252) return false;
        if (data.every((d) => d)) return true;
        const m = parity.length;
        if (!m) return false;
        const missing = [];
        for (let i = 0; i < k; i += 1) if (!data[i]) missing.push(i);
        const presentP = [];
        let width = 0;
        for (let i = 0; i < k; i += 1) if (data[i] && data[i].length > width) width = data[i].length;
        for (let i = 0; i < m; i += 1) {
            if (parity[i]) {
                presentP.push(i);
                if (parity[i].length > width) width = parity[i].length;
            }
        }
        if (!width || missing.length > presentP.length) return false;
        const usedP = presentP.slice(0, missing.length);
        const missN = missing.length;
        const known = data.map((d) => {
            if (!d) return null;
            const p = new Uint8Array(width);
            p.set(d);
            return p;
        });
        const rhs = usedP.map((p) => {
            const rec = new Uint8Array(width);
            rec.set(parity[p]);
            const row = new Uint8Array(width);
            for (let b = 0; b < width; b += 1) {
                let s = rec[b];
                for (let d = 0; d < k; d += 1) {
                    if (known[d]) s ^= gfMul(cauchy(p, d, m), known[d][b]);
                }
                row[b] = s;
            }
            return row;
        });
        const mat = usedP.map((p) => {
            const row = new Uint8Array(missN);
            for (let c = 0; c < missN; c += 1) row[c] = cauchy(p, missing[c], m);
            return row;
        });
        const inv = invertMatrix(mat);
        if (!inv) return false;
        for (let c = 0; c < missN; c += 1) {
            const out = new Uint8Array(width);
            for (let b = 0; b < width; b += 1) {
                let s = 0;
                for (let r = 0; r < missN; r += 1) s ^= gfMul(inv[c][r], rhs[r][b]);
                out[b] = s;
            }
            data[missing[c]] = out;
        }
        return true;
    }

    function concatFecAu(parts) {
        let n = 0;
        for (const p of parts) n += p.length;
        const blob = new Uint8Array(n);
        let o = 0;
        for (const p of parts) {
            blob.set(p, o);
            o += p.length;
        }
        if (blob.length < 4) return null;
        const len = (blob[0] | (blob[1] << 8) | (blob[2] << 16) | (blob[3] << 24)) >>> 0;
        if (len > blob.length - 4) return null;
        return blob.subarray(4, 4 + len);
    }

    class DatagramAssembler {
        constructor() {
            this.pending = new Map();
            this.partial = new Map();
            this.held = new Map();
            this.lastSeq = -1;
            this.holeSince = 0;
        }
        push(buf, now) {
            const t = now == null ? Date.now() : now;
            const frag = parseVideoDatagram(buf);
            const m = frag ? parityCount(frag.frags) : 0;
            if (!frag || frag.frags <= 0 || frag.frag >= frag.frags + m) {
                return this.expire(t);
            }
            if (this.lastSeq >= 0) {
                const age = seqDelta(frag.seq, this.lastSeq);
                if (age === 0 || age > 32768) return this.expire(t);
            }
            if (this.held.has(frag.seq)) return this.expire(t);
            if (this.pending.size >= 4) {
                for (const [seq] of this.pending) {
                    if (seqDelta(frag.seq, seq) > 1 && seqDelta(frag.seq, seq) < 32768) {
                        this.pending.delete(seq);
                    }
                }
            }
            const blockIndex = frag.blockIndex || 0;
            const blockCount = frag.blockCount || 1;
            const slotKey = frag.seq + blockIndex * 65536;
            let slots = this.pending.get(slotKey);
            const isParity = frag.frag >= frag.frags;
            if (!slots || slots.frags !== frag.frags) {
                if (isParity) return this.expire(t);
                slots = {
                    frags: frag.frags,
                    key: frag.key,
                    ptsNs: frag.ptsNs,
                    sentNs: frag.sentNs,
                    parts: Array.from({ length: frag.frags }, () => null),
                    parity: Array.from({ length: m }, () => null),
                };
                this.pending.set(slotKey, slots);
            }
            if (isParity) slots.parity[frag.frag - frag.frags] = frag.payload;
            else slots.parts[frag.frag] = frag.payload;
            const haveData = slots.parts.every((p) => p != null);
            const have = slots.parts.filter((p) => p != null).length
                + slots.parity.filter((p) => p != null).length;
            if (!haveData && have < frag.frags) return this.expire(t);
            if (!haveData) {
                const data = slots.parts.slice();
                if (!recoverFec(data, slots.parity)) return this.expire(t);
                slots.parts = data;
            }
            this.pending.delete(slotKey);
            const au = m > 0 ? concatFecAu(slots.parts) : concatBytes(slots.parts);
            if (!au) return this.expire(t);
            let frameAu = au;
            if (blockCount > 1) {
                let acc = this.partial.get(frag.seq);
                if (!acc || acc.count !== blockCount) {
                    acc = {
                        count: blockCount,
                        parts: Array(blockCount).fill(null),
                        key: slots.key,
                        ptsNs: slots.ptsNs,
                        sentNs: slots.sentNs,
                    };
                    this.partial.set(frag.seq, acc);
                }
                acc.parts[blockIndex] = au;
                acc.key = slots.key;
                if (acc.parts.some((part) => part == null)) return this.expire(t);
                this.partial.delete(frag.seq);
                frameAu = concatBytes(acc.parts);
                slots.key = acc.key;
                slots.ptsNs = acc.ptsNs;
                slots.sentNs = acc.sentNs;
            }
            const expired = this.expire(t);
            return expired.concat(this.accept({
                type: "video",
                seq: frag.seq,
                key: slots.key,
                ptsNs: slots.ptsNs,
                sentNs: slots.sentNs,
                au: frameAu,
            }, t));
        }
        accept(frame, now) {
            const out = [];
            if (this.lastSeq >= 0) {
                const age = seqDelta(frame.seq, this.lastSeq);
                if (age === 0 || age > 32768) return out;
            }
            if (this.lastSeq < 0) {
                this.lastSeq = frame.seq;
                out.push(frame);
                return out;
            }
            const delta = seqDelta(frame.seq, this.lastSeq);
            if (delta === 1) {
                this.lastSeq = frame.seq;
                out.push(frame);
                this.drainHeld(out);
                return out;
            }
            if (frame.key) {
                this.held.clear();
                this.holeSince = 0;
                this.lastSeq = frame.seq;
                out.push(frame);
                return out;
            }
            this.held.set(frame.seq, frame);
            if (!this.holeSince) this.holeSince = now;
            this.trimHeld();
            return out;
        }
        drainHeld(out) {
            for (;;) {
                const next = (this.lastSeq + 1) & 0xffff;
                const frame = this.held.get(next);
                if (!frame) break;
                this.held.delete(next);
                this.lastSeq = next;
                out.push(frame);
            }
            if (this.held.size === 0) this.holeSince = 0;
        }
        trimHeld() {
            if (this.held.size <= MAX_HELD) return;
            const seqs = Array.from(this.held.keys()).sort(
                (a, b) => seqDelta(a, this.lastSeq) - seqDelta(b, this.lastSeq),
            );
            for (const seq of seqs.slice(MAX_HELD)) this.held.delete(seq);
        }
        expire(now) {
            const t = now == null ? Date.now() : now;
            if (this.partial.size > MAX_PARTIAL) {
                const seqs = Array.from(this.partial.keys()).sort(
                    (a, b) => seqDelta(b, this.lastSeq) - seqDelta(a, this.lastSeq),
                );
                for (const seq of seqs.slice(MAX_PARTIAL)) this.partial.delete(seq);
            }
            if (this.held.size === 0) return [];
            if (t - this.holeSince < HOLE_WAIT_MS) return [];
            const dropped = Math.max(1, this.held.size);
            this.held.clear();
            this.holeSince = 0;
            return [{ type: "gap", dropped }];
        }

        onReliableKeyframe() {
            this.pending.clear();
            this.held.clear();
            this.holeSince = 0;
            this.lastSeq = -1;
        }
    }

    class FrameReader {
        constructor() {
            this.buf = new Uint8Array(0);
        }
        push(chunk) {
            if (this.buf.length + chunk.length > MAX_FRAME) {
                this.buf = new Uint8Array(0);
            }
            const next = new Uint8Array(this.buf.length + chunk.length);
            next.set(this.buf, 0);
            next.set(chunk, this.buf.length);
            this.buf = next;
        }
        pop() {
            const split = splitFrame(this.buf);
            if (!split) return null;
            this.buf = this.buf.subarray(split.used);
            return decodeMessage(split.body);
        }
    }

    return {
        TYPE_VIDEO, TYPE_HELLO, TYPE_HELLO_ACK, TYPE_PING, TYPE_PONG, TYPE_IDR, TYPE_BYE,
        MAX_FRAME,
        startCodeLen, extractSpsPps, codecStringFromSps, withStartCode,
        unescapeRbsp, classifyAccessUnit, sliceKind,
        encodeFrame, splitFrame, encodeHello, encodeCtrl, encodePing, decodeMessage,
        hashFromBase64, pickWtHost, FrameReader,
        parseVideoDatagram, DatagramAssembler, seqDelta, DATAGRAM_HEADER,
        HOLE_WAIT_MS, parityCount, recoverFec, concatFecAu,
    };
}));
