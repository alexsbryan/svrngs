// Magnets' cap (MAGNETS.md, "The whole app"): at most 1,000 lines of shipped
// source, its HTML, CSS, JavaScript, SQL and word pack together. Shipped is
// every such file of the folder but tests/, which never ships; nouns.txt is
// svrngs's list, not the app's. The check returns why it broke, or null, and
// the second test plants a folder past the cap, and one with nothing to
// count, and watches it catch each.

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

test('the cap catches a folder past it, and one with nothing to count', () => {
  const past = mkdtempSync(join(tmpdir(), 'magnets-past-'));
  cpSync(APP, past, { recursive: true });
  writeFileSync(join(past, 'planted.js'), '\n'.repeat(CAP));
  assert.match(String(overCap(past)), /shipped lines, past 1000/);
  assert.match(String(overCap(mkdtempSync(join(tmpdir(), 'magnets-none-')))), /nothing shipped/);
});
