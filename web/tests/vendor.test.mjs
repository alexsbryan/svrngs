// The pages' v2 libraries, vendored (page-scaffold (b); web/vendor/README.md):
// Preact 11.1.1 and htm 3.1.1, each file the bytes the README pins, with
// its two edits and nothing else, and the three loading together under
// node as one Preact.

import assert from 'node:assert/strict';
import { cpSync, mkdtempSync, readFileSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { test } from 'node:test';

import { pins, unpinned } from '../../apps/pinned.mjs';

const VENDOR = new URL('../vendor', import.meta.url).pathname;
const SCRIPTS = ['preact.js', 'hooks.js', 'htm.js'];

test('every vendored file is the one pinned', () => {
  assert.deepEqual(pins(VENDOR).map(([f]) => f), [...SCRIPTS, 'LICENSE.preact', 'LICENSE.htm']);
  assert.deepEqual(unpinned(VENDOR), []);
});

test('one byte changed in hooks.js is caught, by name', () => {
  const dir = mkdtempSync(join(tmpdir(), 'svrngs-vendor-'));
  cpSync(VENDOR, dir, { recursive: true });
  const f = join(dir, 'hooks.js');
  const bytes = readFileSync(f);
  bytes[100] ^= 1;
  writeFileSync(f, bytes);
  assert.deepEqual(unpinned(VENDOR, dir).map((s) => s.split(' ')[0]), ['hooks.js']);
});

test('no file keeps a source map line, and hooks.js imports one Preact, versioned', () => {
  for (const f of SCRIPTS) {
    assert.doesNotMatch(readFileSync(join(VENDOR, f), 'utf8'), /sourceMappingURL/, f);
  }
  const imports = (f) => [...readFileSync(join(VENDOR, f), 'utf8').matchAll(/from\s*"([^"]*)"/g)].map((m) => m[1]);
  assert.deepEqual(imports('hooks.js'), ['./preact.js?v=__VERSION__']);
  assert.deepEqual(imports('preact.js'), []);
  assert.deepEqual(imports('htm.js'), []);
});

test('htm bound to Preact writes a vnode, and the hooks load beside it', async () => {
  const { h } = await import('../vendor/preact.js?v=__VERSION__');
  const htm = (await import('../vendor/htm.js?v=__VERSION__')).default;
  const { useState } = await import('../vendor/hooks.js?v=__VERSION__');
  const v = htm.bind(h)`<button id="go" disabled=${true}>Bring</button>`;
  assert.equal(v.type, 'button');
  assert.deepEqual([v.props.id, v.props.disabled, v.props.children], ['go', true, 'Bring']);
  assert.equal(typeof useState, 'function');
});
