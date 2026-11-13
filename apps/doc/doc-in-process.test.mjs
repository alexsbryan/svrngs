// SPDX-License-Identifier: AGPL-3.0-or-later
// The doc end to end in one process: the real server's doc (server.mjs
// `doc`, everything `start` listens with but the listener) and pages as
// sync.js runs them, each over an in-memory socket carrying the headers
// svrngs's door sets on an upgrade (a stand-in for the door: svrngs
// crates/svrngs-node/src/door.rs strips a client's copies and sets its own).
// Time is node:test's mocked clock, moved a step at a time, so a bar on
// QUIET_MS is judged on the time the page and the server read, never a
// loaded host's. The lid is what a closed lid does to the server's process:
// nothing it says reaches a page, and what a page sends or opens waits for
// it to wake. `node --test apps/doc/`, and scripts/check.sh's "app laws".

import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { mkdtempSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { test } from 'node:test';
import { PROSE_FRAGMENT } from './adapter.js';
import { doc } from './server.mjs';
import { QUIET_MS, sync } from './sync.js';
import { Y, awarenessProtocol } from './vendor/doc-bundle.js';

const keyOf = (person) => createHash('sha256').update(person).digest('hex');
/** The test's mocked timers (each test enables its own). */
let mock;
/** Every message and step already queued, run: setImmediate is the real one. */
const settle = () => new Promise((r) => setImmediate(r));
/** The mocked clock moved on `ms`, a step at a time, the work each step wakes run. */
async function sleep(ms) {
  for (let left = ms; left > 0; left -= 50) {
    mock.tick(Math.min(50, left));
    await settle();
  }
}
/** Virtual ms until `f()` holds, failing past `within`. */
async function until(what, f, within = 15_000) {
  const t0 = Date.now();
  await settle();
  for (;;) {
    if (f()) return Date.now() - t0;
    if (Date.now() - t0 > within) throw new Error(`${what}: not within ${within} ms`);
    await sleep(50);
  }
}

/** The server's doc in this process, its lid, and how a page reaches it. */
async function server(keep) {
  const d = await doc({ keep });
  const lid = { shut: false, waiting: [] };
  /** What reaches the server: at once, or when it wakes. */
  const toServer = (f) => (lid.shut ? lid.waiting.push(f) : queueMicrotask(f));
  return {
    d,
    stop: () => (lid.shut = true),
    go: () => {
      lid.shut = false;
      lid.waiting.splice(0).forEach((f) => queueMicrotask(f));
    },
    end: () => d.close(),
    /** GET /here, as sync.js `poke` asks it: answered by the server's own handler. */
    poke: () =>
      new Promise((done) =>
        toServer(() =>
          d.handle(
            { method: 'GET', url: '/here', headers: {} },
            {
              writeHead() {
                return this;
              },
              end: () => done(),
            },
          ),
        ),
      ),
    /** A browser's WebSocket, opened with `headers` as the door would set them. */
    socketAs(headers) {
      return class {
        constructor() {
          Object.assign(this, { readyState: 0, binaryType: 'blob' });
          this.peer = {
            send: (bytes) => {
              if (lid.shut) return;
              const data = Uint8Array.from(bytes).buffer;
              queueMicrotask(() => this.readyState === 1 && this.onmessage?.({ data }));
            },
            close: () => queueMicrotask(() => this.end()),
            onmessage: () => {},
            onclose: () => {},
          };
          toServer(() => {
            if (this.readyState !== 0) return;
            this.readyState = 1;
            d.opened({ headers }, this.peer);
            this.onopen?.();
          });
        }
        send(data) {
          if (this.readyState === 1) {
            const bytes = Uint8Array.from(data);
            toServer(() => this.peer.onmessage(bytes));
          }
        }
        close() {
          const was = this.readyState;
          this.end();
          if (was === 1) toServer(() => this.peer.onclose());
        }
        end() {
          if (this.readyState === 3) return;
          this.readyState = 3;
          this.onclose?.();
        }
      };
    },
  };
}

/** A page: its own doc, its sync, and what the sync told it. */
function page(srv, person) {
  const doc = new Y.Doc();
  const awareness = new awarenessProtocol.Awareness(doc);
  const seen = { hello: null, lines: null, away: [], here: 0, joined: [], pokes: 0, synced: 0 };
  const headers = person
    ? { 'x-svrngs-person': Buffer.from(person).toString('latin1'), 'x-svrngs-key': keyOf(person) }
    : {};
  const s = sync({
    url: 'ws://127.0.0.1:4100/sync',
    doc,
    awareness,
    WebSocket: srv.socketAs(headers),
    poke: () => ((seen.pokes += 1), srv.poke()),
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

test('two pages write at once and read one doc, each named as the door named their socket', async (t) => {
  t.mock.timers.enable({ apis: ['setInterval', 'setTimeout', 'Date'], now: 1_000_000 });
  mock = t.mock.timers;
  const keep = mkdtempSync(join(tmpdir(), 'doc-'));
  const srv = await server(keep);
  const bo = page(srv, 'Bo');
  const zoe = page(srv, 'Zoë');
  t.after(async () => (bo.close(), zoe.close(), srv.end(), rmSync(keep, { recursive: true })));
  await until('both are greeted', () => bo.seen.hello && zoe.seen.hello);
  // The page lets its person type once this has been said (app.js).
  await until("both are told they hold the server's doc", () => bo.seen.synced && zoe.seen.synced);
  assert.deepEqual([bo.seen.hello.you, zoe.seen.hello.you], ['Bo', 'Zoë']);
  bo.write('bring chips');
  zoe.write('bring a speaker');
  await until('both read both paragraphs', () => bo.text().length === 2 && zoe.text().length === 2);
  assert.deepEqual(bo.text().sort(), ['bring a speaker', 'bring chips']);
  assert.deepEqual(bo.text(), zoe.text(), 'the two pages read the paragraphs in one order');
  await until(
    'each paragraph says who last edited it',
    () => bo.seen.lines?.filter(Boolean).length === 2,
  );
  assert.deepEqual(
    bo.seen.lines.map((e) => e.person),
    bo.text().map((p) => (p === 'bring chips' ? 'Bo' : 'Zoë')),
  );
  await until(
    "each sees the other's caret, named",
    () => bo.names().join() === 'Bo,Zoë' && zoe.names().join() === 'Bo,Zoë',
  );
});

test('a page opened without svrngs is named nobody, and its paragraph says so', async (t) => {
  t.mock.timers.enable({ apis: ['setInterval', 'setTimeout', 'Date'], now: 1_000_000 });
  mock = t.mock.timers;
  const keep = mkdtempSync(join(tmpdir(), 'doc-'));
  const srv = await server(keep);
  const anon = page(srv, null);
  t.after(async () => (anon.close(), srv.end(), rmSync(keep, { recursive: true })));
  await until('greeted', () => anon.seen.hello);
  assert.equal(anon.seen.hello.you, null);
  anon.write('hello');
  await until('its line arrives', () => anon.seen.lines?.[0]);
  assert.equal(anon.seen.lines[0].person, null);
});

test('a lender gone quiet is goodnight within QUIET_MS and a tick, and what was written meanwhile joins once it answers', async (t) => {
  t.mock.timers.enable({ apis: ['setInterval', 'setTimeout', 'Date'], now: 1_000_000 });
  mock = t.mock.timers;
  const keep = mkdtempSync(join(tmpdir(), 'doc-'));
  const srv = await server(keep);
  const bo = page(srv, 'Bo');
  const zoe = page(srv, 'Zoë');
  t.after(async () => (bo.close(), zoe.close(), srv.end(), rmSync(keep, { recursive: true })));
  bo.write('before');
  await until('both read it', () => zoe.text().includes('before'));

  const stopped = Date.now();
  srv.stop();
  // A bound of its own, not QUIET_MS's: a page that never says goodnight
  // fails here rather than waiting with it.
  await until('both say goodnight', () => bo.seen.away.length && zoe.seen.away.length, 8_000);
  for (const p of [bo, zoe]) {
    const after = p.seen.away[0].at - stopped;
    assert.ok(
      after >= QUIET_MS - 1_000 && after <= QUIET_MS + 1_500,
      `goodnight ${after} ms after the lid, not within ${QUIET_MS} ms and a tick`,
    );
  }
  assert.deepEqual(bo.names(), ['Bo'], 'no other caret stays drawn while the server is quiet');
  bo.write('while away, by Bo');
  zoe.write('while away, by Zoë');
  assert.equal(bo.s.state, 'away');

  await sleep(4_000);
  const woke = Date.now();
  srv.go();
  await until('both join again', () => bo.seen.joined.length && zoe.seen.joined.length, 20_000);
  await until(
    'both read all three',
    () => bo.text().length === 3 && zoe.text().length === 3,
    10_000,
  );
  assert.deepEqual(bo.text(), zoe.text());
  assert.deepEqual(bo.seen.joined[0].kept, 1, 'Bo is told his one edit joined');
  assert.ok(
    bo.seen.pokes > 0 && zoe.seen.pokes > 0,
    'each page asked the server plainly before a new socket',
  );
  console.log(
    `# goodnight after ${bo.seen.away[0].at - stopped} and ${zoe.seen.away[0].at - stopped} ms; joined ${bo.seen.joined[0].at - woke} and ${zoe.seen.joined[0].at - woke} ms after the lid opened`,
  );
});

test('the server reads the doc back whole after a restart', async (t) => {
  t.mock.timers.enable({ apis: ['setInterval', 'setTimeout', 'Date'], now: 1_000_000 });
  mock = t.mock.timers;
  const keep = mkdtempSync(join(tmpdir(), 'doc-'));
  t.after(() => rmSync(keep, { recursive: true }));
  let srv = await server(keep);
  const bo = page(srv, 'Bo');
  bo.write('kept');
  await until('Bo is told who wrote it', () => bo.seen.lines?.[0]?.person === 'Bo');
  bo.close();
  srv.end();
  srv = await server(keep);
  const cy = page(srv, 'Cy');
  t.after(async () => (cy.close(), srv.end()));
  await until('Cy reads it', () => cy.text().includes('kept'));
  await until('and who wrote it', () => cy.seen.lines?.[0]?.person === 'Bo');
});
