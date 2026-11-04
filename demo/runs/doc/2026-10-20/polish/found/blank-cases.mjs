// node blank-cases.mjs, with DOC=<apps/doc> and TMPD=<a scratch dir>, Playwright resolvable: its answer is blank-cases.out.
// Three pages on one empty doc, three ways of starting: what paragraphs the server ends up with.
import { chromium } from 'playwright';
import { spawn } from 'node:child_process';
import { mkdtempSync } from 'node:fs';
import { tmpdir } from 'node:os';
const DOC = process.env.DOC;
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
async function server() {
  const p = spawn(process.execPath, [`${DOC}/server.mjs`, '--port', '0', '--keep', mkdtempSync(`${process.env.TMPD}/k-`)]);
  let s = ''; p.stdout.on('data', (d) => (s += d));
  while (!/127\.0\.0\.1:(\d+)/.test(s)) await sleep(50);
  return { url: `http://127.0.0.1:${s.match(/:(\d+)\//)[1]}/`, end: () => p.kill() };
}
const end = (page) => page.evaluate(() => { const last = document.querySelector('.tiptap').lastElementChild; const r = document.createRange(); r.selectNodeContents(last); r.collapse(false); const s = getSelection(); s.removeAllRanges(); s.addRange(r); });
const paras = (page) => page.evaluate(() => [...document.querySelectorAll('.tiptap > *')].map((p) => { const c = p.cloneNode(true); c.querySelectorAll('.collaboration-carets__caret').forEach((x) => x.remove()); return c.textContent; }));
const b = await chromium.launch();
for (const kase of ['enter-first, at once, on an empty doc', 'typing first, at once, on an empty doc', 'one first line, then enter-first, at once']) {
  const srv = await server();
  const pages = [];
  for (let i = 0; i < 3; i++) { const p = await (await b.newContext()).newPage(); await p.goto(srv.url); await p.waitForSelector('.tiptap'); pages.push(p); }
  await sleep(800);
  // Waits until every page shows the first line: without the wait, on a
  // loaded Mac, a page could still be on its own starting line when it
  // edited (blank-cases-unwaited.out), which is the cause itself.
  if (kase.startsWith('one first')) { await pages[0].locator('.tiptap').focus(); await end(pages[0]); await pages[0].keyboard.type('Friday'); for (const p of pages) await p.waitForFunction(() => document.querySelector('.tiptap').innerText.includes('Friday'), null, { timeout: 15000 }); }
  await Promise.all(pages.map(async (p, i) => { await p.locator('.tiptap').focus(); await end(p); await sleep(30); if (!kase.startsWith('typing')) await p.keyboard.press('Enter'); await p.keyboard.type(`line ${'ABC'[i]}`); }));
  await sleep(2000);
  console.log(JSON.stringify({ kase, paragraphs: await paras(pages[0]), same: JSON.stringify(await paras(pages[1])) === JSON.stringify(await paras(pages[0])) }));
  srv.end();
}
await b.close();
