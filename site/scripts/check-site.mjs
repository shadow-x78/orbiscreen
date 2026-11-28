#!/usr/bin/env node
// ─────────────────────────────────────────────
// Orbiscreen - Pre-deploy site checks
// https://github.com/shadow-x78/orbiscreen
// ─────────────────────────────────────────────

import { readFileSync, readdirSync, existsSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, resolve, relative, sep } from 'node:path';

const here = dirname(fileURLToPath(import.meta.url));
const siteRoot = resolve(here, '..');
const dist = resolve(siteRoot, 'dist');
const SITE_URL = 'https://orbiscreen.com';

const errors = [];
const warnings = [];

function error(message) {
  errors.push(message);
}
function warn(message) {
  warnings.push(message);
}

function walk(dir, out = []) {
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const full = resolve(dir, entry.name);
    if (entry.isDirectory()) walk(full, out);
    else out.push(full);
  }
  return out;
}

if (!existsSync(dist)) {
  console.error('check-site: dist/ not found; run npm run build first');
  process.exit(1);
}

const files = walk(dist);
const htmlFiles = files.filter((file) => file.endsWith('.html'));
const rel = (file) => relative(dist, file).split(sep).join('/');

function urlForHtml(file) {
  let path = rel(file).replace(/\.html$/, '');
  if (path === 'index') return '/';
  if (path === 'ar/index') return '/ar';
  if (path.endsWith('/index')) path = path.slice(0, -'/index'.length);
  return `/${path}`;
}

function mirrorFor(url) {
  if (url === '/') return '/index.md';
  if (url === '/ar') return '/ar/index.md';
  if (url.endsWith('/')) return `${url}index.md`;
  return `${url}.md`;
}

function distPathForHref(href) {
  const clean = href.split('#')[0].split('?')[0];
  if (!clean || clean === '/') return resolve(dist, 'index.html');
  if (/\.(md|txt|xml|webmanifest|svg|png|jpg|jpeg|webp|avif|ico|json|mp4|webm|gif|pdf|zip)$/.test(clean)) {
    return resolve(dist, clean.replace(/^\//, ''));
  }
  const trimmed = clean.replace(/\/$/, '');
  if (trimmed === '') return resolve(dist, 'index.html');
  return resolve(dist, `${trimmed.replace(/^\//, '')}/index.html`);
}

const sitemapPath = resolve(dist, 'sitemap.xml');
if (!existsSync(sitemapPath)) {
  error('sitemap.xml is missing');
}
const sitemap = existsSync(sitemapPath) ? readFileSync(sitemapPath, 'utf8') : '';
const sitemapUrls = new Set([...sitemap.matchAll(/<loc>([^<]+)<\/loc>/g)].map((m) => m[1]));

const idCache = new Map();
function collectIds(file) {
  const html = readFileSync(file, 'utf8');
  const ids = new Set();
  const seen = new Map();
  for (const match of html.matchAll(/\sid="([^"]+)"/g)) {
    const id = match[1];
    seen.set(id, (seen.get(id) ?? 0) + 1);
    ids.add(id);
  }
  for (const [id, count] of seen) {
    if (count > 1) {
      error(`${relative(dist, file).split(sep).join('/')}: duplicate id "${id}" (${count}x); the anchor lands on the first one`);
    }
  }
  idCache.set(file, ids);
  return ids;
}

const pageUrls = new Map();
for (const file of htmlFiles) {
  const url = urlForHtml(file);
  pageUrls.set(url, file);
  if (url === '/404') continue;
  if (!sitemapUrls.has(`${SITE_URL}${url}`)) {
    error(`sitemap is missing ${SITE_URL}${url}`);
  }
}

const titles = new Map();
const descriptions = new Map();

for (const file of htmlFiles) {
  const name = rel(file);
  const html = readFileSync(file, 'utf8');
  const is404 = name === '404.html';

  const title = html.match(/<title>([\s\S]*?)<\/title>/i)?.[1]?.trim() ?? '';
  if (!title) error(`${name}: missing <title>`);
  else {
    if (titles.has(title)) error(`${name}: duplicate title with ${titles.get(title)}`);
    titles.set(title, name);
    if (title.length > 65) warn(`${name}: title is ${title.length} characters`);
  }

  const description = html.match(/<meta name="description" content="([^"]*)"/i)?.[1]?.trim() ?? '';
  if (!description) error(`${name}: missing meta description`);
  else {
    if (descriptions.has(description)) error(`${name}: duplicate description with ${descriptions.get(description)}`);
    descriptions.set(description, name);
    if (description.length < 110) warn(`${name}: description is ${description.length} characters`);
    if (description.length > 175) warn(`${name}: description is ${description.length} characters`);
  }

  const canonical = html.match(/<link rel="canonical" href="([^"]+)"/i)?.[1] ?? '';
  if (!is404) {
    if (!canonical) error(`${name}: missing canonical`);
    else if (!canonical.startsWith(SITE_URL)) error(`${name}: canonical is not on ${SITE_URL} (${canonical})`);
  }

  if (!is404) {
    for (const lang of ['en', 'ar', 'x-default']) {
      if (!new RegExp(`<link rel="alternate" hreflang="${lang}"`, 'i').test(html)) {
        error(`${name}: missing hreflang ${lang}`);
      }
    }
  }

  const h1s = html.match(/<h1\b/gi)?.length ?? 0;
  if (h1s !== 1) error(`${name}: expected exactly one <h1>, found ${h1s}`);

  const levels = [...html.matchAll(/<h([1-6])\b/gi)].map((match) => Number(match[1]));
  for (let index = 1; index < levels.length; index += 1) {
    if (levels[index] - levels[index - 1] > 1) {
      error(`${name}: heading level jumps from h${levels[index - 1]} to h${levels[index]}`);
    }
  }

  for (const match of html.matchAll(/aria-controls="([^"]+)"/g)) {
    for (const id of match[1].split(/\s+/)) {
      if (id && !new RegExp(`\\sid="${id.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')}"`).test(html)) {
        error(`${name}: aria-controls points at missing id "${id}"`);
      }
    }
  }

  for (const img of html.matchAll(/<img\b[^>]*>/gi)) {
    if (!/\balt=/.test(img[0])) error(`${name}: <img> without alt: ${img[0].slice(0, 80)}`);
  }

  for (const script of html.matchAll(/<script type="application\/ld\+json"[^>]*>([\s\S]*?)<\/script>/gi)) {
    try {
      JSON.parse(script[1]);
    } catch (failure) {
      error(`${name}: invalid JSON-LD (${failure.message})`);
    }
  }

  if (!is404) {
    const markdownPath = mirrorFor(urlForHtml(file));
    if (!existsSync(distPathForHref(markdownPath))) {
      error(`${name}: missing Markdown mirror ${markdownPath}`);
    }
    if (!new RegExp(`type="text/markdown"`, 'i').test(html)) {
      error(`${name}: missing text/markdown alternate link`);
    }
  }

  for (const link of html.matchAll(/<a\b[^>]*href="([^"]+)"/gi)) {
    const href = link[1];
    if (/^(https?:|mailto:|tel:|data:)/i.test(href)) continue;
    if (href === '#' || href === '') continue;
    const samePage = href.startsWith('#');
    const target = samePage ? file : distPathForHref(href);
    if (!existsSync(target)) {
      error(`${name}: broken internal link ${href}`);
      continue;
    }
    if (!href.includes('#')) continue;
    const fragment = href.split('#')[1];
    if (!fragment) continue;
    const ids = idCache.get(target) ?? collectIds(target);
    let decoded = fragment;
    try {
      decoded = decodeURIComponent(fragment);
    } catch {

    }
    if (!ids.has(decoded)) error(`${name}: broken anchor ${href}`);
  }
}

