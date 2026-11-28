// Magnets' page in a browser against the stand-in (standin.mjs), B's own:
// what only a browser does with it. Chromium as a desktop puts a word in by
// mouse and takes it out, then does it by keyboard, focus left where the
// keys can go on and drawn where it is; WebKit as an iPhone does it by touch.
// A computer and a phone on one board see each other's moves, pushed. And
// the page says still syncing, unreachable and a refused move in board.js's
// words, drawing no fresh fridge meanwhile. The board's order, SQL and words
// are board.test.mjs's laws. Where SVRNGS_ROUTE names one (its JSON in
// board.test.mjs's head), each runs against the real route on a computer's
// svrngs (web/tests/route.mjs), opened there
// first, as the scene crates/svrngs/tests/scenes/magnets.rs runs them, but
// the states', whose switches are the stand-in's alone. Without Playwright
// or its browser this fails naming the install (`launch` exits 3); it never
// skips.

import assert from 'node:assert/strict';
import http from 'node:http';
import { test } from 'node:test';

import { launch } from '../../../web/tests/page.mjs';
import * as route from '../../../web/tests/route.mjs';
import { CHANGED, COPIED, NIGHT, refused, SAID } from '../board.js';
import { LINK, serve as standinAt } from './standin.mjs';

const WAIT = { timeout: 15_000 };
// The real route's origin is opened by svrngs first, as a person's browser is.
const OPEN = { unopened: !route.real, b: true };
const serve = route.real ? route.serve : standinAt;

/** The board opened in `context`: its page errors and debug traces kept, and what it shows read. */
async function open(context, at) {
  const page = await context.newPage();
  const [errors, traced] = [[], []];
  page.on('pageerror', (e) => errors.push(String(e)));
  page.on('console', (m) => m.type() === 'debug' && traced.push(m.text()));
  await page.goto(at);
  const words = (list) => page.$$eval(`#${list} button`, (bs) => bs.map((b) => b.textContent));
  return {
    page,
    errors,
    traced,
    tray: () => words('tray'),
    /** Until the sentence's words are `want`. */
    reads: (want) =>
      page.waitForFunction(
        (w) =>
          [...document.querySelectorAll('#placed button')].map((b) => b.textContent).join(' ') ===
          w,
        want.join(' '),
        WAIT,
      ),
    /** Until the page says `want`. */
    says: (want) =>
      page.waitForFunction((w) => document.querySelector('#said').textContent === w, want, WAIT),
    shown: () => page.locator('#board').waitFor(WAIT),
    word: (list, name) => page.locator(`#${list}`).getByRole('button', { name, exact: true }),
    focus: () => page.evaluate(() => document.activeElement?.textContent),
    /** A word's move from its popover: the word in the sentence pressed, then the move named with it. */
    moves: async (word, name, how = 'click') => {
      await page.locator('#placed').getByRole('button', { name: word, exact: true })[how]();
      await page.getByRole('button', { name, exact: true })[how]();
    },
    /** The words of the sentence outlined as new, each said "new" before it. */
    fresh: () =>
      page.$$eval('#placed .fresh', (bs) =>
        bs.map((b) => `${b.previousElementSibling?.textContent} ${b.textContent}`),
      ),
  };
}

/**
 * What the page hands the clipboard and the share sheet, kept on `window`,
 * so a test reads what it sent and no real pasteboard is touched; and what
 * the engine has, `window.sheet`: a share sheet only where it has one of
 * its own (the Mac's WebKit does, Linux's WPE doesn't), never one a test
 * made up.
 */
const HANDED = () => {
  const keep = (as) => async (given) => {
    window.handed = { as, given };
  };
  const share = typeof navigator.share === 'function';
  window.sheet = share;
  Object.defineProperty(navigator, 'clipboard', { value: { writeText: keep('copied') } });
  if (share) navigator.share = keep('shared');
};

