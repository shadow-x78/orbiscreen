#!/usr/bin/env bash
# ─────────────────────────────────────────────
# Orbiscreen - Version Consistency Check
# https://github.com/shadow-x78/orbiscreen
# ─────────────────────────────────────────────

# Verifies that every place the project records its version agrees with the Cargo
# workspace. The version is duplicated across ~25 files because each ecosystem needs it
# in its own format, and nothing enforced that they moved together until a release
# shipped a mismatched Android versionName.
#
# Usage: scripts/check-versions.sh
# Exits non-zero and prints one line per mismatch.

set -euo pipefail

cd "$(dirname "$0")/.."

expected="$(grep -m1 -oE '^version = "[^"]+"' Cargo.toml | sed 's/.*"\(.*\)"/\1/')"
if [ -z "$expected" ]; then
    echo "check-versions: cannot read the workspace version from Cargo.toml" >&2
    exit 1
fi

# Android versionCode must advance by exactly one per release; it is a single monotonic
# integer, not a semantic version.
expected_code=$(
    grep -oE 'versionCode = [0-9]+' clients/android/app/build.gradle.kts |
        head -1 | grep -oE '[0-9]+'
)

failures=0

# Each location states the version in its own syntax; the sed expression reduces the match
# to the bare version so they can be compared to the workspace value.
check() {
    local file="$1" pattern="$2" extract="$3"
    local found
    found="$(grep -m1 -oE "$pattern" "$file" 2>/dev/null | head -1 | sed -n "$extract" || true)"
    if [ -z "$found" ]; then
        printf 'check-versions: %-46s no version found\n' "$file" >&2
        failures=$((failures + 1))
    elif [ "$found" != "$expected" ]; then
        printf 'check-versions: %-46s %s (workspace says %s)\n' "$file" "$found" "$expected" >&2
        failures=$((failures + 1))
    fi
}

check "PKGBUILD" '^pkgver=[0-9.]+' 's/^pkgver=//p'
check "crates/orbiscreen-gui/tauri.conf.json" '"version": "[0-9.]+"' 's/.*"\([0-9.]*\)".*/\1/p'
check "data/orbiscreen-copr.spec" '^%global version [0-9.]+' 's/^%global version *//p'
check "clients/android/app/build.gradle.kts" 'versionName = "[0-9.]+"' 's/.*"\([0-9.]*\)".*/\1/p'

# Cargo.lock carries the version of every workspace member.
if ! grep -qE "^version = \"$expected\"$" Cargo.lock; then
    printf 'check-versions: %-46s does not contain version %s\n' "Cargo.lock" "$expected" >&2
    failures=$((failures + 1))
fi

# versionCode is monotonic, so only assert that it is present and numeric.
if [ -z "$expected_code" ]; then
    echo "check-versions: versionCode not found in build.gradle.kts" >&2
    failures=$((failures + 1))
fi

# Badges and the release matrix are documentation; a stale one is a real bug report.
for doc in README.md README_AR.md SECURITY.md CODE_OF_CONDUCT.md; do
    if grep -q 'img.shields.io/badge/version-' "$doc" &&
        ! grep -q "img.shields.io/badge/version-$expected-" "$doc"; then
        printf 'check-versions: %-46s badge is not %s\n' "$doc" "$expected" >&2
        failures=$((failures + 1))
    fi
done

for doc in docs/*.md; do
    if grep -q 'img.shields.io/badge/version-' "$doc" &&
        ! grep -q "img.shields.io/badge/version-$expected-" "$doc"; then
        printf 'check-versions: %-46s badge is not %s\n' "$doc" "$expected" >&2
        failures=$((failures + 1))
    fi
done

# The release matrix quotes the versionCode, so the two must not drift apart either.
for doc in docs/PACKAGING.md docs/PACKAGING_AR.md; do
    if grep -q 'versionCode' "$doc" && ! grep -q "$expected_code" "$doc"; then
        printf 'check-versions: %-46s quotes a versionCode other than %s\n' \
            "$doc" "$expected_code" >&2
        failures=$((failures + 1))
    fi
done

if [ "$failures" -ne 0 ]; then
    echo "check-versions: $failures mismatch(es) against workspace version $expected" >&2
    exit 1
fi

echo "check-versions: all version locations agree on $expected (Android versionCode $expected_code)"