// Ana's home page in Chromium, as she uses it: opened at the address
// `svrngs open --print` gives, then one act of hers, pressed and typed as a
// person does. It prints what her row says and what /lent answers her page
// (fetched by the page, so its worker proves it), and saves a screenshot.
//
//   node ana.mjs <address> lend|add|refuse|stop <screenshot.png>
//
// The header's value comes in $VALUE, never on the command line.
import { chromium } from '../../../../web/node_modules/playwright/index.mjs';

const [address, act, shot] = process.argv.slice(2);
const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1000, height: 1600 } });
const seen = { act };
const row = () => page.locator('#lent li', { hasText: 'coder' });
try {
  await page.goto(address);
  // What she lends is under This computer, beside All your groups.
  await page.getByRole('link', { name: 'This computer' }).click({ timeout: 60_000 });
  await page.waitForSelector('#lent', { timeout: 60_000 });
  if (act === 'lend') {
    await page.click("#running button:has-text('Look for programs')");
    const found = page.locator('#running li', { hasText: 'localhost:8301' });
    await found.waitFor({ timeout: 60_000 });
    seen.found = await found.innerText();
    await found.locator('input[name=name]').fill('coder');
    await found.locator('input[name=kind]').fill('llama-server');
    await found.locator('input[type=checkbox]').first().check();
    await found.locator('button', { hasText: 'Lend' }).click();
    // A program whose terms let apps send to it is drawn first, then lent
    // on the second press (home.rs `drawn`).
    await found.getByText('press Lend again').waitFor({ timeout: 30_000 });
    seen.drawn = await found.innerText();
    await found.locator('button', { hasText: 'Lend' }).click();
    await row().waitFor({ timeout: 30_000 });
  } else {
    await row().waitFor({ timeout: 30_000 });
    await row().locator('summary').click();
    if (act === 'add' || act === 'refuse') {
      if (act === 'refuse') await row().locator('input[name=header]').fill('x-svrngs-key');
      await row().locator('input[name=secret]').fill(process.env.VALUE ?? '');
      await row().locator('button', { hasText: 'Add this header' }).click();
      const told = act === 'add' ? 'svrngs adds authorization' : 'svrngs can’t add';
      await row().getByText(told).waitFor({ timeout: 30_000 });
    } else if (act === 'stop') {
      await row().locator('button', { hasText: 'Stop adding authorization' }).click();
      await row().getByText('wants a header on each request').waitFor({ timeout: 30_000 });
    }
  }
  seen.row = await row().innerText();
  seen.lent = await page.evaluate(() => fetch('/lent').then((r) => r.json()));
} catch (error) {
  seen.error = String(error);
} finally {
  await page.screenshot({ path: shot, fullPage: true }).catch(() => {});
  await browser.close();
}
console.log(JSON.stringify(seen));
process.exit(seen.error ? 1 : 0);
