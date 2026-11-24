// ─────────────────────────────────────────────
// Orbiscreen - Comment policy check for the site and worker
// ─────────────────────────────────────────────

import { readdirSync, readFileSync, statSync } from 'node:fs';
import { join, relative, resolve } from 'node:path';

const site = resolve(new URL('..', import.meta.url).pathname);
const repo = resolve(site, '..');
const extensions = new Set(['.astro', '.ts', '.mjs', '.css']);
const targets = [
  join(site, 'src'),
  join(site, 'scripts'),
  join(site, 'astro.config.mjs'),
  join(repo, 'worker/index.mjs'),
  join(repo, 'scripts/build-site.mjs'),
];
const directives = /(@ts-|@ts\b|eslint|istanbul|prettier|@license|@preserve|@jsx|@flow|sourceMappingURL|@vite|@vitest|@jest|v8 ignore|c8 ignore|#region|#endregion|shellcheck)/;
let failures = 0;

function walk(path) {
  const out = [];
  const stat = statSync(path, { throwIfNoEntry: false });
  if (!stat) return out;
  if (stat.isFile()) return [path];
  for (const name of readdirSync(path)) {
    if (name === 'content' && path.endsWith('/src')) continue;
    out.push(...walk(join(path, name)));
  }
  return out;
}

function bannerEnd(lines) {
  let i = 0;
  let block = false;
  while (i < lines.length) {
    const s = lines[i].trim();
    if (block) { if (s.includes('*/') || s.includes('-->')) block = false; i += 1; continue; }
    if (s === '' || s === '---' || s.startsWith('#!')) { i += 1; continue; }
    if (s.startsWith('//')) { i += 1; continue; }
    if (s.startsWith('/*')) { if (!s.includes('*/')) block = true; i += 1; continue; }
    if (s.startsWith('<!--')) { if (!s.includes('-->')) block = true; i += 1; continue; }
    break;
  }
  return i;
}

function checkFile(path) {
  if (!extensions.has(path.slice(path.lastIndexOf('.')))) return;
  const text = readFileSync(path, 'utf8');
  const lines = text.split('\n');
  const start = bannerEnd(lines);
  let inTemplate = false;
  for (let i = start; i < lines.length; i += 1) {
    const line = lines[i];
    const s = line.trim();
    if (directives.test(line)) continue;
    const ticks = (line.match(/`/g) || []).length;
    if (inTemplate) {
      if (ticks % 2 === 1) inTemplate = false;
      continue;
    }
    if (ticks % 2 === 1) inTemplate = true;
    const code = line
      .replace(/'(?:[^'\\]|\\.)*'/g, "''")
      .replace(/"(?:[^"\\]|\\.)*"/g, '""')
      .replace(/`(?:[^`\\]|\\.)*`/g, '``');
    const comment =
      s.startsWith('//') ||
      s.startsWith('/*') ||
      s.startsWith('<!--') ||
      (ticks % 2 === 0 && (/[^:"/]\s+\/\//.test(code) || /[^:"/]\s+\/\*/.test(code)));
    if (comment) {
      console.error(`${relative(repo, path)}:${i + 1}: comment is not allowed`);
      failures += 1;
    }
  }
}

for (const target of targets) {
  for (const file of walk(target)) {
    if (file.includes('/node_modules/')) continue;
    if (file.includes('/src/content/')) continue;
    if (file.endsWith('check-comments.mjs')) continue;
    checkFile(file);
  }
}

if (failures) {
  console.error(`comment policy: ${failures} violation(s)`);
  process.exit(1);
}
console.log('site comment policy holds');
