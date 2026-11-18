// Bo at his home page in Chromium. `money`: Elm's Shared apps list Money.
// `chat`: Ana's deepseek-flash opened from Elm, one message sent, its answer.
//
//   KEY=<key> node bo.mjs money|chat <address> <shots>
import { chromium, devices } from '../../../../web/node_modules/playwright/index.mjs';

const [phase, address, shots] = process.argv.slice(2);
const browser = await chromium.launch();
const { defaultBrowserType: _, ...phone } = devices['Pixel 7'];
const context = await browser.newContext(phase === 'chat' ? phone : {});
const home = await context.newPage();
const seen = { phase };
let page = home;
try {
  await home.goto(address);
  await home.getByRole('link', { name: /^Elm/ }).first().click({ timeout: 60_000 });
  if (phase === 'money') {
    const money = home.getByRole('link', { name: 'Money', exact: true }).first();
    await money.waitFor({ timeout: 120_000 });
    seen.listed = await money.innerText();
    await home.screenshot({ path: `${shots}/b-money.png`, fullPage: true });
  } else {
    const lend = home.getByRole('link', { name: /deepseek-flash/ }).first();
    await lend.waitFor({ timeout: 120_000 });
    seen.listed = await lend.innerText();
    const [opened] = await Promise.all([
      context.waitForEvent('page', { timeout: 10_000 }).catch(() => null),
      lend.click(),
    ]);
    page = opened ?? home;
    await page.waitForSelector('#text', { timeout: 60_000 });
    // Spot's line under its title says whose it is once it has asked (page.js).
    await page.waitForFunction(() => !/Asking/.test(document.getElementById('standing').textContent), null, { timeout: 60_000 });
    await page.fill('#text', 'Say hello to Bo.');
    await page.click('#send');
    await page.waitForFunction(() => document.querySelectorAll('.msg').length >= 2, null, { timeout: 60_000 });
    await page.waitForFunction(() => document.getElementById('send').textContent === 'Send', null, { timeout: 60_000 });
    seen.title = await page.title();
    seen.answer = await page.locator('.msg').last().innerText();
    await page.screenshot({ path: `${shots}/b-chat.png` });
  }
} catch (error) {
  seen.error = String(error).split('\n')[0];
  seen.standing = await page.textContent('#standing').catch(() => null);
  await page.screenshot({ path: `${shots}/b-${phase}-error.png`, fullPage: true }).catch(() => {});
} finally {
  await browser.close();
}
console.log(JSON.stringify(seen));
