// SPDX-License-Identifier: AGPL-3.0-or-later
// The doc end to end on one host: the real server (server.mjs) as its own
// process, and pages as sync.js runs them, each over a socket that sets the
// headers svrngs's door sets on an upgrade (a stand-in for the door: svrngs
// crates/svrngs-node/src/door.rs strips a client's copies and sets its own).
// The lid is SIGSTOP on the server's process, which is what a closed lid
// does to it. `node --test apps/doc/`, and scripts/check.sh's "app laws".

import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { createHash, randomBytes } from 'node:crypto';
import { mkdtempSync, rmSync } from 'node:fs';
import http from 'node:http';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { test } from 'node:test';
import { fileURLToPath } from 'node:url';
import { PROSE_FRAGMENT } from './adapter.js';
import { QUIET_MS, sync } from './sync.js';
import { Y, awarenessProtocol } from './vendor/doc-bundle.js';
import { frame, frames } from './ws.mjs';

const SERVER = fileURLToPath(new URL('./server.mjs', import.meta.url));
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
        headers: { connection: 'Upgrade', upgrade: 'websocket', 'sec-websocket-version': '13', 'sec-websocket-key': randomBytes(16).toString('base64'), ...headers },
      });
      this.req.on('upgrade', (_res, socket, head) => {
        this.socket = socket;
        this.readyState = 1;
        let buf = Buffer.alloc(0);
        const take = (d) => {
          const { out, rest } = frames(Buffer.concat([buf, d]));
          buf = rest;
          for (const f of out) {
            if (f.op === 2) this.onmessage?.({ data: f.data.buffer.slice(f.data.byteOffset, f.data.byteOffset + f.data.length) });
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

/** The server as its own process; `stop()` and `go()` are the lid. */
async function server(keep) {
  const proc = spawn(process.execPath, [SERVER, '--port', '0', '--keep', keep], { stdio: ['ignore', 'pipe', 'inherit'] });
  let said = '';
  proc.stdout.on('data', (d) => (said += d));
  await until('the server says its port', () => /127\.0\.0\.1:(\d+)/.test(said), 10_000);
  return {
    port: Number(said.match(/127\.0\.0\.1:(\d+)/)[1]),
    stop: () => proc.kill('SIGSTOP'),
    go: () => proc.kill('SIGCONT'),
    end: () => new Promise((done) => (proc.kill('SIGCONT'), proc.kill('SIGTERM'), proc.on('exit', done))),
  };
}

/** A page: its own doc, its sync, and what the sync told it. */
function page(port, person) {
  const doc = new Y.Doc();
  const awareness = new awarenessProtocol.Awareness(doc);
  const seen = { hello: null, lines: null, away: [], here: 0, joined: [], pokes: 0 };
  const headers = person ? { 'x-svrngs-person': Buffer.from(person).toString('latin1'), 'x-svrngs-key': keyOf(person) } : {};
  const s = sync({
    url: `ws://127.0.0.1:${port}/sync`,
    doc,
    awareness,
    WebSocket: socketAs(headers),
    poke: () => ((seen.pokes += 1), fetch(`http://127.0.0.1:${port}/here`, { signal: AbortSignal.timeout(5000) })),
    on: {
      hello: (h) => {
        seen.hello = h;
        awareness.setLocalStateField('user', { name: h.you });
      },
      lines: (l) => (seen.lines = l),
      away: (a) => seen.away.push({ ...a, at: Date.now() }),
      here: () => (seen.here += 1),
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
    text: () => frag.toArray().map((p) => p.toArray().map((t) => t.toString()).join('')),
    names: () => [...awareness.getStates().values()].map((s) => s.user?.name).filter(Boolean).sort(),
  };
}

test('two pages write at once and read one doc, each named as the door named their socket', async (t) => {
  const keep = mkdtempSync(join(tmpdir(), 'doc-'));
  const srv = await server(keep);
  const bo = page(srv.port, 'Bo');
  const zoe = page(srv.port, 'Zoë');
  t.after(async () => (bo.close(), zoe.close(), await srv.end(), rmSync(keep, { recursive: true })));
  await until('both are greeted', () => bo.seen.hello && zoe.seen.hello);
  assert.deepEqual([bo.seen.hello.you, zoe.seen.hello.you], ['Bo', 'Zoë']);
  bo.write('bring chips');
  zoe.write('bring a speaker');
  await until('both read both paragraphs', () => bo.text().length === 2 && zoe.text().length === 2);
  assert.deepEqual(bo.text().sort(), ['bring a speaker', 'bring chips']);
  assert.deepEqual(bo.text(), zoe.text(), 'the two pages read the paragraphs in one order');
  await until('each paragraph says who last edited it', () => bo.seen.lines?.filter(Boolean).length === 2);
  assert.deepEqual(
    bo.seen.lines.map((e) => e.person),
    bo.text().map((p) => (p === 'bring chips' ? 'Bo' : 'Zoë')),
  );
  await until('each sees the other\'s caret, named', () => bo.names().join() === 'Bo,Zoë' && zoe.names().join() === 'Bo,Zoë');
});

test('a page opened without svrngs is named nobody, and its paragraph says so', async (t) => {
  const keep = mkdtempSync(join(tmpdir(), 'doc-'));
  const srv = await server(keep);
  const anon = page(srv.port, null);
  t.after(async () => (anon.close(), await srv.end(), rmSync(keep, { recursive: true })));
  await until('greeted', () => anon.seen.hello);
  assert.equal(anon.seen.hello.you, null);
  anon.write('hello');
  await until('its line arrives', () => anon.seen.lines?.[0]);
  assert.equal(anon.seen.lines[0].person, null);
});

test('a lender gone quiet is goodnight within QUIET_MS and a tick, and what was written meanwhile joins once it answers', async (t) => {
  const keep = mkdtempSync(join(tmpdir(), 'doc-'));
  const srv = await server(keep);
  const bo = page(srv.port, 'Bo');
  const zoe = page(srv.port, 'Zoë');
  t.after(async () => (bo.close(), zoe.close(), await srv.end(), rmSync(keep, { recursive: true })));
  bo.write('before');
  await until('both read it', () => zoe.text().includes('before'));

  const stopped = Date.now();
  srv.stop();
  // A bound of its own, not QUIET_MS's: a page that never says goodnight
  // fails here rather than waiting with it.
  await until('both say goodnight', () => bo.seen.away.length && zoe.seen.away.length, 8_000);
  for (const p of [bo, zoe]) {
    const after = p.seen.away[0].at - stopped;
    assert.ok(after >= QUIET_MS - 1_000 && after <= QUIET_MS + 1_500, `goodnight ${after} ms after the lid, not within ${QUIET_MS} ms and a tick`);
  }
  assert.deepEqual(bo.names(), ['Bo'], 'no other caret stays drawn while the server is quiet');
  bo.write('while away, by Bo');
  zoe.write('while away, by Zoë');
  assert.equal(bo.s.state, 'away');

  await sleep(4_000);
  const woke = Date.now();
  srv.go();
  await until('both join again', () => bo.seen.joined.length && zoe.seen.joined.length, 20_000);
  await until('both read all three', () => bo.text().length === 3 && zoe.text().length === 3, 10_000);
  assert.deepEqual(bo.text(), zoe.text());
  assert.deepEqual(bo.seen.joined[0].kept, 1, 'Bo is told his one edit joined');
  assert.ok(bo.seen.pokes > 0 && zoe.seen.pokes > 0, 'each page asked the server plainly before a new socket');
  console.log(`# goodnight after ${bo.seen.away[0].at - stopped} and ${zoe.seen.away[0].at - stopped} ms; joined ${bo.seen.joined[0].at - woke} and ${zoe.seen.joined[0].at - woke} ms after the lid opened`);
});

test('the server reads the doc back whole after a restart', async (t) => {
  const keep = mkdtempSync(join(tmpdir(), 'doc-'));
  t.after(() => rmSync(keep, { recursive: true }));
  let srv = await server(keep);
  const bo = page(srv.port, 'Bo');
  bo.write('kept');
  await until('Bo is told who wrote it', () => bo.seen.lines?.[0]?.person === 'Bo');
  bo.close();
  await srv.end();
  srv = await server(keep);
  const cy = page(srv.port, 'Cy');
  t.after(async () => (cy.close(), await srv.end()));
  await until('Cy reads it', () => cy.text().includes('kept'));
  await until('and who wrote it', () => cy.seen.lines?.[0]?.person === 'Bo');
});
