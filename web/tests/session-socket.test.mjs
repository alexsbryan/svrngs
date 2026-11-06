// The socket a lend's page gets on a computer (session-client.js) reads as a
// browser's: Action Cable finds a socket's state with `for (state in
// adapters.WebSocket)`, so constants a `for in` cannot see leave it never
// open: it opens socket after socket and subscribes on none. Planted, watched
// failing: `Object.defineProperty` with no `enumerable`. web/shim.js, the
// phone's, reaches them by `Object.assign`.

import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { test } from 'node:test';

const SOURCE = readFileSync(
  new URL('../../crates/svrngs-node/src/home/session-client.js', import.meta.url),
  'utf8',
);

test('a `for in` over the socket finds its states, on the class and on a socket', () => {
  const window = { WebSocket: class {} };
  new Function('window', 'navigator', 'location', SOURCE)(window, {}, { href: 'http://a.localhost/' });
  const states = (of) => {
    const found = [];
    for (const name in of) found.push(name);
    return found.filter((name) => name === name.toUpperCase()).sort();
  };
  const want = ['CLOSED', 'CLOSING', 'CONNECTING', 'OPEN'];
  assert.deepEqual(states(window.WebSocket), want);
  assert.deepEqual(states(window.WebSocket.prototype), want);
  assert.equal(window.WebSocket.OPEN, 1);
});
