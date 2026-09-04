#!/usr/bin/env bash
# ─────────────────────────────────────────────
# Orbiscreen - Packit Version Extractor
# https://github.com/shadow-x78/orbiscreen
# ─────────────────────────────────────────────

set -euo pipefail
grep -m1 -oP '(?<=^version = ")[^"]+' Cargo.toml
