// ─────────────────────────────────────────────
// Orbiscreen - Version and changelog access
// https://github.com/shadow-x78/orbiscreen
// ─────────────────────────────────────────────

import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, resolve } from 'node:path';

const here = dirname(fileURLToPath(import.meta.url));
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
const repoRoot = findRepoRoot(here);

function readWorkspaceVersion(): string {
  const cargo = readFileSync(resolve(repoRoot, 'Cargo.toml'), 'utf8');
  const section = cargo.split(/^\[workspace\.package\]$/m)[1];
  if (!section) throw new Error('Cargo.toml: [workspace.package] section not found');
  const match = section.match(/^\s*version\s*=\s*"([^"]+)"/m);
  if (!match) throw new Error('Cargo.toml: workspace version not found');
  return match[1];
}

function readRustVersion(): string {
  const cargo = readFileSync(resolve(repoRoot, 'Cargo.toml'), 'utf8');
  const match = cargo.match(/^\s*rust-version\s*=\s*"([^"]+)"/m);
  return match ? match[1] : '1.92';
}

export const VERSION = readWorkspaceVersion();
export const VERSION_TAG = `v${VERSION}`;
export const RUST_VERSION = readRustVersion();
