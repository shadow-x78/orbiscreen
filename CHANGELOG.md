# Changelog

All entries follow the `orbiscreen | vX.Y.Z | <type>: <description>` commit convention.
Each release line is a release commit; bullets under it are the per-change commits that ship in it.

---

## orbiscreen | v0.10.1 | release: Android UI overhaul, black-screen fix, live discovery

```
orbiscreen | v0.10.1 | feat: migrate Android UI to Material 3 + Jetpack Compose
orbiscreen | v0.10.1 | feat: add Spacedesk-style live NSD discovery with manual host entry
orbiscreen | v0.10.1 | feat: add optional subnet scanner for networks without mDNS
orbiscreen | v0.10.1 | feat: persist and surface recent host with a Material chip
orbiscreen | v0.10.1 | fix: resolve black screen by setting explicit MPEG-TS MIME type
orbiscreen | v0.10.1 | fix: stream via OkHttpDataSource with zero read-timeout for live feeds
orbiscreen | v0.10.1 | fix: surface ExoPlayer errors through StreamEvent.Error instead of silent black
orbiscreen | v0.10.1 | refactor: replace delta-based TouchInjector with absolute InputDispatcher
orbiscreen | v0.10.1 | feat: support multi-touch, wheel, stylus and keyboard events
orbiscreen | v0.10.1 | feat: add host control toolbar (lock, blank, ctrl-alt-del, files, retry)
orbiscreen | v0.10.1 | feat: add soft keyboard overlay with system IME handoff
orbiscreen | v0.10.1 | feat: query host /api/info to map Android touches to host resolution
orbiscreen | v0.10.1 | feat: add settings screen with theme, decoder, scanner, recent host
orbiscreen | v0.10.1 | chore: bump Android versionCode to 9 and versionName to 0.10.1
orbiscreen | v0.10.1 | chore: bump workspace Cargo.toml version to 0.10.1
orbiscreen | v0.10.1 | build: add Compose BOM, navigation-compose, material-icons-extended
orbiscreen | v0.10.1 | build: add proguard rules for Media3, OkHttp, Compose and NSD
orbiscreen | v0.10.1 | build: add lint.xml opting in to androidx.media3 UnstableApi
orbiscreen | v0.10.1 | docs: rewrite README for Material 3 UI and v0.10.1 highlights
orbiscreen | v0.10.1 | docs: update Android architecture diagram with new package layout
```

---

## orbiscreen | v0.9.0 | chore: remove old changelog history and clarify package signing

```
orbiscreen | v0.9.0 | chore: remove old changelog history and clarify package signing
orbiscreen | v0.9.0 | feat: implement universal artifact signing
orbiscreen | v0.9.0 | feat: sign Android release APK with production keystore
orbiscreen | v0.9.0 | feat: sign RPM, DEB and AppImage with GPG keys
orbiscreen | v0.9.0 | docs: document the signing process in SECURITY.md
```

---

## orbiscreen | v0.8.7 | refactor: native player + mDNS + touch injection

```
orbiscreen | v0.8.7 | refactor: replace WebView/WebRTC path with native GStreamer + ExoPlayer
orbiscreen | v0.8.7 | feat: add mDNS zero-config discovery on Android
orbiscreen | v0.8.7 | feat: stream touch events into the Linux Wayland compositor via uinput and evdev
orbiscreen | v0.8.7 | fix: drop the failing EVDI dependency and use the XDG Desktop Portal for capture
orbiscreen | v0.8.7 | ci: resolve cargo fmt, clippy and Cargo.lock drift
orbiscreen | v0.8.7 | docs: update ARCHITECTURE.md for the native pipeline
orbiscreen | v0.8.7 | style: cargo fmt
orbiscreen | v0.8.7 | fix: resolve Android compilation error after Kotlin upgrade
```

---

## orbiscreen | v0.7.4 | fix: complete uninstall + missing web client

```
orbiscreen | v0.7.4 | feat: add orbiscreen uninstall command for clean removal
orbiscreen | v0.7.4 | fix: ship the web client files (index.html, app.js, style.css) in every package
orbiscreen | v0.7.4 | fix: daemon resolves web client across ~/.local/share, /usr/share, /app/share
```

---

## orbiscreen | v0.7.3 | docs: full reformatting

```
orbiscreen | v0.7.3 | ci: inject dynamic packaging versions into release.yml
orbiscreen | v0.7.3 | ci: generate sha256 checksums for every artifact
orbiscreen | v0.7.3 | fix: correct Flatpak ID and Debian version injection
orbiscreen | v0.7.3 | docs: reformat README, ARCHITECTURE, DBUS_SPEC, PACKAGING, TROUBLESHOOTING
```

---

## orbiscreen | v0.7.2 | fix: daemon CPU loop + uninstall script

```
orbiscreen | v0.7.2 | fix: rate-limit the capture frame pump with tokio::time::sleep
orbiscreen | v0.7.2 | chore: add scripts/uninstall.sh and install.sh uninstall path
```

---

## orbiscreen | v0.7.1 | fix: Android crash + SDK target

```
orbiscreen | v0.7.1 | fix: wrap MainActivity init in try/catch and remove stray --> from index.html
orbiscreen | v0.7.1 | chore: bump Android targetSdk to 35 and enable V1/V2/V3 signing
orbiscreen | v0.7.1 | chore: allow cleartext traffic for local networks only in network_security_config.xml
```
