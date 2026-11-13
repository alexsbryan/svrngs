// SPDX-License-Identifier: AGPL-3.0-or-later
// The doc's socket, P's own (docs/seams.tsv): a page's socket upgraded at
// /sync on the server's real listener (server.mjs `start`, ws.mjs `accept`
// and its frames), carrying the headers svrngs's door sets on an upgrade.
// What the doc does over it is doc.test.mjs's, in one process.

import assert from 'node:assert/strict';
import { createHash, randomBytes } from 'node:crypto';
import { mkdtempSync, rmSync } from 'node:fs';
import http from 'node:http';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { test } from 'node:test';
import { PROSE_FRAGMENT } from './adapter.js';
import { start } from './server.mjs';
import { sync } from './sync.js';
import { Y, awarenessProtocol } from './vendor/doc-bundle.js';
import { frame, frames } from './ws.mjs';

const keyOf = (person) => createHash('sha256').update(person).digest('hex');
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
async function until(what, f, within = 15_000) {
  const t0 = Date.now();
  for (;;) {
    const v = f();
    if (v) return Date.now() - t0;
    if (Date.now() - t0 > within) throw new Error(`${what}: not within ${within} ms`);
    await sleep(50);
  }
}

/** A browser's WebSocket, in Node, opened with `headers` as the door would set them. */
function socketAs(headers) {
  return class {
    constructor(url) {
      Object.assign(this, { readyState: 0, binaryType: 'blob' });
      const u = new URL(url.replace(/^ws/, 'http'));
      this.req = http.request({
        host: u.hostname,
        port: u.port,
        path: u.pathname,
        headers: {
          connection: 'Upgrade',
          upgrade: 'websocket',
          'sec-websocket-version': '13',
          'sec-websocket-key': randomBytes(16).toString('base64'),
          ...headers,
        },
      });
      this.req.on('upgrade', (_res, socket, head) => {
        this.socket = socket;
        this.readyState = 1;
        let buf = Buffer.alloc(0);
        const take = (d) => {
          const { out, rest } = frames(Buffer.concat([buf, d]));
          buf = rest;
          for (const f of out) {
            if (f.op === 2)
              this.onmessage?.({
                data: f.data.buffer.slice(f.data.byteOffset, f.data.byteOffset + f.data.length),
              });
            if (f.op === 8) this.end();
          }
        };
        socket.on('data', take);
        socket.on('close', () => this.end());
        socket.on('error', () => this.end());
        this.onopen?.();
        if (head.length) take(head);
      });
      this.req.on('error', () => this.end());
      this.req.end();
    }
    send(data) {
      if (this.readyState === 1) this.socket.write(frame(2, data, randomBytes(4)));
    }
    close() {
      if (this.readyState === 1) this.socket.end(frame(8, Buffer.from([3, 232]), randomBytes(4)));
      else this.req.destroy();
      this.end();
    }
    end() {
      if (this.readyState === 3) return;
      this.readyState = 3;
      this.onclose?.();
    }
  };
}

/** A page: its own doc, its sync, and what the sync told it. */
function page(port, person) {
  const doc = new Y.Doc();
  const awareness = new awarenessProtocol.Awareness(doc);
  const seen = { hello: null, lines: null, away: [], here: 0, joined: [], pokes: 0, synced: 0 };
  const headers = person
    ? { 'x-svrngs-person': Buffer.from(person).toString('latin1'), 'x-svrngs-key': keyOf(person) }
    : {};
  const s = sync({
    url: `ws://127.0.0.1:${port}/sync`,
    doc,
    awareness,
    WebSocket: socketAs(headers),
    poke: () => (
      (seen.pokes += 1),
      fetch(`http://127.0.0.1:${port}/here`, { signal: AbortSignal.timeout(5000) })
    ),
    on: {
      hello: (h) => {
        seen.hello = h;
        awareness.setLocalStateField('user', { name: h.you });
      },
      lines: (l) => (seen.lines = l),
      away: (a) => seen.away.push({ ...a, at: Date.now() }),
      here: () => (seen.here += 1),
      synced: () => (seen.synced += 1),
      joined: (j) => seen.joined.push({ ...j, at: Date.now() }),
    },
  }).start();
  const frag = doc.getXmlFragment(PROSE_FRAGMENT);
  return {
    doc,
    awareness,
    seen,
    s,
    // An Awareness keeps a timer of its own (y-protocols' outdated check),
    // which would hold the test's process open.
    close: () => (s.stop(), awareness.destroy()),
    write(text) {
      const p = new Y.XmlElement('paragraph');
      const t = new Y.XmlText();
      t.insert(0, text);
      p.insert(0, [t]);
      frag.push([p]);
    },
    text: () =>
      frag.toArray().map((p) =>
        p
          .toArray()
          .map((t) => t.toString())
          .join(''),
      ),
    names: () =>
      [...awareness.getStates().values()]
        .map((s) => s.user?.name)
        .filter(Boolean)
        .sort(),
  };
}

test("a page's socket is upgraded at /sync, greeted as the door named it, and its edit reaches another", async (t) => {
  const keep = mkdtempSync(join(tmpdir(), 'doc-'));
  const srv = await start({ port: 0, keep });
  const bo = page(srv.port, 'Bo');
  const zoe = page(srv.port, 'Zoë');
  t.after(
    async () => (bo.close(), zoe.close(), await srv.close(), rmSync(keep, { recursive: true })),
  );
  await until('both are greeted', () => bo.seen.hello && zoe.seen.hello);
  await until("both are told they hold the server's doc", () => bo.seen.synced && zoe.seen.synced);
  assert.deepEqual([bo.seen.hello.you, zoe.seen.hello.you], ['Bo', 'Zoë']);
  bo.write('bring chips');
  await until('Zoë reads it', () => zoe.text().includes('bring chips'));
});
