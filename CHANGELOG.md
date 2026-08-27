<!--
  Orbiscreen - Changelog (GPL-3.0-or-later)
  https://github.com/shadow-x78/orbiscreen
-->
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [v0.13.0] - 2026-08-27

Desktop-environment parity release: the full KDE-level experience (a real compositor-native virtual display, no root, no dialogs) now extends to wlroots compositors, portal grants persist across runs on GNOME, input injection is rootless on X11 and portal-free on wlroots, and per-frame allocations/copies were removed across the whole pipeline.

### Added — discovery & diagnostics
- **`orbiscreen doctor`** (`--json` for the GTK panel): prints the detected session/compositor, the exact ordered capture plan `auto` will follow, EVDI module state, portal presence on the session bus, saved permission grants, swaymsg/hyprctl availability, wlroots virtual-output IPC reachability, and `/dev/uinput` writability — each finding paired with the fix.
- **`orbiscreen doctor --fix [--yes]`**: detects the distro from `/etc/os-release` (dnf/apt/pacman/zypper incl. `ID_LIKE` derivatives), offers to install the EVDI package (`evdi` / `evdi-dkms` + headers), loads the module, and re-verifies.
- **Central environment analyzer** (`capabilities.rs`): session + compositor detection from `XDG_SESSION_TYPE`, `WAYLAND_DISPLAY`, `XDG_CURRENT_DESKTOP`, `DESKTOP_SESSION`, `KDE_FULL_SESSION`, `HYPRLAND_INSTANCE_SIGNATURE`, `SWAYSOCK`, `GAMESCOPE_WAYLAND_DISPLAY`, with a full detection-matrix test suite.
- **Capability-driven capture plan**: `auto` no longer uses a hardcoded chain; it resolves the try-chain from detected capabilities and logs the resolved plan and the reason every step succeeds or falls through. Plans: KDE `kwin-virtual → portal`; wlroots `wlroots-virtual → wlr-screencopy → portal → evdi`; other Wayland `portal`; X11 `evdi → x11-root`.
- GTK panel shows the active capture backend in its status row.

### Added — wlroots capture & virtual displays
- **New `wlr-screencopy` capture backend**: `zwlr_screencopy_manager_v1` with SHM buffers (memfd-backed), damage-aware copies where the compositor supports them, per-output capture by name, stride padding removed, XRGB→opaque alpha normalization, strict frame validation, and clean teardown. No portal, no share dialog.
- **New capture preference `screencopy`** (`[capture] preferred = "screencopy"`), accepted by config validation.
- **Compositor-native virtual outputs on wlroots** (`WlrootsVirtualOutput`): the daemon creates a headless output via Sway IPC (`SWAYSOCK`, native i3-ipc framing, `create output` + mode) or Hyprland IPC (`HYPRLAND_INSTANCE_SIGNATURE` socket, `output create/destroy`), waits until the output is advertised and active, captures it by name with screencopy, and removes the output on stop/crash/drop. IPC failure falls back cleanly to mirroring an existing output; the doctor explains which IPC (if any) is reachable.
- **CI integration test on headless sway**: a dedicated job spawns sway with `WLR_BACKENDS=headless` and exercises real screencopy capture, virtual-output create/list/drop lifecycle, and output teardown.

### Added — GNOME / portal UX
- **Dialog-free portal sessions**: ScreenCast permissions are persisted via restore tokens (`PersistMode::ExplicitlyRevoked`) in `$XDG_STATE_HOME/orbiscreen/portal.json`; a failed/stale token automatically retries with a fresh selection. After the first grant, GNOME streams start instantly with no dialog.
- The RemoteDesktop **input** session persists its grant the same way (separate token).
- `doctor` reports whether each grant is saved.

### Added — rootless / portal-free input
- **wlroots-native input injection** (`virtual-keyboard-unstable-v1` + `wlr-virtual-pointer-unstable-v1`, protocol XML vendored): absolute pointer events and keyboard injection directly on the Wayland socket — no `xdg-desktop-portal-wlr` needed. Pointer coordinates are normalized to the captured output when one is known.
- **New input order on Wayland**: wlroots-native → RemoteDesktop portal → uinput, each with an explanatory fallback warning.
- **XTEST injector for X11**: rootless pointer/keyboard injection via xcb-xtest for any user; uinput remains the stronger fallback when `/dev/uinput` is available.

### Added — frame-pipeline efficiency
- **Pooled frame buffers** (`FramePool`/`PooledFrameBuffer` in `orbiscreen-core`): X11, wlr-screencopy, portal, and KWin capture now fill recycled buffers instead of allocating per frame; the encoder wraps pooled buffers directly into GStreamer buffers (`gst_buffer_new_wrapped` semantics) — the previous per-frame `appsrc` alloc+copy is gone, and each buffer returns to the pool when GStreamer releases the frame.
- **X11 capture upgraded to MIT-SHM**: one persistent shared image (memfd + `attach_fd`, requires MIT-SHM ≥ 1.2) that the X server writes into directly — no per-frame reply payload over the socket — with automatic fallback to plain `GetImage` when the extension is absent (verified live against Xwayland).
- **Identical-frame skipping on X11 mirroring**: a fast 128-bit frame hash suppresses duplicate frames, so an idle mirrored desktop no longer burns CPU re-encoding unchanged content (keepalive pacing unchanged).

### Tests
- Frame-assembly unit tests (stride stripping, premultiplied alpha, truncation), frame-pool recycling/cap tests, hash tests, distro-detection tests for `doctor --fix`, capability matrix tests, plus the sway-headless and live-DISPLAY X11 integration tests.

### Deferred to a follow-up
- **DMA-BUF zero-copy** (planned phase-5 item): requires per-hardware validation that cannot be done in headless CI; the SHM path stays the default.
- **GTK panel EVDI wizard** (planned phase-6 item): `doctor --fix` covers the guided flow on the CLI in the meantime.
- KDE/sway/Xvfb side-by-side performance measurements: to be published once measured on reference hardware.

## [v0.12.6] - 2026-08-27

Full-project audit round: every diagnostic gate (fmt, clippy `-D warnings`, build, tests, audit, machete, deny, shellcheck, gradle assemble+lint) re-run and all findings fixed.