/**
 * Whether the engine of `context` hands a page a streamed write whole once
 * it is sent: a server on 127.0.0.1 streams a short event, then one of 400
 * bytes, and the page reads its body until both are in, for 5 s. Linux's
 * WebKit (Playwright's WPE) gave a page only the first ~124 bytes of each
 * write after the first, the rest only with the next write, so a board read
 * by subscription there draws each push one push late (apps/magnets/index.js
 * never saw the bytes: its `fetch` body's reader was given them so). Where
 * it doesn't, what a page sees pushed can't be judged by timing.
 */
async function streamsWhole(context) {
  const server = http.createServer((req, res) => {
    if (req.url !== '/stream') return res.writeHead(200, { 'content-type': 'text/html' }).end('');
    res.writeHead(200, { 'content-type': 'text/event-stream', 'cache-control': 'no-store' });
    res.write('data: {}\n\n');
    setTimeout(() => res.write(`data: ${'x'.repeat(400)}\n\n`), 200);
  });
  await new Promise((done) => server.listen(0, '127.0.0.1', done));
  const page = await context.newPage();
  try {
    await page.goto(`http://127.0.0.1:${server.address().port}/`);
    return await page.evaluate(async () => {
      const reader = (await fetch('stream')).body.getReader();
      const late = new Promise((done) => setTimeout(() => done(null), 5_000));
      let got = 0;
      while (got < 418) {
        const read = await Promise.race([reader.read(), late]);
        if (!read || read.done) break;
        got += read.value.length;
      }
      reader.cancel();
      return got === 418;
    });
  } finally {
    await page.close();
    server.closeAllConnections();
    server.close();
  }
}

/** Every debug line `p` must have traced: each branch a move took. */
function traced(p, what) {
  for (const w of what)
    assert.ok(
      p.traced.some((l) => l.startsWith(`Magnets: ${w}`)),
      `traced ${w}: ${p.traced.join(' | ')}`,
    );
}

