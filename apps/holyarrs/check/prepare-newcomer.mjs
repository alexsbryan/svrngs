// Ana creates the invitation through the stock UI. The copied URL is kept
// verbatim: a wrong origin must be observed, never repaired backstage.
import assert from 'node:assert/strict';
import { readFileSync, writeFileSync } from 'node:fs';
import { arrived, launch } from '../../../web/tests/page.mjs';

const args = JSON.parse(process.argv[2]);
const state = JSON.parse(readFileSync(args.state));
const { context, close } = await launch('chromium');
const home = await context.newPage();
let page = home;
try {
  if (args.mode === 'shared') {
    await home.goto(args.owner_link);
    await home.waitForSelector('#status.good', { timeout: 90_000 });
    await home.locator('#lent button', { hasText: 'holyarrs' }).click();
  } else {
    await home.goto(args.home);
    await arrived(home, 90_000);
    [page] = await Promise.all([
      context.waitForEvent('page'),
      home.getByRole('link', { name: /^Open .*holyarrs$/ }).click(),
    ]);
  }
  await page.getByLabel('Name', { exact: true }).fill(state.ana.name);
  await page.getByLabel('Password', { exact: true }).fill(state.ana.password);
  await page.getByRole('button', { name: 'Sign in', exact: true }).click();
  await page.waitForURL((u) => u.pathname === '/');
  const origin = new URL(page.url()).origin;
  await page.goto(`${origin}/settings/users`);
  await page.getByRole('button', { name: 'Invite someone', exact: true }).click();
  await page.getByLabel('Note for yourself').fill('Bo');
  await page.getByRole('button', { name: 'Make an invite link', exact: true }).click();
  const invitation = (await page.locator('.fresh code').innerText()).trim();
  assert.equal(
    new URL(invitation).origin,
    origin,
    'the copied invitation is the one the app actually produced',
  );
  await page.screenshot({ path: `${state.dir}/ana-created-invitation.png` });
  writeFileSync(
    `${state.dir}/app-invitation.json`,
    JSON.stringify({ origin, invitation, mode: args.mode }, null, 2),
    { mode: 0o600 },
  );
  console.log(JSON.stringify({ origin, invitation, mode: args.mode }));
} finally {
  await close();
}
