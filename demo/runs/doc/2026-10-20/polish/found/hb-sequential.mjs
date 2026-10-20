// HB=<the same dir> node hb-sequential.mjs: one page at a time, Enter between lines, on each engine, with and without HardBreak.
import { chromium, webkit, devices } from 'playwright';
import { spawn } from 'node:child_process';
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
for (const kind of ['with', 'without']) for (const [eng, dev] of [[chromium, 'Pixel 7'], [webkit, 'iPhone 13'], [chromium, null]]) {
  const p = spawn(process.execPath, [`${process.env.HB}/${kind}/server.mjs`, '--port', '0', '--keep', `${process.env.HB}/k-${kind}-${eng.name()}-${dev}-${Date.now()}`]); let s = ''; p.stdout.on('data', (d) => (s += d));
  while (!/:(\d+)\//.test(s)) await sleep(50);
  const b = await eng.launch(); const pg = await (await b.newContext(dev ? devices[dev] : {})).newPage();
  await pg.goto(`http://127.0.0.1:${s.match(/:(\d+)\//)[1]}/`); await pg.waitForSelector('.tiptap[contenteditable="true"]');
  await pg.locator('.tiptap').focus(); await pg.keyboard.type('one');
  for (let i = 0; i < 3; i++) { await pg.keyboard.press('Enter'); await pg.keyboard.type(`line ${i}`); }
  await sleep(800);
  console.log(kind.padEnd(8), (dev ?? 'desktop').padEnd(10), await pg.evaluate(() => document.querySelector('.tiptap').innerHTML.replace(/<span class="collaboration[^]*?<\/span>/g, '')));
  await b.close(); p.kill();
}