test('a word goes in, moves, is sent and kept, by mouse, then by keyboard (chromium)', async () => {
  const standin = await serve();
  const { context, close } = await launch('chromium', OPEN);
  await context.addInitScript(HANDED);
  try {
    const p = await open(context, standin.at);
    await p.shown();
    assert.equal((await p.tray())[0], 'goose');

    // A word in leaves its place in the tray empty: the word beside it
    // stays where it was, under the thumb (xenia, demo/runs/magnets/
    // 2026-11-17-xenia: "the word list rearranged after the first tap").
    const before = await p.word('tray', 'invoice').boundingBox();
    await p.word('tray', 'goose').click();
    await p.reads(['goose']);
    assert.ok(!(await p.tray()).includes('goose'), 'out of the tray');
    const keeping = p.page.getByRole('button', { name: 'Keep it and start a new one' });
    assert.equal(await keeping.count(), 0, 'no keeping a sentence of one word');
    assert.deepEqual(await p.word('tray', 'invoice').boundingBox(), before, 'invoice stays put');
    // A word in the sentence opens its moves; Take out puts it back in the pile.
    await p.moves('goose', 'Take “goose” out');
    await p.reads([]);
    assert.equal((await p.tray())[0], 'goose', 'back at its place in the tray');

    // Earlier and Later swap a word with its neighbor; each is named with its word.
    for (const w of ['goose', 'invoice', 'moon']) await p.word('tray', w).click();
    await p.reads(['goose', 'invoice', 'moon']);
    await p.moves('moon', 'Move “moon” earlier');
    await p.reads(['goose', 'moon', 'invoice']);
    await p.moves('goose', 'Move “goose” later');
    await p.reads(['moon', 'goose', 'invoice']);
    await p.word('placed', 'moon').click();
    assert.equal(
      await p.page.getByRole('button', { name: 'Move “moon” earlier' }).count(),
      0,
      'the first word has no Earlier',
    );
    await p.page.keyboard.press('Escape');

    // Send it copies the sentence and its link, and says where to paste it.
    await p.page.getByRole('button', { name: 'Send it to your group' }).click();
    await p.page.waitForFunction(
      (w) => document.querySelector('#sent').textContent === w,
      COPIED,
      WAIT,
    );
    const { as, given } = await p.page.evaluate(() => window.handed);
    assert.equal(as, 'copied');
    assert.match(given, /^I can explain the moon goose invoice\.( Change it: \S+)?$/);
    if (!route.real) assert.ok(given.endsWith(LINK), given);

    // Keep it and start a new one: the fridge empties under the next prompt,
    // the sentence goes on the list, and Put it back returns it.
    const prompt = () => p.page.textContent('#prompt');
    await keeping.click();
    await p.reads([]);
    assert.equal(await prompt(), 'Nobody warned me about the');
    const back = p.page.getByRole('button', {
      name: 'Put it back I can explain the moon goose invoice',
    });
    await p.word('tray', 'tiny').click();
    await p.reads(['tiny']);
    await back.click();
    await p.reads(['moon', 'goose', 'invoice']);
    assert.equal(await prompt(), 'I can explain the');
    await p.page
      .getByRole('button', { name: 'Put it back Nobody warned me about the tiny' })
      .waitFor(WAIT);
    for (const w of ['moon', 'goose', 'invoice']) await p.moves(w, `Take “${w}” out`);
    await p.reads([]);

    // The keyboard alone, from the page opened again: Tab reaches the first
    // word, Enter puts it on, and focus stays in the tray on the word that
    // took its place, drawn.
    await p.page.reload();
    await p.shown();
    await p.page.keyboard.press('Tab');
    assert.equal(await p.focus(), 'goose');
    await p.page.keyboard.press('Enter');
    await p.reads(['goose']);
    assert.equal(await p.focus(), 'invoice');
    const ring = await p.page.evaluate(() => getComputedStyle(document.activeElement).outlineStyle);
    assert.notEqual(ring, 'none', 'focus is drawn');
    // Shift+Tab back past Send it to the word in the sentence, and Space
    // opens its moves, focus on Take out, which Enter presses; focus goes
    // with the word, as the sentence has no word left.
    await p.page.keyboard.press('Shift+Tab');
    assert.equal(await p.focus(), 'Send it to your group');
    await p.page.keyboard.press('Shift+Tab');
    assert.equal(await p.focus(), 'goose');
    await p.page.keyboard.press('Space');
    assert.equal(await p.focus(), 'Take “goose” out');
    await p.page.keyboard.press('Enter');
    await p.reads([]);
    assert.equal(await p.focus(), 'goose');

    const [{ rows }] = await standin.db.run([
      { sql: 'SELECT id, position FROM magnets WHERE position IS NOT NULL' },
    ]);
    assert.deepEqual(rows, [], 'every word back in the tray');
    traced(p, [
      'the word pack read',
      'the board answered',
      'a magnet goes on the sentence',
      'a magnet goes back to the tray',
      'a magnet moves earlier',
      'a magnet moves later',
      'the sentence is copied',
      'the sentence is kept, and a new one starts',
      'a kept sentence goes back on the fridge',
      'the move was made',
    ]);
    assert.deepEqual(p.errors, []);
  } finally {
    await close();
    standin.stop();
  }
});

