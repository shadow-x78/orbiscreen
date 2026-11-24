#!/usr/bin/env bash
# ─────────────────────────────────────────────
# Orbiscreen - Version Synchronization (per-commit)
# https://github.com/shadow-x78/orbiscreen
# ─────────────────────────────────────────────

# Rewrites the project version in every location that carries it and prepends an entry to
# each append-only log, so that a commit's tree agrees with the version in its own header.
# The append-only logs (CHANGELOG.md, debian/changelog) are only ever prepended to.
#
# Usage: set-version.sh <version> <android-version-code> <summary>

set -euo pipefail
cd "${REPO_ROOT:-$PWD}"

version="$1"
code="$2"
summary="$3"

previous="$(grep -m1 -oE '^version = "[^"]+"' Cargo.toml | sed 's/.*"\(.*\)"/\1/')"
previous_code="$(grep -oE 'versionCode = [0-9]+' clients/android/app/build.gradle.kts | grep -oE '[0-9]+')"

if [ "$previous" = "$version" ]; then
    echo "set-version: already at $version"
    exit 0
fi

# CHANGELOG.md and debian/changelog accumulate history and are only ever prepended to.
# Cargo.lock is excluded from the blanket replace because its sha256 checksums can contain
# the version as a substring; it gets a targeted update at the end of this script.
append_only="CHANGELOG.md debian/changelog Cargo.lock"

files=$(git ls-files | grep -vE '\.(png|jpg|ico|jar|ttf|svg|zip)$')
for file in $files; do
    case " $append_only " in
        *" $file "*) continue ;;
    esac
    if [ "$file" = "data/orbiscreen-copr.spec" ]; then
        # The spec states its version as a literal until commit 7 turns it into a macro.
        if grep -q '^%global version ' "$file"; then
            sed -i -E "s|^%global version .*|%global version ${version}|" "$file"
        else
            sed -i -E "s|^Version: +[0-9.]+|Version:        ${version}|" "$file"
        fi
        continue
    fi
    # The dots are escaped so the version only matches itself: an unescaped "0.43.0"
    # also matches "0x4320" and rewrites the 7680x4320 resolution in the README.
    if grep -qF "$previous" "$file" 2>/dev/null; then
        sed -i "s/${previous//./\\.}/${version}/g" "$file"
    fi
done

# Android versionCode is an independent monotonic integer.
for file in $files; do
    case " $append_only " in
        *" $file "*) continue ;;
    esac
    grep -qF "$previous_code" "$file" 2>/dev/null || continue
    sed -i "s/versionCode = ${previous_code}/versionCode = ${code}/g; s/\`${previous_code}\`/\`${code}\`/g" "$file"
done

# The release date is the day the version is cut, not a fixed stamp: a frozen
# date puts every release in the changelog on the same day.
stamp="$(date -u +%Y-%m-%d)"
author_line="-- shadow-x7 <shadow-x7@red-shadows-rs.com>  $(date -R)"

python3 - "$version" "$code" "$summary" "$stamp" <<'PY'
import re, sys
version, code, summary, stamp = sys.argv[1:5]
text = open("CHANGELOG.md", encoding="utf-8").read()
entry = (
    f"## [v{version}] - {stamp}\n\n"
    f"### Changed\n"
    f"- {summary}\n"
    f"- Version bumped to {version} across the Cargo workspace, Android "
    f"(`versionCode` {code}), Tauri, PKGBUILD, Debian, COPR, and documentation badges.\n\n"
)
m = re.search(r"^## \[v[0-9]", text, re.M)
if m:
    text = text[: m.start()] + entry + text[m.start():]
else:
    text = text.rstrip("\n") + "\n\n" + entry
open("CHANGELOG.md", "w", encoding="utf-8").write(text)
PY

{
    printf 'orbiscreen (%s~ubuntunoble1) noble; urgency=medium\n\n' "$version"
    printf '  * Release %s for Ubuntu noble.\n' "$version"
    printf '  * %s\n' "$summary"
    printf '  * Bump version to %s across all packages; Android versionCode %s.\n\n' "$version" "$code"
    printf '%s\n\n' "$author_line"
    cat debian/changelog
} > /tmp/kilo/debian.new
mv /tmp/kilo/debian.new debian/changelog

if [ -f data/orbiscreen-copr.spec ]; then
    today="$(LC_ALL=C date -u '+%a %b %d %Y')" python3 - "$version" "$summary" <<'PY'
import os, re, sys
version, summary = sys.argv[1:3]
today = os.environ["today"]
path = "data/orbiscreen-copr.spec"
text = open(path, encoding="utf-8").read()
head = (
    f"* {today} shadow-x7 <shadow-x7@red-shadows-rs.com>"
    f" - {version}-1\n- Release {version}: {summary}\n\n"
)
text = re.sub(r"^(%changelog\n)", lambda m: m.group(1) + head, text, count=1, flags=re.M)
open(path, "w", encoding="utf-8").write(text)
PY
fi

# Cargo.lock carries the workspace member versions. Only the version line of a
# package whose name starts with orbiscreen may change: a blanket replace corrupts the
# sha256 checksums, whose hex digits can contain the version as a substring.
python3 - "$previous" "$version" <<'PYLOCK'
import re, sys
previous, version = sys.argv[1], sys.argv[2]
text = open("Cargo.lock", encoding="utf-8").read()
def _flush(block):
    name = next((l for l in block if l.startswith("name = ")), "")
    if not name.startswith('name = "orbiscreen'):
        return block
    result = []
    for line in block:
        if line.startswith("version = ") and previous in line:
            line = line.replace(previous, version)
        result.append(line)
    return result


out, block = [], []
for line in text.splitlines(keepends=True):
    if line.startswith("[["):
        out.extend(_flush(block)); block = []
    block.append(line)
out.extend(_flush(block))
open("Cargo.lock", "w", encoding="utf-8").write("".join(out))
PYLOCK

echo "set-version: ${previous} -> ${version} (Android versionCode ${previous_code} -> ${code})"