### Fixed
- **EVDI pump never terminated on fatal errors:** after the kernel closed the event channel or the registered buffer vanished, the pump thread retried the failed operation forever and warned every 50 ms while the daemon kept streaming stale keepalive frames. Terminal errors now end the source, which flows through as a clean capture-pump shutdown.
- **KWin virtual-display open blocked a tokio worker:** `KwinVirtualCapture::open` performs blocking wayland round-trips, permission-file retries (up to 2.5 s) and a 5 s handshake deadline; called from an async context it stalled a runtime thread for up to ~7.5 s. The open now runs on `spawn_blocking`.
- **Damage pump zombie after compositor close:** when the compositor closed the layer-shell surface the pump kept attaching/committing to the dead surface forever and swallowed roundtrip errors. It now exits on `Closed` and on connection errors.
- **Truncated X11 `GetImage` replies were zero-padded**, masking real capture failures as black frames — short replies are now capture errors.
- **Mouse buttons 7 and 8 mapped to the same uinput code** (`0x117`); buttons 4–8 now map distinctly onto `BTN_SIDE`..`BTN_TASK` (`0x113`–`0x117`).
- **Encoder input queue allowed 60 raw frames** (`set_max_bytes`), ~475 MB at 1080p and ~2 GB at 4K before backpressure engaged; capped at 4 frames.
- Transport join-buffer mutex locks are now poisoning-tolerant instead of panicking a serving task if another thread ever panicked mid-update.
- `display` config gained upper clamps (7680×4320), matching the existing lower bounds and the two-sided clamps for refresh rate and bitrate.
- A relative `XDG_CONFIG_HOME` is now ignored per the XDG spec (same filter `XDG_DATA_HOME` already had).
- The MPEG-TS integration test's monotonicity check compared every timestamp against the *first* frame instead of its predecessor — it now catches real ordering regressions.

### Fixed (Android)
- **Tapping Refresh killed discovery permanently:** `DiscoveryService.stop()` cancelled the caller-supplied `viewModelScope`, freezing the host list and silently ignoring every subsequent restart. The service now owns its own scope, and `restart()` waits for the NsdManager stop callback before discovering again (also removing a start/stop race that could throw on some devices).
- **Auto-reconnect built the player with an empty token:** the reconnect job called `build()`, whose first action was `reconnectJob?.cancel()` — cancelling the very coroutine it ran in; the resulting `CancellationException` was swallowed by the token fetch, so every reconnect streamed unauthenticated into a 401 loop. External builds cancel pending reconnects; the reconnect path no longer cancels itself and no longer swallows cancellation.
- The `DiscoveryService`/gateway provider captured the Activity context into a ViewModel — now the application context.
- "Forget recent host" in Settings left the stale card on screen until navigation; the row now recomposes immediately.
- `roundIcon` pointed at the square launcher mipmap; it now references `@mipmap/ic_launcher_round`.

### Security
- `event-listener` 5.4.1 → 5.4.2 (RUSTSEC-2026-0221, unsound `!Send` tag across threads). The remaining `derivative` unmaintained advisory is unfixable upstream (`evdi` 0.8.0 is the final release) and stays an accepted warning.
- The damage pump's shared-memory file moved from a predictable fixed path in `/tmp` to an anonymous `memfd_create` region — no world-writable pathname, no cross-instance interference, nothing left on disk.
- Android release workflow: fails fast when `ANDROID_KEY_PASSWORD` is unset, rejects an unsigned APK after `assembleRelease` (previously uploadable), and removes the decoded keystore in an `if: always()` cleanup step.

### Removed
- `transport.webrtc_port_range` config field — dead since the WebRTC teardown (v0.12.1): normalized but never read. Existing config files containing it still parse (the key is ignored).
- Dead code: `VirtualDisplay::open_at`/indexed node selection and the `spec()` getter, `EncodeError::EncoderUnavailable`, Android `HostApi.health`/`sendControl`, `InputDispatcher.wheel`/`stylus`, `DiscoveryViewModel.saveRecent`, `StreamViewModel.toggleToolbar` + the never-changing `toolbarVisible` (toolbar is always shown), the navigation route's unused `label` parameter, the unreachable "idle" branch of the discovery status banner, 15 unused theme colors, 10 unused strings, 8 unused XML colors and dead imports across the client.
- The empty deb `postinst` script and the `packaging/` directory (the AppImage build merged into `scripts/build-appimage.sh`, with `package-appimage.sh` kept as the stable entry point used by the release workflow).

### Changed
- `scripts/install.sh` now also builds and installs `orbiscreen-gtk`; previously the installed desktop entry's `Exec=orbiscreen-gtk` pointed at a binary the script never installed (the GTK build is best-effort so the daemon still installs where GTK 4 dev libraries are missing). The desktop entry's `Icon=` now matches the icon filename every installer ships, and `uninstall.sh` removes the GTK binary too.
- `package-deb.sh` builds both binaries when missing (previously copied them blindly) and carries a valid RFC822 `Maintainer` address; `package-rpm.sh` checks for both binaries, not only the daemon.
- AppImage build script: fixed a shellcheck SC2227 redirection placed between `find` actions.

### Documentation
- `DBUS_SPEC.md` / `DBUS_SPEC_AR.md`: `GetConfig` example matches the current schema (removed the deleted `display.count` and `webrtc_port_range` lines, added the `[capture]` section).
- Bug-report template no longer suggests a `RUST_LOG` target for the long-removed `webrtc_rs` crate.
- `SECURITY.md` / `PACKAGING.md` / `PACKAGING_AR.md` version matrices updated to the current release.

## [v0.12.5] - 2026-08-26

### Fixed
- **Unbounded frame channel in the portal/mirror capture path:** the Wayland portal backend passed raw BGRA frames (~8 MB/frame at 1080p) over an unbounded channel, so a stalled consumer could grow memory without limit — the exact failure mode the KWin backend's bounded queue (v0.12.0) was built to prevent. The portal path now uses the same bounded channel (capacity 2) with drop-on-full and a debug log.
- **Config silently ignored under systemd:** the default `--config` was the relative path `orbiscreen.toml`, and no install path (manual / deb / rpm) set `WorkingDirectory=` or passed `--config`, so the service resolved the file against `$HOME` and silently fell back to defaults — custom resolution, port or encoder choices never took effect. The default is now the XDG path `$XDG_CONFIG_HOME/orbiscreen/orbiscreen.toml` (`~/.config/orbiscreen/orbiscreen.toml` when unset), used identically by the daemon, the GTK panel and the systemd user unit, and documented in the README and INSTALL guide.
- **Session token duplicated in the stream URL:** the Android client sent the token both as an `Authorization: Bearer` header (the actual authentication source) and as a `?token=` query parameter; the query parameter is gone, removing a leak path through verbose HTTP/ExoPlayer logs.
- **X11 capture errors lost their detail:** `GetImage` reply failures were collapsed to a constant error code `0`; real X11 protocol error codes are now surfaced, and connection failures are reported as a distinct connect error.
- **Android: deprecated OkHttp API** — `RequestBody.create(...)` in the control API client replaced with the `toRequestBody(...)` extension, matching the rest of the client.

