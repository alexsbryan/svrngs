// Grapevine's page in a browser, B's own (docs/seams.tsv): Chromium as a
// desktop and WebKit as an iPhone (web/tests/page.mjs, `launch`), the server
// run alone on a copy of demo.json, a simulation acting for Sam, Jo and Kim,
// each played by `?as=`. What only a browser does with the page: its script
// (server.mjs SCRIPT) sends Sam's ask as JSON and reloads, and the page then
// says it; a form it cannot send says why beside it. §10's scene, every word
// and every page's markup, is scene.test.mjs's, on the server's handler with
// no browser. Without Playwright or its browser this fails naming the
// install (`launch` exits 3); it never skips.

import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { copyFileSync, mkdtempSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { test } from 'node:test';

import { launch } from '../../web/tests/page.mjs';

/** §10's town, demo.json, on a copy of its own: Kim's tile saw listed in Waller block. */
async function serve() {
  const dir = mkdtempSync(join(tmpdir(), 'grapevine-page-'));
  copyFileSync(new URL('./demo.json', import.meta.url).pathname, join(dir, 'town.json'));
  writeFileSync(
    join(dir, 'config.json'),
    JSON.stringify({ file: join(dir, 'town.json'), acting: ['Sam', 'Jo', 'Kim'], data: dir }),
  );
  const child = spawn(
    process.execPath,
    [new URL('./server.mjs', import.meta.url).pathname, join(dir, 'config.json')],
    {
      stdio: ['ignore', 'pipe', 'pipe'],
      env: { ...process.env, GRAPEVINE_DEBUG: '1' },
    },
  );
  // `launch` exits 3 when a browser cannot start, past this test's `finally`:
  // the server ends with the process, or it outlives it, as Events' did and
  // held node --test open (capability-step-limit (a)).
  process.once('exit', () => child.kill());
  let traced = '';
  child.stderr.on('data', (b) => (traced += b));
  const line = await new Promise((done, fail) => {
    child.stdout.once('data', (b) => done(String(b)));
    child.once('exit', (code) =>
      fail(new Error(`the server exited ${code} before it listened: ${traced}`)),
    );
  });
  return { at: JSON.parse(line).at, traced: () => traced, stop: () => child.kill() };
}

for (const engine of ['chromium', 'webkit']) {
  test(`the page's script sends Sam's ask and reloads, and says a refusal beside its form (${engine})`, async () => {
    const server = await serve();
    const { context, close } = await launch(engine);
    try {
      const page = await context.newPage();
      const errors = [];
      page.on('pageerror', (e) => errors.push(String(e)));
      await page.goto(`${server.at}?as=Sam`);
      assert.match(await page.innerText('body'), /This is a simulation/);
      const tool = page.locator('.kind[data-kind=tool] form[data-to="/ask"]');
      await tool.locator('[name="f.what"]').fill('tile saw');
      await tool.locator('[name="f.when"]').fill('Saturday');
      await tool.locator('[name=group]').selectOption({ label: '1400 Page' });
      await tool.locator('[name=hops]').selectOption({ label: 'One ring away' });
      await Promise.all([
        page.waitForEvent('load'),
        tool.getByRole('button', { name: 'Ask', exact: true }).click(),
      ]);
      assert.match(await page.innerText('body'), /You asked: tile saw, Saturday/);

      // A date the server refuses: no reload, its words beside the form.
      const again = page.locator('.kind[data-kind=tool] form[data-to="/ask"]');
      await again.locator('[name="f.what"]').fill('ladder');
      await again.locator('[name="f.when"]').fill('Sunday');
      await again.locator('[name=until]').fill('2000-01-01');
      await again.getByRole('button', { name: 'Ask', exact: true }).click();
      await page.waitForFunction(
        () => document.querySelector('.kind[data-kind=tool] .said').textContent !== '',
      );
      assert.notEqual(await again.locator('.said').textContent(), '');
      assert.deepEqual(errors, []);
    } finally {
      await close();
      server.stop();
    }
  });
}
