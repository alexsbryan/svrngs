// How many of WebKit's UI-process threads are held in AppKit's
// `-[NSAnimation _runBlocking]` after ten pages, each in a context closed
// after it loads, with the defaults given as arguments
// (`node leak.mjs -NSAutomaticWindowAnimationsEnabled NO`) passed to
// Playwright.app through a wrapper, since Playwright refuses an argument
// that does not start with '-'. → run-leak.txt.
import { createRequire } from 'node:module';
import { execSync } from 'node:child_process';
import { chmodSync, mkdtempSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';

const require = createRequire(new URL('../../../../web/package.json', import.meta.url));
const { webkit } = require('playwright');

const extra = process.argv.slice(2);
const wrapper = `${mkdtempSync(`${tmpdir()}/wk-`)}/webkit.sh`;
writeFileSync(wrapper, `#!/bin/sh\nexec "${webkit.executablePath()}" "$@" ${extra.join(' ')}\n`);
chmodSync(wrapper, 0o755);
const browser = await webkit.launch({ executablePath: wrapper });
const pid = execSync("pgrep -n -f 'Playwright.app/Contents/MacOS/Playwright'").toString().trim();
for (let i = 0; i < 10; i++) {
  const context = await browser.newContext();
  await (await context.newPage()).goto('data:text/html,x');
  await context.close();
}
const held = () =>
  String(execSync(`sample ${pid} 1 2>/dev/null | grep -c 'NSAnimation _runBlocking' || true`)).trim();
const after = held();
await new Promise((done) => setTimeout(done, 15_000));
console.log(
  `${extra.join(' ') || '(no defaults)'}: ${after} threads in NSAnimation _runBlocking after 10 pages, ${held()} 15 s later`,
);
await browser.close();