### Security
- **Android `network_security_config.xml`:** removed the dead `<domain-config>` LAN list — Android `<domain>` entries do not match CIDR ranges, so the list matched only network/broadcast addresses and enforced nothing. Cleartext HTTP remains globally permitted by design (the app only contacts LAN hosts the user selects via mDNS or manual entry); the decision is now stated explicitly in `SECURITY.md`.

### Removed
- **`display.count` config field:** it was read for display purposes only and never created more than one virtual display, so it looked configurable but had no effect. Removed from the config schema, `list-displays` output, GTK panel and tests until real multi-display support lands.
- Unused `gstreamer-video` dependency from `orbiscreen-encode` (confirmed by `cargo machete`).
- Orphaned allow-list entries in `deny.toml` (licenses encountered by no current dependency).

### Changed
- Comment policy applied project-wide: explanatory comments removed from code files while every source, config and template file carries a consistent top-of-file header (GPL-3.0-or-later notice + repository link, banner-style for config/CI files); third-party vendored code (`clients/web/vendor/mpegts.js`) untouched.
- Missing final newlines added across manifests, config and web files (`.editorconfig` `insert_final_newline` compliance).

### Documentation
- README / README_AR and docs/INSTALL / INSTALL_AR: XDG config location, `--config` override, and a dedicated **Configuration** install section.
- SECURITY.md: cleartext-HTTP decision for the Android client documented next to the token threat model.

## [v0.12.4] - 2026-08-26

### Added
- **Client diagnostics in `/health`:** new `stream_starts` and `auth_failures` counters, plus a warning line with the peer address on every rejected request — makes "phone shows nothing" diagnosable from the server side in one curl.
- Web client now plays with ~100 ms of buffer: IO stash disabled and live-latency chasing tuned to chase whenever buffered latency exceeds 1 s down to 200 ms (`liveSync` enabled).

### Changed
- `scripts/verify-stream.sh` no longer hard-requires ffprobe; it falls back to ffmpeg-only detection when ffprobe is missing or broken.

## [v0.12.3] - 2026-08-25

### Fixed
- **Growing playback latency ("slow stream"):** stream timestamps advanced one frame duration per pushed packet regardless of real elapsed time, so during idle periods PTS ran many times slower than the wall clock and live players accumulated delay they could never chase back. Timestamps now follow the wall clock: keepalives stamp 500 ms apart, active streaming stamps at real time.
- **Garbage frames right after connecting:** new clients previously started receiving packets from the middle of a GOP — deltas without their references, which decoders render as corruption until the next natural keyframe. The transport now keeps the current GOP since its last keyframe and replays it to every joining client under the pump lock before going live, making joins gap-free, duplicate-free and instant.
- **Silent mid-stream packet loss:** a slow client's mux queue used to drop chunks quietly, corrupting that client's decode until an arbitrary keyframe. Queues no longer emit holes — on overflow the client's stream ends cleanly instead, and clients rejoin at a keyframe via the self-healing reconnect. After any broadcast lag, clients also freeze on the last good frame and resume at the next keyframe rather than decoding orphaned deltas.

### Changed
- Removed the obsolete EVDI install hint from RPM `%post` and deb `postinst` scriptlets (EVDI is opt-in; KDE Wayland needs no kernel module) and corrected the deb description accordingly.
- Added `ORBISCREEN_ENCODER_DUMP=<path>` diagnostics hook: appends raw Annex-B encoder output to a file for bitstream debugging.

## [v0.12.2] - 2026-08-25

### Fixed
- **Black stream on idle desktop (the "no image at all" bug):** compositors deliver screencast frames on damage only, and a freshly created virtual display is completely static — the capture received exactly one pre-paint black buffer and froze on it forever. A new damage pump keeps the virtual output recompositing (~2 fps) via an invisible, click-transparent layer-shell surface, so the capture always reflects the real display content.
- **Corrupted H.264 bitstream from forced key units:** `UpstreamForceKeyUnitEvent` on the x264enc src pad produced malformed I-frames (decoder errors like `out of range intra chroma pred mode`, black concealment output) regardless of threading mode. Forced key units are gone; clean joins are guaranteed by the intact delta chain plus natural IDRs (`key-int-max` lowered 30 → 10, worst-case idle join ≤ 5 s, active join ≤ 200 ms).
- **Startup blocked forever on the input portal:** the daemon now waits at most 20 s for the RemoteDesktop portal and starts streaming anyway with a clear warning when it hangs or needs interactive approval; remote control comes online on the next restart once granted.

### Changed
- x264enc runs with `sliced-threads=false` and 2 frame threads: slice-level threading (implicit in `zerolatency`) is fragile around keyframe requests and produced 18 slices per frame; frame-level threading keeps each access unit atomic.

## [v0.12.1] - 2026-08-25

### Added
- **Mirror capture mode:** `[capture] preferred = "mirror"` streams your real desktop (picked in the portal share dialog) instead of a second virtual monitor — for when you want to *see* your screen rather than extend it.
- **Detailed `/health`:** now returns JSON with `version`, `encoder`, `frames_forwarded`, `active_clients`, `total_clients` and `uptime_seconds`, making stream diagnosis a single curl.
- **`scripts/verify-stream.sh`:** end-to-end stream sanity check — records a few seconds of `/stream`, decodes with ffprobe and measures frame brightness to catch black/empty-stream regressions automatically.
- **Full project audit hardening:** GitHub Actions pinned to commit SHAs (supply-chain), `gitleaks`/`shellcheck`/`cargo-deny` clean runs documented, comment policy applied (comment-free code files, banner-style headers on config files).

