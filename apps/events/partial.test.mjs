// A group whose record this computer has not fully received (#151; the
// queue's fence 2): the server run against a stand-in for svrngs's record
// API whose log says `complete: false` for two of Sam's three groups. Each
// is said as that where its posts or answers are shown — on the home list
// and in each section of the host's view — and a group with nothing
// received is never said to have nothing posted. Red on the plant the
// finding names, `complete` dropped outside the path's group: The Club
// said "Nothing posted yet." and dWeb's answers shown as if all were here.
// Beside it, against the same stand-in, two people called bo (keys-names).

import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { mkdtempSync, writeFileSync } from 'node:fs';
import http from 'node:http';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { test } from 'node:test';

const E = '0123456789abcdef';
const dinner = { kind: 'event', event: E, what: 'dinner', when: 'Friday 7', where: 'the house' };
const posted = (id) => [{ id, ts_unix: 1000, person: 'sam', person_key: 'k-sam', payload: dinner }];
const PARTIAL = {
  house: { name: 'The House', people: ['sam', 'bo'], ops: posted('h1'), complete: true },
  dweb: { name: 'dWeb', people: ['sam', 'cy'], ops: posted('d1'), complete: false },
  club: { name: 'The Club', people: ['sam', 'di'], ops: [], complete: false },
};
const PARTLY = /may not have reached this computer yet/;

/** svrngs's record API as the store reads it (store.mjs `record`). */
function svrngs(GROUPS = PARTIAL) {
  const server = http.createServer((req, res) => {
    const url = new URL(req.url, 'http://x');
    const g = GROUPS[url.searchParams.get('namespace')];
    const answer = url.pathname === '/v1/groups'
      ? { groups: Object.entries(GROUPS).map(([id, x]) => ({ id, name: x.name })) }
      : url.pathname === '/v1/rail/people' && g
        ? { people: g.people.map((p) => (typeof p === 'string' ? { person: p, person_key: `k-${p}` } : p)) }
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

// Two people called bo (keys-names): the second is invited and answers as
// themselves, told apart by when each joined, and their "I'm coming" is a
// record of their own, never a correction of the first bo's answer. Red at
// 0c9dc094: the fold kept people by name, so the second bo was shown the
// first's guests and their change of mind corrected the first's act.
test('two people called bo are two: each answers as themselves, told apart by when they joined', async () => {
  const oct = (d) => ({ at: Date.UTC(2026, 9, d, 12) / 1000 });
  const people = [{ person: 'sam', person_key: 'k-sam', joined: 'started' }, { person: 'bo', person_key: 'k-bo1', joined: oct(1) }, { person: 'bo', person_key: 'k-bo2', joined: oct(3) }];
  const ops = [...posted('h1'), { id: 'h2', ts_unix: 1100, person: 'bo', person_key: 'k-bo1', payload: { kind: 'coming', event: E, guests: ['Maya'] } }];
  const api = await svrngs({ house: { name: 'The House', people, ops, complete: true } });
  const app = await events(`http://127.0.0.1:${api.address().port}`);
  try {
    const as = (key, path, init = {}) => fetch(`${app.at}${path}`, { ...init, redirect: 'manual', headers: { 'x-svrngs-key': key, ...init.headers } });
    const second = await (await as('k-bo2', `/g/house/e/${E}`)).text();
    assert.match(second, /<li>bo \(joined 1 Oct\) \+ Maya<\/li>/, "the first bo's answer, said as theirs");
    assert.match(second, /<ul class="invited"><li>bo \(joined 1 Oct\)<\/li><li>bo \(joined 3 Oct\)<\/li><\/ul>/);
    assert.match(second, /<button>I'm coming<\/button>/, 'the second bo has not answered');
    assert.doesNotMatch(second, /value="Maya"/, "nor are the first bo's guests theirs");
    assert.match(await (await as('k-bo1', `/g/house/e/${E}`)).text(), /value="Maya"/);
    const coming = await as('k-bo2', `/g/house/e/${E}/coming`, { method: 'POST', headers: { 'content-type': 'application/x-www-form-urlencoded' }, body: 'guests=' });
    assert.deepEqual(JSON.parse(coming.headers.get('x-svrngs-append')), [{ group: 'house', op: { kind: 'coming', event: E } }], 'a record of their own');
  } finally {
    app.stop();
    api.close();
  }
});
