#!/usr/bin/env bash
# ─────────────────────────────────────────────
# Orbiscreen - Code Comment Policy Guard
# https://github.com/shadow-x78/orbiscreen
# ─────────────────────────────────────────────

# Fails when a source file (Rust, Kotlin, JavaScript, TypeScript, HTML)
# contains any comment: standalone lines, trailing remarks, or blocks.
# The file-top credit banner (license or third-party notice, including a
# leading DOCTYPE and blank lines) is the only tolerated occurrence.
# Configuration files (.env*, *.toml, *.sh, *.xml, *.yml, systemd units) keep
# their comments and are out of scope here.

# ── Exit Codes ──
# 0: policy holds, 1: violations found

set -u

root="$(cd "$(dirname "$0")/.." && pwd)"
status=0

# ── Banner Skipper ──
# Emits the file body after the leading banner block, then the caller greps
# that body for violations.
skip_banner='
BEGIN { state = 0 }
state == 0 {
    if ($0 ~ /^[[:space:]]*$/) next
    if ($0 ~ /^[[:space:]]*<![Dd][Oo][Cc][Tt][Yy][Pp][Ee]/) next
    if ($0 ~ /^[[:space:]]*\/\//) next
    if ($0 ~ /^[[:space:]]*\/\*/ || $0 ~ /^[[:space:]]*\*/) {
        if ($0 ~ /\*\//) next
        state = 2
        next
    }
    if ($0 ~ /^[[:space:]]*<!--/) {
        if ($0 ~ /-->/) next
        state = 3
        next
    }
    state = 1
}
state == 2 { if ($0 ~ /\*\//) state = 0; next }
state == 3 { if ($0 ~ /-->/) state = 0; next }
state == 1 { print }
'

# ── Violation Scan ──
while IFS= read -r file; do
    hits=$(awk "$skip_banner" "$file" 2>/dev/null \
        | grep -Pn '(^\s*//)|([^:"/]\s+//)|(^\s*/\*)|([^:"/]\s+/\*)|(^\s*<!--)')
    if [ -n "$hits" ]; then
        echo "::error file=${file#"$root"/}::code comments are not allowed"
        status=1
    fi
done < <(find \
    "$root/crates" \
    "$root/clients/web" \
    "$root/clients/android/app/src" \
    -type f \( -name '*.rs' -o -name '*.kt' -o -name '*.kts' \
        -o -name '*.js' -o -name '*.ts' -o -name '*.html' \) \
    ! -path '*/build/*' \
    ! -path '*/gen/*' \
    ! -path '*/target/*' \
    ! -path '*/node_modules/*' 2>/dev/null)

if [ "$status" -eq 0 ]; then
    echo "comment policy holds"
fi
exit "$status"
