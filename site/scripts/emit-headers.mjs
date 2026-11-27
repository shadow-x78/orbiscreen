#!/usr/bin/env node
// ─────────────────────────────────────────────
// Orbiscreen - Post-build headers, CSP and llms-full
// https://github.com/shadow-x78/orbiscreen
// ─────────────────────────────────────────────

import { createHash } from 'node:crypto';
import { readFileSync, writeFileSync, readdirSync, existsSync, statSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, resolve, relative, sep } from 'node:path';

const here = dirname(fileURLToPath(import.meta.url));
const siteRoot = resolve(here, '..');
const dist = resolve(siteRoot, 'dist');
const SITE_URL = 'https://orbiscreen.com';

if (!existsSync(dist)) {
  console.error('emit-headers: dist/ not found; run astro build first');
  process.exit(1);
}

function walk(dir, out = []) {
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const full = resolve(dir, entry.name);
    if (entry.isDirectory()) walk(full, out);
    else out.push(full);
  }
  return out;
}

const files = walk(dist);
const htmlFiles = files.filter((file) => file.endsWith('.html'));
const mdFiles = files.filter((file) => file.endsWith('.md'));

const hashes = new Set();
for (const file of htmlFiles) {
  const html = readFileSync(file, 'utf8');

  for (const match of html.matchAll(
    /<script(?![^>]*\bsrc=)[^>]*>([\s\S]*?)<\/script>/gi,
  )) {
    const body = match[1];
    if (!body || body.trim() === '') continue;
    hashes.add(`'sha256-${createHash('sha256').update(body, 'utf8').digest('base64')}'`);
  }
}

const scriptHashes = [...hashes].sort().join(' ');

const csp = [
  "default-src 'self'",
  "base-uri 'self'",
  "object-src 'none'",
  "frame-ancestors 'none'",
  "form-action 'self'",
  `script-src 'self'${scriptHashes ? ` ${scriptHashes}` : ''} https://static.cloudflareinsights.com`,
  "style-src 'self' 'unsafe-inline'",
  "img-src 'self' data:",
  "font-src 'self'",
  "connect-src 'self' https://cloudflareinsights.com",
  "manifest-src 'self'",
  'upgrade-insecure-requests',
].join('; ');

const globalHeaders = [
  `  Content-Security-Policy: ${csp}`,
  '  Strict-Transport-Security: max-age=31536000; includeSubDomains',
  '  X-Content-Type-Options: nosniff',
  '  X-Frame-Options: DENY',
  '  Referrer-Policy: strict-origin-when-cross-origin',
  '  Permissions-Policy: camera=(), microphone=(), geolocation=(), payment=(), usb=(), interest-cohort=()',
  '  Cross-Origin-Opener-Policy: same-origin',
].join('\n');

function canonicalFor(file) {
  const rel = relative(dist, file).split(sep).join('/');
  const withoutExt = rel.replace(/\.md$/, '');
  if (withoutExt === 'index') return `${SITE_URL}/`;
  if (withoutExt === 'ar/index') return `${SITE_URL}/ar`;
  return `${SITE_URL}/${withoutExt}`;
}

const mirrorRules = mdFiles
  .map((file) => {
    const rel = relative(dist, file).split(sep).join('/');
    return `/${rel}\n  Content-Type: text/markdown; charset=utf-8\n  Link: <${canonicalFor(file)}>; rel="canonical"`;
  })
  .join('\n\n');

const headers = `# ─────────────────────────────────────────────
# Orbiscreen - Cloudflare Pages headers (generated)
# ─────────────────────────────────────────────

/*
${globalHeaders}

/_astro/*
  Cache-Control: public, max-age=31536000, immutable

# Icons and the social preview change only with a redesign.
/favicon.ico
  Cache-Control: public, max-age=604800

/favicon.svg
  Cache-Control: public, max-age=604800

/favicon-*.png
  Cache-Control: public, max-age=604800

/icon-*.png
  Cache-Control: public, max-age=604800

/apple-touch-icon.png
  Cache-Control: public, max-age=604800

/og/*
  Cache-Control: public, max-age=604800

${mirrorRules}
`;

writeFileSync(resolve(dist, '_headers'), headers, 'utf8');

function orderKey(file) {
  const rel = relative(dist, file).split(sep).join('/');
  if (rel === 'index.md') return '0000';
  if (rel.startsWith('ar/index')) return '0001';
  if (rel === 'features.md') return '0010';
  if (rel === 'install.md') return '0011';
  if (rel === 'compare.md') return '0012';
  if (rel === 'faq.md') return '0013';
  if (rel === 'docs.md') return '0020';
  if (rel.startsWith('docs/')) return `0030${rel}`;
  if (rel === 'changelog.md') return '0040';
  if (rel === 'license.md') return '0050';
  if (rel === 'security.md') return '0051';
  if (rel === 'privacy.md') return '0052';
  return `1000${rel}`;
}

const ordered = [...mdFiles].sort((a, b) => orderKey(a).localeCompare(orderKey(b)));
const full = [
  '# Orbiscreen: full text',
  '',
  `Generated from ${SITE_URL}. Every section below is also available as an HTML page and as a Markdown mirror.`,
  '',
  ...ordered.map((file) => {
    const rel = relative(dist, file).split(sep).join('/');
    return `\n\n<!-- ${rel} -->\n\n${readFileSync(file, 'utf8').trim()}\n`;
  }),
].join('');

writeFileSync(resolve(dist, 'llms-full.txt'), full, 'utf8');

const size = statSync(resolve(dist, 'llms-full.txt')).size;
console.log(
  `emit-headers: ${htmlFiles.length} pages, ${hashes.size} inline script hashes, ${mdFiles.length} mirrors, llms-full ${Math.round(size / 1024)} KiB`,
);