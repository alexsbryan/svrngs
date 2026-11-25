// Plays the customer in a headed Chromium: presses the player's buttons, and
// reads back each attempt as the app recorded it.
//
//   node drive.mjs <out dir> <step>...    steps: play, help, fix, again, end
//
// Each step is a press; play, fix and again wait for the attempt's verdict.
// Every attempt is a fresh page, so a fresh Hls instance and media element.
import { chromium } from '../../../../web/node_modules/playwright/index.mjs';
import { appendFileSync } from 'node:fs';
import { join } from 'node:path';

const [out, ...steps] = process.argv.slice(2);
const browser = await chromium.launch({ headless: false });
appendFileSync(join(out, 'browser.txt'), `chromium ${browser.version()}\n`);
const ctx = await browser.newContext({ viewport: { width: 1000, height: 720 } });
let page = await ctx.newPage();
await page.goto('http://127.0.0.1:8402/');
for (const step of steps) {
  if (['play', 'fix', 'again'].includes(step)) {
    if (step === 'play') { await page.close(); page = await ctx.newPage(); await page.goto('http://127.0.0.1:8402/'); }
    const before = await page.textContent('#status');
    await page.click(`#${step}`);
    await page.waitForFunction((b) => { const t = document.getElementById('status').textContent; return t !== b && !t.endsWith('…'); }, before, { timeout: 90_000, polling: 250 });
  } else if (step === 'help') {
    await page.click('#help');
    await page.click('#confirm');
  } else await page.click(`#${step}`);
  console.log(step, '→', await page.textContent('#status'));
}
await browser.close();
