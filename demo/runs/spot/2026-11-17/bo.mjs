// Bo at his home page in Chromium, at a phone's size: opened at the address
// `svrngs open --print` gives, his group opened, Ana's lend opened from it,
// then one message after another typed and sent on Spot's page. It prints
// what each answer said, the page's title, and whether the page's text or
// storage holds $KEY; a screenshot after each.
//
//   KEY=<key> node bo.mjs <address> <screenshots dir> <message> [<message> ...]
import { chromium, devices } from '../../../../web/node_modules/playwright/index.mjs';

const [address, shots, ...messages] = process.argv.slice(2);
const browser = await chromium.launch();
const { defaultBrowserType: _, ...phone } = devices['Pixel 7'];
const context = await browser.newContext(phone);
const home = await context.newPage();
const seen = { answers: [] };
try {
  await home.goto(address);
  await home.getByRole('link', { name: /Spot Run/ }).first().click({ timeout: 60_000 });
  const lend = home.getByRole('link', { name: /deepseek-flash \(spot\)/ }).first();
  await lend.waitFor({ timeout: 60_000 });
  seen.listed = await lend.innerText();
  await home.screenshot({ path: `${shots}/b1-home.png`, fullPage: true });
  const [opened] = await Promise.all([
    context.waitForEvent('page', { timeout: 10_000 }).catch(() => null),
    lend.click(),
  ]);
  const page = opened ?? home;
  await page.waitForSelector('#text', { timeout: 60_000 });
  await page.waitForFunction(() => !/Asking/.test(document.getElementById('standing').textContent));
  seen.title = await page.title();
  seen.standing = await page.textContent('#standing');
  seen.at = new URL(page.url()).host.replace(/^[^.]+/, '<label>');
  for (const [i, message] of messages.entries()) {
    const before = await page.$$eval('.msg', (all) => all.length);
    await page.fill('#text', message);
    await page.click('#send');
    await page.waitForFunction((n) => document.querySelectorAll('.msg').length >= n + 2, before);
    await page.waitForFunction(() => document.getElementById('send').textContent === 'Send');
    const last = page.locator('.msg').last();
    seen.answers.push({
      sent: message,
      said: await last.innerText(),
      strong: await last.locator('strong').count(),
      error: (await last.getAttribute('class')).includes('error'),
    });
    await page.screenshot({ path: `${shots}/b${i + 2}-chat.png` });
  }
  seen.standingAfter = await page.textContent('#standing');
  const held = await page.evaluate(() => document.body.innerText + JSON.stringify({ ...localStorage }));
  seen.keyInBrowser = held.includes(process.env.KEY);
} catch (error) {
  seen.error = String(error);
  await home.screenshot({ path: `${shots}/b-error.png`, fullPage: true }).catch(() => {});
} finally {
  await browser.close();
}
console.log(JSON.stringify(seen));
process.exit(seen.error ? 1 : 0);