for (const [url, file] of pageUrls) {
  if (url === '/404') continue;
  const html = readFileSync(file, 'utf8');
  const en = html.match(/<link rel="alternate" hreflang="en" href="([^"]+)"/i)?.[1];
  const ar = html.match(/<link rel="alternate" hreflang="ar" href="([^"]+)"/i)?.[1];
  if (!en || !ar) continue;
  for (const target of [en, ar]) {
    const path = target.replace(SITE_URL, '');
    if (!existsSync(distPathForHref(path))) {
      error(`${rel(file)}: hreflang target ${path} has no page`);
    }
  }
}

for (const required of ['llms.txt', 'llms-full.txt', 'robots.txt', 'rss.xml', 'sitemap-index.xml', 'site.webmanifest']) {
  if (!existsSync(resolve(dist, required))) error(`missing ${required}`);
}

const robots = existsSync(resolve(dist, 'robots.txt')) ? readFileSync(resolve(dist, 'robots.txt'), 'utf8') : '';
for (const bot of ['GPTBot', 'OAI-SearchBot', 'ClaudeBot', 'PerplexityBot', 'Google-Extended', 'CCBot']) {
  if (!robots.includes(`User-agent: ${bot}`)) error(`robots.txt does not name ${bot}`);
}
if (!robots.includes(`Sitemap: ${SITE_URL}/sitemap-index.xml`)) error('robots.txt does not point at the sitemap');

function checkXml(file, { requireItems = false } = {}) {
  if (!existsSync(file)) return;
  const text = readFileSync(file, 'utf8');
  const name = relative(dist, file).split(sep).join('/');
  if (!text.startsWith('<?xml')) error(`${name}: missing XML declaration`);

  for (const match of text.matchAll(/&(?!(?:amp|lt|gt|quot|apos|#\d+|#x[0-9a-fA-F]+);)/g)) {
    const at = text.slice(Math.max(0, match.index - 40), match.index + 40).replace(/\s+/g, ' ');
    error(`${name}: unescaped & near "${at}"`);
    break;
  }
  if (/Invalid Date/.test(text)) error(`${name}: contains "Invalid Date"`);
  if (requireItems && !/<item>/.test(text)) error(`${name}: has no items`);
}

checkXml(resolve(dist, 'sitemap.xml'));
checkXml(resolve(dist, 'sitemap-index.xml'));
checkXml(resolve(dist, 'rss.xml'), { requireItems: true });

const sitemapFile = resolve(dist, 'sitemap.xml');
if (existsSync(sitemapFile)) {
  for (const match of readFileSync(sitemapFile, 'utf8').matchAll(/<loc>([^<]+)<\/loc>/g)) {
    if (!match[1].startsWith(`${SITE_URL}/`)) error(`sitemap.xml: loc is not on ${SITE_URL}: ${match[1]}`);
  }
}

const headersFile = resolve(dist, '_headers');
if (existsSync(headersFile)) {
}

for (const message of warnings) console.warn(`warn  ${message}`);
for (const message of errors) console.error(`error ${message}`);

console.log(
  `check-site: ${htmlFiles.length} pages, ${sitemapUrls.size} sitemap URLs, ${warnings.length} warnings, ${errors.length} errors`,
);

if (errors.length > 0) process.exit(1);