test('a word goes in, is sent and comes out by touch (webkit as an iPhone)', async (t) => {
  const standin = await serve();
  const { context, close } = await launch('webkit', OPEN);
  await context.addInitScript(HANDED);
  try {
    const p = await open(context, standin.at);
    await p.shown();
    await p.word('tray', 'goose').tap();
    await p.reads(['goose']);
    // On a phone, Send it opens the share sheet with the sentence and its
    // link. An engine as an iPhone that has no share sheet, or whose pointer
    // is not a finger (Linux's WPE has neither), is a computer to the page,
    // which copies them: so judged, and the share sheet said unjudged.
    await p.page.getByRole('button', { name: 'Send it to your group' }).tap();
    await p.page.waitForFunction(() => window.handed, null, WAIT);
    const { engine, handed } = await p.page.evaluate(() => ({
      // Whether its pointer is a finger, read as the page reads it, once
      // it ran: the Mac's WebKit says no at the document's start.
      engine: { share: window.sheet, coarse: matchMedia('(pointer: coarse)').matches },
      handed: window.handed,
    }));
    const sheet = engine.share && engine.coarse;
    if (!sheet)
      t.diagnostic(
        `could-not-judge: the share sheet, as this WebKit ${engine.share ? '' : 'has no navigator.share'}${engine.share || engine.coarse ? '' : ' and '}${engine.coarse ? '' : 'matches no (pointer: coarse)'}; its copy judged`,
      );
    assert.equal(handed.as, sheet ? 'shared' : 'copied');
    const given = sheet ? handed.given.text : handed.given;
    assert.match(given, /^I can explain the goose\.( Change it: \S+)?$/);
    if (!sheet)
      await p.page.waitForFunction(
        (w) => document.querySelector('#sent').textContent === w,
        COPIED,
        WAIT,
      );
    await p.moves('goose', 'Take “goose” out', 'tap');
    await p.reads([]);
    assert.equal((await p.tray())[0], 'goose');
    assert.deepEqual(p.errors, []);
  } finally {
    await close();
    standin.stop();
  }
});

test('a computer and a phone on one board each see the other’s moves, new ones marked', async (t) => {
  const standin = await serve();
  const [computer, phone] = [await launch('chromium', OPEN), await launch('webkit', OPEN)];
  try {
    // What the phone is pushed is judged only where its engine hands a
    // page what was streamed (`streamsWhole`); elsewhere the phone opens
    // the board again to see it, and the computer's half is judged always.
    const whole = await streamsWhole(phone.context);
    if (!whole)
      t.diagnostic(
        'could-not-judge: moves pushed to the phone, as its engine holds back what was streamed until more comes; the phone read them by opening the board again',
      );
    const pushed = async (p, want) => {
      if (!whole) await p.page.reload();
      await p.reads(want);
    };
    const a = await open(computer.context, standin.at);
    const b = await open(phone.context, standin.at);
    await Promise.all([a.shown(), b.shown()]);
    await a.word('tray', 'goose').click();
    await a.reads(['goose']);
    await pushed(b, ['goose']);
    await b.word('tray', 'invoice').tap();
    await a.reads(['goose', 'invoice']);
    await pushed(b, ['goose', 'invoice']);
    // A word that arrives while someone watches is outlined, said "new";
    // never their own.
    assert.deepEqual(await a.fresh(), ['new invoice'], 'the phone’s word, new to the computer');
    if (whole) assert.deepEqual(await b.fresh(), [], 'nothing new to the one who moved it');
    await b.moves('goose', 'Take “goose” out', 'tap');
    await a.reads(['invoice']);
    await pushed(b, ['invoice']);

    // The computer comes back after the phone moved: what changed since it
    // last looked is said, and outlined.
    await a.page.goto('about:blank');
    await b.word('tray', 'moon').tap();
    await pushed(b, ['invoice', 'moon']);
    await a.page.goto(standin.at);
    await a.says(CHANGED);
    assert.equal(
      await a.page.getAttribute('#said', 'data-state'),
      'changed',
      'said, never as a fault',
    );
    await a.reads(['invoice', 'moon']);
    assert.deepEqual(await a.fresh(), ['new moon'], 'only what came while it was away');
    assert.deepEqual([...a.errors, ...b.errors], []);
  } finally {
    await Promise.all([computer.close(), phone.close()]);
    standin.stop();
  }
});

