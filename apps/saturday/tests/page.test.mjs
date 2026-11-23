// Saturday's page in a browser against the stand-in (standin.mjs), as
// Magnets' page laws are (apps/magnets/tests/page.test.mjs): what only a
// browser does with it. Ana on a computer (Chromium) and Bo on a phone
// (WebKit as an iPhone) plan one Saturday and see each other's changes,
// pushed, and what Ana is typing survives Bo's. The keyboard alone adds a
// plan and votes. And the page says still syncing, unreachable and a write
// refused in board.js's words. The board's reads and writes are
// board.test.mjs's laws. Without Playwright or its browser this fails
// naming the install; it never skips.

import assert from 'node:assert/strict';
import { test } from 'node:test';

import { launch } from '../../../web/tests/page.mjs';
import { SAID, total, unsaved } from '../board.js';
import { serve } from './standin.mjs';

const WAIT = { timeout: 15_000 };
const OPEN = { unopened: true, b: true };

/** The board opened in `context` at `at`: its page errors and debug traces kept, and what it shows read. */
async function open(context, at) {
  const page = await context.newPage();
  const [errors, traced] = [[], []];
  page.on('pageerror', (e) => errors.push(String(e)));
  page.on('console', (m) => m.type() === 'debug' && traced.push(m.text()));
  await page.goto(at);
  const plan = (title) => page.getByRole('article', { name: title, exact: true });
  return {
    page,
    errors,
    traced,
    plan,
    shown: () => page.locator('#board').waitFor(WAIT),
    /** Until the page says `want`. */
    says: (want) =>
      page.waitForFunction((w) => document.querySelector('#said').textContent === w, want, WAIT),
    /** Until `title`'s plan reads `text` somewhere in it. */
    reads: (title, text) => plan(title).getByText(text, { exact: false }).first().waitFor(WAIT),
  };
}

/** Every debug line `p` must have traced: each branch it took. */
function traced(p, what) {
  for (const w of what)
    assert.ok(
      p.traced.some((l) => l.startsWith(`Saturday: ${w}`)),
      `traced ${w}: ${p.traced.join(' | ')}`,
    );
}

test('Ana on a computer and Bo on a phone plan one Saturday', async () => {
  const standin = await serve();
  const [computer, phone] = [await launch('chromium', OPEN), await launch('webkit', OPEN)];
  try {
    const ana = await open(computer.context, standin.as.Ana);
    const bo = await open(phone.context, standin.as.Bo);
    await Promise.all([ana.shown(), bo.shown()]);

    // Ana adds a plan; Bo sees it, and votes for it.
    await ana.page.getByLabel('What').fill('Picnic');
    await ana.page.getByLabel('When').fill('14:00');
    await ana.page.getByLabel('Where').fill('Park');
    await ana.page.getByRole('button', { name: 'Add it to the board' }).click();
    await bo.reads('Picnic', '14:00 · Park');
    await bo.plan('Picnic').getByRole('button', { name: 'Vote for this' }).tap();
    await ana.reads('Picnic', total(1));
    assert.equal(
      await bo
        .plan('Picnic')
        .getByRole('button', { name: 'Vote for this' })
        .getAttribute('aria-pressed'),
      'true',
      'Bo’s vote is shown pressed to Bo',
    );
    assert.equal(
      await ana
        .plan('Picnic')
        .getByRole('button', { name: 'Vote for this' })
        .getAttribute('aria-pressed'),
      'false',
      'and not to Ana',
    );

    // Ana pastes a list; it all lands, and Bo sees it.
    await ana.plan('Picnic').getByText('Paste a list of things to bring').click();
    await ana.plan('Picnic').getByLabel('One thing a line').fill('Ice\n- chips\nPlates\n');
    await ana.plan('Picnic').getByRole('button', { name: 'Add these to the list' }).click();
    await bo.plan('Picnic').getByRole('button', { name: 'I’ll bring Plates' }).waitFor(WAIT);
    const listed = await bo.plan('Picnic').locator('.things > li').allTextContents();
    assert.equal(listed.length, 3, `three things: ${listed}`);

    // Ana starts a claim of her own and leaves it unsent; Bo claims the
    // ice meanwhile, and Ana's words and focus are where she left them.
    const box = ana.plan('Picnic').getByLabel('I’ll bring');
    await box.fill('sunscreen');
    await bo.plan('Picnic').getByRole('button', { name: 'I’ll bring Ice' }).tap();
    await ana.reads('Picnic', 'Ice (Bo bringing it)');
    assert.equal(await box.inputValue(), 'sunscreen', 'Ana’s words kept');
    assert.ok(await box.evaluate((el) => el === document.activeElement), 'Ana’s focus kept');
    assert.equal(
      await ana
        .plan('Picnic')
        .getByRole('button', { name: /^(Change|Take back) / })
        .count(),
      0,
      'Ana is shown no Change or Take back on Bo’s claim',
    );
    await box.press('Enter');
    await bo.reads('Picnic', 'Ana: sunscreen');

    // Bo changes his claim, then takes it back; Ana sees each.
    await bo.plan('Picnic').getByRole('button', { name: 'Change “Ice”' }).tap();
    const his = bo.plan('Picnic').getByLabel('I’ll bring');
    assert.equal(await his.inputValue(), 'Ice', 'his claim’s words in the box');
    await his.fill('two bags of ice');
    await bo.plan('Picnic').getByRole('button', { name: 'Save the change' }).tap();
    await ana.reads('Picnic', 'Bo: two bags of ice');
    await bo.plan('Picnic').getByRole('button', { name: 'Take back “two bags of ice”' }).tap();
    await ana.page.waitForFunction(
      () => !document.querySelector('#plans').textContent.includes('two bags of ice'),
      null,
      WAIT,
    );
    await ana.plan('Picnic').getByRole('button', { name: 'I’ll bring Ice' }).waitFor(WAIT);

    const [{ rows }] = await standin.db.run([
      { sql: 'SELECT person_key, what, withdrawn FROM claims ORDER BY what' },
    ]);
    assert.deepEqual(
      rows,
      [
        { person_key: 'key-ana', what: 'sunscreen', withdrawn: 0 },
        { person_key: 'key-bo', what: 'two bags of ice', withdrawn: 1 },
      ],
      'each claim its claimer’s row, the one taken back kept',
    );
    traced(ana, [
      'who asks',
      'plans answered',
      'things answered',
      'saving: “Picnic”',
      'the write was saved',
    ]);
    traced(bo, ['changing a claim', 'saving: Taking back']);
    assert.deepEqual([...ana.errors, ...bo.errors], []);
  } finally {
    await Promise.all([computer.close(), phone.close()]);
    standin.stop();
  }
});

