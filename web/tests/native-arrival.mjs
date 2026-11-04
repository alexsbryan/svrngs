// SPDX-License-Identifier: AGPL-3.0-or-later
// Real issuer /ask, no /redeem; then a native local open with a nondefault folder.
import assert from 'node:assert/strict';
import { launch, pageAt, serve } from './page.mjs';
import { createResolver } from '../../resolver/handler.mjs';

const args = JSON.parse(process.argv[2]);
const server = await serve(args.site);
const origin = pageAt(server);
const { context, close } = await launch(args.engine, { device: 'Desktop Chrome' });
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
  await page.goto(args.open);
  const section = page.locator(`#group-${args.group}`);
  await section.locator('.folder').waitFor();
  assert.ok((await section.locator('.folder').textContent()).includes(args.folder));
  assert.equal(
    await section.getByRole('button', { name: 'Open this folder', exact: true }).count(),
    1,
  );
  assert.equal(new URL(page.url()).hash, `#group-${args.group}`);
  console.log(JSON.stringify({ line, kept, folder: args.folder, hash: new URL(page.url()).hash }));
} catch (error) {
  console.error(`${error}\n${logs.join('\n')}`);
  process.exitCode = 1;
} finally {
  await close();
  server.close();
}
