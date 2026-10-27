// Give's page in a browser (give-app-keep (a); GIVE.md §3 scene 1): a
// computer's page, so in Chromium alone (CHARTER fence 13). A server of the
// test's own serves page/ under a carried page's policy, read from the
// trust floor (scripts/trust_floor.txt, `page carried`), with a shelf of
// two cards, and answers `svrngs/acts` as svrngs's door does
// (crates/svrngs-node/src/carried.rs `acts`): who asks, and give's acts,
// each new one written as the computer the test names. A write from a
// phone is refused as the door refuses it.
//
// Cy's computer can't keep apps yet, so each row asks once to install what
// it needs, and "Not now" leaves it unasked after. Once cy's computer says
// it can keep, cy starts Minecraft, says yes to its makers' terms, and the
// row says it is kept by cy's computer; on Bo's page, kept by cy's. Bo's
// own start on a computer that can't keep is told what it needs. No page
// says container, image, volume, snapshot, keeper or port forwarding.
// Without Playwright or its browser this fails naming the install
// (`launch` exits 3); it never skips.

import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import http from 'node:http';
import { extname, join, normalize } from 'node:path';
import { test } from 'node:test';

import { launch } from '../../web/tests/page.mjs';

const HERE = new URL('.', import.meta.url).pathname;
const ROOT = join(HERE, '../..');
/** What a person never reads here (CHARTER fence 6), and the fold's own words. */
const NOT_SAID = /\b(containers?|images?|volumes?|snapshots?|keepers?|port forwarding|fold|acts?|refused)\b/i;
const MINECRAFT_TERMS = "Minecraft's makers ask whoever runs a server to agree to their terms.";
const CARDS = [
  { kind: 'minecraft', title: 'Minecraft', about: 'A world your group plays in.', door: 'tcp', asks: [{ say: MINECRAFT_TERMS, link: 'https://aka.ms/MinecraftEULA' }], version: 'm1' },
  { kind: 'standin', title: 'Stand-in', about: 'A page that counts how often it has started.', door: 'http', asks: [], version: 's1' },
];

/** A carried page's policy as the floor pins it: its directives under `page carried`. */
async function carriedPolicy() {
  const floor = await readFile(join(ROOT, 'scripts/trust_floor.txt'), 'utf8');
  const block = floor.split('\npage carried ')[1].split('\n').slice(1);
  const lines = block.slice(0, block.findIndex((l) => !l.startsWith('  ')));
  return lines.map((l) => l.trim().split(' | ')[0]).join('; ');
}

const TYPES = { '.html': 'text/html; charset=utf-8', '.js': 'text/javascript; charset=utf-8', '.mjs': 'text/javascript; charset=utf-8', '.css': 'text/css; charset=utf-8', '.json': 'application/json' };

/** The app at an origin of its own: who asks is `x-person` and `x-key`, the computer it is open on `x-computer`. */
async function serve() {
  const policy = await carriedPolicy();
  const ops = [];
  const record = (actor, person, person_key, payload) =>
    ops.push({ id: `op${ops.length + 1}`, actor, person, person_key, ts_unix: ops.length + 1, voided: false, payload });
  const server = http.createServer(async (req, res) => {
    res.setHeader('content-security-policy', policy);
    const [you, key, computer] = ['x-person', 'x-key', 'x-computer'].map((h) => req.headers[h]);
    const path = new URL(req.url, 'http://app').pathname;
    const json = (status, body) => res.writeHead(status, { 'content-type': 'application/json' }).end(JSON.stringify(body));
    if (path === '/svrngs/acts') {
      if (req.method === 'GET') return json(200, { you, you_key: key, group: 'elm', people: ['cy', 'bo'], spans: null, ops });
      let body = '';
      for await (const chunk of req) body += chunk;
      if (computer === 'phone') return json(403, { error: 'Open give from your home page to write in it.' });
      record(computer, you, key, JSON.parse(body).op);
      return json(200, {});
    }
    if (path === '/cards.json') return json(200, CARDS);
    const file = join(HERE, 'page', normalize(path === '/' ? 'index.html' : path));
    try {
      res.writeHead(200, { 'content-type': TYPES[extname(file)] ?? 'application/octet-stream' }).end(await readFile(file));
    } catch {
      res.writeHead(404).end();
    }
  });
  await new Promise((done) => server.listen(0, '127.0.0.1', done));
  return { at: `http://127.0.0.1:${server.address().port}/`, ops, record, stop: () => server.close() };
}

