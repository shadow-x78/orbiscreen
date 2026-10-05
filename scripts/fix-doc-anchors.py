#!/usr/bin/env python3
"""Repoint the troubleshooting guides' legacy table-of-contents anchors.

Both guides were reorganised into real sections, but the "Still Stuck?" link lists still
used the pre-reorganisation slugs, so those links jumped back to the top of the page
instead of reaching a section. This rewrites each legacy anchor to the heading that now
carries its content and reports anything it could not place.

Run from the repository root:
    python3 scripts/fix-doc-anchors.py docs/TROUBLESHOOTING.md docs/TROUBLESHOOTING_AR.md
"""
import pathlib
import re
import sys

ENGLISH = {
    "ci-fmt": "ci-action-format-lint-cargo-fmt---check",
    "ci-clippy": "ci-action-clippy-deny-warnings",
    "ci-build": "ci-action-build-cargo-build---workspace---locked",
    "ci-test": "ci-action-test-cargo-test---workspace---locked",
    "ci-deny": "ci-action-run-cargo-deny",
    "ci-android": "ci-action-android-assembledebug-lintdebug",
    "ci-ppa": "ci-action-publish-to-launchpad-ppa-ppayml",
    "ci-deb": "ci-action-publish-to-launchpad-ppa-ppayml",
    "ci-fedora": "ci-action-fedora-rpm--ci-fedoryml",
    "ci-rpm": "ci-action-fedora-rpm--ci-fedoryml",
    "ci-release": "ci-action-release-matrix-releaseyml",
    "ci-failures": "ci-workflow-actions-githubworkflowsciyml",
    "re-run-job": "ci-workflow-actions-githubworkflowsciyml",
    "runtime-evdi": "runtime-kde-plasma-virtual-display-without-evdi-or-root",
    "runtime-kwin": "runtime-kde-plasma-virtual-display-without-evdi-or-root",
    "runtime-exit": "runtime-orbiscreen-start-fails---kernel-module-is-not-installed",
    "runtime-exits": "runtime-orbiscreen-start-fails---kernel-module-is-not-installed",
    "runtime-no-display": "runtime-orbiscreen-start-fails---kernel-module-is-not-installed",
    "runtime-no-screen": "runtime-orbiscreen-start-fails---kernel-module-is-not-installed",
    "runtime-capture": "runtime-capture-backend-unavailable-on-wayland",
    "runtime-touch": "runtime-capture-backend-unavailable-on-wayland",
    "runtime-wayland": "runtime-capture-backend-unavailable-on-wayland",
    "runtime-unsafe": "runtime-unsafe_op_in_unsafe_fn-missing_debug_implementations",
    "runtime-lints": "runtime-unsafe_op_in_unsafe_fn-missing_debug_implementations",
    "runtime-dbus": "daemon-not-found-on-d-bus",
    "runtime-issues": "runtime",
    "android-adb": "android-chromeos-adb-connection-fails-on-asus-chromebook-cm3001",
    "android-chromebook-adb": "android-chromeos-adb-connection-fails-on-asus-chromebook-cm3001",
    "android-stylus": "android-stylus-pen-not-drawing-incorrect-pressure-or-app-crash-on-lenovo-tab",
    "android-lenovo-tab": "android-stylus-pen-not-drawing-incorrect-pressure-or-app-crash-on-lenovo-tab",
    "android-touchpad": "android-dragging-windows-or-selecting-files-in-touchpad-mode",
    "android-touchpad-drag": "android-dragging-windows-or-selecting-files-in-touchpad-mode",
    "android-crash": "android-app-crashes-or-process-dies-when-tapping-connect",
    "android-app-crash": "android-app-crashes-or-process-dies-when-tapping-connect",
    "android-connect-crash": "android-app-crashes-or-process-dies-when-tapping-connect",
    "android-launch-crash": "android-app-crashes-immediately-on-launch",
    "android-black-screen": "android-black-screen-after-connect",
    "android-no-hosts": "android-discovery-list-is-empty-even-though-hosts-are-on-the-same-wi-fi",
    "android-discovery": "android-discovery-list-is-empty-even-though-hosts-are-on-the-same-wi-fi",
    "android-touch-offset": "android-touch-is-rotated-misaligned",
    "android-rotation": "android-touch-is-rotated-misaligned",
    "android-control-404": "android-control-toolbar-actions-return-404",
    "android-404": "android-control-toolbar-actions-return-404",
    "android-toolbar-404": "android-control-toolbar-actions-return-404",
    "android-looking-for-host": "android-usb-connection-shows-looking-for-host",
    "android-usb": "android-usb-connection-shows-looking-for-host",
    "android-usb-host": "android-usb-connection-shows-looking-for-host",
    "android-client-issues": "android-client-devices",
    "streaming-wifi-latency": "streaming-high-latency-stutter-or-slow-mouse-movement-on-5ghz-wi-fi",
    "streaming-latency": "streaming-high-latency-stutter-or-slow-mouse-movement-on-5ghz-wi-fi",
    "streaming-reconnect": "streaming-stream-error-causes-infinite-reconnect-flicker-instead-of-detecting-disconnect",
    "streaming-flicker": "streaming-stream-error-causes-infinite-reconnect-flicker-instead-of-detecting-disconnect",
    "stream-disconnect-retry": "streaming-stream-error-causes-infinite-reconnect-flicker-instead-of-detecting-disconnect",
    "cursor-clamping": "multi-monitor-x11-mouse-cursor-escapes-virtual-display-to-other-physical-screens",
    "cursor-escape": "multi-monitor-x11-mouse-cursor-escapes-virtual-display-to-other-physical-screens",
    "wrong-screen": "client-shows-the-wrong-screen-primary-desktop-instead-of-virtual-display",
    "web-no-picture": "web-client-loads-but-shows-no-picture",
    "no-encoder": "no-encoder-available---stream-starts-but-errors-out-x264-missing",
    "token-401": "401-unauthorized-from-stream-input-or-apicontrol-token",
    "unauthorized": "401-unauthorized-from-stream-input-or-apicontrol-token",
    "dbus-missing": "daemon-not-found-on-d-bus",
    "daemon-cpu": "daemon-100-cpu-usage-or-freeze",
    "local-build": "local-build-issues",
    "local-build-issues": "local-build-issues",
    "local-sdk": "android-sdk-not-found",
    "local-uinput": "uinput-permission-denied",
    "local-perm": "uinput-permission-denied",
    "net-timeout": "network--connection-issues",
    "network--connection-issues": "network--connection-issues",
    "comp-mutter": "gnome-wayland-mutter",
    "comp-x11": "x11",
    "comp-wlroots": "sway-wlroots",
    "compositor-specific-issues": "compositor-specific-issues",
    "app-sw-decoder": "media-codec-failure-frames-drop-or-stutter",
    "verify-stream": "verify-stream-works",
    "setup-dev-env": "setup-dev-env-works",
    "still-stuck": "still-stuck",
}

