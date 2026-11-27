# Orbiscreen demo

A 24-second, 720 × 720 minimal motion piece with clean typography on black.
Four simple scenes: intro, extended display, features, and end card.

## Files

| File | Size | What it is |
|---|---|---|
| `orbiscreen-demo.mp4` | 120 KB | Full 24.0 s, 720 × 720, 30 fps, H.264 |
| `orbiscreen-demo-480.mp4` | 70 KB | Full 24.0 s, 480 × 480, H.264 |
| `orbiscreen-demo-preview.mp4` | 90 KB | 12 s cut: intro, extended display, features, end card |
| `orbiscreen-demo-poster.png` | 26 KB | 720 × 720 poster from the intro |

The READMEs embed the poster and link to the full video. The site plays the
preview. There is no GIF: the old 480 × 480 / 10 fps GIF was the low-quality
version and has been removed.

## Source

- `demo/index.html` is the animation. It has no build step.
- `window.__setTime(seconds)` renders an exact frame; `?t=12.5` does the same in
  a browser.
- `?capture=1` disables the real-time loop so the frame capture is deterministic.
- Fonts come from `site/node_modules` so the type matches the site.

## Rebuild

One command rebuilds the video, the 480 copy, the preview cut, and the poster:

```bash
./demo/build.sh
```

It starts a local server, captures 720 frames at 1440 px with Playwright
(`demo/capture.py`), downsamples to 720 with lanczos, and encodes with x264
(CRF 14). Requirements: Python 3 with Playwright and Google Chrome (the `chrome` channel), plus `ffmpeg`.

The duration is read from `window.__total`, so changing `TOTAL` in
`demo/index.html` changes the video length without editing the capture script.
`demo/frames/` is ignored and removed after encoding.
