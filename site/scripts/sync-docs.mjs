#!/usr/bin/env node
// ─────────────────────────────────────────────
// Orbiscreen - Documentation sync
// docs/*.md (EN + AR) -> src/content/docs
// https://github.com/shadow-x78/orbiscreen
// ─────────────────────────────────────────────

import {
  readFileSync,
  writeFileSync,
  mkdirSync,
  rmSync,
  existsSync,
  readdirSync,
  statSync,
} from 'node:fs';
import { execFileSync } from 'node:child_process';
import GithubSlugger from 'github-slugger';
import { fileURLToPath } from 'node:url';
import { dirname, resolve, basename } from 'node:path';

const here = dirname(fileURLToPath(import.meta.url));
const siteRoot = resolve(here, '..');
const repoRoot = resolve(siteRoot, '..');
const docsDir = resolve(repoRoot, 'docs');
const changelogFile = resolve(repoRoot, 'CHANGELOG.md');
const outDir = resolve(siteRoot, 'src/content/docs');
const metaPath = resolve(siteRoot, 'src/data/docs-meta.json');

const meta = JSON.parse(readFileSync(metaPath, 'utf8')).docs;
const bySource = new Map(meta.map((entry) => [entry.source, entry]));

function fail(message) {
  console.error(`sync-docs: ${message}`);
  process.exit(1);
}

/**
 * The newest date in CHANGELOG.md. Used as a deterministic fallback when git
 * history is unavailable (Cloudflare Pages clones shallow by default), so a
 * build never invents "today" as the modification date.
 */
