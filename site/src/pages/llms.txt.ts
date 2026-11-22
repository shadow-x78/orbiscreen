import type { APIRoute } from 'astro';
import docsMeta from '../data/docs-meta.json';
import { GITHUB_URL, RELEASES_URL, SITE_URL, SHORT_DESCRIPTION } from '../lib/site';
import { RUST_VERSION, VERSION } from '../lib/version';

export const GET: APIRoute = () => {
  const docs = docsMeta.docs
    .map((doc) => `- [${doc.title}](${SITE_URL}/docs/${doc.slug}): ${doc.description}`)
    .join('\n');
  const docsAr = docsMeta.docs
    .map((doc) => `- [${doc.titleAr}](${SITE_URL}/ar/docs/${doc.slug}): ${doc.descriptionAr}`)
    .join('\n');

  const body = `# Orbiscreen

> ${SHORT_DESCRIPTION}

Orbiscreen is a Rust workspace that runs one daemon on a Linux host and streams a virtual monitor to an Android tablet, phone, or Chromium browser. It is not a mirror and not a remote desktop: the host creates an independent virtual output, encodes it as H.264, and injects the client input back as native Linux events.

## Facts

- Version: ${VERSION}
- License: GPL-3.0-or-later
- Host platforms: Linux on Wayland and X11 (KDE Plasma, GNOME 46+, COSMIC, Sway, Hyprland, XFCE, MATE, LXQt, Cinnamon)
- Clients: Android app (Kotlin, Jetpack Compose, Material 3) and Chromium browsers (WebTransport and WebCodecs)
- Transports: UDP Annex-B with Reed-Solomon FEC, WebTransport, USB Android Open Accessory, HTTP MPEG-TS fallback
- Encoders: NVENC, VA-API, x264
- Measured transport delay: 2-4 ms on a 5 GHz LAN
- Measured glass-to-glass latency: 25-40 ms on a tuned 5 GHz Wi-Fi link
- Maximum virtual display resolution: 7680x4320
- Stylus: 4095 pressure levels, tilt, and in-air hover through uinput
- Root: not required on KDE Plasma and wlroots; EVDI provides the display on GNOME, COSMIC and X11
- Source language: Rust ${RUST_VERSION}+
- Repository: ${GITHUB_URL}
- Releases: ${RELEASES_URL}

## Pages

- [Home](${SITE_URL}/): what Orbiscreen does, how it streams, and the supported desktops
- [Features](${SITE_URL}/features): virtual display paths, encoding profile, input injection, transports, clients, control and security
- [Setup](${SITE_URL}/install): package commands for Ubuntu, Fedora, Arch, AppImage and source, plus the Android client
- [Compare](${SITE_URL}/compare): capability table against Spacedesk, Deskreen, Weylus and Sidecar, with the measurement method
- [FAQ](${SITE_URL}/faq): extended versus mirrored, Wayland without root, stylus, USB, latency, tokens, privacy
- [Changelog](${SITE_URL}/changelog): release notes with the reasoning behind each change
- [Security](${SITE_URL}/security): private disclosure channel, scope, supported versions
- [License](${SITE_URL}/license): GPL-3.0-or-later in practice
- [Privacy](${SITE_URL}/privacy): no cookies, no accounts, no telemetry

## Documentation

${docs}

## الوثائق بالعربية

- [الرئيسية](${SITE_URL}/ar)
- [المزايا](${SITE_URL}/ar/features)
- [التثبيت](${SITE_URL}/ar/install)
- [المقارنة](${SITE_URL}/ar/compare)
- [الأسئلة](${SITE_URL}/ar/faq)
- [سجل التغييرات](${SITE_URL}/ar/changelog)

${docsAr}

## Machine-readable

- [Sitemap](${SITE_URL}/sitemap.xml)
- [RSS feed](${SITE_URL}/rss.xml)
- Markdown mirrors: append \`.md\` to any page path, for example ${SITE_URL}/features.md or ${SITE_URL}/docs/architecture.md
- Content negotiation: the same markup is returned when a request sends \`Accept: text/markdown\`, and every HTML response advertises its mirror in a \`Link: rel="alternate"; type="text/markdown"\` header
- [Full text dump](${SITE_URL}/llms-full.txt)
`;

  return new Response(body, {
    headers: { 'Content-Type': 'text/plain; charset=utf-8' },
  });
};