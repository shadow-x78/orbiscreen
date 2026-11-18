#!/usr/bin/env bash
# ─────────────────────────────────────────────
# Orbiscreen - Application Icon Generator
# https://github.com/shadow-x78/orbiscreen
# ─────────────────────────────────────────────

# Renders the committed PNG icon set from the SVG masters, at each target size instead of
# downscaling one large raster. Every size therefore gets the same crisp geometry, and the small
# master (16/24/32) keeps the ring readable when the whole mark is only a dozen pixels tall.
#
#   scripts/generate-icons.sh           # rewrite the PNGs
#   scripts/generate-icons.sh --check   # verify the committed PNGs only, write nothing
#
# Sources:
#   data/orbiscreen.svg                     main master (identical copy in assets/logo/)
#   assets/logo/orbiscreen-logo-small.svg   small-size master (16/24/32)
#
# Outputs:
#   gui/icons/{16,22,24,32,48,64,128,256,512}x*.png, gui/icons/icon.png,
#   gui/icons/128x128@2x.png, gui/icons/icon.ico, and the Android legacy mipmaps.

set -euo pipefail

cd "$(dirname "$0")/.."
REPO_ROOT="$(pwd)"

CHECK=0
for arg in "$@"; do
    case "$arg" in
    --check) CHECK=1 ;;
    -h | --help)
        sed -n '2,20p' "$0" | sed 's/^# \{0,1\}//'
        exit 0
        ;;
    *)
        echo "error: unknown argument '$arg' (try --check)" >&2
        exit 1
        ;;
    esac
done

MAIN_SVG="$REPO_ROOT/data/orbiscreen.svg"
MIRROR_SVG="$REPO_ROOT/assets/logo/orbiscreen-logo.svg"
SMALL_SVG="$REPO_ROOT/assets/logo/orbiscreen-logo-small.svg"
WEB_FAVICON="$REPO_ROOT/clients/web/favicon.svg"
WEB_FAVICON_PNG="$REPO_ROOT/clients/web/favicon.png"
WEB_TOUCH_PNG="$REPO_ROOT/clients/web/apple-touch-icon.png"
ICON_DIR="$REPO_ROOT/gui/icons"
ANDROID_RES="$REPO_ROOT/clients/android/app/src/main/res"
ANDROID_BG="#11111B"

# Coverage window for the mark's alpha bounding box: 78-86% of the canvas. The main master is
# 82.0%, the small master 84.2%; anything outside means a master was edited by accident.
# Coverage is measured on a high-resolution render of the master, never on a small PNG: at 16px
# one pixel is 6.25% of the canvas, so a per-file check would only measure quantization noise.
MIN_COVER=78
MAX_COVER=86
MEASURE_SIZE=1024

fail() {
    echo "error: $*" >&2
    exit 1
}

for svg in "$MAIN_SVG" "$MIRROR_SVG" "$SMALL_SVG"; do
    [ -f "$svg" ] || fail "missing $svg"
done

# The two copies of the main mark must stay byte-identical; drift silently splits the branding.
cmp -s "$MAIN_SVG" "$MIRROR_SVG" \
    || fail "data/orbiscreen.svg and assets/logo/orbiscreen-logo.svg differ; keep them identical"

# The web favicon is the same mark written without the CSS classes, so compare the geometry and
# the colour numerically instead of the bytes.
check_web_favicon() {
    local expected actual
    expected="$(tr -d '\n' < "$MAIN_SVG" | sed -n 's/.*class="orbi-s" cx="\([0-9.]*\)" cy="\([0-9.]*\)" r="\([0-9.]*\)" stroke-width="\([0-9.]*\)".*/\1 \2 \3 \4/p')"
    actual="$(tr -d '\n' < "$WEB_FAVICON" | sed -n 's/.*<circle cx="\([0-9.]*\)" cy="\([0-9.]*\)" r="\([0-9.]*\)" fill="none" stroke="#89B4FA" stroke-width="\([0-9.]*\)".*/\1 \2 \3 \4/p')"
    [ -n "$actual" ] || fail "clients/web/favicon.svg: could not read the ring geometry"
    [ "$expected" = "$actual" ] \
        || fail "clients/web/favicon.svg ring is '$actual', the master is '$expected'"

    local dot_expected dot_actual
    dot_expected="$(tr -d '\n' < "$MAIN_SVG" | sed -n 's/.*class="orbi-f" cx="\([0-9.]*\)" cy="\([0-9.]*\)" r="\([0-9.]*\)".*/\1 \2 \3/p')"
    dot_actual="$(tr -d '\n' < "$WEB_FAVICON" | sed -n 's/.*<circle cx="\([0-9.]*\)" cy="\([0-9.]*\)" r="\([0-9.]*\)" fill="#89B4FA".*/\1 \2 \3/p')"
    [ -n "$dot_actual" ] || fail "clients/web/favicon.svg: could not read the dot geometry"
    [ "$dot_expected" = "$dot_actual" ] \
        || fail "clients/web/favicon.svg dot is '$dot_actual', the master is '$dot_expected'"
}
check_web_favicon

