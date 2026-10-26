// SPDX-License-Identifier: AGPL-3.0-or-later
// The doc's server: its page, and one socket per open page, through which
// every edit and caret passes. Lent with `svrngs share`, so each page
// reaches it through svrngs's door, which names the person on the socket's
// upgrade (`x-svrngs-person`, `x-svrngs-key`): the server writes each edit
// down as theirs (`wrote`), and tells every page who last edited each
// paragraph (protocol.js `LINES`). It keeps the doc as those acts, one JSON
// line each, in `<keep>/acts.jsonl`, so a restart reads it back whole.
//
//   node apps/doc/server.mjs --port <port> --keep <dir> [--svrngs <svrngs's api file>]
//
// `--svrngs` is the `api` file in svrngs's data dir (`~/.svrngs/api`): the
// server asks it, with the token in the `token` file beside it, once a minute for this computer's name in its groups, so
// every page can say whose doc it is, and whose computer is not answering
// when it says goodnight. Without it, the pages name nobody.
// `DOC_DEBUG=1` traces each decision as a JSON line on stderr.

import { appendFileSync, existsSync, mkdirSync, readFileSync } from 'node:fs';
import http from 'node:http';
import { dirname, extname, join, normalize, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { bytesToB64, b64ToBytes, createAttribution } from './adapter.js';
import { AWARENESS, HELLO, HERE, HERE_EVERY_MS, LINES, SYNC, bare, read, said } from './protocol.js';
import { Y, awarenessProtocol, decoding, encoding, syncProtocol } from './vendor/doc-bundle.js';
import { accept } from './ws.mjs';

const HERE_DIR = dirname(fileURLToPath(import.meta.url));
const TYPES = { '.html': 'text/html; charset=utf-8', '.js': 'text/javascript; charset=utf-8', '.mjs': 'text/javascript; charset=utf-8' };
/** What the page may fetch: its own files, nothing else of this directory. */
const PAGE = new Set(['index.html', 'app.js', 'sync.js', 'adapter.js', 'protocol.js', 'vendor/doc-bundle.js']);
/** How long after an edit every page is told the paragraphs' last editors again. */
const LINES_AFTER_MS = 250;
/** How often the lender's name is asked of svrngs. */
const LENDER_EVERY_MS = 60_000;

const trace = process.env.DOC_DEBUG ? (what, fields = {}) => console.error(JSON.stringify({ at: new Date().toISOString(), what, ...fields })) : () => {};

/** A header svrngs's door set, read as the UTF-8 it wrote (door.rs sets the name's bytes). */
const told = (req, name) => {
  const v = req.headers[name];
  return typeof v === 'string' && v.length ? Buffer.from(v, 'latin1').toString('utf8') : null;
};

/**
 * Start the server; resolves to `{port, close}` once it listens.
 * `keep` is the directory the doc's acts are kept in; `api` svrngs's api
 * file, or none.
 */
export async function start({ port = 0, keep, api = null }) {
  if (!keep) throw new Error('say where the doc is kept: --keep <dir>');
  mkdirSync(keep, { recursive: true });
  const file = join(keep, 'acts.jsonl');
  const doc = new Y.Doc();
  const awareness = new awarenessProtocol.Awareness(doc);
  awareness.setLocalState(null);
  const attribution = createAttribution();
  const pages = new Set();
  let seq = 0;
  let lastTs = 0;
  let lender = null;

  // The doc as kept: every act, applied in the order written.
  if (existsSync(file)) {
    const acts = readFileSync(file, 'utf8').split('\n').filter(Boolean).map((l) => JSON.parse(l));
    Y.transact(doc, () => acts.forEach((a) => Y.applyUpdate(doc, b64ToBytes(a.update))), 'kept');
    attribution.absorb(acts);
    seq = acts.reduce((m, a) => Math.max(m, a.seq), 0);
    lastTs = acts.reduce((m, a) => Math.max(m, a.ts), 0);
    trace('doc: read back what was kept', { acts: acts.length, file });
  }

  const send = (page, bytes) => page.peer.send(bytes);
  const toAll = (bytes, except = null) => pages.forEach((p) => p !== except && send(p, bytes));

  let linesTimer = null;
  const linesSoon = () => {
    linesTimer ??= setTimeout(() => {
      linesTimer = null;
      toAll(said(LINES, attribution.entries()));
    }, LINES_AFTER_MS);
  };

  // An edit from a page: written down as the person's whose socket carried
  // it, then sent to every other page. One that came from the kept file is
  // not written again.
  doc.on('update', (update, origin) => {
    if (!pages.has(origin)) return;
    const ts = Math.max(Date.now() / 1000, lastTs + 0.001);
    lastTs = ts;
    seq += 1;
    const act = { id: String(seq), seq, ts, actor: origin.key ?? '', person: origin.person, update: bytesToB64(update) };
    appendFileSync(file, `${JSON.stringify(act)}\n`);
    attribution.absorb([act]);
    trace('doc: an edit written as its socket\'s person', { seq, person: act.person, bytes: update.length });
    const e = encoding.createEncoder();
    encoding.writeVarUint(e, SYNC);
    syncProtocol.writeUpdate(e, update);
    toAll(encoding.toUint8Array(e), origin);
    linesSoon();
  });

  // A caret or a name: kept against the page whose socket said it, so its
  // carets go when the socket does, and sent to every page.
  awareness.on('update', ({ added, updated, removed }, origin) => {
    if (pages.has(origin)) {
      added.forEach((id) => origin.ids.add(id));
      removed.forEach((id) => origin.ids.delete(id));
    }
    const changed = [...added, ...updated, ...removed];
    const e = encoding.createEncoder();
    encoding.writeVarUint(e, AWARENESS);
    encoding.writeVarUint8Array(e, awarenessProtocol.encodeAwarenessUpdate(awareness, changed));
    toAll(encoding.toUint8Array(e));
  });

  const opened = (req, peer) => {
    const page = { peer, person: told(req, 'x-svrngs-person'), key: told(req, 'x-svrngs-key'), ids: new Set() };
    pages.add(page);
    trace('socket: a page opened', { person: page.person, named: page.person !== null, pages: pages.size });
    send(page, said(HELLO, { you: page.person, key: page.key, lender }));
    const e = encoding.createEncoder();
    encoding.writeVarUint(e, SYNC);
    syncProtocol.writeSyncStep1(e, doc);
    send(page, encoding.toUint8Array(e));
    const states = [...awareness.getStates().keys()];
    if (states.length) {
      const a = encoding.createEncoder();
      encoding.writeVarUint(a, AWARENESS);
      encoding.writeVarUint8Array(a, awarenessProtocol.encodeAwarenessUpdate(awareness, states));
      send(page, encoding.toUint8Array(a));
    }
    send(page, said(LINES, attribution.entries()));
    peer.onmessage = (data) => {
      if (typeof data === 'string') return trace('socket: refused — a text message, which the doc never sends', { person: page.person });
      let m;
      try {
        m = read(data);
        if (m.type === SYNC) {
          const reply = encoding.createEncoder();
          encoding.writeVarUint(reply, SYNC);
          syncProtocol.readSyncMessage(m.body, reply, doc, page);
          if (encoding.length(reply) > 1) send(page, encoding.toUint8Array(reply));
        } else if (m.type === AWARENESS) {
          awarenessProtocol.applyAwarenessUpdate(awareness, decoding.readVarUint8Array(m.body), page);
        } else {
          trace('socket: refused — a message of a type pages do not send', { type: m.type, person: page.person });
        }
      } catch (e) {
        trace('socket: refused — a message that does not read', { type: m?.type, person: page.person, error: String(e) });
      }
    };
    peer.onclose = () => {
      pages.delete(page);
      awarenessProtocol.removeAwarenessStates(awareness, [...page.ids], null);
      trace('socket: a page closed', { person: page.person, pages: pages.size });
    };
  };

  const here = setInterval(() => toAll(bare(HERE)), HERE_EVERY_MS);

  const askLender = async () => {
    if (!api) return;
    try {
      const base = readFileSync(api, 'utf8').trim().replace(/\/$/, '');
      const token = readFileSync(join(dirname(api), 'token'), 'utf8').trim();
      const asked = await fetch(`${base}/v1/groups`, { headers: { authorization: `Bearer ${token}` } });
      const groups = (await asked.json()).groups ?? [];
      const names = [...new Set(groups.map((g) => g.person).filter(Boolean))];
      const now = names.length === 1 ? names[0] : null;
      trace('lender: asked svrngs this computer\'s name', { names, lender: now, said: names.length > 1 ? 'two names in its groups, so none' : undefined });
      if (now !== lender) {
        lender = now;
        pages.forEach((p) => send(p, said(HELLO, { you: p.person, key: p.key, lender })));
      }
    } catch (e) {
      trace('lender: svrngs did not answer, so the lender stays as it was', { api, lender, error: String(e) });
    }
  };
  await askLender();
  const asking = setInterval(askLender, LENDER_EVERY_MS);

  const server = http.createServer((req, res) => {
    const path = decodeURIComponent(new URL(req.url, 'http://doc').pathname).replace(/^\/+/, '') || 'index.html';
    const name = normalize(path);
    // What a page asks before a new socket (sync.js `poke`): nothing, at once.
    if (req.method === 'GET' && name === 'here') return res.writeHead(204, { 'cache-control': 'no-store' }).end();
    if (req.method !== 'GET' || !PAGE.has(name)) {
      trace('page: refused — not one of the page\'s files', { path });
      res.writeHead(404, { 'content-type': 'text/plain; charset=utf-8' }).end('There is no such page here.');
      return;
    }
    res.writeHead(200, { 'content-type': TYPES[extname(name)] ?? 'application/octet-stream', 'cache-control': 'no-cache' });
    res.end(readFileSync(join(HERE_DIR, name)));
  });
  server.on('upgrade', (req, socket, head) => {
    if (new URL(req.url, 'http://doc').pathname !== '/sync') {
      trace('socket: refused — an upgrade not at /sync', { path: req.url });
      return socket.end('HTTP/1.1 404 Not Found\r\nconnection: close\r\ncontent-length: 0\r\n\r\n');
    }
    accept(req, socket, head, (peer) => opened(req, peer));
  });
  await new Promise((done) => server.listen(port, '127.0.0.1', done));
  trace('server: listening on loopback alone', { port: server.address().port, keep });
  return {
    port: server.address().port,
    close: () => {
      clearInterval(here);
      clearInterval(asking);
      clearTimeout(linesTimer);
      pages.forEach((p) => p.peer.close(1001));
      return new Promise((done) => server.close(done));
    },
  };
}

if (process.argv[1] && fileURLToPath(import.meta.url) === resolve(process.argv[1])) {
  const arg = (name) => {
    const i = process.argv.indexOf(name);
    return i > 0 ? process.argv[i + 1] : null;
  };
  const port = Number(arg('--port'));
  if (!Number.isInteger(port) || !arg('--keep')) {
    console.error('node apps/doc/server.mjs --port <port> --keep <dir> [--svrngs <svrngs\'s api file>]');
    process.exit(2);
  }
  const { port: at } = await start({ port, keep: arg('--keep'), api: arg('--svrngs') });
  console.log(`The doc is at http://127.0.0.1:${at}/, kept in ${arg('--keep')}. Lend it: svrngs share ${at} --name doc --kind doc`);
}
