# Orbiscreen demo

A 1080×1080 motion piece in the style of the project site: blue on near-black,
bold Inter headings, a monitor and tablet, a terminal, and the logo end card.

## Files

| File | Size | What it is |
|---|---|---|
| `orbiscreen-demo.mp4` | 4.7 MB | Full 32.4 s, 1080×1080, 30 fps, H.264 |
| `orbiscreen-demo-480.mp4` | 1.0 MB | Full 32.4 s, 480×480, H.264 |
| `orbiscreen-demo-preview.mp4` | 2.7 MB | 18.1 s cut: intro, display, transport, end card |
| `orbiscreen-demo.gif` | 4.7 MB | 18.1 s preview, 480×480, 10 fps, looping |
| `orbiscreen-demo-poster.png` | 144 KB | 1080×1080 poster from the intro |

The README embeds the GIF and links to the full video.

## Source

- `demo/index.html` is the animation. It has no build step.
- `window.__setTime(seconds)` renders any exact frame; without it the page loops in real time.
- Fonts come from `site/node_modules` so the type matches the site.

## Rebuild

```bash
python3 -m http.server 8765 --bind 127.0.0.1
python3 demo/capture.py

ffmpeg -y -framerate 30 -i demo/frames/%04d.png \
  -c:v libx264 -preset slow -crf 18 -pix_fmt yuv420p -movflags +faststart \
  assets/demo/orbiscreen-demo.mp4

ffmpeg -y -i assets/demo/orbiscreen-demo.mp4 \
  -vf "scale=480:480:flags=lanczos" \
  -c:v libx264 -preset slow -crf 21 -pix_fmt yuv420p -movflags +faststart \
  assets/demo/orbiscreen-demo-480.mp4

ffmpeg -y -i assets/demo/orbiscreen-demo.mp4 \
  -filter_complex "[0:v]trim=0:3.6,setpts=PTS-STARTPTS[v0];[0:v]trim=3.6:8.7,setpts=PTS-STARTPTS[v1];[0:v]trim=8.7:13.9,setpts=PTS-STARTPTS[v2];[0:v]trim=28.2:32.4,setpts=PTS-STARTPTS[v3];[v0][v1][v2][v3]concat=n=4:v=1:a=0[out]" \
  -map "[out]" -c:v libx264 -preset slow -crf 18 -pix_fmt yuv420p -movflags +faststart \
  assets/demo/orbiscreen-demo-preview.mp4

ffmpeg -y -i assets/demo/orbiscreen-demo-preview.mp4 \
  -vf "fps=10,scale=480:480:flags=lanczos,split[s0][s1];[s0]palettegen=max_colors=48:stats_mode=diff[p];[s1][p]paletteuse=dither=bayer:bayer_scale=5" \
  -loop 0 assets/demo/orbiscreen-demo.gif

ffmpeg -y -i assets/demo/orbiscreen-demo.mp4 -vf "select=eq(n\,48)" -frames:v 1 \
  assets/demo/orbiscreen-demo-poster.png
```

`demo/frames/` is ignored; delete it after encoding.