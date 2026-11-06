import assert from 'node:assert/strict';
import { readFile, writeFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';

const { chromium } = await import(pathToFileURL(resolve('web/node_modules/playwright/index.mjs')).href);
const runtime = resolve('target/relay-demo');
const scene = JSON.parse(await readFile(resolve(runtime, 'public-scene.json'), 'utf8'));
const browser = await chromium.launch({ headless: true });
const logs = [];
const result = { mode: process.argv[2] ?? 'observe', address: scene.publication.address };
try {
  const context = await browser.newContext();
  const page = await context.newPage();
  page.on('console', message => logs.push(message.type() + ': ' + message.text()));
  page.on('pageerror', error => logs.push('pageerror: ' + error.message));
  await page.goto(scene.publication.address, { waitUntil: 'domcontentloaded', timeout: 30000 });
  await page.waitForFunction(() => {
    const input = document.querySelector('#name');
    return (input && input.getClientRects().length) || document.body.innerText.includes('did not answer') || document.body.innerText.includes('invitation has ended');
  }, null, { timeout: 90000 }).catch(() => {});
  result.nameForm = await page.locator('#name').isVisible();
  result.text = await page.locator('body').innerText();
  result.layers = logs.filter(line => line.includes('LAYER ') || line.includes('JOIN:'));
  if (process.argv[2] === 'install') {
    assert.ok(result.nameForm, 'public native name form must be reachable');
    await page.locator('#name').fill('Public browser proof');
    await page.getByRole('button', { name: 'Get the installation command' }).click();
    await page.waitForFunction(() => document.body.innerText.includes('curl -fsSL'), null, { timeout: 20000 });
    result.installation = await page.locator('body').innerText();
    assert.ok(result.installation.includes('https://group.svrnme.sh/install.sh'));
    assert.ok(result.installation.includes('--as'));
    assert.ok(!result.installation.includes('.ts.net'));
    console.log('PASS real public Chromium short-address navigation -> issuer ask -> native name form -> canonical installation command');
  } else {
    console.log('Observed public browser name form:', result.nameForm);
    console.log('Observed issuer dial established:', logs.some(line => line.includes('QUIC connection established')));
    console.log('Observed computer-did-not-answer UI:', result.text.includes('did not answer'));
  }
} finally {
  await writeFile(resolve(runtime, 'public-browser-' + result.mode + '.json'), JSON.stringify({ ...result, logs }, null, 2) + '\n', { mode: 0o600 });
  await browser.close();
}