const ON_STANDIN = {
  skip: route.real ? 'its switches are the stand-in’s: the real route has none' : false,
};
// A move is held by holding the page's request in Playwright's route: on
// the real route, svrngs's worker sends it, and the route never saw it
// (the scene, 2026-11-28: the held move was made at once).
const HELD = {
  skip: route.real
    ? 'a move is held in Playwright’s route, which svrngs’s worker passes by'
    : false,
};
test(
  'a word you put on is never new to you, though another’s move reaches you first (chromium)',
  HELD,
  async () => {
    const standin = await serve();
    const [one, other] = [await launch('chromium', OPEN), await launch('chromium', OPEN)];
    try {
      const a = await open(one.context, standin.at);
      const b = await open(other.context, standin.at);
      await Promise.all([a.shown(), b.shown()]);
      // b's move held on its way, while a's is made and pushed to b first:
      // the answer b is given then is from before b's own move.
      let release;
      const held = new Promise((done) => (release = done));
      await b.page.route('**/svrngs/sql', async (r) =>
        r.request().method() === 'POST' ? (await held, r.continue()) : r.continue(),
      );
      await b.word('tray', 'goose').click();
      await a.word('tray', 'moon').click();
      await a.reads(['moon']);
      await b.reads(['moon']);
      release();
      // Both on, in whichever order their places fall.
      await b.page.waitForFunction(
        () => document.querySelectorAll('#placed button').length === 2,
        null,
        WAIT,
      );
      assert.ok(!(await b.fresh()).includes('new goose'), 'b’s own word is never new to b');
      assert.deepEqual([...a.errors, ...b.errors], []);
    } finally {
      await Promise.all([one.close(), other.close()]);
      standin.stop();
    }
  },
);

test(
  'still syncing, unreachable and a refused move are said, no fresh fridge drawn, and the last sentence kept through goodnight (chromium)',
  ON_STANDIN,
  async () => {
    const standin = await serve();
    const { context, close } = await launch('chromium', OPEN);
    const words =
      'You’re moving words faster than your group takes them. Wait a moment, then try again.';
    try {
      await standin.set({ syncing: true });
      const p = await open(context, standin.at);
      await p.says(SAID.syncing);
      assert.equal(await p.page.getByRole('button').count(), 0, 'no word to tap while syncing');
      await standin.set({ syncing: false });
      await p.shown();
      await p.says('');

      await standin.set({ refusing: { status: 429, refused: 'too-fast', words } });
      await p.word('tray', 'goose').click();
      await p.says(refused('goose', words));
      assert.equal(await p.page.getAttribute('#said', 'data-state'), 'refused');
      assert.ok((await p.tray()).includes('goose'), 'the refused word stays in the tray');
      await standin.set({ refusing: null });
      await p.word('tray', 'goose').click();
      await p.reads(['goose']);
      await p.says('');

      // Unreachable: the open stream cut, then the board asked again and
      // answered 503, and only then the switch lifted. Meanwhile the last
      // sentence stays, inert, in the goodnight look.
      await standin.set({ unreachable: true });
      await p.says(NIGHT);
      assert.ok(await p.word('placed', 'goose').isVisible(), 'the last sentence stays on screen');
      const inert = p.page.locator('button:enabled:visible');
      assert.equal(await inert.count(), 0, 'no word to tap while unreachable');
      assert.equal(
        await p.page.evaluate(() => getComputedStyle(document.body).backgroundColor),
        'rgb(25, 25, 112)',
        'midnightblue, as svrngs says goodnight',
      );
      for (let waited = 0; !p.traced.some((l) => l.includes('status: 503')); waited += 100) {
        assert.ok(waited < WAIT.timeout, `no 503 asked: ${p.traced.join(' | ')}`);
        await new Promise((done) => setTimeout(done, 100));
      }
      await p.says(NIGHT);
      await standin.set({ unreachable: false });
      await p.says('');
      await p.reads(['goose']);
      assert.equal(await p.page.locator('body.night').count(), 0, 'the morning look again');
      traced(p, [
        'the board is still syncing',
        'svrngs/sql refused',
        'the move was refused',
        'the board could not be read',
      ]);
      assert.deepEqual(p.errors, []);
    } finally {
      await close();
      standin.stop();
    }
  },
);
