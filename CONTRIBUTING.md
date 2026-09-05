# Contributing to Orbiscreen

Contributions to Orbiscreen are welcome. Please note that this project is released with a [Contributor Code of Conduct](CODE_OF_CONDUCT.md). By participating in this project you agree to abide by its terms.

## 🌿 Day-to-Day Work

- **Branches:** `feature/`, `fix/`, `docs/`, `chore/` prefixes (e.g. `feature/wayland-clipboard`), branched from `main`.
- **Commits:** `orbiscreen | <type>: <description>` for work, `orbiscreen | vX.Y.Z | <type>: <description>` for release commits. Types: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`.
- **Style:** All Rust code formatted with `cargo fmt --all` and clean under `cargo clippy --workspace --all-targets --locked -- -D warnings` (GTK excluded on hosts without the gtk4 system libraries). Kotlin code follows the existing Compose idioms. Every file header follows the Orbiscreen style:

```rust
// Orbiscreen - <module name> (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen
```

- **Toolchain:** to install Rust and all build dependencies on Fedora-, Debian-, or Arch-based distros: `./scripts/setup-dev-env.sh`
- **Pull requests:** from a fork, targeting `main`, with tests green, a `CHANGELOG.md` entry, and the PR template checklist filled in.

## 🚀 Release Process (maintainers)

A release is ONE commit containing the pending work plus the version bump:

1. Bump the version in **all version homes** so nothing can disagree with the tag:
   - `Cargo.toml` (`[workspace.package]`) followed by `cargo update -w` so `Cargo.lock` matches (the release workflow builds `--locked`)
   - `clients/android/app/build.gradle.kts`: `versionName` and `versionCode` (+1 every release; both are checked by the Play/F-Droid tooling)
   - the version badges in `README.md`, `README_AR.md` (`version-X.Y.Z`), and the badge lines in every doc: `docs/ARCHITECTURE.md`(+AR), `docs/DBUS_SPEC.md`(+AR), `docs/DE_SUPPORT.md`(+AR), `docs/PACKAGING.md`(+AR), `docs/TROUBLESHOOTING.md`(+AR), and `SECURITY.md`
   - the release-matrix line in `docs/PACKAGING.md`/`docs/PACKAGING_AR.md` and the scope heading in `SECURITY.md`
2. Rotate the pending `CHANGELOG.md` entry into a `## [vX.Y.Z] - <date>` block dated today (the release notes are extracted from this heading by the workflow).
3. Pick the bump by CHANGELOG convention: `✨ Added` work = minor (`0.14.0`), `🐛 Fixed`/`🎨 Changed`/`🧹 Cleanup` only = patch (`0.13.3`).
4. Verify locally, commit everything as one release commit, and tag it:

```bash
cargo fmt --all --check
cargo clippy --workspace --exclude orbiscreen-gtk --all-targets --locked -- -D warnings
cargo test --workspace --exclude orbiscreen-gtk --locked
cargo deny check

git add -A
git commit -m "orbiscreen | vX.Y.Z | <type>: one-line summary of the release"
git tag -a vX.Y.Z -m "vX.Y.Z - <one-line summary>"
```

Then push the result:

```bash
git push origin main vX.Y.Z
```

The tag push triggers the [release workflow](.github/workflows/release.yml), which gates on fmt/clippy/tests/cargo-deny, then builds, signs, and attaches the tarball, `.deb`, `.rpm`, AppImage, and Android APK (each with a SHA256 checksum) to the GitHub release.

---

<div align="center">

Built by <a href="https://github.com/shadow-x78">shadow-x78</a> ·
[Code of Conduct](CODE_OF_CONDUCT.md) ·
[Back to README](README.md)

<sub>&copy; 2026 Orbiscreen (shadow-x78)</sub>

</div>