ARABIC = {
    "ci-fmt": "إجراء-ci-format",
    "ci-clippy": "إجراء-ci-clippy",
    "ci-build": "إجراء-ci-build",
    "ci-test": "إجراء-ci-test",
    "ci-deny": "إجراء-ci-run-cargo-deny",
    "ci-android": "إجراء-ci-android",
    "ci-deb": "إجراء-ci-publish-to-launchpad",
    "ci-rpm": "إجراء-ci-fedora-rpm",
    "ci-release": "إجراء-ci-release-matrix",
    "ci-failures": "إخفاقات-سير-عمل-ci",
    "re-run-job": "إجراءات-سير-عمل-ci",
    "runtime-evdi": "وقت-التشغيل-kde-plasma",
    "runtime-kwin": "وقت-التشغيل-kde-plasma",
    "runtime-exit": "وقت-التشغيل-فشل",
    "runtime-exits": "وقت-التشغيل-فشل",
    "runtime-no-display": "وقت-التشغيل-فشل",
    "runtime-no-screen": "وقت-التشغيل-فشل",
    "runtime-capture": "وقت-التشغيل-واجهة-الالتقاط",
    "runtime-touch": "وقت-التشغيل-واجهة-الالتقاط",
    "runtime-wayland": "وقت-التشغيل-واجهة-الالتقاط",
    "runtime-unsafe": "وقت-التشغيل-unsafe",
    "runtime-lints": "وقت-التشغيل-unsafe",
    "runtime-dbus": "الـ-daemon",
    "runtime-issues": "وقت-التشغيل",
    "android-adb": "android-chromeos",
    "android-chromebook-adb": "android-chromeos",
    "android-stylus": "android-القلم",
    "android-lenovo-tab": "android-القلم",
    "android-touchpad": "android-سحب-النوافذ",
    "android-touchpad-drag": "android-سحب-النوافذ",
    "android-crash": "android-التطبيق-يتعطل-أو-تموت",
    "android-app-crash": "android-التطبيق-يتعطل-أو-تموت",
    "android-connect-crash": "android-التطبيق-يتعطل-أو-تموت",
    "android-launch-crash": "android-التطبيق-يتعطل-فوراً",
    "android-black-screen": "android-شاشة-سوداء",
    "android-no-hosts": "android-قائمة-الاكتشاف",
    "android-discovery": "android-قائمة-الاكتشاف",
    "android-touch-offset": "android-اللمس-مُدوَّر",
    "android-rotation": "android-اللمس-مُدوَّر",
    "android-control-404": "android-إجراءات-شريط-التحكم",
    "android-404": "android-إجراءات-شريط-التحكم",
    "android-toolbar-404": "android-إجراءات-شريط-التحكم",
    "android-looking-for-host": "android-اتصال-usb",
    "android-usb": "android-اتصال-usb",
    "android-usb-host": "android-اتصال-usb",
    "android-client-issues": "أجهزة-وعميل-android",
    "streaming-wifi-latency": "البث-بطء-شديد",
    "streaming-latency": "البث-بطء-شديد",
    "streaming-reconnect": "البث-وميض-وإعادة-اتصال",
    "streaming-flicker": "البث-وميض-وإعادة-اتصال",
    "stream-disconnect-retry": "البث-وميض-وإعادة-اتصال",
    "cursor-clamping": "تعدد-الشاشات",
    "cursor-escape": "تعدد-الشاشات",
    "wrong-screen": "العميل-يعرض-الشاشة-الخطأ",
    "web-no-picture": "عميل-الويب",
    "no-encoder": "لا-يوجد-مُرمَّز",
    "token-401": "رفض-401",
    "unauthorized": "رفض-401",
    "dbus-missing": "الـ-daemon",
    "daemon-cpu": "الـ-daemon",
    "local-build": "مشاكل-البناء-المحلي",
    "local-build-issues": "مشاكل-البناء-المحلي",
    "local-sdk": "لم-يُعثر-على-android-sdk",
    "local-uinput": "رفض-صلاحية-uinput",
    "local-perm": "رفض-صلاحية-uinput",
    "net-timeout": "مشاكل-الشبكة-والاتصال",
    "network--connection-issues": "مشاكل-الشبكة-والاتصال",
    "comp-mutter": "gnome-wayland-mutter",
    "comp-x11": "x11",
    "comp-wlroots": "sway-hyprland-wlroots",
    "compositor-specific-issues": "المُركِّبات",
    "app-sw-decoder": "فيديو",
    "verify-stream": "verify-stream",
    "setup-dev-env": "setup-dev-env",
    "still-stuck": "ما-زلت-عالقاً",
}


