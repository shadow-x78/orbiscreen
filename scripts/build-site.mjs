#!/usr/bin/env node
// ─────────────────────────────────────────────
// Orbiscreen - build the website from the repository root
// https://github.com/shadow-x78/orbiscreen
// ─────────────────────────────────────────────
// Cloudflare's builder starts at the repository root and runs `npm run build`,
// so the site's dependencies have to be installed from here. Two details make a
// plain `cd site && npm ci` unreliable:
//
//  1. npm 12 rejects the `allow-scripts` config when a project install runs
//     inside an npm script ("--allow-scripts is not allowed in project-scoped
//     installs"), and npm forwards that config as an environment variable to
//     nested scripts. A developer with `allow-scripts` in ~/.npmrc therefore
//     cannot run `npm run build` at all, while CI can. The variable is removed
//     from the child environment so both behave the same.
//  2. `npm ci` deletes node_modules, which would kill a dev server on every
//     build. Dependencies are installed only when they are missing or when the
//     lockfile is newer than the installed tree.

import { spawnSync } from 'node:child_process';
import { existsSync, statSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, resolve } from 'node:path';

const here = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(here, '..');
const siteRoot = resolve(repoRoot, 'site');

function sanitisedEnv() {
  const env = { ...process.env };
  for (const key of Object.keys(env)) {
    if (/^npm_config_allow[-_]scripts$/i.test(key) || key === 'NPM_CONFIG_ALLOW_SCRIPTS') {
      delete env[key];
    }
  }
  return env;
}

function needsInstall() {
  if (process.env.ORBISCREEN_FORCE_INSTALL === '1') return true;
  const modules = resolve(siteRoot, 'node_modules');
  if (!existsSync(modules)) return true;

  const installed = resolve(modules, '.package-lock.json');
  const lock = resolve(siteRoot, 'package-lock.json');
  if (!existsSync(installed) || !existsSync(lock)) return true;
  return statSync(lock).mtimeMs > statSync(installed).mtimeMs;
}

function run(command, args, cwd) {
  const result = spawnSync(command, args, { cwd, stdio: 'inherit', env: sanitisedEnv() });
  if (result.error) {
    console.error(`build-site: ${command} failed to start: ${result.error.message}`);
    process.exit(1);
  }
  if (result.status !== 0) {
    process.exit(result.status ?? 1);
  }
}

if (needsInstall()) {
  console.log('build-site: installing site dependencies');
  run('npm', ['ci', '--no-audit', '--no-fund'], siteRoot);
} else {
  console.log('build-site: site dependencies are current');
}

run('npm', ['run', 'build'], siteRoot);