test('a group starts an app from the shelf, and every member reads who keeps it (chromium)', async () => {
  const server = await serve();
  const { context, close } = await launch('chromium');
  const errors = [];
  try {
    const page = await context.newPage();
    page.on('pageerror', (e) => errors.push(String(e)));
    const traced = [];
    page.on('console', (m) => m.type() === 'debug' && traced.push(m.text()));
    const said = [];
    const text = () => page.innerText('main');
    const as = async (person, key, computer) => {
      await context.setExtraHTTPHeaders({ 'x-person': person, 'x-key': key, 'x-computer': computer });
      await page.goto(server.at);
      await page.waitForFunction(() => !document.body.innerText.includes('Reading what your group keeps'), null, { timeout: 60_000 });
      said.push(await text());
    };
    const press = async (name, row) => {
      const before = await text();
      const scope = row ? page.locator('li', { hasText: row }) : page;
      await scope.getByRole('button', { name, exact: true }).first().click();
      await page.waitForFunction((b) => document.querySelector('main').innerText !== b, before, { timeout: 30_000 });
      said.push(await text());
    };
    const row = (title) => page.locator('li', { hasText: title }).innerText();

    // Cy's computer can't keep yet: each row asks, once, for the install.
    await as('cy', 'ka', 'a1');
    assert.match(await row('Minecraft'), /To keep Minecraft on this computer, svrngs needs Podman \(free, one install\) and Give’s program for computers\. Get Podman · Get Give’s program/);
    assert.equal(await page.getByRole('link', { name: 'Get Podman' }).first().getAttribute('href'), 'https://podman.io/docs/installation');
    await press('Not now', 'Minecraft');
    assert.doesNotMatch(await text(), /needs Podman/);
    await as('cy', 'ka', 'a1');
    assert.doesNotMatch(await text(), /needs Podman/, 'asked once');

    // Cy's computer says it can keep, as the keeper program does; cy starts
    // Minecraft, is asked its makers' terms, and agrees.
    server.record('a1', 'cy', 'ka', { kind: 'ready' });
    await as('cy', 'ka', 'a1');
    await press('Start it for the group', 'Minecraft');
    assert.match(await row('Minecraft'), new RegExp(`${MINECRAFT_TERMS} Read them`));
    assert.equal(await page.getByRole('link', { name: 'Read them' }).getAttribute('href'), 'https://aka.ms/MinecraftEULA');
    await press('Agree and start', 'Minecraft');
    assert.match(await row('Minecraft'), /Minecraft · kept by your computer/);
    const started = server.ops.at(-1);
    assert.deepEqual(
      [started.actor, started.payload],
      ['a1', { kind: 'started', app: 'minecraft', card: 'm1', agreed: true }],
      'the start names the card version and the yes, written by the computer the page is open on',
    );
    assert.match(await row('Stand-in'), /Start it for the group/, 'the other app is still to start');

    // Bo reads who keeps it; Bo's own start, on a computer that can't keep,
    // is told what that computer needs.
    await as('bo', 'kb', 'b1');
    assert.match(await row('Minecraft'), /Minecraft · kept by cy’s computer/);
    await press('Start it for the group', 'Stand-in');
    assert.match(await row('Stand-in'), /This computer can’t keep apps yet\. To keep them, svrngs needs Podman/);
    assert.match(await row('Stand-in'), /Start it for the group/);

    // A phone's write is refused by svrngs, and said.
    await as('bo', 'kb', 'phone');
    await press('Start it for the group', 'Stand-in');
    assert.match(await text(), /Stand-in didn’t start: Open give from your home page to write in it\./);

    for (const words of said) assert.doesNotMatch(words, NOT_SAID);
    for (const what of ['cards.json read', 'svrngs/acts answered', 'folded', 'started, written or refused', 'the fold refused an act']) {
      assert.ok(traced.some((l) => l.startsWith(`Give: ${what} `)), `traced: ${what}`);
    }
    assert.deepEqual(errors, []);
  } finally {
    await close();
    server.stop();
  }
});

