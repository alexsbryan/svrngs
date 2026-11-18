// Ana at her home page in Chromium, by the words docs/guide/examples.md
// gives. One phase a run; each prints what it saw as JSON, and a
// screenshot on the way and on an error.
//
//   node ana.mjs sandbox <address> <shots>   make Sandbox, bring Money, its folder, Money's heading
//   node ana.mjs money <address> <shots>     Money opened again: its heading
//   node ana.mjs lend <address> <shots> <port> <key>   Already running here, then its header
import { chromium } from '../../../../web/node_modules/playwright/index.mjs';

const [phase, address, shots, port, key] = process.argv.slice(2);
const browser = await chromium.launch();
const context = await browser.newContext();
const page = await context.newPage();
const seen = { phase };
const shot = (name) => page.screenshot({ path: `${shots}/a-${phase}-${name}.png`, fullPage: true });
// A group, from the page's own "All your groups" (an open is spent once used).
const group = async (name) => {
  await page.getByRole('link', { name: 'All your groups' }).first().click();
  await page.getByRole('link', { name: new RegExp(`^${name}`) }).first().click({ timeout: 60_000 });
};
const heading = async (name) => {
  const [opened] = await Promise.all([
    context.waitForEvent('page', { timeout: 10_000 }).catch(() => null),
    page.getByRole('link', { name, exact: true }).first().click({ timeout: 60_000 }),
  ]);
  const app = opened ?? page;
  await app.waitForSelector('h2', { timeout: 60_000 });
  return app.$$eval('h2', (all) => all.map((h) => h.textContent));
};
try {
  await page.goto(address);
  if (phase === 'sandbox') {
    await page.getByRole('link', { name: 'Make a group' }).first().click();
    const form = page.locator('form, section').filter({ hasText: 'A name for the group' }).last();
    await form.getByLabel('A name for the group').fill('Sandbox');
    await form.getByLabel('Your name in it').fill('ana');
    await form.getByRole('button', { name: 'Make a group' }).click();
    await page.getByText(/You made Sandbox/).waitFor({ timeout: 60_000 });
    await group('Sandbox');
    await page.locator('summary:visible').filter({ hasText: /^Add an app$/ }).first().click();
    seen.howLink = await page.getByRole('link', { name: /See how they.re made/ }).count();
    await page.getByRole('checkbox', { name: /Money/ }).check({ timeout: 60_000 });
    await page.getByRole('button', { name: 'Bring the ticked ones to Sandbox' }).click();
    await page.getByText(/You brought Money to Sandbox/).waitFor({ timeout: 60_000 });
    await shot('brought');
    await page.locator('summary:visible').filter({ hasText: /^Folder location$/ }).first().click();
    seen.folder = await page.locator('p.folder:visible').first().textContent();
    seen.headings = await heading('Money');
    await group('Elm');
    await page.locator('summary:visible').filter({ hasText: /^Folder location$/ }).first().click();
    seen.elmFolder = await page.locator('p.folder:visible').first().textContent();
  } else if (phase === 'money') {
    await group('Sandbox');
    seen.headings = await heading('Money');
  } else if (phase === 'lend') {
    await page.getByRole('link', { name: 'This computer' }).first().click();
    await page.getByRole('button', { name: 'Look for programs running on this computer' }).click();
    // Each program found is a line of its own (running-here.js `Found`).
    const row = page.locator('li').filter({ hasText: `Something at http://localhost:${port}.` });
    await row.waitFor({ timeout: 60_000 });
    await row.getByLabel('Its name').fill('deepseek-flash');
    await row.getByLabel('Which program').fill('spot');
    await row.getByRole('checkbox', { name: /Elm/ }).check();
    await row.getByRole('button', { name: /Lend it to Elm/ }).click();
    await shot('lent');
    // "Under What you lend, press its line to open it" (docs/guide/lend.md).
    await page.locator('summary').filter({ hasText: /^deepseek-flash:/ }).click({ timeout: 60_000 });
    await page.getByLabel('What it says').fill(`Bearer ${key}`);
    await page.getByRole('button', { name: 'Add this header' }).click();
    seen.added = await page.getByText(/svrngs adds authorization/).first().textContent({ timeout: 60_000 });
    await shot('header');
  }
} catch (error) {
  seen.error = String(error).split('\n')[0];
  await shot('error').catch(() => {});
} finally {
  await browser.close();
}
console.log(JSON.stringify(seen));