### Changed
- **EVDI is opt-in.** `[capture] preferred = "evdi"` requests the EVDI DRM virtual display explicitly; on Wayland, `auto` never touches EVDI anymore, so the recurring `EVDI kernel module not active` line is gone on KDE and the portal is reached directly on other compositors. On X11, `auto` still uses EVDI when its module is already loaded.
- The encoder-to-transport video channel is bounded (64 packets) so a stalled transport backpressures the pipeline instead of growing memory without limit.

### Fixed
- **New clients saw an endless black screen on idle virtual displays.** KWin delivers virtual-display frames on damage only: a static desktop stops producing frames entirely, and keepalive re-pushes encoded as deltas — which `h264parse`/`mpegtsmux` hold back until the first keyframe, so a freshly connected client received nothing. The daemon now re-pushes the last frame every 500 ms **with a forced IDR**, so any new client decodes within half a second no matter how idle the display is.
- **Android: the app never recovers after a daemon restart.** The stream token is rotated on every daemon start, but the Android client cached it forever — every reconnect looped on 401 with a black screen. The player now re-fetches a fresh token before each reconnect/retry, and input/control pick up rotated tokens automatically (periodic refresh + `InputDispatcher.updateToken`), mirroring the web client's self-healing reconnect.
- **Android: `build.gradle.kts` hard-failed configuration without a signing keystore**, blocking `lintDebug`/`assembleDebug` on dev machines. Signing is now conditional (release builds unsigned with a warning when secrets are absent) and CI explicitly rejects unsigned APKs.

## [v0.12.0] - 2026-08-24

### Added
- **KWin virtual display backend (no root, no portal):** on KDE Plasma the daemon now creates a real virtual monitor (`Virtual-ORBISCREEN`, visible in Display Settings) through KWin's `zkde_screencast_unstable_v1` Wayland protocol and streams it over PipeWire directly — bypassing xdg-desktop-portal entirely, so no share dialog appears and the portal's crash-prone path is avoided. KWin only exposes the protocol to allow-listed executables, so the daemon maintains `~/.local/share/applications/orbiscreen.kwin.desktop` (user-writable, no sudo) with `X-KDE-Wayland-Interfaces=zkde_screencast_unstable_v1` and its own executable path, refreshing the KService cache and retrying the connection until the grant is visible. Closing the stream removes the virtual output automatically. New `[capture] preferred = "auto" | "kwin-virtual" | "portal"` config option; `auto` (default) tries KWin first on Wayland and falls back to the portal on non-KDE compositors, while explicit preferences fail loudly on non-Wayland sessions instead of silently capturing the real desktop.
- **Web client self-healing reconnect:** after a daemon restart the stream token changes, and any already-open browser tab used to loop forever on a silent black screen (the overlay was hidden and the stale token was never refreshed). The client now re-shows the status overlay on stream loss and re-fetches `/client/config.json` before each reconnect, so tabs recover automatically once the daemon is back.

### Changed
- Capture fallback log no longer claims a portal dialog is always required; the hint is logged only when the portal path is actually taken.
- Frame validation is shared between the portal and KWin backends (`sample_to_captured_frame`), so size-mismatch and malformed-sample diagnostics can no longer drift; the KWin backend uses a bounded frame queue (drops under stall instead of growing without limit).
- A compositor-side close of the virtual output is reported as terminal (`CaptureSession::is_ended`) and stops the capture pump instead of retrying forever with warning spam.

## [v0.11.2] - 2026-08-23

### 🐛 Fixed
- **X11 input registration:** the uinput device registered keys only up to `KEY_KPDOT` (code 83), so the kernel silently dropped everything above it — arrow keys, Insert/Delete/Home/End/PageUp/PageDown, right Ctrl/Alt, Meta, F11/F12 and most numpad keys never reached the host on the X11 path; wheel input sent `REL_WHEEL` without registering the axis at all. The virtual touchscreen now registers the full key range (`KEY_MAX`) plus `REL_WHEEL`, making every mapped client key functional.
- **Stream task panic on short packets:** the non-NAL debug log sliced `bytes[..4]` unguarded, so any packet shorter than 4 bytes killed the per-client streaming task; the slice is now length-clamped.
- **Daemon exit on D-Bus failure:** when the session bus was unavailable, dropping the D-Bus handles closed the shutdown watch channel, so the daemon exited immediately after start with a misleading "D-Bus Stop received" log; a keep-alive sender now holds the channel open until a real stop.
- **RPM version drift:** `package-rpm.sh` hardcoded a stale `VERSION=0.6.0` fallback; it now extracts the workspace version from `Cargo.toml` dynamically like the deb builder.
- **Duplicate ADB reverse setup:** both the daemon startup path and `Transport::serve()` ran `setup_reverse_for_all` on every launch (a blocking call inside async context); it now lives only in the transport layer.
- **Misleading installer log:** `install.sh` printed "Reloading systemd user daemon..." before writing the unit file and never actually reloaded; the line is gone.
- **Desktop entry validation:** `Categories` carried two main categories (`Utility` + `System`); `desktop-file-validate` is now fully clean.
- **Wayland capture diagnostics:** the portal pipeline now watches the GStreamer bus and logs real negotiation/caps errors instead of failing silently with zero frames, and `pipewiresrc` output is pinned to system memory so compositors offering DMA-BUF buffers cannot stall negotiation.
- **Portal log noise:** the ashpd/zbus property-cache warnings are filtered from the GTK binary's logs as well (the daemon already did).
- **Portal denial handling:** dismissing or denying the screen-share prompt now fails with an explicit "user denied the ScreenCast permission" message instead of the cryptic "Portal request didn't succeed with no information".
- **Monotonic stream timestamps:** captured frames were pushed to the encoder with `PTS=0`, producing non-monotonic chunk timestamps (first keyframe stamped at a huge running-time base, everything else at 0) that made `mpegtsmux` stall right after the first AU — clients froze on the initial, often-black frame. The pump now stamps every frame with `frame_index × 1/fps`, so the MPEG-TS stream flows continuously and late-joining clients pick up on the next keyframe.
- **H.264 byte-stream format (root cause of the black screen):** `x264enc` defaults to `byte-stream=false`, and the encoder pipeline's appsink accepted any stream-format — so `h264parse` silently converted the output to AVCC (length-prefixed NALs) while the transport advertised `stream-format=byte-stream`. The transport's `h264parse` then waited forever for Annex B start codes that never arrived, and `mpegtsmux` emitted zero bytes: every client saw a frozen black frame. The encoder now forces `byte-stream=true` and its appsink pins `stream-format=byte-stream, alignment=au`; an integration test feeds real encoder output through the muxer and asserts MPEG-TS bytes come out.
- **Accurate EVDI fallback message:** without the evdi kernel module the daemon no longer claims "clients will see the host's main display" — portal capture lets you pick any display, including virtual monitors, in the share dialog (now logged at info, not warn).
- **Universal H.264 profile:** the encoder negotiated Y444 input, producing High 4:4:4 Predictive video that Android hardware decoders cannot play — phones showed a black frame even while browsers with software decoding managed. The pipeline now pins `NV12` before `x264enc`, yielding Constrained Baseline output that decodes everywhere (verified with ffprobe on a live stream: ~7 MB over 7 s at 1080p60).
- **Per-client pipeline lifetime:** a teardown guard placed in the HTTP handler scope set the client's muxer pipeline to flushing as soon as the handler returned, so `/stream` delivered HTTP 200 with zero bytes. The guard now lives inside the streaming task itself, which also guarantees clean NULL transitions when clients disconnect or the daemon shuts down — silencing the burst of GStreamer "Trying to dispose element … PLAYING" CRITICALs.
- **Web control gating (mouse hijack):** moving the mouse over the browser player forwarded every movement as absolute pointer injection, yanking the host cursor across monitors. Control now requires an explicit click that engages Pointer Lock ("Click to control" hint, Esc to release); wheel and keyboard follow the same gate.
- **evdi module helper bundled:** `install-evdi-module.sh` ships inside the deb/RPM packages with a post-install hint, and it now builds the module automatically from DisplayLink main when no prebuilt `evdi.ko` is found (required on kernels newer than the vendored 1.9.1 sources support).

