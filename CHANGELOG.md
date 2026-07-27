# Changelog

The headline lines below are release commits. Each bullet under them is the
per-change commit that ships in the release, written in the project's
`orbiscreen | <scope>: <description>` style.

---

## orbiscreen | v0.10.1 | release: build fixes for Android Material 3 client

```
orbiscreen | android | build: remove debug.keystore fallback from release signingConfig
orbiscreen | android | build: fail the release build if orbiscreen-release.keystore is missing
orbiscreen | android | build: refresh Cargo.lock for Android 0.10.0 workspace
orbiscreen | docs  | changelog: split v0.10.0 and v0.10.1 release notes
```

---

## orbiscreen | v0.10.0 | release: Android Material 3 UI overhaul, black-screen fix, live discovery

```
orbiscreen | android | ui: migrate to Material 3 + Jetpack Compose
orbiscreen | android | discovery: add Spacedesk-style live NSD scan with manual host entry
orbiscreen | android | discovery: add optional subnet scanner for networks without mDNS
orbiscreen | android | discovery: persist and surface recent host with a Material chip
orbiscreen | android | player: fix black screen by setting explicit MPEG-TS MIME type
orbiscreen | android | player: stream via OkHttpDataSource with zero read-timeout for live feeds
orbiscreen | android | player: surface ExoPlayer errors through StreamEvent.Error instead of silent black
orbiscreen | android | input: replace delta-based TouchInjector with absolute InputDispatcher
orbiscreen | android | input: support multi-touch, wheel, stylus and keyboard events
orbiscreen | android | stream: add host control toolbar (lock, blank, ctrl-alt-del, files, retry)
orbiscreen | android | stream: add soft keyboard overlay with system IME handoff
orbiscreen | android | stream: query host /api/info to map Android touches to host resolution
orbiscreen | android | settings: add settings screen with theme, decoder, scanner, recent host
orbiscreen | android | build: bump Android versionCode to 9 and versionName to 0.10.0
orbiscreen | build: bump workspace Cargo.toml version to 0.10.0
orbiscreen | android | build: add Compose BOM, navigation-compose, material-icons-extended
orbiscreen | android | build: add proguard rules for Media3, OkHttp, Compose and NSD
orbiscreen | android | build: add lint.xml opting in to androidx.media3 UnstableApi
orbiscreen | docs: rewrite README for Material 3 UI and v0.10.0 highlights
orbiscreen | docs: update Android architecture diagram with new package layout
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