# ── Rasterizer ──
# rsvg-convert renders these files most faithfully; ImageMagick's internal MSVG delegate gets
# the geometry subtly wrong, so it is the second choice and is verified by the coverage check.
if command -v rsvg-convert >/dev/null 2>&1; then
    RASTERIZER=rsvg
elif command -v magick >/dev/null 2>&1; then
    RASTERIZER=magick
elif command -v convert >/dev/null 2>&1; then
    RASTERIZER=convert
else
    fail "no SVG rasterizer found (need 'rsvg-convert', 'magick' or 'convert')"
fi

render() {
    local svg="$1" size="$2" out="$3"
    case "$RASTERIZER" in
    rsvg) rsvg-convert -w "$size" -h "$size" "$svg" -o "$out" ;;
    magick) magick -background none -density 1152 "$svg" -resize "${size}x${size}" "$out" ;;
    convert) convert -background none -density 1152 "$svg" -resize "${size}x${size}" "$out" ;;
    esac
    strip_metadata "$out"
}

# PNG encoders embed tIME/text chunks that change on every run, so a regeneration would always
# show a diff even when the pixels match. -strip plus defining the tIME chunk away makes the
# output reproducible: two runs produce identical bytes.
strip_metadata() {
    local file="$1"
    if command -v magick >/dev/null 2>&1; then
        magick "$file" -strip -define png:exclude-chunks=date,time "$file"
    elif command -v convert >/dev/null 2>&1; then
        convert "$file" -strip -define png:exclude-chunks=date,time "$file"
    fi
}

# Coverage of an SVG master: render it large, trim the transparent border, compare the widest
# remaining side with the canvas. Rendering large keeps the result independent of pixel rounding.
master_coverage() {
    local svg="$1" what="$2" tmp trimmed width height longest cover
    tmp="$(mktemp --suffix=.png)"
    render "$svg" "$MEASURE_SIZE" "$tmp"
    trimmed="$(magick "$tmp" -alpha extract -threshold 50% -trim -format "%w %h" info:)"
    rm -f "$tmp"
    width="${trimmed%% *}"
    height="${trimmed##* }"
    [ -n "$width" ] && [ -n "$height" ] || fail "cannot measure the coverage of $what"
    longest="$width"
    [ "$height" -gt "$longest" ] && longest="$height"
    cover=$((100 * longest / MEASURE_SIZE))
    if [ "$cover" -lt "$MIN_COVER" ] || [ "$cover" -gt "$MAX_COVER" ]; then
        fail "$what covers ${cover}% of the canvas, expected ${MIN_COVER}-${MAX_COVER}%"
    fi
    echo "$what covers ${cover}% of the canvas"
}

# A committed PNG must exist, be exactly square at the requested size, draw something, and keep
# its ink centred (or be exactly the canvas for the Android plates, which bake a background).
verify() {
    local file="$1" size="$2" what="$3" opaque="${4:-0}" dimensions box
    [ -f "$file" ] || fail "$what missing: ${file#"$REPO_ROOT"/} (run scripts/generate-icons.sh)"
    dimensions="$(magick identify -format "%wx%h" "$file" 2>/dev/null)" \
        || fail "$what is not a readable image: ${file#"$REPO_ROOT"/}"
    [ "$dimensions" = "${size}x${size}" ] \
        || fail "$what is $dimensions, expected ${size}x${size}: ${file#"$REPO_ROOT"/}"

    if [ "$opaque" -eq 1 ]; then
        # The Android plates bake the brand background, so an alpha box says nothing.
        return 0
    fi

    # "WxH+X+Y" of the alpha bounding box at half opacity.
    box="$(magick "$file" -alpha extract -threshold 50% -format "%@" info: 2>/dev/null)" \
        || fail "cannot measure $what"
    [ -n "$box" ] && [ "$box" != "0x0" ] || fail "$what draws nothing: ${file#"$REPO_ROOT"/}"

    local w="${box%%x*}"
    local rest="${box#*x}"
    local h="${rest%%+*}"
    local x="${rest#*+}"
    local y="${x#*+}"
    x="${x%%+*}"
    local left="$x"
    local right=$((size - x - w))
    local top="$y"
    local bottom=$((size - y - h))
    local tolerance=$((size / 8 + 1))
    local dx=$((left - right)) dy=$((top - bottom))
    [ "${dx#-}" -le "$tolerance" ] && [ "${dy#-}" -le "$tolerance" ] \
        || fail "$what is off-centre (margins L$left R$right T$top B$bottom): ${file#"$REPO_ROOT"/}"
}

# sizes_from_small: the whole mark is only a dozen pixels tall, so it uses the heavier master.
sizes_from_small="16 22 24 32"
sizes_from_main="48 64 128 256 512"