### 🔒 Security
- **systemd hardening:** `NoNewPrivileges=true` added to all shipped user units (install.sh, deb builder, RPM spec).
- **Android permissions pruned:** removed unused `VIBRATE`, `ACCESS_FINE_LOCATION` and `ACCESS_COARSE_LOCATION` from the manifest.
- Full security re-audit: constant-time token comparison, bounded queues, `/api/control` auth + fixed-argument commands, no WebView in the Android client, OkHttp timeouts everywhere, secrets only via CI env — verified sound, no changes required.

### 🗑️ Removed
- Dead transport API: `find_usb_device()`, `remove_reverse()`, `first_device_serial()` (+ their tests); the dead `StreamUrl.mimeType()` with its suppress/import; the unused synchronous `InputInjector::open()`.
- The unreferenced `gen-key-script` GPG helper.
- Residual inline comments across Rust/Kotlin/web/shell/spec/drawable sources — code files keep only the standard two-line license headers, while env-style config files (`gradle.properties`, ProGuard rules, `deny.toml`, `rustfmt.toml`, `.gitignore`, `.editorconfig`, RPM spec, desktop entry, `lint.xml`, AndroidManifest) carry uniform decorative section banners instead.

### 🔧 Changed
- Release workflow builds with `--locked`; Android `versionCode` 17 → 18.

## [v0.11.1] - 2026-08-16

### 🐛 Fixed
- **Android discovery staleness:** vanished mDNS services are now actually removed from the host list (the lost-service event previously carried no address, so stale entries lingered forever); subnet-sweep candidates are verified with the daemon's own `/api/info` before being listed.
- **Structured concurrency:** the subnet scanner rethrows `CancellationException` so leaving the discovery screen genuinely stops in-flight probes; the player's supervisor scope and OkHttp dispatcher are shut down on release.
- **Android 12+ gateway detection:** `WifiManager.dhcpInfo` (deprecated, null without location grant) is replaced with `ConnectivityManager.getLinkProperties()`, so the subnet sweep works on modern Android; SDK level raised (`compileSdk 35`) to match `targetSdk 35`.
- **Web remote control:** pressed pointer buttons are tracked in a set and all released on `pointercancel`/`pointerleave` (previously only button 1 and only while buttons were held, leaving stuck buttons on the host); keystrokes are no longer forwarded while the stream is still connecting; playback rejection now triggers the reconnect scheduler.

### 🗑️ Removed
- **Dead code across all crates:** unused dependencies (`evdi-sys`, `libc`, `gstreamer-video`), unused public API (`open_many`, `remove_all_nodes`, `device_node_index`, `write_debug_ppm`, `stream()`, `TouchCalibration`, `fullname()`, the dead `query_status`/`call_get_status` chain), and the `Start()` no-op method from the D-Bus surface (starting stays a systemd job).
- **Unused client surface:** `TouchCalibration`, the legacy pre-Compose `activity_main.xml`, unused Gradle dependencies (`appcompat`, `security-crypto`, `media3-exoplayer-hls`), the dead `syncWebClient` asset step, unused strings/colors resources, and the half-wired `label` navigation argument.
- **Stale packaging assets:** unreferenced icons in `data/` (only `orbiscreen.svg` is used), the superseded `packaging/debian/` dir, and the never-built `packaging/flatpak/` manifest.
- **Inline comments** were stripped project-wide; only the per-file license header remains.

### 🔧 Changed
- `rand` bumped to 0.9 in `orbiscreen-transport`; Android `versionCode` 16 → 17.

## [v0.11.0] - 2026-08-16

### 💥 Breaking
- **Token auth for media & control endpoints:** `/stream`, `/input`, and `/api/control` now require a per-session access token (32 random bytes, regenerated on every daemon start). Clients present it via `Authorization: Bearer <token>` or `?token=<token>`. Android clients get it from mDNS discovery TXT / `/client/config.json`; the bundled web client bootstraps from `/client/config.json`. `/health`, `/api/info`, `/client/*` remain public.
- **Keystore removed from the repository:** the Android release signing key is no longer shipped with the source (it remains in git history for anyone who pulled earlier). Builders must supply their own signing key; APKs signed with the old key have a different signature, so uninstall the old app before installing newly signed releases.
- **Dead transport paths removed:** the legacy `/ws` WebSocket and WebRTC `/sdp` signaling endpoints are gone. WebRTC is fully replaced by MPEG-TS over HTTP; `webrtc_port_range` is kept only as a compatibility placeholder.
- **`orbiscreen stop` is now real:** instead of a no-op, it asks the running daemon to shut itself down gracefully through the D-Bus session service (`Stop()`), reporting "daemon is not running" when the service is absent.
- **Android Stream toolbar:** the dead "Open files" action (no backend since the WebRTC era) was removed from the control toolbar.

