// Grapevine's page in a browser (gv-page (b)): Chromium as a desktop and
// WebKit as an iPhone (web/tests/page.mjs, `launch`), the server run alone on
// a file of its own, a simulation acting for Sam, Jo and Kim, each played by
// `?as=`. GRAPEVINE_DEMO §10's scene, pressed as the page shows it: Sam asks
// in 1400 Page for a tile saw, one ring away; Jo carries it to Waller block;
// Kim, who listed one, is told she might have it and offers; Sam reads the
// offer, which Jo's page says went by and never what it said; both say yes
// and each sees the other's name; Sam takes it back, and the carry is gone
// from Kim's page within one loop. Every page says it is a simulation, every
// conversation says the shortcut (§6), no page says a word a reader must
// look up, and what a kind's `show` says is drawn as text, never as HTML.
// Without Playwright or its browser this fails naming the install
// (`launch` exits 3); it never skips.

import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { mkdtempSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { test } from 'node:test';

import { launch } from '../../web/tests/page.mjs';
import { RELAY_EVERY } from './relay.mjs';

/** Words the page never says (the queue's fence 9). */
const NOT_SAID = /\b(keys?|members?|records?|acts?|hash(es)?|handles?|hops?|seal)\b/i;
/** What only Sam and Kim may ever read. */
const SEALED = ['Waller and Pierce', 'Sam Reyes', 'Kim Okafor'];

/** §10's town on a file of its own: Kim's tile saw listed in Waller block. */
async function serve() {
  const dir = mkdtempSync(join(tmpdir(), 'grapevine-page-'));
  writeFileSync(join(dir, 'town.json'), JSON.stringify({
    groups: { page: { name: '1400 Page', people: ['Sam', 'Jo'] }, waller: { name: 'Waller block', people: ['Jo', 'Kim'] } },
    ops: { waller: [{ id: 'l1', ts_unix: 1_760_000_000, person: 'Kim', payload: { app: 'grapevine-listings', kind: 'tool', listing: { what: 'tile saw' } } }] },
  }));
  writeFileSync(join(dir, 'config.json'), JSON.stringify({ file: join(dir, 'town.json'), acting: ['Sam', 'Jo', 'Kim'], data: dir }));
  const child = spawn(process.execPath, [new URL('./server.mjs', import.meta.url).pathname, join(dir, 'config.json')], {
    stdio: ['ignore', 'pipe', 'pipe'],
    env: { ...process.env, GRAPEVINE_DEBUG: '1' },
  });
  let traced = '';
  child.stderr.on('data', (b) => (traced += b));
  const line = await new Promise((done, fail) => {
    child.stdout.once('data', (b) => done(String(b)));
    child.once('exit', (code) => fail(new Error(`the server exited ${code} before it listened: ${traced}`)));
  });
  return { at: JSON.parse(line).at, traced: () => traced, stop: () => child.kill() };
}

for (const engine of ['chromium', 'webkit']) {
  test(`§10's scene, by ?as=, every page a simulation (${engine})`, async () => {
    const server = await serve();
    const { context, close } = await launch(engine);
    try {
      const page = await context.newPage();
      const pages = [];
      const as = async (who) => {
        await page.goto(`${server.at}?as=${who}`);
        const text = await page.innerText('body');
        pages.push([who, text]);
        return text;
      };
      /** `who`'s page, read again until `want` holds of it, within two loops. */
      const until = async (who, want, what) => {
        const by = Date.now() + 2 * RELAY_EVERY + 2_000;
        for (;;) {
          const text = await as(who);
          if (want(text)) return text;
          if (Date.now() > by) assert.fail(`${who}'s page never showed ${what} within two loops:\n${text}`);
          await new Promise((r) => setTimeout(r, 300));
        }
      };
      const press = async (scope, name) => {
        await Promise.all([page.waitForEvent('load'), scope.getByRole('button', { name, exact: true }).click()]);
      };

      // 1. Sam asks in 1400 Page for a tile saw on Saturday, one ring away.
      await as('Sam');
      const tool = page.locator('.kind[data-kind=tool] form[data-to="/ask"]');
      assert.deepEqual(await tool.locator('[name^="f."]').evaluateAll((es) => es.map((e) => [e.name, e.type])), [['f.what', 'text'], ['f.when', 'text']], "the tool form is the kind's fields");
      await tool.locator('[name="f.what"]').fill('tile saw');
      await tool.locator('[name="f.when"]').fill('Saturday');
      await tool.locator('[name=group]').selectOption({ label: '1400 Page' });
      await tool.locator('[name=hops]').selectOption({ label: 'One ring away' });
      await press(tool, 'Ask');
      assert.match(await as('Sam'), /You asked: tile saw, Saturday/);

      // 2. Jo's page shows the ask and "Carry to Waller block". Jo taps it.
      assert.match(await as('Jo'), /Sam asks: tile saw, Saturday/);
      await press(page.locator('section[data-group=page] .ask'), 'Carry to Waller block');

      // 3. Kim's page: carried by Jo, and the tool kind's notice. Kim offers.
      const kim = await as('Kim');
      assert.match(kim, /A neighbour of Jo’s needs: tile saw, Saturday — carried by Jo/);
      assert.match(kim, /You might have this\./);
      assert.doesNotMatch(await page.locator('section[data-group=waller]').innerText(), /Sam/, "Kim's page never names who asked");
      const offer = page.locator('section[data-group=waller] .ask form[data-to="/reply"]');
      await offer.locator('[name="f.when"]').fill('Saturday morning');
      await offer.locator('[name="f.where"]').fill('Waller and Pierce');
      await press(offer, 'Offer');
      assert.match(await as('Kim'), /It goes through Jo’s computer/);

      // 4. Sam's page opens the offer, through Jo's loop; Jo's says a sealed message went by.
      await until('Sam', (t) => /Someone who saw it offered: when Saturday morning; where Waller and Pierce/.test(t), 'the offer');
      const jo = await until('Jo', (t) => /A sealed message went by toward the person who asked: \d+ bytes/.test(t), 'that a message went by');
      for (const s of SEALED) assert.ok(!jo.includes(s), `Jo's page holds ${s}`);

      // 5. Sam says yes, then Kim; each sees the other's name.
      await as('Sam');
      const yes = page.locator('.thread.asker form[data-to="/yes"]');
      await yes.locator('[name=name]').fill('Sam Reyes');
      await press(yes, 'Yes');
      const before = await until('Kim', (t) => /The person who asked said yes/.test(t), "Sam's yes");
      assert.ok(!before.includes('Sam Reyes'), "Sam's name waits for Kim's yes");
      const back = page.locator('.thread.replier form[data-to="/yes"]');
      await back.locator('[name=name]').fill('Kim Okafor');
      await press(back, 'Yes');
      assert.match(await as('Kim'), /You both said yes: they’re Sam Reyes/);
      await until('Sam', (t) => /You both said yes: they’re Kim Okafor/.test(t), "Kim's yes");

      // 6. Sam takes it back, and the carry in Waller is gone within one loop.
      await as('Sam');
      await press(page.locator('section[data-group=page] .ask'), 'Take it back');
      await until('Kim', (t) => !/tile saw/.test(t.split('Your conversations')[0]), 'the carry gone');

      // A kind's `show` is drawn as text: Kim asks a question with markup in it.
      await as('Kim');
      const question = page.locator('.kind[data-kind=ask] form[data-to="/ask"]');
      await question.locator('[name="f.question"]').fill('Anyone have a <b>ladder</b>?');
      await question.locator('[name=group]').selectOption({ label: 'Waller block' });
      await press(question, 'Ask');
      assert.match(await as('Jo'), /Kim asks: Anyone have a <b>ladder<\/b>\?/);
      assert.equal(await page.locator('.ask b').count(), 0, "the page drew the kind's own HTML");

      await page.goto(server.at);
      pages.push(['nobody', await page.innerText('body')]);
      for (const [who, text] of pages) {
        assert.match(text, /This is a simulation/, `${who}'s page says it is a simulation`);
        assert.doesNotMatch(text, NOT_SAID, `${who}'s page says a word a reader must look up`);
      }
      for (const who of ['Sam', 'Jo', 'Kim']) {
        await as(who);
        const threads = await page.locator('.thread').count();
        assert.ok(threads > 0, `${who} has a conversation`);
        assert.equal(await page.locator('.thread .shortcut').count(), threads, `every one of ${who}'s conversations says the shortcut`);
      }
      for (const s of SEALED) assert.ok(!server.traced().includes(s), `the server's trace holds ${s}`);
    } finally {
      await close();
      server.stop();
    }
  });
}
