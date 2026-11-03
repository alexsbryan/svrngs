// A lend's worker reads the frames the door's socket sends (web/lend.js
// `relayed` and `frameAt`; RFC 6455 §5.2) however the reads split them:
// each whole message told to the page's port once all of it has come, a
// head that comes in two reads waited for, and the stream's end told
// after. lend.js is a service worker whose imports are a browser's, so
// the functions are cut from its text and run in a thread of their own,
// which a read that never ends cannot hold past its time. Watched
// failing at 4b7160c0, which read a head's longer length before it had
// all come: a 70,000-byte frame split at its head's bytes 2 to 6 was told
// over and over, and its read never ended.

import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { test } from 'node:test';
import { Worker } from 'node:worker_threads';

const LEND = readFileSync(new URL('../lend.js', import.meta.url), 'utf8');
// The function `name` as lend.js declares it, up to its closing brace.
const defined = (name) => {
  const at = LEND.search(new RegExp(`^(async )?function ${name}\\(`, 'm'));
  assert.notEqual(at, -1, `lend.js declares ${name}`);
  return LEND.slice(at, LEND.indexOf('\n}\n', at) + 2);
};
const SOURCE = ['relayed', 'frameAt', 'gathered', 'closedBy'].map(defined).join('\n');

// In the thread: `relayed` over a reader that gives each of `chunks`, then
// ends; each thing it tells the port posted back, up to `TOLD`, so a loop
// that never ends cannot fill the test's memory before it is stopped.
const TOLD = 5;
const THREAD = `
const { parentPort, workerData } = require('node:worker_threads');
const reading = require('node:vm').runInThisContext(workerData.source + '\\nrelayed;');
const chunks = workerData.chunks.map((chunk) => new Uint8Array(chunk));
const reader = {
  read: async () => (chunks.length ? { value: chunks.shift(), done: false } : { done: true }),
};
const writer = { write: async () => {}, close: async () => {} };
let posted = 0;
const port = { postMessage: (told) => ++posted <= ${TOLD} && parentPort.postMessage(told) };
void reading(reader, writer, port);
`;

// What the port is told of `chunks`, read in turn, up to the stream's end;
// refused when that end is not told within `within` ms, or `TOLD` things
// are told before it, more than any case here sends.
function told(chunks, within = 10_000) {
  return new Promise((done, failed) => {
    const all = [];
    const thread = new Worker(THREAD, { eval: true, workerData: { source: SOURCE, chunks } });
    const late = setTimeout(() => {
      void thread.terminate();
      failed(new Error(`the read did not end within ${within} ms, ${all.length} told`));
    }, within);
    thread.on('message', (one) => {
      all.push(one);
      if ('closed' in one || all.length === TOLD) {
        clearTimeout(late);
        void thread.terminate();
        if ('closed' in one) done(all);
        else failed(new Error(`${TOLD} told and the read not ended`));
      }
    });
    thread.on('error', (error) => {
      clearTimeout(late);
      failed(error);
    });
  });
}

// A final binary frame of `size` bytes as a door sends it, unmasked: its
// head, by the length's three forms, then the bytes.
function sent(size) {
  const data = Uint8Array.from({ length: size }, (_, i) => i % 251);
  const length =
    size < 126
      ? [size]
      : size < 2 ** 16
        ? [126, size >> 8, size & 255]
        : [127, 0, 0, 0, 0, size >>> 24, (size >> 16) & 255, (size >> 8) & 255, size & 255];
  return { data, bytes: Uint8Array.from([0x82, ...length, ...data]) };
}

// 70,000 bytes take the 8-byte length (a head of 10 bytes); 300 take the
// 2-byte (a head of 4). Split at each byte of the head, and in the bytes.
const SPLITS = [
  [70_000, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 5_000]],
  [300, [1, 2, 3, 4, 100]],
];
test('a frame whose head comes split across reads is told whole, then the end', async (t) => {
  for (const [size, splits] of SPLITS)
    for (const split of splits)
      await t.test(`${size} bytes, split at ${split}`, async () => {
        const { data, bytes } = sent(size);
        const all = await told([bytes.slice(0, split), bytes.slice(split)]);
        assert.equal(all.length, 2, `${all.length} told`);
        assert.deepEqual(new Uint8Array(all[0].message), data, 'the message whole');
        assert.deepEqual(all[1], { closed: 1006, reason: 'the stream ended' });
      });
});

test('two frames in one read are told in turn', async () => {
  const [one, two] = [sent(70_000), sent(5)];
  const all = await told([Uint8Array.from([...one.bytes, ...two.bytes])]);
  assert.deepEqual(
    all.slice(0, 2).map((m) => new Uint8Array(m.message)),
    [one.data, two.data],
  );
  assert.equal(all.length, 3);
});
