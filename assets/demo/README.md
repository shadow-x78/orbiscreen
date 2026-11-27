# Orbiscreen demo

A 34-second, 1080 × 1080 motion piece in the site's dark palette: `#0e1116`
base, `#151a21`/`#1b222b` surfaces, `#4c8dff` signal lines, and a running
timecode. It walks through the extended display, transport, input, setup,
clients, and the end card.

## Files

| File | Size | What it is |
|---|---|---|
| `orbiscreen-demo.mp4` | 1.2 MB | Full 34.0 s, 1080 × 1080, 30 fps, H.264 |
| `orbiscreen-demo-480.mp4` | 336 KB | Full 34.0 s, 480 × 480, H.264 |
| `orbiscreen-demo-preview.mp4` | 696 KB | 17.8 s cut: intro, display, transport, end card |
| `orbiscreen-demo-poster.png` | 188 KB | 1080 × 1080 poster from the intro |

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

It starts a local server, captures 1020 frames at 2160 px with Playwright
(`demo/capture.py`), downsamples to 1080 with lanczos, and encodes with x264
(CRF 16). Requirements: Python 3 with Playwright and Google Chrome (the `chrome` channel), plus `ffmpeg`.

The duration is read from `window.__total`, so changing `TOTAL` in
`demo/index.html` changes the video length without editing the capture script.
`demo/frames/` is ignored and removed after encoding.
