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
  // The page the group's https links print on, as svrngs says it beside
  // the group (carried.rs `acts`, give-page-host): none here unless a test
  // names one.
  const said = { page: null };
  // Each act at its time, now unless the test names one: the page reads
  // the clock to say whether a computer has stopped answering.
  const record = (actor, person, person_key, payload, ts = Math.floor(Date.now() / 1000)) =>
    ops.push({ id: `op${ops.length + 1}`, actor, person, person_key, ts_unix: ts, voided: false, payload });
  const server = http.createServer(async (req, res) => {
    res.setHeader('content-security-policy', policy);
    const [you, key, computer] = ['x-person', 'x-key', 'x-computer'].map((h) => req.headers[h]);
    const path = new URL(req.url, 'http://app').pathname;
    const json = (status, body) => res.writeHead(status, { 'content-type': 'application/json' }).end(JSON.stringify(body));
    if (path === '/svrngs/acts') {
      if (req.method === 'GET') return json(200, { you, you_key: key, group: 'elm', page: said.page, people: ['cy', 'bo'], spans: null, ops });
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
  const port = server.address().port;
  // svrngs stopped, as an update restarts it: nothing answers at the
  // page's origin, and every connection open to it is dropped.
  const down = () => new Promise((done) => (server.close(done), server.closeAllConnections()));
  const up = () => new Promise((done) => server.listen(port, '127.0.0.1', done));
  return { at: `http://127.0.0.1:${port}/`, ops, said, record, down, up, stop: () => server.close() };
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
    assert.match(await row('Minecraft'), /To keep Minecraft on this computer, svrngs needs Podman \(free, one install\) and Give’s program for computers\. Get Podman, then paste this line into a terminal for Give’s program:/);
    assert.equal(await page.getByRole('link', { name: 'Get Podman' }).first().getAttribute('href'), 'https://podman.io/docs/installation');
    // The one line install.sh takes Give's program by (give-install (b)):
    // svrngs's own line, with --give (crates/svrngs/tests/install.rs runs
    // it), from the page svrngs is published at (README.md), as this
    // computer keeps no page of its own (give-page-host).
    const line = `curl -fsSL https://group.svrnme.sh/install.sh | sh -s -- 'https://group.svrnme.sh/' --give`;
    assert.equal(await page.locator('li', { hasText: 'Minecraft' }).locator('code').innerText(), line);
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

    // Cy's computer says again, as Give's program does when it changes, that
    // it can't run Stand-in here for want of Rosetta (give-install (c)): the
    // row says what to turn on, and offers no start until it says it can.
    server.record('a1', 'cy', 'ka', { kind: 'ready', cannot: { standin: 'rosetta' } });
    await as('cy', 'ka', 'a1');
    assert.match(await row('Stand-in'), /To keep Stand-in on this computer, turn on Rosetta in Podman Desktop’s settings, then start Podman again\./);
    assert.doesNotMatch(await row('Stand-in'), /Start it for the group/);
    assert.match(await row('Minecraft'), /Minecraft · kept by your computer/);
    server.record('a1', 'cy', 'ka', { kind: 'ready', cannot: {} });
    await as('cy', 'ka', 'a1');
    assert.match(await row('Stand-in'), /Start it for the group/, 'said it can run it, so it can be started');

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
    for (const what of ['cards.json read', 'svrngs/acts answered', 'folded', 'started, written or refused', 'the fold refused an act', 'this computer can’t run it']) {
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

/** A time as the page says it: hours and minutes today, with the day before. */
function when(ts) {
  const d = new Date(ts * 1000);
  const today = d.toDateString() === new Date().toDateString();
  const hm = { hour: '2-digit', minute: '2-digit', hourCycle: 'h23' };
  return d.toLocaleString([], today ? hm : { month: 'short', day: 'numeric', ...hm });
}

// Scenes 4 and 5 (give-app-takeover): cy's computer keeps Minecraft and
// copies it every five minutes, then stops answering. Dee, whose computer
// can keep, is asked "Keep it here?" with the newest copy's time and its
// makers' terms; Ed, whose computer can't, reads only that it stopped
// answering. Dee says yes; the copy doesn't start, and dee is offered the
// copy before it. Then cy's computer comes back and keeps what it had as
// a copy: every member reads it, and bo starts it instead, which asks the
// same terms. Nothing is merged and nothing is taken away: the copy is
// still listed after.
test('a member keeps an app whose computer stopped answering, and a returning computer’s state can be started instead (chromium)', async () => {
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

    const now = Math.floor(Date.now() / 1000);
    const at = now - 3600;
    server.record('a1', 'cy', 'ka', { kind: 'ready' }, at - 10);
    server.record('b1', 'bo', 'kb', { kind: 'ready' }, at - 10);
    server.record('c1', 'dee', 'kc', { kind: 'ready' }, at - 10);
    server.record('a1', 'cy', 'ka', { kind: 'started', app: 'minecraft', card: 'm1', agreed: true }, at);
    const started = server.ops.at(-1).id;
    const copy = (n, ts, over = started, dir = `give/minecraft/a1/${n}`) =>
      server.record('a1', 'cy', 'ka', { kind: 'copied', app: 'minecraft', card: 'm1', over, from: dir, sum: `sum${n}` }, ts);
    copy(1, at + 300);
    copy(2, at + 600);

    // Cy's computer's last copy, then nothing for three of its intervals.
    copy(3, now - 901);
    await as('ed', 'ke', 'e1');
    assert.match(await row('Minecraft'), new RegExp(`Minecraft’s computer hasn’t answered since ${when(now - 901)}\\.`));
    assert.doesNotMatch(await row('Minecraft'), /Keep it here|You have a copy/, 'a computer that can’t keep is not asked');

    await as('dee', 'kc', 'c1');
    assert.match(
      await row('Minecraft'),
      new RegExp(`Minecraft’s computer hasn’t answered since ${when(now - 901)}\\. You have a copy from ${when(now - 901)}\\. Keep it here\\?[\\s\\S]*${MINECRAFT_TERMS} Read them`),
    );
    await press('Agree and keep it here', 'Minecraft');
    assert.deepEqual(last(), ['c1', { kind: 'taken', app: 'minecraft', card: 'm1', over: started, from: 'give/minecraft/a1/3', sum: 'sum3', agreed: true }]);
    const t1 = server.ops.at(-1).id;
    assert.match(await row('Minecraft'), /Minecraft · kept by your computer/);
    assert.doesNotMatch(await row('Minecraft'), /hasn’t answered|Keep it here/, 'a computer that answers: nobody is asked');

    // The copy doesn't start on dee's computer, as Give's program says.
    server.record('c1', 'dee', 'kc', { kind: 'failed', app: 'minecraft', over: t1 });
    await as('dee', 'kc', 'c1');
    assert.match(await row('Minecraft'), new RegExp(`This copy didn’t start\\. Try the copy from ${when(at + 600)}\\?`));
    await press('Agree and keep it here', 'Minecraft');
    assert.deepEqual(last(), ['c1', { kind: 'taken', app: 'minecraft', card: 'm1', over: t1, from: 'give/minecraft/a1/2', sum: 'sum2', agreed: true }]);
    const t2 = server.ops.at(-1).id;
    assert.match(await row('Minecraft'), /Minecraft · kept by your computer/);
    assert.doesNotMatch(await row('Minecraft'), /didn’t start/);

    // Cy's computer comes back, and keeps what it had as a copy.
    copy(4, now, started, 'give/minecraft/a1/had/4');
    await as('cy', 'ka', 'a1');
    assert.match(await row('Minecraft'), /Minecraft · kept by dee’s computer/);
    assert.match(await row('Minecraft'), new RegExp(`Minecraft as your computer had it at ${when(now)}`));
    await as('bo', 'kb', 'b1');
    assert.match(await row('Minecraft'), new RegExp(`Minecraft as cy’s computer had it at ${when(now)}`));
    await press('Start instead', 'Minecraft');
    assert.match(await row('Minecraft'), new RegExp(`${MINECRAFT_TERMS} Read them`), 'the same question');
    await press('Agree and start', 'Minecraft');
    assert.deepEqual(last(), ['b1', { kind: 'taken', app: 'minecraft', card: 'm1', over: t2, from: 'give/minecraft/a1/had/4', sum: 'sum4', agreed: true }]);
    assert.match(await row('Minecraft'), /Minecraft · kept by your computer/);
    assert.match(await row('Minecraft'), /Minecraft as cy’s computer had it/, 'nothing taken away');

    for (const words of said) assert.doesNotMatch(words, NOT_SAID);
    for (const what of ['taken, written or refused', 'who answers, as it stands']) {
      assert.ok(traced.some((l) => l.startsWith(`Give: ${what} `)), `traced: ${what}`);
    }
    assert.deepEqual(errors, []);
  } finally {
    await close();
    server.stop();
  }
});

// Give's page left open while its computer's svrngs is down (give-page-down;
// finding 2 of demo/runs/give/2026-10-26-scenes/NEEDS_HUMAN.md): each
// round's read can't reach svrngs. The page says so within one round, in a
// person's words, and shows nothing from before as current, so no press
// acts on what it last read (AGENTS.md 6: an absence is reported, never
// defaulted). Refused is a page that goes on showing its last read, as it
// did when the failed read threw out of `show`. Once svrngs answers again
// the page reads as before, with no reload.
test('a page left open while svrngs is down says it can’t read the group, and reads it again once svrngs is back (chromium)', async () => {
  const server = await serve();
  const { context, close } = await launch('chromium');
  const errors = [];
  try {
    const page = await context.newPage();
    page.on('pageerror', (e) => errors.push(String(e)));
    const traced = [];
    page.on('console', (m) => m.type() === 'debug' && traced.push(m.text()));
    const text = () => page.innerText('main');
    const row = (title) => page.locator('li', { hasText: title }).innerText();
    const buttons = () => page.getByRole('button').count();
    // One round is 3 seconds (index.js `setInterval`); a read that can't
    // connect fails at once, so a round and its read fit in this.
    const ROUND = 6_000;

    server.record('a1', 'cy', 'ka', { kind: 'ready' });
    server.record('b1', 'bo', 'kb', { kind: 'ready' });
    server.record('a1', 'cy', 'ka', { kind: 'started', app: 'minecraft', card: 'm1', agreed: true });
    await context.setExtraHTTPHeaders({ 'x-person': 'cy', 'x-key': 'ka', 'x-computer': 'a1' });
    await page.goto(server.at);
    await page.waitForFunction(() => document.body.innerText.includes('kept by'), null, { timeout: 60_000 });
    assert.match(await row('Minecraft'), /Minecraft · kept by your computer[\s\S]*Move it to…/);
    assert.ok((await buttons()) > 0, 'the page offers its presses while svrngs answers');
    await page.evaluate(() => (window.loaded = 'once'));

    await server.down();
    await page.waitForFunction(() => document.body.innerText.includes('can’t be read just now'), null, { timeout: ROUND });
    const said = await text();
    assert.match(said, /What your group keeps can’t be read just now: svrngs on this computer isn’t answering\. This page will try again by itself\./);
    assert.doesNotMatch(said, /kept by|Minecraft|Stand-in/, 'nothing from before is offered as current');
    assert.equal(await buttons(), 0, 'no press acts on what the page last read');
    assert.doesNotMatch(said, NOT_SAID);
    assert.ok(traced.some((l) => l.startsWith('Give: svrngs/acts not reached ')), 'traced: svrngs/acts not reached');

    await server.up();
    await page.waitForFunction(() => document.body.innerText.includes('kept by'), null, { timeout: ROUND });
    assert.match(await row('Minecraft'), /Minecraft · kept by your computer[\s\S]*Move it to…/);
    assert.doesNotMatch(await text(), /can’t be read/);
    assert.equal(await page.evaluate(() => window.loaded), 'once', 'read again with no reload');
    assert.deepEqual(errors, []);
  } finally {
    await close();
    server.stop();
  }
});

// A press while svrngs is down (give-page-down (c)): cy has Minecraft's
// makers' terms open, svrngs stops, and cy presses "Agree and start". The
// round reads nothing while an ask is open (index.js `setInterval`), so the
// press is the only thing that can say it. Refused is a press that fails
// silently, its ask left on screen as if it were waiting, as when the failed
// write threw out of `write`. The page says nothing reached the group, the
// ask stays open, and once svrngs answers the same press starts it, once.
test('a press while svrngs is down says nothing reached the group, and the ask stays open to press again (chromium)', async () => {
  const server = await serve();
  const { context, close } = await launch('chromium');
  const errors = [];
  try {
    const page = await context.newPage();
    page.on('pageerror', (e) => errors.push(String(e)));
    const traced = [];
    page.on('console', (m) => m.type() === 'debug' && traced.push(m.text()));
    const text = () => page.innerText('main');
    const row = (title) => page.locator('li', { hasText: title }).innerText();
    const press = (name) => page.locator('li', { hasText: 'Minecraft' }).getByRole('button', { name, exact: true }).click();

    server.record('a1', 'cy', 'ka', { kind: 'ready' });
    await context.setExtraHTTPHeaders({ 'x-person': 'cy', 'x-key': 'ka', 'x-computer': 'a1' });
    await page.goto(server.at);
    await page.waitForFunction(() => document.body.innerText.includes('Start it for the group'), null, { timeout: 60_000 });
    await press('Start it for the group');
    await page.waitForFunction(() => document.body.innerText.includes('Agree and start'), null, { timeout: 30_000 });
    const written = server.ops.length;

    await server.down();
    await press('Agree and start');
    await page.waitForFunction(() => document.body.innerText.includes('nothing reached your group'), null, { timeout: 30_000 });
    const said = await text();
    assert.match(said, /Minecraft didn’t start: svrngs on this computer isn’t answering, so nothing reached your group\. Press again in a moment\./);
    assert.match(await row('Minecraft'), new RegExp(`${MINECRAFT_TERMS} Read them[\\s\\S]*Agree and start`), 'the ask stays open to press again');
    assert.equal(server.ops.length, written, 'nothing was written');
    assert.doesNotMatch(said, NOT_SAID);
    assert.ok(traced.some((l) => l.startsWith('Give: started, not written: svrngs not reached ')), 'traced: not written');

    await server.up();
    await press('Agree and start');
    await page.waitForFunction(() => document.body.innerText.includes('kept by'), null, { timeout: 30_000 });
    assert.match(await row('Minecraft'), /Minecraft · kept by your computer/);
    assert.doesNotMatch(await text(), /nothing reached your group/);
    assert.deepEqual(server.ops.slice(written).map((o) => o.payload), [{ kind: 'started', app: 'minecraft', card: 'm1', agreed: true }], 'started once');
    assert.deepEqual(errors, []);
  } finally {
    await close();
    server.stop();
  }
});

test('Give’s install line names the page this computer’s group links are on, ours where it keeps none (chromium)', async () => {
  // give-page-host: a group on a page host of its own gets svrngs and
  // Give's program from it, so the line names the page svrngs says beside
  // the group, with the slash install.sh puts after it; none, ours (the
  // first test), as is no word from a svrngs before this row. One svrngs can't read, or one a terminal would read as
  // more than an address, gives no line: never ours in its place (AGENTS.md
  // 6), never a line that runs what the address spells. Red at 47d46706:
  // the line was ours whatever the page.
  const server = await serve();
  const { context, close } = await launch('chromium');
  try {
    const page = await context.newPage();
    const traced = [];
    page.on('console', (m) => m.type() === 'debug' && traced.push(m.text()));
    await context.setExtraHTTPHeaders({ 'x-person': 'cy', 'x-key': 'ka', 'x-computer': 'a1' });
    const open = async (said) => {
      server.said.page = said;
      await page.goto(server.at);
      await page.waitForSelector('li');
      return page.locator('li', { hasText: 'Minecraft' });
    };
    let row = await open('https://links.example.org/our/house');
    const theirs = 'https://links.example.org/our/house/';
    assert.equal(await row.locator('code').innerText(), `curl -fsSL ${theirs}install.sh | sh -s -- '${theirs}' --give`);

    // A svrngs from before give-page-host says no page: ours, as it was.
    row = await open(undefined);
    const ours = 'https://group.svrnme.sh/';
    assert.equal(await row.locator('code').innerText(), `curl -fsSL ${ours}install.sh | sh -s -- '${ours}' --give`);
    assert.ok(traced.some((t) => t.includes('as one before give-page-host')), traced.join('\n'));

    row = await open({ unread: 'not an http(s) address without a \'#\'' });
    assert.equal(await row.locator('code').count(), 0, 'no line, never ours in its place');
    assert.match(await row.innerText(), /Get Podman\. The line for Give’s program can’t be given here just now: this computer can’t read the page your group’s links are on\./);
    assert.ok(traced.some((t) => t.includes('no line for Give’s program') && t.includes('not an http(s) address')), traced.join('\n'));

    row = await open("https://links.example.org/$(touch x)'/");
    assert.equal(await row.locator('code').count(), 0, 'no line a terminal would run more of');
    assert.match(await row.innerText(), /The line for Give’s program can’t be given here just now/);
    assert.doesNotMatch(await page.innerText('main'), NOT_SAID);
  } finally {
    await close();
    server.stop();
  }
});
