// Events' page in a browser, B's own (docs/seams.tsv): Chromium as a desktop
// and WebKit as an iPhone (web/tests/page.mjs, `launch`), the server run
// alone on a file of its own, each person named by the header a proxy would
// set. What only a browser does with the page: Sam's Post pressed, its
// checkboxes sent and its 303 followed to the event; the Copy button's
// script says what it did, and in Chromium the clipboard holds the link.
// Who sees whose answers, and every word, is answers.test.mjs's, on the
// server's handler with no browser. Without Playwright or its browser this
// fails naming the install (`launch` exits 3); it never skips.

import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { mkdtempSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { test } from 'node:test';

import { launch } from '../../web/tests/page.mjs';

const WHO = 'remote-user';

/** The server on a file of its own, holding `ops` ({<group>: [op]}) already written. */
async function serve(ops = {}) {
  const dir = mkdtempSync(join(tmpdir(), 'events-page-'));
  writeFileSync(
    join(dir, 'store.json'),
    JSON.stringify({
      groups: {
        house: { name: 'The House', people: ['sam', 'bo', 'ama'] },
        dweb: { name: 'dWeb', people: ['sam', 'cy'] },
      },
      ops,
    }),
  );
  writeFileSync(
    join(dir, 'config.json'),
    JSON.stringify({ file: join(dir, 'store.json'), who: WHO }),
  );
  const child = spawn(
    process.execPath,
    [new URL('./server.mjs', import.meta.url).pathname, join(dir, 'config.json')],
    { stdio: ['ignore', 'pipe', 'inherit'] },
  );
  // `launch` exits 3 when a browser cannot start, past this test's `finally`:
  // the server ends with the process, or it outlives it, and node --test,
  // holding its stderr, never exits (capability-step-limit (a)).
  process.once('exit', () => child.kill());
  const line = await new Promise((done, fail) => {
    child.stdout.once('data', (b) => done(String(b)));
    child.once('exit', (code) => fail(new Error(`the server exited ${code} before it listened`)));
  });
  return { at: `http://127.0.0.1:${JSON.parse(line).port}`, stop: () => child.kill() };
}

for (const engine of ['chromium', 'webkit']) {
  test(`a post is pressed and its link copied, as the browser does it (${engine})`, async () => {
    const server = await serve();
    const { context, close } = await launch(engine);
    if (engine === 'chromium')
      await context.grantPermissions(['clipboard-read', 'clipboard-write']);
    try {
      const page = await context.newPage();
      await context.setExtraHTTPHeaders({ [WHO]: 'sam' });
      await page.goto(`${server.at}/`);
      await page.fill('input[name=what]', 'dinner');
      await page.fill('input[name=when]', 'Friday 7');
      await page.fill('input[name=where]', 'the house');
      await page.check('input[value=house]');
      await page.check('input[value=dweb]');
      await Promise.all([
        page.waitForNavigation(),
        page.getByRole('button', { name: 'Post', exact: true }).click(),
      ]);
      const house = new URL(page.url()).pathname;
      assert.match(house, /^\/g\/house\/e\/[0-9a-f]{16}$/);
      assert.equal(await page.locator('section[data-group]').count(), 2, 'posted to both groups');

      await context.setExtraHTTPHeaders({
        [WHO]: 'sam',
        'x-svrngs-label': 'lbl',
        'x-svrngs-pages': 'house=https://group.example/ dweb=none',
      });
      await page.goto(`${server.at}${house}`);
      const link = await page.locator('section[data-group=house] input.link').inputValue();
      assert.equal(link, `https://group.example/#open=lbl&at=${house}`);
      const copy = page.locator('section[data-group=house] input.link + button');
      assert.equal(await copy.textContent(), 'Copy the link to this event');
      await copy.click();
      await page.waitForFunction(
        () =>
          document.querySelector('section[data-group=house] input.link + button').textContent !==
          'Copy the link to this event',
      );
      const told = await copy.textContent();
      assert.ok(
        ['Copied', 'Copy it from the box'].includes(told),
        `the Copy button says what it did: ${told}`,
      );
      if (engine === 'chromium')
        assert.equal(await page.evaluate(() => navigator.clipboard.readText()), link);
    } finally {
      await close();
      server.stop();
    }
  });
}
