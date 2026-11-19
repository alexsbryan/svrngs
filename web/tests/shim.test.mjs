// A lent page's WebSocket sends what a browser's sends (web/shim.js
// `Socket`; WHATWG WebSockets, `send`): a string, an ArrayBuffer, a Blob's
// bytes, and of any ArrayBufferView — a typed array, a DataView — the
// bytes it views, and no others of the buffer under it. Each is carried to
// the lend's worker over the socket's port as `{send}`. The shim runs as
// a page runs it, in this process's one global, given what of a browser it
// reads as it starts. Watched failing at 4b7160c0, whose `send` called a
// DataView's `slice`, which a DataView has none of, and threw.
//
// It also shows a subresource refused while the share is spent in a banner
// with svrngs's words (share-refusal; RELAY_COMMONS design 3, "The
// words"), by fetch and by XMLHttpRequest, so a stalled player says why.

import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { test } from 'node:test';
import { runInThisContext } from 'node:vm';

// The worker the page's sockets are handed to: each socket's port, as the
// shim posts it.
const ports = [];
// The elements the shim appends to the page, in order.
const appended = [];
// What the page's own `fetch` answers, replaced by each test.
let answer = () => Promise.resolve(new Response('ok'));
// Node 21 on gives globalThis a `navigator` with a getter alone, so it is
// defined over, not assigned (configurable, as Node's is).
Object.defineProperty(globalThis, 'navigator', {
  value: {
    serviceWorker: {
      controller: { postMessage: (asked, [port]) => asked.socket && ports.push(port) },
      addEventListener: () => {},
    },
  },
  configurable: true,
  writable: true,
});
Object.assign(globalThis, {
  window: globalThis,
  location: { href: 'https://lend.example/' },
  document: {
    currentScript: { src: 'https://lend.example/shim.js?l=lend&c=' },
    createElement: (tag) => ({
      tag,
      textContent: '',
      style: {},
      attributes: {},
      setAttribute(name, value) {
        this.attributes[name] = value;
      },
    }),
    documentElement: { append: (element) => appended.push(element) },
  },
  Document: class {},
  // The page's XHR, as the shim wraps it: a request the test drives to
  // HEADERS_RECEIVED and DONE itself.
  XMLHttpRequest: class {
    readyState = 0;
    responseType = '';
    responseText = '';
    response = null;
    headers = {};
    listeners = [];
    open() {}
    addEventListener(type, heard) {
      if (type === 'readystatechange') this.listeners.push(heard);
    }
    getResponseHeader(name) {
      return this.headers[name.toLowerCase()] ?? null;
    }
    done() {
      this.readyState = 4;
      for (const heard of this.listeners) heard();
    }
  },
  fetch: (...args) => answer(...args),
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

// The share's words (door.rs `spent_words`; share-refusal; RELAY_COMMONS
// design 3, "The words"): a subresource refused `share` shows a banner
// with the answer's words, once, replaced by the next — so a film that
// waits says why. Planted, watched failing: the banner never shown, and
// `appended` was empty.
const SPENT =
  'Sam’s computer has used its share of the public way for now. Play it at full speed on the same Wi-Fi as Sam’s computer, or in the svrngs app, soon.';
// The banner chain runs in a microtask after the wrapper's answer.
const settled = () => new Promise((done) => setTimeout(done, 0));
// The answer a refused subresource gets from the door, as svrngs marks it.
const refused = (words) =>
  new Response(JSON.stringify({ error: words, kind: 'share' }), {
    status: 429,
    headers: { 'x-svrngs-refused': 'share' },
  });

test('a subresource refused while the share is spent shows the answer’s words in a banner, by fetch', async () => {
  answer = async () => refused(SPENT);
  const response = await window.fetch('https://lend.example/segment');
  assert.equal(response.status, 429, 'the answer itself is kept');
  await settled();
  assert.equal(appended.length, 1, 'one banner');
  assert.equal(appended[0].textContent, SPENT);
  assert.match(appended[0].style.cssText, /position:fixed/);
  // The next refusal says its words in the same banner, not a second one.
  answer = async () => refused('Sam’s computer has used its share for now.');
  await window.fetch('https://lend.example/segment');
  await settled();
  assert.equal(appended.length, 1);
  assert.equal(appended[0].textContent, 'Sam’s computer has used its share for now.');
  // An answer that is not svrngs's own shows nothing.
  answer = async () => new Response('fine', { status: 200 });
  await window.fetch('https://lend.example/segment');
  await settled();
  assert.equal(appended.length, 1);
  assert.equal(appended[0].textContent, 'Sam’s computer has used its share for now.');
});

test('a subresource refused while the share is spent shows the answer’s words in a banner, by XMLHttpRequest', async () => {
  const asked = new XMLHttpRequest();
  asked.open('GET', 'https://lend.example/segment');
  asked.readyState = 2;
  for (const heard of asked.listeners) heard();
  asked.headers['x-svrngs-refused'] = 'share';
  asked.responseText = JSON.stringify({ error: SPENT, kind: 'share' });
  asked.done();
  assert.equal(appended.at(-1).textContent, SPENT);
});
