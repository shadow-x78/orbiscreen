#!/usr/bin/env bash
set -euo pipefail
grep -m1 -oP '(?<=^version = ")[^"]+' Cargo.toml
