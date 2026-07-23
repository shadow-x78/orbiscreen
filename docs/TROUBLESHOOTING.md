# Troubleshooting - Orbiscreen

---

## 🌐 Language

<a href="TROUBLESHOOTING.md">🇬🇧 English</a> · <a href="TROUBLESHOOTING_AR.md">🇸🇦 العربية</a>

---

## 📋 Table of Contents

### CI Workflow Actions (`.github/workflows/ci.yml`)

- [Action: `Check formatting` (`cargo fmt --all -- --check`)](#ci-fmt)
- [Action: `Clippy (deny warnings)` (`cargo clippy --workspace --all-targets --locked -- -D warnings`)](#ci-clippy)
- [Action: `Build` (`cargo build --workspace --locked`)](#ci-build)
- [Action: `Test` (`cargo test --workspace --locked`)](#ci-test)
- [Action: `Run cargo-deny` (`cargo deny check`)](#ci-deny)

### Runtime

- [Runtime: `orbiscreen start` fails - `kernel module is not installed`](#runtime-evdi)
- [Runtime: capture backend unavailable on Wayland](#runtime-wayland)
- [Runtime: `unsafe_op_in_unsafe_fn` / `missing_debug_implementations` lint warnings](#runtime-lints)

### Still Stuck?

- [Build still failing? Check the action logs](#still-stuck)
- [Re-run a single CI job](#re-run-job)

---

<a id="ci-fmt"></a>
## 🧪 CI Action: `Check formatting` (`cargo fmt --all -- --check`)

**Symptom:**
```
Diff in /path/to/file.rs:
   println!("x");
-  println!("y");
+  println!("z");
```

**Cause:**
Rust source files don't match `cargo fmt`'s formatting.

**Fix:**
```bash
cargo fmt --all
git add -A
git commit -m "orbiscreen | v0.1.1 | style: cargo fmt --all"
```

**Why this happens:**
- Added a file without running `cargo fmt` before committing.
- Edited `rustfmt.toml` to change formatting rules (e.g. `max_width`).

**Prevention:**
The `.github/workflows/ci.yml` runs `cargo fmt --all -- --check` on every
PR. Run `cargo fmt --all` locally before pushing.

---

<a id="ci-clippy"></a>
## 🧪 CI Action: `Clippy (deny warnings)` (`cargo clippy --workspace --all-targets --locked -- -D warnings`)

**Symptom:**
```
error: this operation is not supported for derived errors
  --> src/lib.rs:42:5
   |
42 |     let result = self.op().expect("...");   // unwrap in production code
   |                              ^^^^^^^^
   |
   = note: `-D warnings` implied by `-D warnings`
```

**Cause:**
`cargo clippy -D warnings` treats every clippy warning as an error. The
most common culprits are:

1. **`let _ = unit_value`** - assigning `()` to a discarded binding.
   Fix: `let _ = expr;` becomes just `expr;`, or use `drop()`.
2. **`unwrap()`/`expect()` in production code** (allowed only in
   `#[cfg(test)]` modules per `CONTRIBUTING`).
3. **Unused imports / variables / fields** - fix or prefix with `_`.
4. **Manual `#[derive]` impl** where Default is derivable - use
   `#[derive(Default)]`.
5. **Code comment quality** - sometimes a clipped clippy lint.
6. **`unsafe_op_in_unsafe_fn`** - when working with `unsafe` blocks, add
   `#[allow(unsafe_code)]` to the inner function.

**Fix:**
```bash
cargo clippy --workspace --all-targets --locked -- -D warnings 2>&1 | head -50
# Apply the suggestions, then:
cargo clippy --workspace --all-targets --locked --fix
# Review the changes and commit
```

**Prevention:**
Run `cargo clippy` locally before pushing.

---

<a id="ci-build"></a>
## 🧪 CI Action: `Build` (`cargo build --workspace --locked`)

**Symptom:**
```
error[E0463]: can't find crate for `webrtc`
  |
  = note: the crate `webrtc` couldn't be found in the registry index
```

**Cause:**
`Cargo.lock` is pinned to versions that are not on `crates.io` anymore.
Or the toolchain pinned in `rust-toolchain.toml` differs from the
runner. With `--locked` enabled, cargo refuses to update the lockfile.

**Fix:**
```bash
# Locally: regenerate the lockfile
cargo update -p webrtc
# Then rebuild
cargo build --workspace --locked
git add Cargo.lock
git commit -m "orbiscreen | v0.1.1 | chore: refresh Cargo.lock for webrtc 0.20.0-rc.3"
```

If the dependency is `webrtc = "0.20.0-rc.3"` and has been removed or
replaced on `crates.io`, you must either:
- Pin a different version (`cargo search webrtc`)
- Wait for a stable release (the workspace pins a release-candidate)

**Prevention:**
Keep `Cargo.lock` committed (already done) and run `cargo update` only
intentionally.

---

<a id="ci-test"></a>
## 🧪 CI Action: `Test` (`cargo test --workspace --locked`)

**Symptom:**
```
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

---- tests::test_encode_default stdout ----
thread 'tests::test_encode_default' panicked at 'called `Result::unwrap()` on an `Err` value: EncodeError("gstreamer pipeline error: appsrc: ...")', src/encode.rs:125:5
```

**Cause:**
Tests assume the host has certain GStreamer plugins installed
(`x264enc`, `vaapih264enc`, `nvh264enc`). On CI these aren't always
present, so the encoder init fails.

**Fix:**
Either install the plugins on CI (the workflow installs
`gstreamer1.0-plugins-good gstreamer1.0-plugins-bad` already - add
`gstreamer1.0-libav gstreamer1.0-plugins-ugly` if x264 is needed), or
make the failing test conditional:

```rust
#[test]
fn frame_duration_ns_matches_framerate() {
    let params = EncodeParams {
        framerate: 60,
        ..EncodeParams::default()
    };
    let mut encoder = Encoder::new(params).ok();
    // Skip if no GStreamer H.264 plugin is installed on the test host.
    if let Some(encoder) = encoder.as_mut() {
        assert_eq!(encoder.frame_duration_ns(), 16_666_666);
    }
}
```

**Prevention:**
Keep CI dependencies up to date. For H.264 encoding on CI,
`gstreamer1.0-plugins-ugly` (which contains `x264enc`) is mandatory.

---

<a id="ci-deny"></a>
## 🧪 CI Action: `Run cargo-deny` (`cargo deny check`)

**Symptom:**
```
advisories FAILED, bans FAILED, licenses ok, sources ok
##[error]Process completed with exit code 3.
```

**Cause:**
The license check passes (GPL-3.0-or-later allowlist includes
`libevdi`'s LGPL-2.1). The bans check fails when `wildcards = "deny"`
flags a path with `*`. The advisories check fails on unmaintained
transitive deps (notably `derivative` via `evdi`).

**Fix:**
For the current setup:
- This is **informational only** - `cargo-deny` is NOT a required
  status check in `ci.yml`, so a deny failure cannot block a PR merge.

To make `cargo-deny` pass cleanly:
1. Update `deny.toml`'s `[advisories]` section to ignore the
   unmaintained `derivative` advisory.
2. Ensure all path globs are resolved before committing.

---

<a id="runtime-evdi"></a>
## 🚀 Runtime: `orbiscreen start` fails - `kernel module is not installed`

**Symptom:**
```
Error: evdi kernel module is not installed
```

**Cause:**
The `evdi` kernel module from DisplayLink has not been loaded on the
host. The daemon needs it to expose a virtual display through DRM/KMS.

**Fix:**
1. Install `evdi` (DKMS build) on the host:
   ```bash
   # Fedora / Nobara
   sudo dnf install dkms gcc make kernel-devel-$(uname -r) displaylink
   sudo modprobe evdi
   ```
   ```bash
   # Ubuntu / Pop!_OS
   sudo apt install dkms
   git clone https://github.com/DisplayLink/evdi.git
   cd evdi && sudo make dkms-install
   sudo modprobe evdi
   ```
2. Verify it loaded:
   ```bash
   lsmod | grep evdi   # should show evdi module
   ls /dev/dri/card*  # should show at least one card
   ```

The full installation walk-through is in `scripts/setup-dev-env.sh`
and `scripts/test-evdi.sh`.

---

<a id="runtime-wayland"></a>
## 🚀 Runtime: capture backend unavailable on Wayland

**Symptom:**
```
Error: capture backend unavailable: Wayland capture requires open_async
```

**Cause:**
The synchronous `CaptureSession::open()` cannot drive the Wayland
ScreenCast portal. On a Wayland session, you must use the async path.

**Fix:**
Use `CaptureSession::open_async()` instead of `open()`:
```rust
// ❌ Wrong on Wayland
let session = CaptureSession::open(width, height)?;

// ✅ Right
let session = CaptureSession::open_async(width, height).await?;
```

The daemon binary (`orbiscreen-daemon`) already uses `open_async`
internally, so this only matters if you call the library directly.

---

<a id="runtime-lints"></a>
## 🚀 Runtime: `unsafe_op_in_unsafe_fn` / `missing_debug_implementations` lint warnings

**Symptom:**
```
warning: unnecessary `unsafe` block
  --> crates/orbiscreen-display/src/lib.rs:142:9
warning: type `VirtualDisplay` does not implement `Debug`
```

**Cause:**
The workspace has these clippy lints enabled at warn level:
- `unsafe_code = "warn"`
- `missing_debug_implementations = "warn"`

**Fix:**
For types that hold non-`Debug` handles (like `evdi::Handle` or
`GStreamer` pipeline), suppress the lint explicitly:
```rust
#[allow(missing_debug_implementations)]
pub struct VirtualDisplay { ... }
```

For `unsafe_op_in_unsafe_fn`:
```rust
#[allow(unsafe_code)]
pub async fn open_at(...) -> ... {
    let unconnected = unsafe { node.open() }?;
    ...
}
```

**Why these lints exist:**
- `missing_debug_implementations` forces explicit handling of opaque
  types so you can't accidentally print internal pointers in logs.
- `unsafe_code = warn` catches stray unsafe blocks outside the few
  deliberately-marked call sites.

---

## 📱 Android Client & Deployment

### Where is the built Android APK file (`app-debug.apk`)?

- **Locally on your system:**
  When you run `./gradlew assembleDebug` inside `clients/android`, the generated APK file is stored at:
  ```text
  clients/android/app/build/outputs/apk/debug/app-debug.apk
  ```

- **On GitHub Actions / Releases:**
  When the `Android build` workflow runs on GitHub Actions, the APK is published under **Artifacts** at the bottom of the workflow run page as **`orbiscreen-android-debug`**.

---

### EVDI Kernel Module Missing / Automatic Wayland Desktop Portal Fallback

**Symptom:**
`orbiscreen probe` reports `display backend: kernel module missing`.

**Explanation:**
Orbiscreen natively supports EVDI kernel driver for dedicated DRM virtual display connectors. However, if the EVDI module is not loaded on your Linux system (e.g. running stock Arch, Fedora, or Ubuntu without EVDI), `orbiscreen-daemon` automatically falls back to Wayland/X11 ScreenCast portal (`xdg-desktop-portal`), allowing instant streaming out-of-the-box on GNOME, KDE, and Sway.

---

### USB Connection & ADB Reverse Port Forwarding

**Symptom:**
Android app displays `Looking for host...` when connected over USB cable.

**Fix:**
Orbiscreen automatically configures `adb reverse tcp:8788 tcp:8788` when started. Ensure:
1. **USB Debugging** is enabled in Android Developer Options.
2. The host device is authorized on your Android phone/tablet prompt.
3. Verify manually using:
   ```bash
   adb devices
   adb reverse tcp:8788 tcp:8788
   ```

---

<a id="still-stuck"></a>
## 🛟 Still Stuck?

<a id="re-run-job"></a>
### Re-run a single CI job

On the failed PR page:
1. Open the **Checks** section.
2. Click the failed check name (e.g. `Build, test & lint (workspace)`).
3. Click **Re-run jobs** → **Re-run failed jobs**.

### Check the action logs

For each failed job, the **Run logs** section shows the exact `cargo`
output. Cross-reference with the sections above.

### Open an issue

If none of the above applies, open an issue using
`.github/ISSUE_TEMPLATE/bug.yml`. Include:
- The exact `cargo` error output
- The CI run URL
- The OS / compositor of the host (if runtime-related)

---

<div align="center">

Built by <a href="https://github.com/shadow-x78">shadow-x78</a> ·
[Back to README](../README.md)

<sub>&copy; 2026 Orbiscreen (shadow-x78)</sub>

</div>