function newestChangelogDate() {
  try {
    const source = readFileSync(changelogFile, 'utf8');
    for (const line of source.split(/\r?\n/)) {
      const match = line.match(/^##\s+\[?v?[0-9]+\.[0-9]+\.[0-9]+[^\]]*\]?\s+-\s+(\d{4}-\d{2}-\d{2})\s*$/);
      if (match) return match[1];
    }
  } catch {
    /* changelog missing */
  }
  return null;
}

function git(args, options = {}) {
  return execFileSync('git', args, {
    cwd: repoRoot,
    encoding: 'utf8',
    stdio: ['ignore', 'pipe', 'ignore'],
    ...options,
  }).trim();
}

/**
 * CI providers clone with depth 1 by default (Cloudflare Pages does), which
 * leaves `git log -- <file>` empty for every document the tip commit did not
 * touch. Instead of requiring a custom build command, deepen the clone here so
 * each page gets its own commit date. Best effort: a read-only checkout or an
 * offline build keeps the CHANGELOG fallback below.
 */
function ensureFullHistory() {
  let shallow;
  try {
    shallow = git(['rev-parse', '--is-shallow-repository']) === 'true';
  } catch {
    // Not a git checkout at all: nothing to deepen.
    return;
  }
  if (!shallow) return;

  try {
    git(['fetch', '--unshallow', '--quiet', '--no-tags'], { timeout: 180_000 });
    console.log('sync-docs: deepened the shallow clone so document dates come from real commits');
  } catch {
    console.warn(
      'sync-docs: could not deepen the shallow clone (offline or read-only checkout); ' +
        'document dates fall back to the newest CHANGELOG entry',
    );
  }
}

ensureFullHistory();

const fallbackDate = newestChangelogDate();
let shallowHistory = false;

function lastCommitDate(file) {
  try {
    const out = git(['log', '-1', '--format=%cs', '--', file]);
    if (/^\d{4}-\d{2}-\d{2}$/.test(out)) return out;
  } catch {
    /* git unavailable */
  }

  // No commit history for this file. Prefer a content-derived date over the
  // checkout clock, and say so once so a real fix (fetch --unshallow) is visible.
  shallowHistory = true;
  if (fallbackDate) return fallbackDate;
  return statSync(file).mtime.toISOString().slice(0, 10);
}

function fenceMask(lines) {
  const mask = new Array(lines.length).fill(false);
  let openFence = null;
  for (let i = 0; i < lines.length; i += 1) {
    const line = lines[i];
    const fence = line.match(/^\s*(`{3,}|~{3,})/);
    if (fence) {
      const marker = fence[1];
      if (openFence === null) {
        openFence = { char: marker[0], length: marker.length };
        mask[i] = true;
        continue;
      }
      if (marker[0] === openFence.char && marker.length >= openFence.length) {
        openFence = null;
        mask[i] = true;
        continue;
      }
    }
    mask[i] = openFence !== null;
  }
  return mask;
}

function headingIndex(lines, mask, pattern, from = 0) {
  for (let i = from; i < lines.length; i += 1) {
    if (!mask[i] && pattern.test(lines[i])) return i;
  }
  return -1;
}

function removeSection(lines, pattern) {
  const mask = fenceMask(lines);
  const start = headingIndex(lines, mask, pattern);
  if (start === -1) return lines;
  let end = lines.length;
  for (let i = start + 1; i < lines.length; i += 1) {
    if (!mask[i] && /^#{1,2}\s/.test(lines[i])) {
      end = i;
      break;
    }
  }
  return [...lines.slice(0, start), ...lines.slice(end)];
}

/**
 * Removes the centred badge wrapper that opens every document: the
 * `<div align="center">` block that contains the H1 and the shields.io badges.
 * Content divs further down (direction wrappers around code and tables) are
 * left untouched.
 */
function stripBadgeBlock(lines) {
  const open = lines.findIndex((line) => /^\s*<div\s+align="center"\s*>\s*$/i.test(line));
  if (open === -1) return lines;

  let close = -1;
  for (let i = open + 1; i < lines.length; i += 1) {
    if (/^\s*<\/div>\s*$/i.test(lines[i])) {
      close = i;
      break;
    }
  }
  if (close === -1) return lines;

  const inner = lines.slice(open + 1, close);
  const looksLikeBadges = inner.every(
    (line) =>
      /^\s*$/.test(line) ||
      /^#\s+\S/.test(line) ||
      /^\s*\[!\[[^\]]*\]\([^)]*\)\]\([^)]*\)\s*$/.test(line) ||
      /^\s*!\[[^\]]*\]\([^)]*\)\s*$/.test(line),
  );
  if (!looksLikeBadges) return lines;

  // The H1 lives inside the badge block and is the page title; keep it.
  const kept = inner.filter((line) => /^#\s+\S/.test(line));
  return [...lines.slice(0, open), ...kept, ...lines.slice(close + 1)];
}

/**
 * Drops the standalone language links that some documents place above the
 * introduction. The site already exposes a language switch in the navbar, so
 * the duplicated row of flags is removed from the generated pages.
 */
function stripLanguageSwitcher(lines) {
  const full = /^\s*<a\s+href="[^"]+">(?:🇬🇧\s*)?English<\/a>\s*·\s*<a\s+href="[^"]+">(?:🇸🇦\s*)?العربية<\/a>\s*$/u;
  const englishOnly = /^\s*<a\s+href="[^"]+">English<\/a>\s*$/u;
  return lines.filter((line) => !full.test(line) && !englishOnly.test(line));
}

/**
 * Some Arabic documents mark sections with raw HTML (`<h2 dir="rtl">…</h2>`)
 * instead of Markdown. Astro only assigns heading ids to Markdown headings, so
 * those sections would be missing from the table of contents and could not be
 * linked. Convert them, keeping their level, and drop the direction marks that
 * the page direction already provides.
 */
function convertHtmlHeadings(lines) {
  const mask = fenceMask(lines);
  return lines.map((line, index) => {
    if (mask[index]) return line;
    const match = line.match(/^\s*<h([1-6])(?:\s[^>]*)?>([\s\S]*?)<\/h\1>\s*$/i);
    if (!match) return line;
    const text = match[2]
      .replace(/<a\s+id="[^"]*"><\/a>/gi, '')
      .replace(/<[^>]+>/g, '')
      .replace(/&(?:rlm|lrm);|&#820[67];/gi, '')
      .replace(/\s+/g, ' ')
      .trim();
    if (!text) return line;
    return `${'#'.repeat(Number(match[1]))} ${text}`;
  });
}

function rewriteTarget(target, prefix) {
  if (/^(https?:|mailto:|tel:|#)/.test(target)) return target;
  const [file, hash] = target.split('#');
  const suffix = hash ? `#${hash}` : '';

  if (file === '../CHANGELOG.md') return `${prefix}/changelog${suffix}`;
  if (file === '../LICENSE') return `${prefix}/license${suffix}`;
  if (file === '../README.md') return `${prefix === '' ? '/' : prefix}${suffix}`;
  if (file === '../README_AR.md') return `/ar${suffix}`;

  const match = file.match(/^([A-Z0-9_]+?)(_AR)?\.md$/);
  if (match) {
    const entry = bySource.get(match[1]);
    if (!entry) fail(`no docs-meta entry for ${file}`);
    return `${prefix}/docs/${entry.slug}${suffix}`;
  }

  return target;
}

/** Rewrites links one line at a time so fenced code and inline code stay intact. */
function rewriteLinks(text, prefix) {
  return text
    .split('\n')
    .map((line) => {
      if (/^\s*(`{3,}|~{3,})/.test(line)) return line;
      return line
        .split(/(`[^`]*`)/)
        .map((segment) => {
          if (segment.startsWith('`') && segment.endsWith('`') && segment.length > 1) return segment;
          return segment
            .replace(/\]\(([^)\s]+)\)/g, (_whole, target) => `](${rewriteTarget(target, prefix)})`)
            .replace(/href="([^"]+\.md)(#[^"]*)?"/g, (_whole, file, hash) =>
              `href="${rewriteTarget(`${file}${hash ?? ''}`, prefix)}"`,
            );
        })
        .join('');
    })
    .join('\n');
}

function transform(source, lang, entry, updated) {
  const prefix = lang === 'ar' ? '/ar' : '';
  const raw = readFileSync(source, 'utf8');

  let lines = raw.split(/\r?\n/);

  lines = stripBadgeBlock(lines);
  lines = stripLanguageSwitcher(lines);
  lines = removeSection(lines, /^##\s+(Language|اللغة)\s*$/);
  lines = removeSection(lines, /^##\s+(Table of Contents|المحتويات)\s*$/);

  // Trim leading separators and blank lines left behind.
  while (lines.length > 0 && (/^\s*$/.test(lines[0]) || /^---+$/.test(lines[0]))) lines.shift();

  // The document's own H1 becomes the page title.
  const mask = fenceMask(lines);
  const titleIndex = headingIndex(lines, mask, /^#\s+\S/);
  if (titleIndex === -1) fail(`${basename(source)}: no H1 title`);
  const rawTitle = lines[titleIndex].replace(/^#\s+/, '').trim();
  const title = rawTitle.replace(/\s*[-—]\s*Orbiscreen\s*$/iu, '').trim() || rawTitle;
  lines.splice(titleIndex, 1);
  lines = convertHtmlHeadings(lines);

  while (lines.length > 0 && (/^\s*$/.test(lines[0]) || /^---+$/.test(lines[0]))) lines.shift();

  // Normalise heading depth so top-level sections become H2.
  const maskAfter = fenceMask(lines);
  let minLevel = 7;
  for (let i = 0; i < lines.length; i += 1) {
    if (maskAfter[i]) continue;
    const m = lines[i].match(/^(#{1,6})\s/);
    if (m) minLevel = Math.min(minLevel, m[1].length);
  }
  if (minLevel < 7 && minLevel !== 2) {
    const shift = 2 - minLevel;
    for (let i = 0; i < lines.length; i += 1) {
      if (maskAfter[i]) continue;
      const m = lines[i].match(/^(#{1,6})\s/);
      if (!m) continue;
      const level = Math.min(6, m[1].length + shift);
      lines[i] = `${'#'.repeat(level)}${lines[i].slice(m[1].length)}`;
    }
  }

  // Collect the heading slugs Astro will generate, then drop any explicit
  // `<a id>` that would shadow one: a duplicate id makes the fragment land on
  // the anchor instead of the section.
  const slugger = new GithubSlugger();
  const anchors = new Set();
  const headingSlugs = new Set();
  const maskFinal = fenceMask(lines);
  for (let i = 0; i < lines.length; i += 1) {
    if (maskFinal[i]) continue;
    const heading = lines[i].match(/^#{1,6}\s+(.+)$/);
    if (heading) {
      const slug = slugger.slug(heading[1].replace(/`/g, ''));
      headingSlugs.add(slug);
      anchors.add(slug);
    }
  }

  // Explicit anchors keep the first occurrence only: a repeated id (the source
  // has a few, and a translation may copy them) would make the fragment land on
  // the wrong section, and a duplicate id is invalid HTML.
  const seenAnchors = new Set(headingSlugs);
  for (let i = 0; i < lines.length; i += 1) {
    if (maskFinal[i]) continue;
    const explicit = [...lines[i].matchAll(/<a\s+id="([^"]+)"><\/a>/g)];
    if (explicit.length === 0) continue;
    let line = lines[i];
    for (const [whole, id] of explicit) {
      if (seenAnchors.has(id)) {
        line = line.replace(whole, '');
        continue;
      }
      seenAnchors.add(id);
      anchors.add(id);
    }
    lines[i] = line.trim() === '' ? '' : line;
  }

  let body = rewriteLinks(lines.join('\n'), prefix);
  body = body.replace(/\n{3,}/g, '\n\n').replace(/^\s+/, '').trimEnd();

  const description = lang === 'ar' ? entry.descriptionAr : entry.description;
  const frontMatter = [
    '---',
    `title: ${JSON.stringify(title)}`,
    `description: ${JSON.stringify(description)}`,
    `lang: ${JSON.stringify(lang)}`,
    `slug: ${JSON.stringify(entry.slug)}`,
    `source: ${JSON.stringify(basename(source))}`,
    `updated: ${JSON.stringify(updated)}`,
    '---',
    '',
    '',
  ].join('\n');

  return { markdown: `${frontMatter}${body}\n`, title, anchors };
}

if (!existsSync(docsDir)) fail(`docs directory not found at ${docsDir}`);

rmSync(outDir, { recursive: true, force: true });
mkdirSync(resolve(outDir, 'ar'), { recursive: true });

const known = new Set(meta.flatMap((entry) => [`${entry.source}.md`, `${entry.source}_AR.md`]));
for (const name of readdirSync(docsDir)) {
  if (!name.endsWith('.md')) continue;
  if (!known.has(name)) fail(`docs/${name} has no docs-meta entry`);
}

const anchorsByKey = new Map();
const rendered = [];

for (const entry of meta) {
  const enFile = resolve(docsDir, `${entry.source}.md`);
  const arFile = resolve(docsDir, `${entry.source}_AR.md`);
  if (!existsSync(enFile)) fail(`missing English source ${entry.source}.md`);
  if (!existsSync(arFile)) fail(`missing Arabic source ${entry.source}_AR.md`);

  for (const [lang, file] of [['en', enFile], ['ar', arFile]]) {
    const result = transform(file, lang, entry, lastCommitDate(file));
    anchorsByKey.set(`${lang}:${entry.slug}`, result.anchors);
    rendered.push({ lang, slug: entry.slug, markdown: result.markdown });
  }
}

// Cross-document links keep their fragment only when the target document has
// that anchor. A translation that lags behind the English source loses the
// fragment instead of shipping a dead jump; the warning names the page.
const droppedAnchors = [];
for (const page of rendered) {
  page.markdown = page.markdown.replace(
    /(\]\()(\/ar)?\/docs\/([a-z0-9-]+)#([^)\s]+)(\))/g,
    (whole, open, arPrefix, targetSlug, rawHash, close) => {
      const lang = arPrefix ? 'ar' : 'en';
      const anchors = anchorsByKey.get(`${lang}:${targetSlug}`);
      if (!anchors) return whole;
      let hash = rawHash;
      try {
        hash = decodeURIComponent(rawHash);
      } catch {
        /* keep the raw fragment */
      }
      if (anchors.has(hash)) return whole;
      droppedAnchors.push(`${page.lang}/docs/${page.slug} -> ${lang}/docs/${targetSlug}#${hash}`);
      return `${open}${arPrefix ?? ''}/docs/${targetSlug}${close}`;
    },
  );
}

let written = 0;
for (const page of rendered) {
  const outFile =
    page.lang === 'ar'
      ? resolve(outDir, 'ar', `${page.slug}.md`)
      : resolve(outDir, `${page.slug}.md`);
  writeFileSync(outFile, page.markdown, 'utf8');
  written += 1;
}

if (shallowHistory) {
  console.warn(
    'sync-docs: no git history for some documents (shallow clone?); using the newest CHANGELOG date. ' +
      'Run `git fetch --unshallow` before building to get per-file dates.',
  );
}

for (const item of droppedAnchors) {
  console.warn(`sync-docs: anchor missing, linking to the page instead: ${item}`);
}

console.log(`sync-docs: wrote ${written} pages for ${meta.length} documents`);
