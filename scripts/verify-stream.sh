#!/usr/bin/env bash
# Orbiscreen - verify-stream (GPL-3.0-or-later)
# https://github.com/shadow-x78/orbiscreen

set -euo pipefail

PORT="${1:-8788}"
DURATION="${2:-4}"
BASE="http://localhost:${PORT}"

fail() {
    echo "FAIL: $1"
    exit 1
}

command -v curl >/dev/null || fail "curl is required"
command -v python3 >/dev/null || fail "python3 is required"
command -v ffmpeg >/dev/null || fail "ffmpeg is required"

HEALTH=$(curl -s --max-time 3 "$BASE/health" || true)
[ -n "$HEALTH" ] || fail "daemon is not responding on $BASE (is it running?)"
echo "[verify-stream] health: $HEALTH"

TOKEN=$(curl -s --max-time 3 "$BASE/client/config.json" \
    | python3 -c "import sys,json;print(json.load(sys.stdin)['token'])") \
    || fail "could not fetch the stream token from config.json"

TMP="$(mktemp -t orbiscreen-verify-XXXXXX.ts)"
trap 'rm -f "$TMP"' EXIT

set +e
curl -sN --max-time "$DURATION" "$BASE/stream?token=$TOKEN" -o "$TMP"
CURL_RC=$?
set -e
if [ "$CURL_RC" -ne 0 ] && [ "$CURL_RC" -ne 28 ]; then
    fail "stream request failed (curl exit $CURL_RC)"
fi

SIZE=$(stat -c%s "$TMP" 2>/dev/null || echo 0)
[ "$SIZE" -gt 10000 ] || fail "stream payload too small ($SIZE B)"

if command -v ffprobe >/dev/null 2>&1 \
    && INFO=$(ffprobe -v error -select_streams v:0 -show_entries \
        stream=codec_name,width,height,avg_frame_rate -of csv=p=0 "$TMP" | head -1); then
    echo "$INFO" | grep -q '^h264,' || fail "no H.264 video stream found (got: '$INFO')"
    echo "[verify-stream] stream: $INFO"
else
    ffmpeg -hide_banner -i "$TMP" -f null - 2>&1 \
        | grep -qE "Video: h264" || fail "no H.264 video stream found"
    echo "[verify-stream] stream: h264 (ffprobe unavailable)"
fi

YAVG=$(ffmpeg -hide_banner -i "$TMP" -vf "signalstats,metadata=print:key=lavfi.signalstats.YAVG" \
    -frames:v 5 -f null - 2>&1 | grep -oE 'YAVG=[0-9.]+' | head -1 | cut -d= -f2)
[ -n "$YAVG" ] || fail "could not measure frame brightness"

if python3 -c "import sys; sys.exit(0 if float('$YAVG') < 20.0 else 1)"; then
    echo "FAIL: decoded frames look black (YAVG=$YAVG)"
    exit 1
fi

echo "PASS: stream is live, H.264 decodes, frames have content (YAVG=$YAVG)"