### ✨ Added
- **evdi is now the primary frame source:** `orbiscreen-display` reads the real virtual-display framebuffer through the evdi DRM interface, so clients see the actual second monitor the compositor draws on (previously the capture fallback was always used). On hosts without the evdi kernel module the daemon degrades gracefully to Wayland portal / X11 capture of the primary desktop and logs a clear warning.
- **Web client:** a browser client is bundled and served by the daemon at `http://<host>:8788/`, playing MPEG-TS live via MediaSource Extensions using the locally vendored `mpegts.js` (no CDN dependency, works offline on the LAN).
- **D-Bus live status:** `GetStatus()` returns a JSON object with live fields (`running`, `frames_forwarded`, `active_clients`, `total_clients`, `encoder`, `capture_backend`); `ListClients()` reports real counts from transport stats; `GetConfig()` dumps the sanitized TOML config; `Stop()` triggers a graceful shutdown.
- **Client statistics:** the transport tracks active/total stream clients and forwarded frames, surfaced via D-Bus and used by the GTK panel.
- **`/api/control` real host tools:** lock (`loginctl`/`xdg-screensaver`), blank/unblank (DPMS via sway/hyprland/xset), and `ctrl_alt_del` injection; arbitrary URL `open` is explicitly rejected.
- **GTK panel rewrite:** the GTK4 app now talks to the daemon over zbus with live status polling, honest Stop-only behavior with toast feedback, and real settings from the daemon config.

