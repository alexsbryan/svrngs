// A lent page's WebSocket sends what a browser's sends (web/shim.js
// `Socket`; WHATWG WebSockets, `send`): a string, an ArrayBuffer, a Blob's
// bytes, and of any ArrayBufferView — a typed array, a DataView — the
// bytes it views, and no others of the buffer under it. Each is carried to
// the lend's worker over the socket's port as `{send}`. The shim runs as
// a page runs it, in this process's one global, given what of a browser it
// reads as it starts. Watched failing at 4b7160c0, whose `send` called a
// DataView's `slice`, which a DataView has none of, and threw.

import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { test } from 'node:test';
import { runInThisContext } from 'node:vm';

// The worker the page's sockets are handed to: each socket's port, as the
// shim posts it.
const ports = [];
Object.assign(globalThis, {
  window: globalThis,
  location: { href: 'https://lend.example/' },
  document: { currentScript: { src: 'https://lend.example/shim.js?l=lend&c=' } },
  Document: class {},
  XMLHttpRequest: class {
    open() {}
  },
  navigator: {
    serviceWorker: {
      controller: { postMessage: (asked, [port]) => asked.socket && ports.push(port) },
      addEventListener: () => {},
    },
  },
});
runInThisContext(readFileSync(new URL('../shim.js', import.meta.url), 'utf8'));

// A socket the worker has said is open, and what it sends, each `{send}`
// as the worker's end of its port is told it.
async function opened() {
  const socket = new window.WebSocket('wss://lend.example/socket');
  const port = ports.at(-1);
  const sent = [];
  port.onmessage = ({ data }) => sent.push(data);
  const open = new Promise((done) => socket.addEventListener('open', done));
  port.postMessage({ open: '' });
  await open;
  const next = () =>
    new Promise((done) => {
      const had = sent.length;
      const look = () => (sent.length > had ? done(sent.at(-1)) : setTimeout(look, 1));
      look();
    });
  return { socket, next, close: () => port.close() };
}

test('a socket sends the bytes each ArrayBufferView views, and nothing of its buffer past them', async () => {
  const { socket, next, close } = await opened();
  const buffer = Uint8Array.from([0, 1, 2, 3, 4, 5, 6, 7]).buffer;
  try {
    for (const [what, view, bytes] of [
      ['a DataView', new DataView(buffer, 2, 3), [2, 3, 4]],
      ['a Uint8Array window', new Uint8Array(buffer, 5, 2), [5, 6]],
      ['a Uint16Array window', new Uint16Array(buffer, 2, 2), [2, 3, 4, 5]],
      ['a whole Uint8Array', new Uint8Array(buffer), [0, 1, 2, 3, 4, 5, 6, 7]],
    ]) {
      socket.send(view);
      const { send } = await next();
      assert.ok(send instanceof ArrayBuffer, `${what} is sent as an ArrayBuffer`);
      assert.deepEqual([...new Uint8Array(send)], bytes, what);
    }
    assert.deepEqual([...new Uint8Array(buffer)], [0, 1, 2, 3, 4, 5, 6, 7], 'the buffer kept');
  } finally {
    close();
  }
});

test('a string, an ArrayBuffer and a Blob are sent as they are', async () => {
  const { socket, next, close } = await opened();
  try {
    socket.send('hello');
    assert.deepEqual(await next(), { send: 'hello' });
    socket.send(Uint8Array.from([9, 8]).buffer);
    assert.deepEqual([...new Uint8Array((await next()).send)], [9, 8]);
    socket.send(new Blob([Uint8Array.from([7, 6, 5])]));
    assert.deepEqual([...new Uint8Array((await next()).send)], [7, 6, 5]);
  } finally {
    close();
  }
});
