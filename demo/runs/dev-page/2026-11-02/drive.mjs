// SPDX-License-Identifier: AGPL-3.0-or-later
// Chrome as a person opens it from the line `node web/tests/dev.mjs` printed:
// its flags as printed (the profile directory Playwright's to make), no
// proxy or certificate setting of Playwright's own. In it, bo joins the
// group from the link ada's `svrngs link --origin http://localhost:<port>/`
// printed, changed to the https page as dev.mjs says; opens Money from the
// group's page; saves an entry; and ada's computer is asked whether it
// counts it (`/v1/rail/log`, the admitted record). With `plain`, the link
// opened as printed, on http://localhost: the plant.
//
//   node drive.mjs '{"line": "<the printed line>", "link": "<svrngs's link>",
//     "data": "<ada's data dir>", "group": "<group id>", "plain": false}'
import { mkdtempSync, readFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { chromium } from '../../../../web/node_modules/playwright/index.mjs';

const args = JSON.parse(process.argv[2]);
const said = args.line.split(' --args ')[1].replace('--user-data-dir="$(mktemp -d)" ', '').split(' ');
const page = said.at(-1);
const flags = said.slice(0, -1);
const link = args.plain ? args.link : args.link.replace(/^http:\/\/localhost:\d+\//, page);
const seen = { flags, link };
const logs = [];
const profile = mkdtempSync(join(tmpdir(), 'dev-page-'));
const context = await chromium.launchPersistentContext(profile, { channel: 'chrome', headless: true, args: flags });
context.on('console', (m) => logs.push(`${m.type()}: ${m.text()}`));
context.on('response', (r) => r.status() >= 400 && logs.push(`answered ${r.status()}: ${r.url().slice(0, 160)}`));
const description = `saved from the dev page ${Date.now()}`;
try {
  const tab = await context.newPage();
  // 1. Join, typing a name when the page asks for one.
  await tab.goto(link);
  await tab.waitForFunction(() => !document.getElementById('join').hidden || /good|bad/.test(document.getElementById('status').className), null, { timeout: 90_000 });
  if (await tab.evaluate(() => !document.getElementById('join').hidden)) {
    await tab.fill('#name', 'bo');
    await tab.click('#join button');
  }
  await tab.waitForSelector('#status.good, #status.bad', { timeout: 90_000 });
  seen.joined = { at: new URL(tab.url()).origin, status: await tab.locator('#status').innerText() };
  // 2. Open Money from the group's page.
  const own = new URL(tab.url()).host;
  await Promise.all([
    tab.waitForURL((u) => u.host !== own && u.host.endsWith(own) && !u.pathname.endsWith('/open.html'), { timeout: 90_000 }),
    tab.locator('#lent button', { hasText: 'Money' }).click({ timeout: 90_000 }),
  ]);
  await tab.waitForFunction(() => document.title === 'ready', null, { timeout: 90_000 });
  seen.opened = new URL(tab.url()).origin;
  // 3. Save an entry.
  seen.saved = await tab.evaluate((w) => window.save(w), description);
} catch (e) {
  seen.failed = String(e).split('\n')[0];
} finally {
  await context.close();
  rmSync(profile, { recursive: true, force: true });
}
// 4. Does ada's computer count it?
const api = readFileSync(join(args.data, 'api'), 'utf8').trim();
const token = readFileSync(join(args.data, 'token'), 'utf8').trim();
const log = await (await fetch(`${api}/v1/rail/log?namespace=${args.group}`, { headers: { authorization: `Bearer ${token}` } })).json();
seen.counted = (log.ops ?? []).filter((o) => JSON.stringify(o).includes(description)).map((o) => ({ person: o.person, op: o.payload ?? o.op }));
seen.logs = logs.filter((l) => /SIGN|refus|not saved|error|answered/i.test(l)).slice(0, 20);
console.log(JSON.stringify(seen, null, 1));
process.exitCode = seen.saved?.status === 200 && seen.counted.length === 1 ? 0 : 1;
