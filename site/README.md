# orbiscreen.com

The marketing and documentation site for Orbiscreen, built with Astro as fully
static output. English lives at the root; Arabic lives under `/ar`.

Everything it publishes comes from this repository, so nothing drifts:

| Source | Becomes |
| --- | --- |
| `docs/*.md` and `docs/*_AR.md` | `/docs/{slug}` and `/ar/docs/{slug}` |
| `CHANGELOG.md` | `/changelog`, `/ar/changelog`, `/rss.xml` |
| `Cargo.toml` (`workspace.package.version`) | every version badge and `softwareVersion` |
| `assets/logo` and `gui/icons` | favicon, app icons, social preview |

## Commands

```bash
npm ci            # install
npm run sync:docs # docs/*.md -> src/content/docs (also runs before build)
npm run dev       # local server on :4321
npm run build     # sync, build, then emit headers and run the site gates
npm run check     # astro check (types)
npm run verify    # the site gates on their own (needs dist/)
```

`npm run build` is the gate that must pass. It fails on a broken internal link,
a broken or duplicate anchor, a missing Markdown mirror, a duplicate title or
description, a missing `hreflang`, a missing `llms.txt`, malformed sitemap or RSS
XML, an RSS item that points at a release the page does not render, an inline
script the generated CSP does not allow, a skipped heading level, a missing
`aria-controls` target, or more or fewer than one `<h1>` on a page.

## What gets generated

- `sitemap.xml` and `sitemap-index.xml` with `hreflang` alternates.
- `llms.txt` (curated index) and `llms-full.txt` (every page as plain text).
- A Markdown mirror at `<path>.md` for every page, advertised with
  `<link rel="alternate" type="text/markdown">`.
- `_headers` with a hash-based CSP built from the scripts in `dist`, plus HSTS
  and the other security headers.
- `_redirects` for legacy document paths.
- `public/robots.txt`, `public/.well-known/security.txt`, `public/site.webmanifest`.
- `public/<key>.txt`: the IndexNow key. `.github/workflows/indexnow.yml` submits
  every sitemap URL to `api.indexnow.org` after a deploy, which is what reaches
  Bing, Yandex, Seznam and Naver without an account. Google still needs the
  Search Console property to be added once.

## Adding a document

1. Add `docs/NAME.md` and `docs/NAME_AR.md`.
2. Add an entry to `src/data/docs-meta.json` (`slug`, `source`, titles,
   descriptions, group).
3. Run `npm run build`.

The build fails if a document has no Arabic pair or no metadata entry. A
translation that lags behind on a heading anchor loses only the fragment: the
link still points at the page, and the build prints which one.

## Deploying

The repository root carries a small entry point so a builder that starts at `/`
works without any extra configuration:

| File | Purpose |
| --- | --- |
| `/package.json` | `build` runs `cd site && npm ci && npm run build`; `deploy` runs `wrangler deploy` |
| `/package-lock.json` | pins the `wrangler` CLI used by the deploy step |
| `/wrangler.jsonc` | serves `./site/dist` as static assets, drops trailing slashes, uses the custom 404, and routes `orbiscreen.com/*` and `www.orbiscreen.com/*` to the worker |
| `/worker/index.js` | redirects `www` to the apex with a 301, then hands every other request to the assets binding |

Cloudflare Workers Builds (or Pages) settings:

- Repository `shadow-x78/orbiscreen`, production branch `main`.
- **Root directory**: `/` — leave it. If your project allows editing it, `site`
  also works with `npm ci && npm run build` and `npx wrangler deploy`.
- **Build command**: `npm ci && npm run build`
- **Deploy command**: `npx wrangler deploy`
- **Version command**: `npx wrangler versions upload`
- **Variable**: `NODE_VERSION` = `22`

`workers_dev` and `preview_urls` are both off, so the worker answers only on the
two routes. Nothing serves the site from a second host, and there is no preview
address to leak into search results.

No custom history command is needed: `sync-docs` dates each document from its
last commit, and when it finds a shallow clone it runs `git fetch --unshallow`
itself. If the fetch is impossible it falls back to the newest `CHANGELOG.md`
date and prints a warning instead of inventing "today".

Deploying is the whole setup: the routes in `wrangler.jsonc` attach both
hostnames, and the worker redirects `www` to the apex with a 301. Nothing in the
build needs a secret.

The two hostnames use Worker routes rather than Workers custom domains because
the zone still holds the A record imported from the previous registrar, and a
custom domain cannot attach over an externally managed record. A route only
needs the record to exist and be proxied, which it is. If that A record is ever
deleted in the dashboard, both entries can become `custom_domain: true`.

### Bot Preference Sync overwrites robots.txt

AI Crawl Control's **Bot Preference Sync** prepends a managed block to
`robots.txt` that disallows most AI crawlers (`Content-Signal: ai-train=no`)
before the repository's own rules. When two groups name the same user agent,
RFC 9309 merges them and the first `Disallow` wins, so the site's `Allow`
never takes effect for those agents.

Turn it off under **AI Crawl Control → Overview → Enable Bot Preference Sync**,
then confirm the served file starts with the site's own header:

```bash
curl -s https://orbiscreen.com/robots.txt | head -12
```

### Cloudflare blocks two of the crawlers robots.txt allows

On the live zone, `ClaudeBot` and `PerplexityBot` receive `403 Your request was
blocked` while `GPTBot`, `OAI-SearchBot`, `ChatGPT-User`, `Google-Extended`,
`CCBot`, `Applebot`, `Googlebot` and `Bingbot` receive `200`. That is Cloudflare's
AI bot blocking at the edge, before the worker runs, so nothing in this
repository can override it — and it defeats the point of `llms.txt` and the
Markdown mirrors for those two engines. Turn it off under **Security → Bots**
("Block AI bots" / AI Crawl Control) and re-check with:

```bash
for ua in ClaudeBot PerplexityBot GPTBot OAI-SearchBot; do
  printf '%-16s ' "$ua"; curl -s -A "$ua" -o /dev/null -w '%{http_code}\n' https://orbiscreen.com/features
done
```

### One zone setting fights the CSP

Cloudflare injects `/cdn-cgi/challenge-platform/scripts/jsd/main.js` as an inline
script when Bot Fight Mode (JavaScript Detections) is on. Its body carries a
per-request ray and timestamp, so no build-time hash can cover it, and the
browser logs one CSP violation per page. The site itself is unaffected.

Turn Bot Fight Mode off under **Security → Bots** on a static site: it adds
nothing here, and the CSP stays strict. Do not weaken `script-src` instead —
once a hash is present, `'unsafe-inline'` is ignored, so relaxing it would not
even silence the warning.

Locally, from the repository root:

```bash
npm ci            # the wrangler CLI for deploys
npm run build     # builds site/ and runs the gates
npm run dev       # wrangler dev serving site/dist on :8787
```

or from `site/` as before: `npm run build`, `npm run deploy`, `npm run worker`.

`.github/workflows/site.yml` runs the same build and gates on every change to
`site/`, `docs/`, `CHANGELOG.md`, or `Cargo.toml`, so a broken page cannot reach
`main` unnoticed.
