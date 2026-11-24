// ─────────────────────────────────────────────
// Orbiscreen - Changelog parsing
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

export const CHANGELOG_PAGE_SIZE = 12;

export interface Release {
  tag: string;
  date: string;
  anchor: string;
  blocks: Block[];
  text: string;
}

export interface ListItem {
  text: string;

  depth: number;
}

export type Block =
  | { type: 'heading'; text: string }
  | { type: 'list'; items: ListItem[] }
  | { type: 'paragraph'; text: string };

function releaseAnchor(version: string): string {
  return `v${version.replace(/\./g, '-')}`;
}

function escapeHtml(value: string): string {
  return value
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;');
}

export function inlineMarkdown(value: string): string {
  let out = escapeHtml(value);
  out = out.replace(/`([^`]+)`/g, '<code>$1</code>');
  out = out.replace(/\*\*([^*]+)\*\*/g, '<strong>$1</strong>');
  out = out.replace(/\[([^\]]+)\]\((https?:[^)]+)\)/g, '<a href="$2" rel="noopener">$1</a>');
  return out;
}

function plainText(blocks: Block[]): string {
  return blocks
    .map((block) => {
      if (block.type === 'list') return block.items.map((item) => item.text).join(' ');
      return block.text;
    })
    .join(' ')
    .replace(/`([^`]+)`/g, '$1')
    .replace(/\*\*([^*]+)\*\*/g, '$1')
    .replace(/\[([^\]]+)\]\(([^)]+)\)/g, '$1')
    .replace(/\s+/g, ' ')
    .trim();
}

function parseBlocks(lines: string[]): Block[] {
  const blocks: Block[] = [];
  let paragraph: string[] = [];
  let list: ListItem[] = [];
  let currentItem: string[] = [];
  let currentDepth = 0;

  const flushParagraph = () => {
    if (paragraph.length > 0) {
      blocks.push({ type: 'paragraph', text: paragraph.join(' ').trim() });
      paragraph = [];
    }
  };
  const flushItem = () => {
    if (currentItem.length > 0) {
      list.push({ text: currentItem.join(' ').trim(), depth: currentDepth });
      currentItem = [];
      currentDepth = 0;
    }
  };
  const flushList = () => {
    flushItem();
    if (list.length > 0) {
      blocks.push({ type: 'list', items: [...list] });
      list = [];
    }
  };

  for (const raw of lines) {
    const line = raw.replace(/\s+$/, '');
    if (/^###\s+/.test(line)) {
      flushParagraph();
      flushList();
      blocks.push({ type: 'heading', text: line.replace(/^###\s+/, '') });
      continue;
    }
    const bullet = line.match(/^(\s*)[-*]\s+(.*)$/);
    if (bullet) {
      flushParagraph();
      flushItem();

      currentDepth = Math.min(2, Math.floor(bullet[1].replace(/\t/g, '  ').length / 2));
      currentItem.push(bullet[2]);
      continue;
    }
    if (line.trim() === '') {
      flushItem();
      if (list.length === 0) flushParagraph();
      continue;
    }
    if (currentItem.length > 0) {
      currentItem.push(line.trim());
      continue;
    }
    paragraph.push(line.trim());
  }
  flushParagraph();
  flushList();
  return blocks;
}

let cache: Release[] | null = null;

export function latestReleaseDate(): string {
  return getReleases()[0]?.date ?? '';
}

export function getReleases(): Release[] {
  if (cache) return cache;
  const source = readFileSync(resolve(repoRoot, 'CHANGELOG.md'), 'utf8');
  const lines = source.split(/\r?\n/);
  const releases: Release[] = [];

  let current: { version: string; date: string; lines: string[] } | null = null;

  for (const line of lines) {
    const match = line.match(/^##\s+\[?v?([0-9]+\.[0-9]+\.[0-9]+[^\]]*)\]?\s*(?:-\s*(.+))?$/);
    if (match) {
      if (current) {
        releases.push({
          tag: `v${current.version}`,
          date: current.date,
          anchor: releaseAnchor(current.version),
          blocks: parseBlocks(current.lines),
          text: '',
        });
      }
      current = { version: match[1].trim(), date: (match[2] ?? '').trim(), lines: [] };
      continue;
    }
    if (current) current.lines.push(line);
  }
  if (current) {
    releases.push({
      tag: `v${current.version}`,
      date: current.date,
      anchor: releaseAnchor(current.version),
      blocks: parseBlocks(current.lines),
      text: '',
    });
  }

  for (const release of releases) release.text = plainText(release.blocks);
  cache = releases;
  return releases;
}