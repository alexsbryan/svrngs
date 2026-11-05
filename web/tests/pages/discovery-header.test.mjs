// The phone's discovery edge names its svrngs version (SCOPE_PARALLEL.md,
// "Shared API shape"): web/page/dial.js `lends` asks a computer for
// `/people/lends` with `x-svrngs-version` set to `core.current_version()`,
// so a lender refuses an old discovery request that cannot name a scope.
// The header is compatibility data, never permission.
//
// Plant, watched failing: with the header line removed, `lends` sent only
// `accept`, and the assertion on `x-svrngs-version` failed.

import assert from 'node:assert/strict';
import { register } from 'node:module';
import { test } from 'node:test';

register('./layout.mjs', import.meta.url);
const { hold, lends } = await import('../../../web/page/dial.js?v=__VERSION__');

// dial.js `patiently` starts a PATIENCE-long timer that holds the process
// open until it fires, even once the dial has answered. Unref every timer
// this test starts, as check.sh's `node --test` runs to the file's end.
const realTimeout = globalThis.setTimeout;
globalThis.setTimeout = (...args) => {
  const timer = realTimeout(...args);
  timer.unref?.();
  return timer;
};

test('the discovery read names the page’s svrngs version', async () => {
  let sent = null;
  const answered = { ok: true, status: 200, text: async () => '[]' };
  const dialled = {
    fetch: (method, path, headers) => {
      sent = { method, path, headers };
      return Promise.resolve(answered);
    },
    close: () => {},
  };
  const dialer = { dial_link: () => Promise.resolve(dialled) };
  const core = {
    relay_of: () => 'a relay',
    issuer_of: () => 'a computer',
    endpoint: () => Promise.resolve(dialer),
    current_version: () => 5,
  };
  hold(new Uint8Array(32));
  const group = { token: 'a token', name: 'House', computers: [] };
  const [, , response] = await lends(core, group, []);
  assert.equal(response, answered);
  assert.equal(sent.method, 'GET');
  assert.equal(sent.path, '/people/lends');
  assert.ok(
    sent.headers.some(([name, value]) => name === 'x-svrngs-version' && value === '5'),
    `x-svrngs-version 5 among ${JSON.stringify(sent.headers)}`,
  );
});
