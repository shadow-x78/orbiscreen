# Contributing to Orbiscreen

We welcome contributions to Orbiscreen! This document outlines our development guidelines.

## 🌿 Branch Naming

Use the following prefixes for your branches:
- `feature/` - For new features
- `fix/` - For bug fixes
- `docs/` - For documentation changes
- `chore/` - For maintenance tasks

Example: `feature/wayland-clipboard`

## 💬 Commit Convention

We enforce a strict commit message format for clarity:

```text
orbiscreen | <type>: <description>
orbiscreen | vX.Y.Z | <type>: <description>
```

- `<type>` can be `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`.
- `vX.Y.Z` should be the current workspace version (use on release commits).
- A `release:` type denotes the bump commit (e.g. `orbiscreen | v0.10.3 | release: bump to 0.10.2`).

Example: `orbiscreen | v0.10.3 | fix: respect display aspect ratio on rotation`

## 💅 Code Style

All Rust code must be formatted and pass Clippy without warnings.
- Format: `cargo fmt --all`
- Lint: `cargo clippy --workspace --all-targets --locked -- -D warnings`

All file headers must match the Orbiscreen style:
```rust
// Orbiscreen - <module name> (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen
```

## ✅ Pull Requests

1. Run `cargo test --workspace` to ensure all tests pass.
2. Follow the checklist provided in the PR template (`.github/PULL_REQUEST_TEMPLATE.md`).
3. Mention the `CHANGELOG.md` entry or the feature phase this PR belongs to.

---

<div align="center">

Built by <a href="https://github.com/shadow-x78">shadow-x78</a> ·
[Back to README](README.md)

<sub>&copy; 2026 Orbiscreen (shadow-x78)</sub>

</div>
