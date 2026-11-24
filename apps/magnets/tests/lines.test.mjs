// Magnets' cap (MAGNETS.md, "The whole app"): at most 1,000 lines of shipped
// source, its HTML, CSS, JavaScript, SQL and word pack together. Shipped is
// every such file of the folder but tests/, which never ships; nouns.txt is
// svrngs's list, not the app's. The check returns why it broke, or null, and
// the second test plants a folder past the cap, and one with nothing to
// count, and watches it catch each. And Magnets is on svrngs's shelf, so
// what is counted is what the shelf brings: its line in
// crates/svrngs-node/src/shelf.txt names every shipped file but index.html
// and index.js, which build.rs brings for every app, and nothing else.

import assert from 'node:assert/strict';
import { cpSync, mkdtempSync, readdirSync, readFileSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { extname, join } from 'node:path';
import { test } from 'node:test';
import { fileURLToPath } from 'node:url';

const APP = fileURLToPath(new URL('..', import.meta.url));
export const CAP = 1_000;
const SHIPPED = new Set(['.html', '.css', '.js', '.mjs', '.sql', '.json']);

/** Each shipped file of `dir` and its lines. */
export function shipped(dir) {
  return readdirSync(dir, { recursive: true })
    .filter((f) => !/^tests([/\\]|$)/.test(f) && SHIPPED.has(extname(f)))
    .sort()
    .map((f) => {
      const text = readFileSync(join(dir, f), 'utf8');
      return [f, text.split('\n').length - (text.endsWith('\n') ? 1 : 0)];
    });
}

export function overCap(dir) {
  const files = shipped(dir);
  const lines = files.reduce((n, [, l]) => n + l, 0);
  if (!files.length) return `nothing shipped in ${dir} to count`;
  return lines > CAP ? `${lines} shipped lines, past ${CAP}: ${JSON.stringify(files)}` : null;
}

test(`Magnets ships at most ${CAP} lines`, () => {
  console.log(`magnets: ${JSON.stringify(shipped(APP))}`);
  assert.equal(overCap(APP), null);
});

const SHELF = fileURLToPath(new URL('../../../crates/svrngs-node/src/shelf.txt', import.meta.url));

/** What the shelf line of the app in `apps/<folder>/` brings, by name, given
 * the shelf's text: index.html, index.js and the files its line lists. */
export function brought(shelf, folder) {
  const line = shelf.split('\n').find((l) => !l.startsWith('#') && l.split('\t')[1] === folder);
  return line ? ['index.html', 'index.js', ...line.split('\t')[2].split(' ')].sort() : [];
}

// Plant, watched failing: Magnets' shelf line without schema.sql, and its
// database's schema isn't brought.
test('the shelf brings every file Magnets ships, and no other', () => {
  const shelf = readFileSync(SHELF, 'utf8');
  assert.deepEqual(
    brought(shelf, 'magnets'),
    shipped(APP).map(([f]) => f),
  );
  const planted = shelf.replace('pack.json schema.sql', 'pack.json');
  assert.notDeepEqual(
    brought(planted, 'magnets'),
    shipped(APP).map(([f]) => f),
  );
});

test('the cap catches a folder past it, and one with nothing to count', () => {
  const past = mkdtempSync(join(tmpdir(), 'magnets-past-'));
  cpSync(APP, past, { recursive: true });
  writeFileSync(join(past, 'planted.js'), '\n'.repeat(CAP));
  assert.match(String(overCap(past)), /shipped lines, past 1000/);
  assert.match(String(overCap(mkdtempSync(join(tmpdir(), 'magnets-none-')))), /nothing shipped/);
});
