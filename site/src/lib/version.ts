// ─────────────────────────────────────────────
// Orbiscreen - Version and changelog access
// https://github.com/shadow-x78/orbiscreen
// ─────────────────────────────────────────────

import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, resolve } from 'node:path';

const here = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(here, '../../..');

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
export const CHANGELOG_ANCHOR = `v${VERSION.replace(/\./g, '-')}`;
