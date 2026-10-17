// A group whose record this computer has not fully received (#151; the
// queue's fence 2): the server run against a stand-in for svrngs's record
// API whose log says `complete: false` for two of Sam's three groups. Each
// is said as that where its posts or answers are shown — on the home list
// and in each section of the host's view — and a group with nothing
// received is never said to have nothing posted. Red on the plant the
// finding names, `complete` dropped outside the path's group: The Club
// said "Nothing posted yet." and dWeb's answers shown as if all were here.

import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { mkdtempSync, writeFileSync } from 'node:fs';
import http from 'node:http';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { test } from 'node:test';

const E = '0123456789abcdef';
const dinner = { kind: 'event', event: E, what: 'dinner', when: 'Friday 7', where: 'the house' };
const posted = (id) => [{ id, ts_unix: 1000, person: 'sam', payload: dinner }];
const GROUPS = {
  house: { name: 'The House', people: ['sam', 'bo'], ops: posted('h1'), complete: true },
  dweb: { name: 'dWeb', people: ['sam', 'cy'], ops: posted('d1'), complete: false },
  club: { name: 'The Club', people: ['sam', 'di'], ops: [], complete: false },
};
const PARTLY = /may not have reached this computer yet/;

/** svrngs's record API as the store reads it (store.mjs `record`). */
function svrngs() {
  const server = http.createServer((req, res) => {
    const url = new URL(req.url, 'http://x');
    const g = GROUPS[url.searchParams.get('namespace')];
    const answer = url.pathname === '/v1/groups'
      ? { groups: Object.entries(GROUPS).map(([id, x]) => ({ id, name: x.name })) }
      : url.pathname === '/v1/rail/people' && g
        ? { people: g.people.map((p) => ({ person: p, person_key: `k-${p}` })) }
        : url.pathname === '/v1/rail/log' && g
          ? { ops: g.ops, complete: g.complete }
          : null;
    res.writeHead(answer ? 200 : 404, { 'content-type': 'application/json' });
    res.end(JSON.stringify(answer ?? { error: 'no such thing here' }));
  });
  return new Promise((done) => server.listen(0, '127.0.0.1', () => done(server)));
}

async function events(api) {
  const dir = mkdtempSync(join(tmpdir(), 'events-partial-'));
  writeFileSync(join(dir, 'config.json'), JSON.stringify({ record: api }));
  const child = spawn(process.execPath, [new URL('./server.mjs', import.meta.url).pathname, join(dir, 'config.json')], { stdio: ['ignore', 'pipe', 'inherit'] });
  const line = await new Promise((done, fail) => {
    child.stdout.once('data', (b) => done(String(b)));
    child.once('exit', (code) => fail(new Error(`the server exited ${code} before it listened`)));
  });
  return { at: `http://127.0.0.1:${JSON.parse(line).port}`, stop: () => child.kill() };
}

/** Each `<section>` of `html` by its heading, its text. */
function sections(html) {
  const out = {};
  for (const s of html.split('<section').slice(1)) {
    const h2 = s.match(/<h2>([^<]*)<\/h2>/)?.[1];
    if (h2) out[h2] = s.split('</section>')[0];
  }
  return out;
}

test("a group not fully received is said so on the home list and in the host's view, never as nothing posted", async () => {
  const api = await svrngs();
  const app = await events(`http://127.0.0.1:${api.address().port}`);
  try {
    const as = async (path) => (await fetch(`${app.at}${path}`, { headers: { 'x-svrngs-key': 'k-sam' } })).text();

    const home = sections(await as('/'));
    assert.doesNotMatch(home['The Club'], /Nothing posted yet/, "The Club's posts may not have reached here");
    assert.match(home['The Club'], PARTLY);
    assert.match(home.dWeb, /dinner/);
    assert.match(home.dWeb, PARTLY);
    assert.match(home['The House'], /dinner/);
    assert.doesNotMatch(home['The House'], PARTLY, 'The House holds all it knows of');

    const hosts = sections(await as(`/g/house/e/${E}`));
    assert.match(hosts.dWeb, PARTLY, "dWeb's answers may not all be here");
    assert.doesNotMatch(hosts['The House'], PARTLY);
  } finally {
    app.stop();
    api.close();
  }
});
