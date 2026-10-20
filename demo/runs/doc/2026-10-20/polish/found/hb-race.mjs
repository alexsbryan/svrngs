// HB=<dir holding with/ and without/, two copies of apps/doc, without/ with HardBreak out of app.js's extensions> [KINDS=with,without] [ROUNDS=6] [DEVS=Pixel 7,Pixel 7,] node hb-race.mjs: three pages press Enter at once at the doc's end, each round.
import { chromium, devices } from 'playwright';
import { spawn } from 'node:child_process';
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const end = (x) => x.evaluate(() => { const l = document.querySelector('.tiptap').lastElementChild; const r = document.createRange(); r.selectNodeContents(l); r.collapse(false); const s = getSelection(); s.removeAllRanges(); s.addRange(r); });
for (const kind of (process.env.KINDS ?? 'with,without').split(',')) {
  let breaks = 0, rounds = 0;
  for (let round = 0; round < Number(process.env.ROUNDS ?? 6); round++) {
    const p = spawn(process.execPath, [`${process.env.HB}/${kind}/server.mjs`, '--port', '0', '--keep', `${process.env.HB}/r-${kind}-${round}-${Date.now()}`]); let s = ''; p.stdout.on('data', (d) => (s += d));
    while (!/:(\d+)\//.test(s)) await sleep(50);
    const url = `http://127.0.0.1:${s.match(/:(\d+)\//)[1]}/`;
    const b = await chromium.launch();
    const pages = [];
    for (const dev of (process.env.DEVS ?? 'Pixel 7,Pixel 7,').split(',').map((d) => d || null)) { const pg = await (await b.newContext(dev ? devices[dev] : {})).newPage(); await pg.goto(url); await pg.waitForSelector('.tiptap[contenteditable="true"]'); pages.push(pg); }
    await pages[2].locator('.tiptap').focus(); await pages[2].keyboard.type('Friday'); 
    for (const pg of pages) await pg.waitForFunction(() => document.querySelector('.tiptap').innerText.includes('Friday'));
    await Promise.all(pages.map(async (pg, i) => { await pg.locator('.tiptap').focus(); await end(pg); await sleep(30); await pg.keyboard.press('Enter'); await pg.keyboard.type(`page ${i} brings something`); }));
    await sleep(1000);
    const html = await pages[0].evaluate(() => document.querySelector('.tiptap').innerHTML);
    const n = (html.match(/<br(?![^>]*ProseMirror-trailingBreak)/g) ?? []).length;
    breaks += n; rounds += 1;
    console.log(kind, 'round', round, n, html.replace(/<span class="collaboration[^]*?<\/span>/g, '').slice(0, 300));
    await b.close(); p.kill();
  }
  console.log(kind, `${breaks} hard break(s) in ${rounds} rounds`);
}
