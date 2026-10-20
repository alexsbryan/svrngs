// SPDX-License-Identifier: AGPL-3.0-or-later
// How an app's vendored files are pinned, read once for every app's tests
// (#208): its vendor/README.md's `| `file` | `sha256` |` table, and each
// file's bytes held to it. canon's and grapevine's tests import it, so a
// change to the table's form is read the same way in both, or fails in both.

import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';

/** The files `vendor`/README.md pins, with their sha256s: [[file, sha256]]. */
export function pins(vendor) {
  const readme = readFileSync(join(vendor, 'README.md'), 'utf8');
  return [...readme.matchAll(/^\| `([^`]+)` \| `([0-9a-f]{64})` \|$/gm)].map((m) => [m[1], m[2]]);
}

/** Each file under `dir` whose bytes are not the ones `vendor`/README.md pins, said. */
export function unpinned(vendor, dir = vendor) {
  return pins(vendor)
    .map(([f, want]) => [f, createHash('sha256').update(readFileSync(join(dir, f))).digest('hex'), want])
    .filter(([, got, want]) => got !== want)
    .map(([f, got, want]) => `${f} is ${got}, pinned ${want}`);
}
