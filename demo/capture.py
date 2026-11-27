import os
import shutil
from pathlib import Path

from playwright.sync_api import sync_playwright

FPS = 30
SCALE = 2  # capture at 1440 and downsample to 720 for crisp text
URL = os.environ.get(
    "DEMO_URL", "http://127.0.0.1:8765/demo/index.html?capture=1"
)
OUT = Path("demo/frames")

if OUT.exists():
    shutil.rmtree(OUT)
OUT.mkdir(parents=True)

with sync_playwright() as p:
    browser = p.chromium.launch(channel="chrome", headless=True)
    page = browser.new_page(
        viewport={"width": 720, "height": 720},
        device_scale_factor=SCALE,
    )
    page.goto(URL, wait_until="networkidle")
    page.wait_for_function("document.fonts.status === 'loaded'")

    total_seconds = float(page.evaluate("window.__total"))
    total = int(round(total_seconds * FPS))
    page.evaluate("window.__setTime(0)")
    page.wait_for_timeout(250)

    for i in range(total):
        t = i / FPS
        page.evaluate("(t) => window.__setTime(t)", t)
        page.screenshot(path=str(OUT / f"{i:04d}.png"), animations="disabled")
        if i % 60 == 0:
            print(f"{i}/{total}", flush=True)

    browser.close()

print(f"captured {total} frames at {FPS} fps, {720 * SCALE}px source", flush=True)