test('the keyboard alone adds a plan and votes, focus drawn where it is (chromium)', async () => {
  const standin = await serve();
  const { context, close } = await launch('chromium', OPEN);
  try {
    const p = await open(context, standin.at);
    await p.shown();
    await p.page.keyboard.press('Tab');
    assert.equal(await p.page.evaluate(() => document.activeElement.id), 'add-title');
    await p.page.keyboard.type('Breakfast');
    await p.page.keyboard.press('Enter');
    await p.plan('Breakfast').waitFor(WAIT);
    const vote = p.plan('Breakfast').getByRole('button', { name: 'Vote for this' });
    await vote.focus();
    await p.page.keyboard.press('Space');
    await p.reads('Breakfast', total(1));
    assert.equal(await vote.getAttribute('aria-pressed'), 'true');
    assert.ok(
      await vote.evaluate((el) => el === document.activeElement),
      'focus stays on the vote',
    );
    const ring = await vote.evaluate((el) => getComputedStyle(el).outlineStyle);
    assert.notEqual(ring, 'none', 'focus is drawn');
    assert.deepEqual(p.errors, []);
  } finally {
    await close();
    standin.stop();
  }
});

test('still syncing, unreachable and a refused write are said (chromium)', async () => {
  const standin = await serve();
  const { context, close } = await launch('chromium', OPEN);
  const words = 'You’re writing faster than your group takes it. Wait a moment, then try again.';
  try {
    await standin.set({ syncing: true });
    const p = await open(context, standin.at);
    await p.says(SAID.syncing);
    assert.equal(await p.page.getByRole('button').count(), 0, 'nothing to press while syncing');
    await standin.set({ syncing: false });
    await p.shown();

    await standin.set({ refusing: { status: 429, refused: 'limit', words } });
    await p.page.getByLabel('What').fill('Picnic');
    await p.page.getByRole('button', { name: 'Add it to the board' }).click();
    await p.says(unsaved('“Picnic”', words));
    assert.equal(await p.page.getAttribute('#said', 'data-state'), 'refused');
    assert.equal(
      await p.page.getByLabel('What').inputValue(),
      'Picnic',
      'what was refused is kept to send again',
    );
    await standin.set({ refusing: null });
    await p.page.getByRole('button', { name: 'Add it to the board' }).click();
    await p.plan('Picnic').waitFor(WAIT);
    await p.says('');

    await standin.set({ unreachable: true });
    await p.says(SAID.unreachable);
    assert.equal(await p.page.getByRole('button').count(), 0, 'nothing to press while unreachable');
    await standin.set({ unreachable: false });
    await p.plan('Picnic').waitFor(WAIT);
    traced(p, ['svrngs refused', 'the write was refused', 'plans could not be read']);
    assert.deepEqual(p.errors, []);
  } finally {
    await close();
    standin.stop();
  }
});
