// ─────────────────────────────────────────────
// Orbiscreen - Repository root discovery
// https://github.com/shadow-x78/orbiscreen
// ─────────────────────────────────────────────

import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, resolve } from 'node:path';

function findRepoRoot(start: string): string {
  let dir = start;
  for (let i = 0; i < 8; i += 1) {
    try {
      if (readFileSync(resolve(dir, 'Cargo.toml'), 'utf8').includes('[workspace.package]')) return dir;
    } catch {}
    const parent = resolve(dir, '..');
    if (parent === dir) break;
    dir = parent;
  }
  throw new Error('Cargo.toml with [workspace.package] not found');
}

export const repoRoot = findRepoRoot(dirname(fileURLToPath(import.meta.url)));
