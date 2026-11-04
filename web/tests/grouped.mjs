// SPDX-License-Identifier: AGPL-3.0-or-later
// Native authenticated shell; synthetic three-group reads and recorded writes.
import assert from 'node:assert/strict';
import { launch } from './page.mjs';

const args = JSON.parse(process.argv[2]);
const { context, close } = await launch(args.engine, { device: 'Desktop Chrome' });
const logs = [];
context.on('console', (message) => logs.push(message.text()));
try {
  const page = await context.newPage();
  page.on('pageerror', (error) => logs.push(`pageerror: ${error}`));
  await page.goto(args.open);
  await page.getByRole('navigation', { name: 'Your groups' }).waitFor({ timeout: 10_000 });
  assert.equal(await page.getByRole('link', { name: 'All your groups', exact: true }).count(), 1);
  console.log(JSON.stringify({ navigation: true }));
} catch (error) {
  console.error(`${error}\n${logs.join('\n')}`);
  process.exitCode = 1;
} finally {
  await close();
}
