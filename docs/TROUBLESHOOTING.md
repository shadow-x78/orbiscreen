<div align="center">

# Troubleshooting — Orbiscreen

[![Version](https://img.shields.io/badge/version-0.1.0-2563eb?style=flat-square&logo=semver)](../CHANGELOG.md)
[![License](https://img.shields.io/badge/license-GPL--3.0-dc2626?style=flat-square)](../LICENSE)
![Language](https://img.shields.io/badge/rust-edition_2021-16a34a?style=flat-square&logo=rust)
![Platform](https://img.shields.io/badge/platform-Linux-9333ea?style=flat-square&logo=linux)

</div>

---

## 🌐 Language

<a href="TROUBLESHOOTING.md">🇬🇧 English</a> · <a href="TROUBLESHOOTING_AR.md">🇸🇦 العربية</a>

---

## 📋 Table of Contents

### GitHub API / Repo Setup

- [Apply branch protection — `"enabled"` rejected as not boolean](#gh-api-enabled)
- [Apply branch protection — `restrictions` rejected on personal repo](#gh-api-restrictions)
- [Apply branch protection — `"restrictions" wasn't supplied`](#gh-api-restrictions-missing)
- [Apply branch protection — `cargo-deny` fails on transitive `derivative`](#gh-api-deny)
- [PR merge blocked — `enforce_admins` and self-approval conflict](#gh-api-self-approval)
- [Push rejected — `protected branch update failed`](#gh-api-push-rejected)

### CI Workflow Actions (`.github/workflows/ci.yml`)

- [Action: `Check formatting` (`cargo fmt --all -- --check`)](#ci-fmt)
- [Action: `Clippy (deny warnings)` (`cargo clippy --workspace --all-targets --locked -- -D warnings`)](#ci-clippy)
- [Action: `Build` (`cargo build --workspace --locked`)](#ci-build)
- [Action: `Test` (`cargo test --workspace --locked`)](#ci-test)
- [Action: `Run cargo-deny` (`cargo deny check`)](#ci-deny)

### Runtime

- [Runtime: `orbiscreen start` fails — `kernel module is not installed`](#runtime-evdi)
- [Runtime: capture backend unavailable on Wayland](#runtime-wayland)
- [Runtime: `unsafe_op_in_unsafe_fn` / `missing_debug_implementations` lint warnings](#runtime-lints)

### Still Stuck?

- [Build still failing? Check the action logs](#still-stuck)
- [Re-run a single CI job](#re-run-job)

---

<a id="gh-api-enabled"></a>
## 🔧 `Apply branch protection` — `"enabled"` rejected as not boolean

**Symptom:**
```json
{
  "message": "Invalid request.",
  "errors": ["For 'allOf/0', {\"enabled\" => true} is not a boolean."],
  "status": 422
}
```

**Cause:**
The top-level toggles (`enforce_admins`, `required_linear_history`,
`allow_force_pushes`, etc.) were wrapped in `{ "enabled": true }`
objects. The schema accepts **boolean literals** at the top level.

**Fix:**
Flatten each toggle:
```jsonc
// ❌ Wrong
{ "enforce_admins": { "enabled": true } }

// ✅ Right
{ "enforce_admins": true }
```

**Apply:**
```bash
gh api -X PUT /repos/shadow-x78/orbiscreen/branches/main/protection \
    --input admin/branch-protection.json
```

---

<a id="gh-api-restrictions"></a>
## 🔧 `Apply branch protection` — `restrictions` rejected on personal repo

**Symptom:**
```json
{
  "message": "Validation Failed",
  "errors": ["Only organization repositories can have users and team restrictions"],
  "status": 422
}
```

**Cause:**
On **personal** repositories, GitHub rejects any value with arrays:
```json
{ "restrictions": { "users": [], "teams": [], "apps": [] } }   // ❌ rejected
```

**Fix:**
For personal repos, use `"restrictions": null`. The field is present
(satisfying the schema) but its value is null (GitHub silently drops
push restrictions — which is exactly what a personal repo needs).

For **organization** repos, populate `users[]` / `teams[]` / `apps[]`.

---

<a id="gh-api-restrictions-missing"></a>
## 🔧 `Apply branch protection` — `"restrictions" wasn't supplied`

**Symptom:**
```json
{ "message": "Invalid request.", "errors": ["\"restrictions\" wasn't supplied."], "status": 422 }
```

**Cause:**
You omitted the `restrictions` key entirely. The schema requires it to
be present (even on personal repos where its value must be `null`).

**Fix:**
Add the key with a `null` value — do not omit it:
```jsonc
// ❌ Wrong — field missing entirely
{ "enforce_admins": true, "required_linear_history": true, ... }

// ✅ Right — field present, value null
{ "enforce_admins": true, "restrictions": null, "required_linear_history": true, ... }
```

---

<a id="gh-api-deny"></a>
## 🔧 `Apply branch protection` — `cargo-deny` fails on transitive `derivative`

**Symptom:**
```
error[unmaintained]: `derivative` is unmaintained; consider using an alternative
  ├─ ID: RUSTSEC-2024-0388
  ├─ derivative v2.2.0
  │   └── evdi v0.8.0
  │       └── orbiscreen-display v0.1.0
advisories FAILED, bans FAILED, licenses ok, sources ok
##[error]Process completed with exit code 3.
```

**Cause:**
`cargo-deny` flags `derivative v2.2.0` as unmaintained. This crate
arrives **transitively** through `evdi v0.8.0`, the kernel-module
binding we use to create virtual displays.

**Fix:**
The branch protection policy only requires the `workspace` check, so
`cargo-deny` does not block merges. Two ways forward:

1. **Leave it informational** — `cargo-deny` continues to surface
   unmaintained transitive deps in CI logs but does not block merges.
   This is the current setup.

2. **Drop `cargo-deny` from the workflow** if its noise becomes a
   problem — remove the `licenses` job from
   `.github/workflows/ci.yml`.

3. **Skip the advisory** in `deny.toml` (last resort — weakens the
   check):
   ```toml
   [advisories]
   ignore = ["RUSTSEC-2024-0388"]
   ```

---

<a id="gh-api-self-approval"></a>
## 🔧 `PR merge` — `enforce_admins` and self-approval conflict

**Symptom:**
```
GraphQL: At least 1 approving review is required by reviewers with write access.
GraphQL: Review Can not approve your own pull request
```

**Cause:**
When `enforce_admins: true` AND `required_approving_review_count: 1`
are both on, **the only reviewer cannot be the author of the PR**.
Solo-developer repos hit this immediately.

**Fix:**
The standard solo-developer workflow is a temporary relax-and-restore:

```bash
# 1. Apply the strict policy (default)
gh api -X PUT /repos/<owner>/orbiscreen/branches/main/protection \
    --input policy.json

# 2. To merge your own PR: temporarily relax the two blocking rules
gh api -X PATCH /repos/<owner>/orbiscreen/branches/main/protection/enforce_admins \
    -f enabled=false
gh api -X PATCH /repos/<owner>/orbiscreen/branches/main/protection/required_pull_request_reviews \
    -f required_approving_review_count=0

# 3. Merge the PR
gh pr merge <N> --squash --admin --delete-branch

# 4. Restore the strict policy
gh api -X PUT /repos/<owner>/orbiscreen/branches/main/protection \
    --input policy.json
```

Once you have a second GitHub account to act as a reviewer, this dance
is no longer needed.

---

<a id="gh-api-push-rejected"></a>
## 🔧 `Push` — `protected branch hook declined`

**Symptom:**
```
remote: error: GH006: Protected branch update failed for refs/heads/main.
remote:
remote: - Changes must be made through a pull request.
remote: - Required status check "workspace" is expected.
```

**Cause:**
Branch protection is active on `main` and requires:
1. Changes come via a pull request (not direct push).
2. The `workspace` status check must pass.

**Fix:**
Use the standard feature-branch + PR flow:
```bash
git checkout -b chore/your-change
git commit -am "orbiscreen | v0.1.0 | chore: your change"
git push -u origin chore/your-change
gh pr create --base main --head chore/your-change
# Wait for the workspace CI to pass
gh pr merge <N> --squash --delete-branch
```

If you need to bypass temporarily, use the relax-and-restore dance from
the previous section.

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
git commit -m "orbiscreen | v0.1.0 | style: cargo fmt --all"
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

1. **`let _ = unit_value`** — assigning `()` to a discarded binding.
   Fix: `let _ = expr;` becomes just `expr;`, or use `drop()`.
2. **`unwrap()`/`expect()` in production code** (allowed only in
   `#[cfg(test)]` modules per `CONTRIBUTING`).
3. **Unused imports / variables / fields** — fix or prefix with `_`.
4. **Manual `#[derive]` impl** where Default is derivable — use
   `#[derive(Default)]`.
5. **Code comment quality** — sometimes a clipped clippy lint.
6. **`unsafe_op_in_unsafe_fn`** — when working with `unsafe` blocks, add
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
git commit -m "orbiscreen | v0.1.0 | chore: refresh Cargo.lock for webrtc 0.20.0-rc.3"
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
`gstreamer1.0-plugins-good gstreamer1.0-plugins-bad` already — add
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
- This is **informational only** — `cargo-deny` is NOT in the
  required status checks (`"contexts": ["workspace"]`).
- See [GH API: cargo-deny fails](#gh-api-deny) above.

To make `cargo-deny` pass cleanly:
1. Update `deny.toml`'s `[advisories]` section to ignore the
   unmaintained `derivative` advisory.
2. Ensure all path globs are resolved before committing.

---

<a id="runtime-evdi"></a>
## 🚀 Runtime: `orbiscreen start` fails — `kernel module is not installed`

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