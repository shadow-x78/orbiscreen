import shutil
from pathlib import Path

from playwright.sync_api import sync_playwright

FPS = 30
DURATION = 32.4
URL = "http://127.0.0.1:8765/demo/index.html?capture=1"
OUT = Path("demo/frames")

if OUT.exists():
    shutil.rmtree(OUT)
OUT.mkdir(parents=True)

with sync_playwright() as p:
    browser = p.chromium.launch(channel="chrome", headless=True)
    page = browser.new_page(viewport={"width": 1080, "height": 1080}, device_scale_factor=1)
    page.goto(URL, wait_until="networkidle")
    page.wait_for_function("document.fonts.status === 'loaded'")
    page.evaluate("window.__setTime(0)")
    page.wait_for_timeout(250)

    total = int(DURATION * FPS)
    for i in range(total):
        t = i / FPS
        page.evaluate("(t) => window.__setTime(t)", t)
        page.screenshot(path=str(OUT / f"{i:04d}.png"), animations="disabled")
        if i % 60 == 0:
            print(f"{i}/{total}", flush=True)

    browser.close()

print(f"captured {total} frames at {FPS} fps", flush=True)
