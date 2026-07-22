---
name: Pull request
about: Submit changes against the main branch
title: ""
labels: []
assignees: []
---

### What does this PR do?

### Why?

Reference the phase (`Phase N`) or `CHANGELOG.md` entry this PR
belongs to.

### How was it tested?

- [ ] `cargo fmt --all`
- [ ] `cargo clippy --workspace --all-targets --locked -- -D warnings`
- [ ] `cargo test --workspace`
- [ ] Manual smoke test on: <!-- distro + compositor -->

### Checklist

- [ ] No new warnings introduced.
- [ ] File headers match UMO style (`// Orbiscreen — <module> (GPL-3.0-or-later)` + GitHub URL).
- [ ] `CHANGELOG.md` updated.