def slugify(title: str) -> str:
    s = title.strip().lower().replace("`", "")
    s = re.sub(r"[^\w\s\u0600-\u06ff-]", "", s)
    return re.sub(r"\s+", "-", s)


def headings(text: str):
    return [slugify(m.group(1)) for m in re.finditer(r"^#{1,6}\s+(.+?)\s*$", text, re.M)]


def main() -> int:
    rc = 0
    for name in sys.argv[1:]:
        table = ARABIC if name.endswith("_AR.md") else ENGLISH
        path = pathlib.Path(name)
        text = path.read_text(encoding="utf-8")
        heads = headings(text)
        known = set(heads)
        repointed = 0
        for legacy, target in table.items():
            if not re.search(rf"\]\(#{re.escape(legacy)}\)", text):
                continue
            dest = target if target in known else next((h for h in heads if h.startswith(target)), None)
            if dest:
                text = re.sub(rf"\]\(#{re.escape(legacy)}\)", f"](#{dest})", text)
                repointed += 1
        path.write_text(text, encoding="utf-8")
        known = set(headings(text))
        dangling = sorted({l for l in re.findall(r"\]\(#([^)]+)\)", text) if l.lower() not in known})
        print(f"{name}: repointed {repointed}, dangling {len(dangling)}")
        for link in dangling:
            print(f"   ! {link}")
        rc |= 1 if dangling else 0
    return rc


if __name__ == "__main__":
    sys.exit(main())