// Scene 3 (give-app-move): cy moves Minecraft to bo's computer. "Move it
// to…" offers it to a computer that can keep; bo is asked, with Minecraft's
// makers' terms. Bo's no leaves everything as it was, and cy is told. On
// bo's yes the row says it is moving, and once cy's computer writes the
// move, every member reads it kept by bo's.
test('a member moves an app to another computer on its person’s yes, and a no leaves it as it was (chromium)', async () => {
  const server = await serve();
  const { context, close } = await launch('chromium');
  const errors = [];
  try {
    const page = await context.newPage();
    page.on('pageerror', (e) => errors.push(String(e)));
    const traced = [];
    page.on('console', (m) => m.type() === 'debug' && traced.push(m.text()));
    const said = [];
    const text = () => page.innerText('main');
    const as = async (person, key, computer) => {
      await context.setExtraHTTPHeaders({ 'x-person': person, 'x-key': key, 'x-computer': computer });
      await page.goto(server.at);
      await page.waitForFunction(() => !document.body.innerText.includes('Reading what your group keeps'), null, { timeout: 60_000 });
      said.push(await text());
    };
    const press = async (name, row) => {
      const before = await text();
      await page.locator('li', { hasText: row }).getByRole('button', { name, exact: true }).first().click();
      await page.waitForFunction((b) => document.querySelector('main').innerText !== b, before, { timeout: 30_000 });
      said.push(await text());
    };
    const row = (title) => page.locator('li', { hasText: title }).innerText();
    const last = () => {
      const op = server.ops.at(-1);
      return [op.actor, op.payload];
    };

    server.record('a1', 'cy', 'ka', { kind: 'ready' });
    server.record('b1', 'bo', 'kb', { kind: 'ready' });
    server.record('a1', 'cy', 'ka', { kind: 'started', app: 'minecraft', card: 'm1', agreed: true });
    const started = server.ops.at(-1).id;

    // Bo, whose computer doesn't keep it, has nothing to move.
    await as('bo', 'kb', 'b1');
    assert.doesNotMatch(await row('Minecraft'), /Move it to/);

    // Cy offers it to bo's computer.
    await as('cy', 'ka', 'a1');
    await press('Move it to…', 'Minecraft');
    assert.match(await row('Minecraft'), /Move Minecraft to:/);
    await press('bo’s computer', 'Minecraft');
    assert.deepEqual(last(), ['a1', { kind: 'offered', app: 'minecraft', over: started, to: 'b1' }]);
    assert.match(await row('Minecraft'), /Minecraft · kept by your computer[\s\S]*Waiting for bo to say yes\./);

    // Bo is asked, with Minecraft's makers' terms, and says not now.
    await as('bo', 'kb', 'b1');
    assert.match(await row('Minecraft'), new RegExp(`cy wants to move Minecraft to your computer\\.[\\s\\S]*${MINECRAFT_TERMS} Read them`));
    await press('Not now', 'Minecraft');
    const first = server.ops.at(-2).id;
    assert.deepEqual(last(), ['b1', { kind: 'declined', app: 'minecraft', offer: first }]);
    assert.match(await row('Minecraft'), /Minecraft · kept by cy’s computer/);
    assert.doesNotMatch(await row('Minecraft'), /wants to move|Agree/, 'a no: bo is not asked again');

    // Cy is told, and everything is as it was: cy offers it again.
    await as('cy', 'ka', 'a1');
    assert.match(await row('Minecraft'), /Minecraft · kept by your computer[\s\S]*bo said not now\./);
    await press('Move it to…', 'Minecraft');
    await press('bo’s computer', 'Minecraft');
    const second = server.ops.at(-1).id;

    // Bo agrees, and keeps it.
    await as('bo', 'kb', 'b1');
    await press('Agree and keep it here', 'Minecraft');
    assert.deepEqual(last(), ['b1', { kind: 'accepted', app: 'minecraft', offer: second, agreed: true }]);
    assert.match(await row('Minecraft'), /Moving to your computer…/);
    await as('cy', 'ka', 'a1');
    assert.match(await row('Minecraft'), /Moving to bo’s computer…/);
    assert.doesNotMatch(await row('Minecraft'), /Move it to/, 'nothing more to offer while it moves');

    // Cy's computer writes the move, as Give's program does.
    server.record('a1', 'cy', 'ka', { kind: 'moved', app: 'minecraft', card: 'm1', over: started, from: 'give/minecraft/a1/9000', sum: 'f00' });
    await as('bo', 'kb', 'b1');
    assert.match(await row('Minecraft'), /Minecraft · kept by your computer/);
    await as('cy', 'ka', 'a1');
    assert.match(await row('Minecraft'), /Minecraft · kept by bo’s computer/);

    for (const words of said) assert.doesNotMatch(words, NOT_SAID);
    for (const what of ['offered, written or refused', 'declined, written or refused', 'accepted, written or refused', 'the offer, as it stands']) {
      assert.ok(traced.some((l) => l.startsWith(`Give: ${what} `)), `traced: ${what}`);
    }
    assert.deepEqual(errors, []);
  } finally {
    await close();
    server.stop();
  }
});
