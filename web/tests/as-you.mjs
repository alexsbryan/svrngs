// How a test asks svrngs as you ("Never send the key", svrngs V1.13.3):
// through apps/client/svrngs.mjs, the one client a program on a computer
// uses, over a data dir of the test's own that names svrngs at `port` and
// holds `token`, as svrngs's own `api` and `token` files do. svrngs proves
// itself on each connection, and each request goes with a code made for
// that connection, never the token.

import { mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

import { svrngs } from '../../apps/client/svrngs.mjs';

const made = new Set();
process.once('exit', () => made.forEach((dir) => rmSync(dir, { recursive: true, force: true })));

/** svrngs at 127.0.0.1:`port`, whose token is `token`: the client's `ask`
 * and `fetch`, as svrngs.mjs gives them. */
export function asYou(port, token) {
  const dir = mkdtempSync(join(tmpdir(), 'svrngs-as-you-'));
  writeFileSync(join(dir, 'api'), `http://127.0.0.1:${port}\n`);
  writeFileSync(join(dir, 'token'), `${String(token).trim()}\n`);
  made.add(dir);
  return svrngs(dir);
}
