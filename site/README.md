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
| `/wrangler.jsonc` | serves `./site/dist` as static assets, drops trailing slashes, uses the custom 404, and attaches `orbiscreen.com` plus `www.orbiscreen.com` as custom domains |
| `/worker/index.js` | redirects `www` to the apex with a 301, then hands every other request to the assets binding |

Cloudflare Workers Builds (or Pages) settings:

- Repository `shadow-x78/orbiscreen`, production branch `main`.
- **Root directory**: `/` — leave it. If your project allows editing it, `site`
  also works with `npm ci && npm run build` and `npx wrangler deploy`.
- **Build command**: `npm ci && npm run build`
- **Deploy command**: `npx wrangler deploy`
- **Version command**: `npx wrangler versions upload`
- **Variable**: `NODE_VERSION` = `22`

No custom history command is needed: `sync-docs` dates each document from its
last commit, and when it finds a shallow clone it runs `git fetch --unshallow`
itself. If the fetch is impossible it falls back to the newest `CHANGELOG.md`
date and prints a warning instead of inventing "today".

Then attach `orbiscreen.com` and `www.orbiscreen.com`, redirect `www` to the
apex, and enable Web Analytics for the project. Nothing in the build needs a
secret.

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