if [ "$CHECK" -eq 0 ]; then
    mkdir -p "$ICON_DIR"
    for size in $sizes_from_small; do
        render "$SMALL_SVG" "$size" "$ICON_DIR/${size}x${size}.png"
    done
    for size in $sizes_from_main; do
        render "$MAIN_SVG" "$size" "$ICON_DIR/${size}x${size}.png"
    done
    # HiDPI pair used by Windows/macOS bundles and by the AppImage desktop entry.
    cp -f "$ICON_DIR/256x256.png" "$ICON_DIR/128x128@2x.png"
    cp -f "$ICON_DIR/512x512.png" "$ICON_DIR/icon.png"

    if command -v magick >/dev/null 2>&1; then
        magick "$ICON_DIR/16x16.png" "$ICON_DIR/24x24.png" "$ICON_DIR/32x32.png" \
            "$ICON_DIR/48x48.png" "$ICON_DIR/64x64.png" "$ICON_DIR/256x256.png" \
            "$ICON_DIR/icon.ico"
    elif command -v convert >/dev/null 2>&1; then
        convert "$ICON_DIR/16x16.png" "$ICON_DIR/24x24.png" "$ICON_DIR/32x32.png" \
            "$ICON_DIR/48x48.png" "$ICON_DIR/64x64.png" "$ICON_DIR/256x256.png" \
            "$ICON_DIR/icon.ico"
    else
        echo "warning: ImageMagick missing, leaving icon.ico untouched" >&2
    fi
fi

# ── Verify the masters and the Linux icon set ──
master_coverage "$MAIN_SVG" "the main master"
master_coverage "$SMALL_SVG" "the small master"

for size in 16 22 24 32; do
    verify "$ICON_DIR/${size}x${size}.png" "$size" "icon ${size}px"
done
for size in 48 64 128 256 512; do
    verify "$ICON_DIR/${size}x${size}.png" "$size" "icon ${size}px"
done
verify "$ICON_DIR/icon.png" 512 "icon.png"

# The web client ships its own rasterised copies of the same mark.
if [ "$CHECK" -eq 0 ]; then
    render "$WEB_FAVICON" 32 "$WEB_FAVICON_PNG"
    # iOS shows a touch icon on an opaque plate, so bake the brand dark canvas in. The mark is
    # rendered transparent first (render() passes -background none), then composited, otherwise
    # ImageMagick rasterizes the SVG on its default white background.
    web_mark="$(mktemp --suffix=.png)"
    render "$WEB_FAVICON" 256 "$web_mark"
    magick -size 256x256 "xc:$ANDROID_BG" "$web_mark" -compose over -composite "$WEB_TOUCH_PNG"
    strip_metadata "$WEB_TOUCH_PNG"
    rm -f "$web_mark"
fi
verify "$WEB_FAVICON_PNG" 32 "web favicon"
verify "$WEB_TOUCH_PNG" 256 "web apple-touch-icon" 1

# ── Android legacy mipmaps ──
# Density buckets and their launcher icon size; the adaptive icon (API 26+) is vector and shares
# the same geometry, so these only serve older launchers and the round fallback.
android_sizes="mdpi:48 hdpi:72 xhdpi:96 xxhdpi:144 xxxhdpi:192"
for entry in $android_sizes; do
    bucket="${entry%%:*}"
    size="${entry##*:}"
    dir="$ANDROID_RES/mipmap-$bucket"
    [ -d "$dir" ] || continue
    for name in ic_launcher ic_launcher_round; do
        file="$dir/$name.png"
        if [ "$CHECK" -eq 0 ]; then
            radius=0
            [ "$name" = "ic_launcher_round" ] && radius=$((size / 2))
            render "$MAIN_SVG" "$size" "$dir/.tmp-$name.png"
            # Legacy launchers draw these on their own plate, so bake the brand dark canvas in
            # and clip the round variant to a circle.
            if [ "$radius" -gt 0 ]; then
                magick -size "${size}x${size}" "xc:$ANDROID_BG" \
                    \( "$dir/.tmp-$name.png" -resize "${size}x${size}" \) \
                    -compose over -composite \
                    \( -size "${size}x${size}" "xc:none" -fill white \
                    -draw "circle $((size / 2)),$((size / 2)) $((size / 2)),0" \) \
                    -alpha off -compose copy_opacity -composite \
                    "$file"
            else
                magick -size "${size}x${size}" "xc:$ANDROID_BG" \
                    \( "$dir/.tmp-$name.png" -resize "${size}x${size}" \) \
                    -compose over -composite "$file"
            fi
            strip_metadata "$file"
            rm -f "$dir/.tmp-$name.png"
        fi
        # These plates bake the brand background, so the alpha box is the whole canvas.
        verify "$file" "$size" "Android $bucket/$name" 1
    done
done

if [ "$CHECK" -eq 1 ]; then
    echo "icons: all committed assets match the masters"
else
    echo "icons: regenerated $(find "$ICON_DIR" -name '*.png' | wc -l) PNG files and the Android mipmaps"
fi
