// One WebKit, pages opened one after another, each in a context of its
// own and closed: the page at which a navigation no longer loads, with
// the defaults given as arguments (`node wall.mjs
// -NSAutomaticWindowAnimationsEnabled NO`) passed to Playwright.app through
// a wrapper, as in leak.mjs. Playwright from web/node_modules. → run-wall.txt.
import { createRequire } from 'node:module';
import { chmodSync, mkdtempSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';

const require = createRequire(new URL('../../../../web/package.json', import.meta.url));
const { webkit } = require('playwright');

const extra = process.argv.slice(2);
const wrapper = `${mkdtempSync(`${tmpdir()}/wk-`)}/webkit.sh`;
writeFileSync(wrapper, `#!/bin/sh\nexec "${webkit.executablePath()}" "$@" ${extra.join(' ')}\n`);
chmodSync(wrapper, 0o755);
const browser = await webkit.launch({ executablePath: wrapper });
console.log(extra.join(' ') || '(no defaults)');
for (let i = 1; i <= 80; i++) {
  const context = await browser.newContext();
  const page = await context.newPage();
  const started = Date.now();
  try {
    await page.goto('data:text/html,<title>x</title>', { timeout: 5000 });
  } catch {
    console.log(`page ${i}: did not load in 5 s`);
    break;
  }
  await context.close();
  if (i % 10 === 0) console.log(`page ${i}: loaded in ${Date.now() - started} ms`);
}
await browser.close();