### 🐛 Fixed
- **Stream/input dimension alignment:** pointer coordinates are scaled through the capture region reported to clients, so touch and mouse land in the right place regardless of display scaling.
- **Bounded channels:** the per-client MPEG-TS path uses a bounded channel and drops chunks for stalled consumers instead of growing daemon memory without limit; stream lag tolerates `Lagged` broadcast errors until the next keyframe instead of disconnecting.
- **Android translation breakage:** removed the dangling `onOpenFiles` reference in StreamScreen/ControlToolbar/strings.xml that kept Kotlin resources out of sync.
- **Packaging:** deb/rpm/AppImage/Flatpak specs and `install.sh` updated to match the current binary layout, client directory, and service unit.
- **GTK panel build:** removed the duplicate `start_status_poller` definition, replaced the removed glib 0.20 `MainContext::channel`/`PRIORITY_DEFAULT` API with an mpsc channel drained on the GTK main loop via `timeout_add_local` (widget handles stay on the main thread, since they are `!Send`), and fixed the Gradle release signing block where the local `keyAlias` val shadowed the DSL property.
- **CI Android signing:** the push-time Android workflow now feeds the keystore from the same repo secrets as the release matrix (`ANDROID_KEYSTORE_B64` etc.), rejects blank signing secrets at configure time, and validates the keystore password/alias with `keytool -list` before the build so mismatched secrets fail immediately instead of after packaging.
- **Player build break:** removed the nonexistent `setTargetLiveOffsetMs` call (not part of media3 1.3.1's `DefaultLivePlaybackSpeedControl.Builder`); the live-edge target is set per media via `MediaItem.LiveConfiguration.setTargetOffsetMs`.
- **Input injection hardening:** pointer buttons outside the registered range (0 or >8) are rejected instead of being silently mapped to left-click, wheel deltas are clamped before the integer cast, and a zero wheel delta no longer emits a bare SYN_REPORT; the Wayland portal injector teardown only spawns the session close when a Tokio runtime context exists, so dropping it outside the runtime can no longer panic from `drop`.

### 🔒 Security
- **mDNS token redaction:** daemon events are now logged by kind only, never via `Debug` (whose `ServiceInfo` rendering can include the TXT record carrying the session access token); the monitor thread also switched from a 60 s receive-timeout to a blocking receive, so daemon events keep being consumed instead of dropping out after the first idle minute.
- Corrected SECURITY.md: the daemon binds `0.0.0.0:8788` (not `127.0.0.1`), endpoints are now token-protected (not "unauthenticated by design"), and the token model + keystore rotation are documented, including limitations against a determined LAN attacker.

### 📚 Docs
- Rewrote `docs/DBUS_SPEC.md` to match the real interface (JSON GetStatus, TOML GetConfig, systemd-only Start); added streaming/client troubleshooting guides (wrong screen via missing evdi, no picture without MSE, missing x264 plugin, 401 token flow, D-Bus service absent); refreshed READMEs for the web client, token model, and evdi requirement.

## [v0.10.7] - 2026-08-12

### 🐛 Fixed
- **Wayland portal stall (1-5 fps):** Encoder `appsrc` was missing `is-live=true` and `do-timestamp=true`. With Wayland's bursty deliverable from `pipewiresrc` and x264's `tune=zerolatency` assumptions, frames backed up in the encoder queue and never reached the appsink. Watchdog showed constant "N frames pushed" with no growth. Now the appsrc timestamps each buffer on arrival, so the encoder treats live input as a stream rather than one giant blob.
- **Stream PTS jumping:** The transport's appsrc previously had `do-timestamp=true` which overrode the encoder's real PTS values, causing mpegtsmux to see wildly out-of-order timestamps and emit nothing. Now we use the real PTS emitted by `h264parse` (after `config-interval=1` ensures SPS/PPS lands on each IDR).
- **NAL garbage filter:** Added a strict H264 start-code validator on the streaming push path so out-of-band packets never reach `gst_base_parse_handle_buffer`, eliminating the `SIGSEGV` seen when a client disconnected mid-frame.

## [v0.10.6] - 2026-08-11

### 🐛 Fixed
- **`/stream` returning 0 bytes:** Streamed MPEG-TS pipeline lacked explicit `video/x-h264` caps on `appsrc`. Without caps, `mpegtsmux` never classified incoming NAL units as keyframe-anchored video and withheld output indefinitely. Clients saw `200 OK` with an empty body.
- **Slow frame pacing (1-2 fps instead of 60):** Wayland capture pipeline lacked explicit frame size caps. The `videoscale video/x-raw,format=BGRA,width=WIDTH,height=HEIGHT` chain was missing so `pipewiresrc` negotiated its own resolution and `videoconvert` overhead ballooned. Added explicit caps plus stride-mismatch detection that drops frames whose buffer size doesn't match `W*H*4`.
- **`/api/info` and `/api/control` 404:** Routes weren't registered with the axum `Router` at all. Added fully wired handlers backed by `AppState` so the Android client can discover resolution / encoder / version and trigger host control actions.
- **SPS/PPS visibility:** Added `h264parse config-interval=1` in the encoder pipeline so parameter sets are reattached to every keyframe, letting clients that join mid-stream sync immediately.
- **Encoder push starvation:** Reduced x264 `key-int-max` to 30 (0.5 s at 60 fps) so clients joining mid-stream get a keyframe within half a second instead of waiting 8+ s for the next IDR.
- **Android launcher icon (legacy PNGs):** Regenerated all five density PNGs from the freshly restyled vector so the install screen and pre-Oreo launchers see the same shape as Android 8+ adaptive icons.
- **Android launcher icon (adaptive):** Artwork group scaled by 1.08 in `ic_launcher_foreground.xml` around the canvas centre so the brand mark reads bigger at small sizes without clipping the safe zone.

## [v0.10.5] - 2026-08-11

### 🐛 Fixed
- **Android Launcher Icon Distortion:** The previous `ic_launcher_foreground.xml` used corner-radius arcs (`A24,24`) on the monitor outline which the VectorDrawable renderer stretched, making the icon look rubbery and off from the brand SVG. Rebuilt all pathData with explicit `h`/`v`/`a` commands matching `data/orbiscreen-app.svg` exactly (monitor fill `#1E1E2E` with blue stroke, inner screen bezel, white accent arc, blue stand neck, slate base). Art now scales to 0.50 within a centered `(128, 124)` translate so it sits perfectly inside the adaptive safe zone.
- **Legacy Launcher PNGs:** Regenerated all `mipmap-*dpi/ic_launcher.png` from the corrected vector so the install-time icon (used by launchers that don't support adaptive icons) renders sharp and non-stretched.
- **x264 `repeat-headers` Crash:** Setting the property unconditionally panicked on newer GStreamer builds where `GstX264Enc` no longer exposes it. Wrapped in `find_property(...).is_some()` since `tune=zerolatency` already enables repeat headers internally.

## [v0.10.4] - 2026-08-11

### 🐛 Fixed
- **Android Launcher Icon Background:** Adaptive icons (`ic_launcher.xml`, `ic_launcher_round.xml`) used `@android:color/transparent` so the launcher showed a black square around the artwork on many OEM launchers - switched to a real `@color/ic_launcher_background` (#FFFFFF) resource.
- **Android Launcher Icon Oversized:** Foreground scale reduced from `0.66` to `0.56` inside `ic_launcher_foreground.xml`, keeping the artwork inside the adaptive safe zone so it renders smaller and non-intrusive after install.

### 🔧 Changed
- **Legacy Launcher PNGs:** Regenerated every `mipmap-*dpi/ic_launcher.png` from `data/orbiscreen-app.svg` on a white background with properly scaled artwork (was full-bleed on transparent).
- **README Restyle:** Reworked both `README.md` and `README_AR.md` to the concise project pattern (comparison table, highlights list, screen-summary table for the Android app instead of long prose, full uninstall command documented).
- **Docs Typography:** Replaced all em dashes (`—`) with standard hyphens across `README*.md`, `CHANGELOG.md`, `SECURITY.md`, and `docs/*.md` for a cleaner, tool-agnostic documentation style.

## [v0.10.3] - 2026-08-10

### 🐛 Fixed
- **EDID Range-Limits Descriptor:** Write the `0xFD` tag at byte 72 (descriptor start) instead of byte 75, so virtual-monitor EDID blocks validate against strict DRM parsers.
- **Mouse Buttons Dropped:** Register `BTN_LEFT..=BTN_TASK` with the uinput device - clicks were silently discarded because only keyboard codes were declared.
- **Wayland Session Leak:** Close the `RemoteDesktop` portal session on drop so the compositor's screen-sharing indicator disappears with the daemon.
- **D-Bus Service Exit:** Keep the D-Bus task alive with a parked future instead of returning `Ok(())` immediately and dropping the connection.
- **Stream Pipeline Panics:** Return `503 SERVICE_UNAVAILABLE` from `/stream` when the GStreamer pipeline can't be built, instead of panicking the axum worker.
- **MPEG-TS Packet Panics:** Replace `unwrap()` calls inside the per-client broadcast task with graceful `warn + break`, so a lagging client can't crash the daemon.
- **mDNS Fullname:** Build the real service fullname (`instance._orbiscreen._tcp.local.`) for `unregister()` so the record is actually removed when the advertiser drops.
- **ADB Scan Robustness:** Skip malformed `adb devices` lines instead of aborting the scan on the first one.
- **Config Hardening:** Add `#[serde(default)]` to all config sections and clamp degenerate values (`fps=0` divide-by-zero, `bitrate` overflow, inverted port ranges, `count=0`).
- **Encoder Bitrate Units:** Pass kbit/s verbatim to VAAPI/NVENC instead of multiplying by 1000 (hardware encoders silently received a 1000× too-high target).
- **Encoder Resource Leaks:** Send EOS before `State::Null` so tail frames flush, cap the appsrc queue (~8 MB), and `Drop`-shutdown the pipeline cleanly.
- **Wheel/Key/Input Bounds:** Round wheel deltas instead of truncating, clamp stylus tilt to ±90°, reject `u16`-overflowing key codes, and `saturating_sub` in `clamp_point` for zero-size specs.
- **Coordinate Scaling:** Scale raw `{x,y}` pointer payloads through the capture region (the stream's source of truth) so hand-written clients land in the right place.

### 📱 Android
- **Wire Protocol Alignment:** `InputDispatcher` now sends the tagged envelopes the daemon actually deserializes (`{"Pointer":{...}}`, `{"Key":{...}}`, `{"Stylus":{"Tilt":{...}}}`) - previously wheel/stylus/key events failed to parse.
- **Linux Key Codes:** The soft keyboard emits evdev codes (`a`=30, space=57, enter=28, backspace=14) instead of Android `KeyEvent` codes, which the host rejected.
- **Stream URL:** Point ExoPlayer at `/stream` (the served route) instead of the nonexistent `/stream.ts?fmt=mp2t`.
- **NSD Lifecycle:** `DiscoveryService.stop()` actually cancels the coroutine scope so `stopServiceDiscovery` runs; refresh no longer leaks discovery sessions.
- **Host:Port Validation:** Reject out-of-range ports and IPv4 octets > 255 in the manual-entry field.
- **Launcher Icon:** White-background adaptive launcher for SDK < 26 (PNG fallbacks), vector foreground scaled into the safe zone for SDK 26+.

### 🌐 Web Client
- **Protocol:** Rewrite `app.js` to send the same tagged `Pointer`/`Key`/`Stylus` envelopes as Android; delete the dead WebRTC `/sdp` negotiation path (`/sdp` always 503s).
- **Pointer Mapping:** Correct for `object-fit: contain` letterboxing so taps land where the video renders, not where the element is.
- **Pointer Capture:** Add `setPointerCapture` + `pointercancel` so drags past the video edge don't leave host buttons stuck.
- **Key Translation:** Map DOM `KeyboardEvent.code` to Linux evdev codes (1:1 table) instead of the broken `keyCode` numerical passthrough.

### 🔧 Changed
- **Icons:** `data/orbiscreen.svg` and `data/orbiscreen-app.svg` now ship a white rounded-rect background; all raster icons (Linux `data/*.png`, every Android `mipmap-*/ic_launcher.png`) are regenerated from the SVG with a white background.
- **AppImage Builder:** Replace the hand-painted placeholder PNG with a `magick` rasterization of the real logo (white background).
- **Docs:** English docs and new `README_AR.md` / `docs/*_AR.md` now follow the reference style (ASCII banner, hex Tailwind badges, `Language` switcher, anchored sections, centered footer).

## [v0.10.2] - 2026-07-27

### 📱 Android
- **Brand Palette:** Replace Material You dynamic color with Catppuccin Mocha / Latte palettes matching `data/orbiscreen-app.svg` so app chrome (status/nav bars, splash, launcher icon background) matches the logo.
- **Splash Screen & Adaptive Launcher:** Add a SplashScreen with brand background and a vector adaptive launcher foreground mirroring the SVG.
- **Connect-time Crash Fix:** Move ExoPlayer construction to the Main thread (previously `playerHolder.build()` ran inside `withContext(Dispatchers.IO)`, which killed the process the moment the user tapped Connect).
- **Error Hardening:** Harden `build()` against construction failures so they surface as `StreamEvent.Error` instead of crashing.

### 🔄 Updated
- **Version Bump:** Bump Cargo workspace version to `0.10.2` and Android `versionName` to `0.10.2` (`versionCode = 10`).
- **Documentation:** Update all documentation files (`README.md`, `CHANGELOG.md`, `CONTRIBUTING.md`, `SECURITY.md`, `docs/*`) to `v0.10.2`.

## [v0.10.1] - 2026-07-27

### 🔄 Updated
- **Workspace Bump:** Bump workspace `Cargo.toml` version to `0.10.1`.
- **Android Bump:** Bump Android `versionName` to `0.10.1`.
- **Cargo.lock:** Refresh `Cargo.lock` path-crate entries for version `0.10.1`.

## [v0.10.0] - 2026-07-27

### ✨ Added
- **Material 3 UI:** Migrate Android client to Material 3 + Jetpack Compose.
- **Live Discovery:** Add Spacedesk-style live NSD scan with manual `host:port` entry.
- **Subnet Scanner:** Add optional subnet scanner for networks without mDNS.
- **Recent Host:** Persist and surface recent host with a Material chip.
- **Absolute Input:** Replace delta-based `TouchInjector` with absolute `InputDispatcher` supporting multi-touch, wheel, stylus, and keyboard events.
- **Control Toolbar:** Add host control toolbar overlay (lock, blank, ctrl-alt-del, files, retry).
- **Soft Keyboard:** Add soft keyboard overlay with system IME handoff.
- **Settings Screen:** Add settings screen with theme, decoder, scanner, and recent host options.

### 🐛 Fixed
- **Black Screen:** Fix black screen on connect by setting explicit MPEG-TS MIME type and using `OkHttpDataSource` with zero read-timeout.
- **Error Handling:** Surface ExoPlayer errors through `StreamEvent.Error` instead of silent black surface.

### 🔄 Updated
- **Build & Dependencies:** Add Compose BOM, `navigation-compose`, `material-icons-extended`, proguard rules for Media3/OkHttp/Compose/NSD, and opt-in lint rules.
- **Documentation:** Comprehensive documentation update across `README.md`, `docs/ARCHITECTURE.md`, `DBUS_SPEC.md`, `INSTALL.md`, `PACKAGING.md`, `TROUBLESHOOTING.md`, and `SECURITY.md`.

## [v0.9.0] - 2026-07-26

### ✨ Added
- **Artifact Signing:** Implement universal artifact signing for Android release APK (production keystore) and Linux packages (RPM, DEB, AppImage GPG keys).

### 📝 Documentation
- Document signing process in `SECURITY.md`.

## [v0.8.7] - 2026-07-26

### ⚡ Optimized
- **Native Player Pipeline:** Replace WebView/WebRTC path with native GStreamer + ExoPlayer.
- **Zero-Config Discovery:** Add mDNS auto-discovery on Android.
- **Touch Injection:** Stream touch events into Linux Wayland compositor via `uinput` and `evdev`.

### 🐛 Fixed
- **Portal Capture Fallback:** Drop failing EVDI dependency and use XDG Desktop Portal for capture.
- **Build & Formatting:** Resolve `cargo fmt`, `clippy`, and Android Kotlin compilation errors.

## [v0.7.4] - 2026-07-25

### ✨ Added
- **Uninstall Command:** Add `orbiscreen uninstall` command for clean removal.

### 🐛 Fixed
- **Web Client Assets:** Bundle web client files (`index.html`, `app.js`, `style.css`) in packages and resolve fallback paths across `~/.local/share`, `/usr/share`, `/app/share`.

## [v0.7.3] - 2026-07-25

### ⚡ CI & Packaging
- Inject dynamic packaging versions and generate SHA256 checksums for all release artifacts.

### 📝 Documentation
- Reformat `README.md`, `ARCHITECTURE.md`, `DBUS_SPEC.md`, `PACKAGING.md`, `TROUBLESHOOTING.md`.

## [v0.7.2] - 2026-07-24

### 🐛 Fixed
- **CPU Loop:** Rate-limit capture frame pump with `tokio::time::sleep` to prevent 100% CPU usage.

## [v0.7.1] - 2026-07-24

### 🐛 Fixed
- **Android Launch Crash:** Wrap `MainActivity` init in try/catch and fix malformed HTML in WebView fallback.
