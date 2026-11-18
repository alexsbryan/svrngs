// Magnets' page in a browser against the stand-in (standin.mjs), B's own:
// what only a browser does with it. Chromium as a desktop puts a word in by
// mouse and takes it out, then does it by keyboard, focus left where the
// keys can go on and drawn where it is; WebKit as an iPhone does it by touch.
// A computer and a phone on one board see each other's moves, pushed. And
// the page says still syncing, unreachable and a refused move in board.js's
// words, drawing no fresh fridge meanwhile. The board's order, SQL and words
// are board.test.mjs's laws. Without Playwright or its browser this fails
// naming the install (`launch` exits 3); it never skips.

import assert from 'node:assert/strict';
import { test } from 'node:test';

import { launch } from '../../../web/tests/page.mjs';
import { refused, SAID } from '../board.js';
import { serve } from './standin.mjs';

const WAIT = { timeout: 15_000 };
const OPEN = { unopened: true, b: true };

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
  };
}

/** Every debug line `p` must have traced: each branch a move took. */
function traced(p, what) {
  for (const w of what)
    assert.ok(
      p.traced.some((l) => l.startsWith(`Magnets: ${w}`)),
      `traced ${w}: ${p.traced.join(' | ')}`,
    );
}

test('a word goes in and out by mouse, then by keyboard (chromium)', async () => {
  const standin = await serve();
  const { context, close } = await launch('chromium', OPEN);
  try {
    const p = await open(context, standin.at);
    await p.shown();
    assert.equal((await p.tray())[0], 'goose');

    await p.word('tray', 'goose').click();
    await p.reads(['goose']);
    assert.ok(!(await p.tray()).includes('goose'), 'out of the tray');
    await p.word('placed', 'goose').click();
    await p.reads([]);
    assert.equal((await p.tray())[0], 'goose', 'back at its place in the tray');

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
    // Shift+Tab back to it in the sentence, and Space takes it out; focus
    // goes with it, as the sentence has no word left.
    await p.page.keyboard.press('Shift+Tab');
    assert.equal(await p.focus(), 'goose');
    await p.page.keyboard.press('Space');
    await p.reads([]);
    assert.equal(await p.focus(), 'goose');

    const [{ rows }] = await standin.db.run([{ sql: 'SELECT id, position FROM magnets' }]);
    assert.deepEqual(rows, [{ id: 'goose', position: null }], 'one row, back in the tray');
    traced(p, [
      'the word pack read',
      'the board answered',
      'a magnet goes on the sentence',
      'a magnet goes back to the tray',
      'the move was made',
    ]);
    assert.deepEqual(p.errors, []);
  } finally {
    await close();
    standin.stop();
  }
});

test('a word goes in and out by touch (webkit as an iPhone)', async () => {
  const standin = await serve();
  const { context, close } = await launch('webkit', OPEN);
  try {
    const p = await open(context, standin.at);
    await p.shown();
    await p.word('tray', 'goose').tap();
    await p.reads(['goose']);
    await p.word('placed', 'goose').tap();
    await p.reads([]);
    assert.equal((await p.tray())[0], 'goose');
    assert.deepEqual(p.errors, []);
  } finally {
    await close();
    standin.stop();
  }
});

test('a computer and a phone on one board each see the other’s moves', async () => {
  const standin = await serve();
  const [computer, phone] = [await launch('chromium', OPEN), await launch('webkit', OPEN)];
  try {
    const a = await open(computer.context, standin.at);
    const b = await open(phone.context, standin.at);
    await Promise.all([a.shown(), b.shown()]);
    await a.word('tray', 'goose').click();
    await b.reads(['goose']);
    await b.word('tray', 'invoice').tap();
    await a.reads(['goose', 'invoice']);
    await b.word('placed', 'goose').tap();
    await a.reads(['invoice']);
    await b.reads(['invoice']);
    assert.deepEqual([...a.errors, ...b.errors], []);
  } finally {
    await Promise.all([computer.close(), phone.close()]);
    standin.stop();
  }
});

test('still syncing, unreachable and a refused move are said, and no fresh fridge drawn (chromium)', async () => {
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
    // answered 503, and only then the switch lifted.
    await standin.set({ unreachable: true });
    await p.says(SAID.unreachable);
    assert.equal(await p.page.getByRole('button').count(), 0, 'no word to tap while unreachable');
    for (let waited = 0; !p.traced.some((l) => l.includes('status: 503')); waited += 100) {
      assert.ok(waited < WAIT.timeout, `no 503 asked: ${p.traced.join(' | ')}`);
      await new Promise((done) => setTimeout(done, 100));
    }
    await p.says(SAID.unreachable);
    await standin.set({ unreachable: false });
    await p.shown();
    await p.reads(['goose']);
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
});
