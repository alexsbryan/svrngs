// SPDX-License-Identifier: AGPL-3.0-or-later
// Real issuer /ask, no /redeem; then a native local open with a nondefault folder.
import assert from 'node:assert/strict';
import { launch, pageAt, serve } from './page.mjs';
import { createResolver } from '../../resolver/handler.mjs';

const args = JSON.parse(process.argv[2]);
const server = await serve(args.site);
const origin = pageAt(server);
const { context, close } = await launch(args.engine, { device: 'Desktop Chrome', b: true });
const logs = [];
context.on('console', (message) => logs.push(message.text()));
try {
  const short = 'https://join.svrngs.test/acid-acorn-acre-acts-afar';
  const resolver = createResolver({
    token: 'fixture-only',
    fetch: async () =>
      Response.json({
        result: JSON.stringify({
          state: 'open',
          phrase: short.split('/').at(-1),
          secret: JSON.stringify({
            token: args.token,
            page: origin,
            until: Math.floor(Date.now() / 1000) + 3600,
          }),
        }),
      }),
  });
  await context.route(short, async (route) => {
    const response = await resolver(
      new Request(short, { headers: { accept: 'text/html' } }),
      '192.0.2.1',
    );
    await route.fulfill({
      status: response.status,
      headers: Object.fromEntries(response.headers),
      body: await response.text(),
    });
  });
  if (process.env.NATIVE_ARRIVAL_PLANT === 'browser-identity') {
    await context.route('**/page/join.js*', async (route) => {
      if (!route.request().url().startsWith(origin)) return route.continue();
      const response = await route.fetch();
      const body = (await response.text()).replace(
        'if (asks(state) && state.joining?.native)',
        'if (false)',
      );
      await route.fulfill({ response, body });
    });
  }
  const page = await context.newPage();
  page.on('pageerror', (error) => logs.push(String(error)));
  await page.goto(short);
  await page.waitForSelector('#join:not([hidden])', { timeout: 30_000 });
  assert.match(await page.locator('#status').textContent(), /ada invited you to house/);
  assert.equal(await page.locator('#computer').isVisible(), false);
  await page.fill('#name', "Bo O'Brien");
  await page.click('#join button');
  await page.waitForSelector('#computer:not([hidden]), #status.good', { timeout: 30_000 });
  const kept = await page.evaluate(async () => (await import('/store.js')).groups());
  assert.deepEqual(kept, [], 'no browser membership persisted');
  const line = await page.locator('#line').textContent();
  assert.ok(line.includes(`curl -fsSL ${origin}install.sh`), line);
  assert.ok(line.includes(`--at '/#group-${args.group}' --as 'Bo O'\\''Brien'`), line);
  assert.equal(await page.locator('#join').isVisible(), false);
  await context.addInitScript(
    ({ group }) => {
      const fetch = window.fetch;
      window.fetch = async (...args) => {
        const url = String(args[0]);
        if (url.endsWith('/v1/groups')) {
          const mode = sessionStorage.getItem('welcome-fixture');
          if (mode === 'loading') await new Promise((resolve) => (window.releaseGroups = resolve));
          if (mode === 'unread') return new Response('fixture unavailable', { status: 503 });
          const response = await fetch(...args);
          const body = await response.json();
          const target = body.groups.find((g) => g.id === group);
          body.groups = [
            target,
            { ...target, id: 'same-name', folders: ['/wrong-folder'] },
            { ...target, id: 'other', name: 'Other group', folders: ['/other-folder'] },
          ];
          if (mode === 'missing') body.groups.shift();
          if (mode === 'unknown-folder') target.folders = null;
          if (mode === 'no-folder') target.folders = [];
          return Response.json(body);
        }
        if (url.endsWith('/folder')) {
          window.openedFolder = JSON.parse(args[1].body);
          return Response.json({});
        }
        return fetch(...args);
      };
    },
    { group: args.group },
  );
  await page.goto(args.open);
  const section = page.locator(`#group-${args.group}`);
  await section.locator('.folder').waitFor({ state: 'attached' });
  assert.equal(
    await page.locator('#home section').count(),
    1,
    'three-group arrival must hide unrelated groups',
  );
  assert.equal(
    await page.getByRole('heading', { name: 'Welcome to house', exact: true }).count(),
    1,
  );
  assert.equal(await page.getByText('Invite people here', { exact: true }).count(), 0);
  assert.ok((await section.locator('.folder').textContent()).includes(args.folder));
  assert.equal(
    await section
      .getByRole('button', { name: /^Open shared files in (Finder|your file manager)$/ })
      .count(),
    1,
  );
  assert.equal(new URL(page.url()).hash, `#group-${args.group}`);
  await section
    .getByRole('button', { name: /^Open shared files in (Finder|your file manager)$/ })
    .click();
  await page.waitForFunction(() => window.openedFolder);
  assert.deepEqual(await page.evaluate(() => window.openedFolder), {
    group: args.group,
    dir: args.folder,
  });
  await page.reload();
  await section.locator('.folder').waitFor({ state: 'attached' });
  assert.equal(await page.locator('#home section').count(), 1, 'refresh keeps the welcome');
  await page.getByRole('link', { name: 'All your groups', exact: true }).click();
  await page.locator('.grouped-nav').waitFor();
  assert.equal(await page.locator('#home section[id^="group-"]').count(), 3);
  await page.evaluate(() => document.dispatchEvent(new Event('visibilitychange')));
  await page.waitForTimeout(6000);
  assert.equal(
    await page.locator('#home section[id^="group-"]').count(),
    3,
    'reread cannot reopen welcome',
  );
  await page.reload();
  await page.locator('.grouped-nav').waitFor();
  assert.equal(
    await page.locator('#home section[id^="group-"]').count(),
    3,
    'ordinary home refresh',
  );
  const home = new URL(page.url()).origin;
  for (const [mode, words] of [
    ['loading', 'Reading your groups…'],
    ['unread', 'cannot read your groups'],
    ['missing', 'not available on this computer'],
    ['unknown-folder', 'cannot say where'],
    ['no-folder', 'no shared folder'],
  ]) {
    await page.evaluate((mode) => sessionStorage.setItem('welcome-fixture', mode), mode);
    await page.goto(`${home}/#group-${args.group}`);
    await page.reload();
    await page.getByText(words, { exact: false }).first().waitFor();
    assert.equal(
      await page.locator('#group-same-name').count(),
      0,
      `${mode} never substitutes same-name group`,
    );
    if (mode === 'loading') {
      await page.waitForFunction(() => typeof window.releaseGroups === 'function');
      await page.evaluate(() => window.releaseGroups());
      await section.locator('.folder').waitFor({ state: 'attached' });
    }
  }
  console.log(JSON.stringify({ line, kept, folder: args.folder, hash: new URL(page.url()).hash }));
} catch (error) {
  console.error(`${error}\n${logs.join('\n')}`);
  process.exitCode = 1;
} finally {
  await close();
  server.close();
}
