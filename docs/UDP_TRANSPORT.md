<div align="center">

# UDP Annex-B Transport - Orbiscreen

[![Version](https://img.shields.io/badge/version-0.25.7-2563eb?style=flat-square&logo=semver)](../CHANGELOG.md)
[![License](https://img.shields.io/badge/license-GPL--3.0-dc2626?style=flat-square)](../LICENSE)
![Rust](https://img.shields.io/badge/rust-1.75%2B-16a34a?style=flat-square&logo=rust)
![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20Android-9333ea?style=flat-square&logo=linux)

</div>

<a href="UDP_TRANSPORT.md">🇬🇧 English</a> · <a href="UDP_TRANSPORT_AR.md">🇸🇦 العربية</a>

---

Android streams H.264 access units over UDP. HTTP MPEG-TS on `/stream` stays for the web client and as fallback.

The daemon advertises `udp_port` on `GET /api/info` (`signaling_port + 1`, default **8789**). Android uses UDP when that port is set and the host is not `127.0.0.1` / `localhost`. USB/AOA cannot carry raw UDP (`adb reverse` is TCP-only), so those paths stay on HTTP.

## Session

1. Client sends **Hello** with the session token (same constant-time compare as HTTP Bearer).
2. Host replies **Hello-Ack** and starts DPLPMTUD. The client waits 1.5 s and treats probes that arrive before the ack as control, not failure.
3. Video is withheld until a datagram size is confirmed. Fragments then use that size.
4. Client pings every 500 ms. Missing all packets for 4 s ends the UDP session.
5. A gap or a lost fragment holds P-frames until the next IDR. The client sends **IDR**; the host reuses `POST /api/control` `action: idr`.
6. Idle clients expire after 5 s without a packet. The last HTTP/UDP client also parks the KWin virtual output.

## Packet format

Every datagram starts with magic `ORB1` and a type byte. Multi-byte fields are little-endian.

| Type | Name | Layout after magic+type |
|------|------|-------------------------|
| 1 | Video | `key u8`, `seq u16`, `frag u16`, `frags u16`, `pts_ns u64`, `sent_ns u64`, payload (28-byte header) |
| 2 | Hello | UTF-8 token |
| 3 | Hello-Ack | empty |
| 4 | Ping | `t0_ns u64` |
| 5 | Pong | `t0_ns u64`, `host_ns u64` |
| 6 | IDR | empty |
| 7 | Probe | `id u16`, zero pad to the probe size |
| 8 | Probe-Ack | `id u16`, `recv u16` (bytes received) |
| 9 | PMTU | `datagram u16` (confirmed size) |

## DPLPMTUD

Search starts at 1200 bytes (576 when `ORBISCREEN_UDP_DROP_ABOVE` is set) and walks up to 1472, or `ORBISCREEN_UDP_MAX_DATAGRAM`. Each probe is padded to the candidate size. Three 250 ms timeouts fail a candidate. Late acks whose id is not the in-flight probe are ignored after the search completes.

The host socket sets `IP_PMTUDISC_PROBE` so a too-large datagram is dropped instead of IP-fragmented.

A 2560×1600 IDR is many fragments. A few percent datagram loss often means no complete keyframe arrives, so the surface stays on the last good frame (or black) while the session stays up. That is the hold-until-IDR rule, not a disconnect.

## Android client

`UdpPlayer` decodes with MediaCodec onto a `SurfaceView` letterboxed to the same content rect as touch mapping (`scaleMode` FIT by default). The stream menu shows send-to-assemble delay as `delay Nms`.

## Latency

`delay` is host-send to client-assemble for one access unit (clock-adjusted `sent_ns`). It does not include capture, encode, or MediaCodec/display after assemble.

On a Samsung Galaxy Tab S5e (SM-T725) over 5 GHz Wi-Fi, 2560×1600@60, VA-API `vah264enc`:

| Path | What is measured | Typical |
|------|------------------|---------|
| Previous (HTTP MPEG-TS + ExoPlayer) | ExoPlayer live offset target (min 8, max 48) | **24 ms** |
| This transport (UDP Annex-B + MediaCodec) | send-to-assemble `delay` | **2–4 ms** (usually 2–3 ms) |

That is about **10×** less player/transport delay than the HTTP live-offset target (~20 ms saved). Encode and display are the same on both paths. HTTP `/stream` still uses the 24 ms ExoPlayer offset.

## Environment

| Variable | Role |
|----------|------|
| `ORBISCREEN_UDP_MAX_DATAGRAM` | Upper bound of the PMTU search (clamped 576–65507, default 1472) |
| `ORBISCREEN_UDP_DROP_ABOVE` | Pretend larger datagrams were lost (PMTU test hook) |
| `ORBISCREEN_UDP_LOSS_PCT` | Randomly drop outgoing datagrams, 0–90 (loss test hook) |
