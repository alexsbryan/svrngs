// Who is asking (GRAPEVINE_DEMO §13.1-§13.3; gv-page (c)): the server
// answers its own person at its own loopback address, and no one else.
// A foreign Host, a foreign Origin and a write sent as anything but JSON
// (a form posting JSON-looking text, as a page on another site can without
// a preflight) are each refused, said, with nothing written. In record mode
// "me" is each group's own, as svrngs's GET /v1/groups answers it, never
// one name for every group. Each law's plant is one line of a copy of
// server.mjs or relay.mjs, its handler asked in this process (no listener)
// and svrngs's API a function; a plant that no longer applies fails by name.

import assert from 'node:assert/strict';
import { cpSync, mkdtempSync, readFileSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { Readable } from 'node:stream';
import { test } from 'node:test';
import { pathToFileURL } from 'node:url';

import { askAct } from './grapevine.js';
import { keyPair } from './seal.mjs';

const HERE = new URL('.', import.meta.url).pathname;
const ASK = {
  kind: 'tool',
  fields: { what: 'tile saw', when: 'Saturday' },
  group: 'page',
  hops: 1,
  until: '2099-01-01',
};
/** The address the server answers at, as its listener would say it. */
const OWN = '127.0.0.1:4100';

/** The server in `dir` (this one, or a planted copy) on `config`, its handler in this process. */
async function serve(dir, config, fetch) {
  const data = mkdtempSync(join(tmpdir(), 'grapevine-who-'));
  writeFileSync(join(data, 'token'), 'the-token');
  const { grapevine } = await import(pathToFileURL(join(dir, 'server.mjs')).href);
  return grapevine({ data, token: join(data, 'token'), ...config }, { fetch, address: () => OWN });
}

/** A request as any program could send it, its Host and Origin its own to choose. */
function ask(app, { method = 'GET', path = '/?as=Sam', host = OWN, origin, type, body } = {}) {
  const req = Readable.from(body === undefined ? [] : [Buffer.from(body)]);
  req.method = method;
  req.url = path;
  req.headers = { host, ...(origin && { origin }), ...(type && { 'content-type': type }) };
  return new Promise((done, fail) => {
    const res = {
      writeHead(status) {
        this.status = status;
        return this;
      },
      end(text = '') {
        done({ status: this.status, text: String(text) });
        return this;
      },
    };
    app.handle(req, res).catch(fail);
  });
}

/** A file of its own: Sam and Jo in 1400 Page. */
function town() {
  const dir = mkdtempSync(join(tmpdir(), 'grapevine-town-'));
  const path = join(dir, 'town.json');
  writeFileSync(
    path,
    JSON.stringify({ groups: { page: { name: '1400 Page', people: ['Sam', 'Jo'] } } }),
  );
  return {
    file: path,
    acting: ['Sam', 'Jo'],
    written: () => (JSON.parse(readFileSync(path, 'utf8')).ops?.page ?? []).length,
  };
}

/**
 * svrngs's loopback API as a computer standing as Jo in 1400 Page and as
 * Joanna in Waller block, and in The yard as no one: what it answers, and
 * each append it is sent.
 */
function svrngs() {
  const sam = keyPair();
  const handle = Buffer.alloc(16, 7).toString('base64');
  const groups = [
    { id: 'page', name: '1400 Page', person: 'Jo' },
    { id: 'waller', name: 'Waller block', person: 'Joanna' },
    { id: 'yard', name: 'The yard', person: null },
  ];
  const people = { page: ['Sam', 'Jo'], waller: ['Joanna', 'Kim'], yard: ['Kim', 'Lee'] };
  const logs = {
    page: [
      {
        id: 'a1',
        ts_unix: 1_760_000_000,
        person: 'Sam',
        payload: askAct({
          kind: 'tool',
          fields: { what: 'tile saw', when: 'Saturday' },
          hops: 1,
          expires: 4_000_000_000,
          seal_to: sam.public,
          handle,
        }),
      },
    ],
    waller: [],
    yard: [],
  };
  const appended = [];
  /** svrngs's record API, as a function: an ask without the token refused (give-token). */
  async function fetch(url, init = {}) {
    if (init.headers?.authorization !== 'Bearer the-token')
      return new Response('{"kind":"no_token"}', { status: 401 });
    const at = new URL(url).searchParams.get('namespace');
    const path = new URL(url).pathname;
    if (path === '/v1/groups') return Response.json({ groups });
    if (path === '/v1/rail/people')
      return Response.json({
        people: people[at].map((p) => ({ person: p, person_key: `key-${p}` })),
      });
    if (path === '/v1/rail/log') return Response.json({ ops: logs[at], complete: true });
    if (path === '/v1/rail/append') {
      appended.push({ group: at, ...JSON.parse(init.body) });
      return Response.json({ id: `w${appended.length}` });
    }
    return new Response('{}', { status: 404 });
  }
  return { fetch, appended };
}

const LAWS = {
  /** A request under another Host, as a rebinding page sends it, is refused and said. */
  async 'a foreign Host is refused'(dir) {
    const t = town();
    const s = await serve(dir, t);
    const got = await ask(s, { host: 'rebound.example:4100' });
    const wrote = await ask(s, {
      method: 'POST',
      path: '/ask?as=Sam',
      host: 'rebound.example:4100',
      type: 'application/json',
      body: JSON.stringify(ASK),
    });
    if (got.status !== 421 || !got.text.includes('answers only at its own address'))
      return `a page under another Host answered ${got.status}`;
    return wrote.status === 421 && t.written() === 0
      ? null
      : `a write under another Host answered ${wrote.status}, ${t.written()} written`;
  },
  /** A write from a page on another site is refused and said, nothing written. */
  async 'a foreign Origin is refused'(dir) {
    const t = town();
    const s = await serve(dir, t);
    const got = await ask(s, {
      method: 'POST',
      path: '/ask?as=Sam',
      origin: 'http://elsewhere.example',
      type: 'application/json',
      body: JSON.stringify(ASK),
    });
    const said = JSON.parse(got.text).said ?? '';
    return got.status === 403 && said.includes('Another site') && t.written() === 0
      ? null
      : `a write from another site answered ${got.status} (${said}), ${t.written()} written`;
  },
  /** A write sent as a form, its body JSON-looking text, is refused and said, nothing written. */
  async 'a write not sent as JSON is refused'(dir) {
    const t = town();
    const s = await serve(dir, t);
    const out = [];
    for (const type of ['application/x-www-form-urlencoded', 'text/plain', undefined]) {
      const got = await ask(s, {
        method: 'POST',
        path: '/ask?as=Sam',
        type,
        body: JSON.stringify(ASK),
      });
      const said = JSON.parse(got.text).said ?? '';
      if (got.status !== 415 || !said.includes('takes only what its own page sends'))
        out.push(`${type ?? 'no type'}: ${got.status} (${said})`);
    }
    const ok = await ask(s, {
      method: 'POST',
      path: '/ask?as=Sam',
      origin: `http://${OWN}`,
      type: 'application/json',
      body: JSON.stringify(ASK),
    });
    if (ok.status !== 200) out.push(`its own page's write answered ${ok.status}: ${ok.text}`);
    return out.length || t.written() !== 1
      ? `${out.join('; ')}; ${t.written()} written, not 1`
      : null;
  },
  /** In record mode "me" is each group's own, as svrngs answers it; never ?as, never one name for all. */
  async '"me" is read per group from svrngs'(dir) {
    const api = svrngs();
    const s = await serve(dir, { record: 'http://127.0.0.1:4200' }, api.fetch);
    const other = await ask(s, { path: '/?as=Kim' });
    if (other.status !== 400) return `?as=Kim in record mode answered ${other.status}`;
    const home = await ask(s, { path: '/' });
    if (!home.text.includes('Carry to Waller block'))
      return 'Jo is not offered a carry to Waller block, where svrngs says this computer stands as Joanna';
    if (home.text.includes('The yard</h2>'))
      return 'The yard, where svrngs names no one here, is shown as one of theirs';
    const carried = await ask(s, {
      method: 'POST',
      path: '/carry',
      type: 'application/json',
      body: JSON.stringify({ ask: 'a1', group: 'waller' }),
    });
    const [w] = api.appended;
    if (carried.status !== 200 || !w)
      return `the carry answered ${carried.status}: ${carried.text}`;
    return w.group === 'waller' && w.on_behalf_of === 'Joanna' && w.payload?.type === 'carry'
      ? null
      : `the carry was written in ${w.group} as ${w.on_behalf_of}, not in waller as Joanna`;
  },
};

/** A copy of the app with `file`'s `from` changed to `to`. */
function planted(file, from, to) {
  const dir = mkdtempSync(join(tmpdir(), 'grapevine-plant-'));
  cpSync(HERE, dir, { recursive: true, filter: (f) => !f.includes('node_modules') });
  const src = readFileSync(join(dir, file), 'utf8');
  assert.equal(src.split(from).length, 2, `the plant no longer applies to ${file}: ${from}`);
  writeFileSync(join(dir, file), src.replace(from, to));
  return dir;
}

const PLANTS = {
  'a foreign Host is refused': ['server.mjs', 'if (req.headers.host !== own) {', 'if (false) {'],
  'a foreign Origin is refused': [
    'server.mjs',
    'if (req.headers.origin !== undefined && req.headers.origin !== `http://${own}`) {',
    'if (false) {',
  ],
  'a write not sent as JSON is refused': [
    'server.mjs',
    "!/^application\\/json\\s*(;|$)/i.test(req.headers['content-type'] ?? '')",
    'false',
  ],
  // one name, the first group's, for every group
  '"me" is read per group from svrngs': [
    'relay.mjs',
    "let me = typeof g.person === 'string' ? g.person : null;",
    "let me = typeof groups[0].person === 'string' ? groups[0].person : null;",
  ],
};

for (const [law, check] of Object.entries(LAWS)) {
  test(law, async () => {
    assert.equal(await check(HERE), null);
  });
  test(`${law}: caught when its guard is dropped`, async (t) => {
    const [file, from, to] = PLANTS[law];
    const why = await check(planted(file, from, to));
    assert.notEqual(why, null, `the plant for ${law} was not caught`);
    t.diagnostic(why);
  });
}
