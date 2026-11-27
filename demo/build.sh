#!/usr/bin/env bash
# Rebuild the demo video and every derived asset from demo/index.html.
# Requires: python3 + Playwright with Google Chrome, ffmpeg, and a free local port.
set -euo pipefail

cd "$(dirname "$0")/.."

PORT="${DEMO_PORT:-8891}"
OUT="assets/demo"

python3 -m http.server "$PORT" --bind 127.0.0.1 >/dev/null 2>&1 &
SERVER_PID=$!
trap 'kill "$SERVER_PID" 2>/dev/null || true' EXIT
sleep 1

DEMO_URL="http://127.0.0.1:${PORT}/demo/index.html?capture=1" python3 demo/capture.py

ffmpeg -y -loglevel error -framerate 30 -i demo/frames/%04d.png \
  -vf "scale=720:720:flags=lanczos" \
  -c:v libx264 -preset slow -crf 14 -pix_fmt yuv420p \
  -profile:v high -level 4.2 -movflags +faststart \
  "$OUT/orbiscreen-demo.mp4"

ffmpeg -y -loglevel error -i "$OUT/orbiscreen-demo.mp4" \
  -vf "scale=480:480:flags=lanczos" \
  -c:v libx264 -preset slow -crf 18 -pix_fmt yuv420p -movflags +faststart \
  "$OUT/orbiscreen-demo-480.mp4"

# 12 s cut: intro, extended display, features, end card.
ffmpeg -y -loglevel error -i "$OUT/orbiscreen-demo.mp4" \
  -filter_complex "[0:v]trim=0:3,setpts=PTS-STARTPTS[v0];[0:v]trim=6:9,setpts=PTS-STARTPTS[v1];[0:v]trim=12:15,setpts=PTS-STARTPTS[v2];[0:v]trim=21:24,setpts=PTS-STARTPTS[v3];[v0][v1][v2][v3]concat=n=4:v=1:a=0[out]" \
  -map "[out]" -c:v libx264 -preset slow -crf 14 -pix_fmt yuv420p -movflags +faststart \
  "$OUT/orbiscreen-demo-preview.mp4"

ffmpeg -y -loglevel error -ss 2 -i "$OUT/orbiscreen-demo.mp4" -frames:v 1 \
  "$OUT/orbiscreen-demo-poster.png"

# keep the website copy in step with the repository assets
for name in orbiscreen-demo.mp4 orbiscreen-demo-480.mp4 orbiscreen-demo-preview.mp4 orbiscreen-demo-poster.png; do
  cp "$OUT/$name" "site/public/assets/demo/$name"
done

rm -rf demo/frames
echo "wrote $OUT + site/public/assets/demo (mp4, 480, preview, poster)"
