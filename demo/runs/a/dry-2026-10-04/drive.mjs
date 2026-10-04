// Drives Jellyfin's own web player through a member's local address, one
// fresh browser profile per mode, recording a HAR with bodies.
// usage: node drive.mjs <mode: none|signin|seed> <local address> <out dir>
//   none    open the address and touch nothing
//   signin  sign in as the viewer, typing its password, then play
//   seed    the web client's stored sign-in names the viewer with a placeholder
//           token ("door"), never a real one, then play: the door sets the header
// Env: PLAYWRIGHT_CORE, CHROME, SECRETS (the dir holding viewer_pw), and for
// signin and seed ITEM and SERVER; for seed, SEED (address, server, user).
import { createRequire } from 'node:module';
import fs from 'node:fs';
const require = createRequire(import.meta.url);
const { chromium } = require(process.env.PLAYWRIGHT_CORE);

const [mode, U, out] = process.argv.slice(2);
fs.mkdirSync(out, { recursive: true });
const log = [];
const note = (s) => { const l = `${new Date().toISOString()} ${s}`; log.push(l); console.log(l); };

const browser = await chromium.launch({
  executablePath: process.env.CHROME,
  headless: true,
  args: ['--autoplay-policy=no-user-gesture-required'],
});
const ctx = await browser.newContext({ recordHar: { path: `${out}/full.har`, content: 'embed' }, viewport: { width: 1280, height: 800 } });
const page = await ctx.newPage();
page.on('console', (m) => { if (m.type() === 'error') note(`console.error ${m.text().slice(0, 200)}`); });
page.on('response', (r) => { if (r.status() >= 400) note(`http ${r.status()} ${r.request().method()} ${r.url().slice(0, 160)}`); });

const shot = async (name) => { await page.screenshot({ path: `${out}/${name}.png` }); note(`screenshot ${name}.png`); };
const seen = async () => page.evaluate(() => ({
  hash: location.hash,
  login: !!document.querySelector('#txtManualName, .manualLoginForm, input[type=password]'),
  password: !!document.querySelector('input[type=password]'),
  cards: document.querySelectorAll('.card').length,
  text: document.body.innerText.slice(0, 300).replace(/\s+/g, ' '),
}));

if (mode === 'seed') {
  // The web client keeps who it is signed in as in localStorage; tell it a
  // user and a placeholder token, never a real one: the door sets the header.
  const meta = JSON.parse(process.env.SEED);
  await page.addInitScript((m) => {
    if (localStorage.getItem('jellyfin_credentials')) return;
    localStorage.setItem('jellyfin_credentials', JSON.stringify({ Servers: [{
      ManualAddress: m.address, LocalAddress: m.address, LastConnectionMode: 2, ManualAddressOnly: true,
      Name: m.serverName, Id: m.serverId, DateLastAccessed: Date.now(), UserId: m.userId, AccessToken: 'door',
    }] }));
  }, meta);
}

try {
  note(`open ${U}`);
  await page.goto(U, { waitUntil: 'load' });
  await page.waitForTimeout(6000);
  note(`after open: ${JSON.stringify(await seen())}`);
  await shot(`${mode}-1-open`);

  if (mode === 'signin') {
    const pw = fs.readFileSync(`${process.env.SECRETS}/viewer_pw`, 'utf8').trim();
    const manual = page.locator('.btnManual:visible');
    if (await manual.count()) { await manual.first().click(); await page.waitForTimeout(1000); }
    await page.fill('#txtManualName', 'viewer');
    await page.fill('#txtManualPassword', pw);
    await page.locator('.manualLoginForm button[type=submit]').click();
    await page.waitForTimeout(6000);
    note(`after sign-in: ${JSON.stringify(await seen())}`);
    await shot(`${mode}-2-signed-in`);
  }

  const stored = await page.evaluate(() => localStorage.getItem('jellyfin_credentials'));
  fs.writeFileSync(`${out}/stored-credentials.json`, stored ?? 'null');
  note(`localStorage jellyfin_credentials present: ${!!stored}`);

  const item = process.env.ITEM;
  if (mode !== 'none' && item) {
    await page.goto(`${U}web/#/details?id=${item}&serverId=${process.env.SERVER}`);
    await page.waitForTimeout(5000);
    note(`details: ${JSON.stringify(await seen())}`);
    await shot(`${mode}-3-details`);
    const play = page.locator('.mainDetailButtons .btnPlay:visible');
    note(`play buttons: ${await play.count()}`);
    if (await play.count()) {
      await play.first().click({ force: true });
      const t0 = Date.now();
      const playing = await page.waitForFunction(() => { const v = document.querySelector('video'); return v && v.currentTime > 0.5 && !v.paused; }, null, { timeout: 30000 }).then(() => true, () => false);
      note(`playing: ${playing} after ${Date.now() - t0} ms`);
      if (playing) {
        await page.waitForTimeout(8000);
        for (const to of [60, 20, 75]) {
          const s0 = Date.now();
          await page.evaluate((t) => { document.querySelector('video').currentTime = t; }, to);
          const ok = await page.waitForFunction((t) => { const v = document.querySelector('video'); return v && !v.paused && !v.seeking && v.readyState >= 3 && v.currentTime >= t - 15; }, to, { timeout: 15000 }).then(() => true, () => false);
          note(`seek to ${to}s: playing again ${ok} after ${Date.now() - s0} ms`);
          await page.waitForTimeout(3000);
        }
      }
      const v = await page.evaluate(() => { const v = document.querySelector('video'); return v && { src: v.currentSrc.slice(0, 200), t: v.currentTime, paused: v.paused, w: v.videoWidth, h: v.videoHeight, rs: v.readyState }; });
      note(`video: ${JSON.stringify(v)}`);
      await shot(`${mode}-4-playing`);
    }
  }
} catch (e) {
  note(`driver stopped: ${String(e).split('\n')[0]}`);
} finally {
  await ctx.close();
  await browser.close();
  fs.writeFileSync(`${out}/driver.log`, log.join('\n') + '\n');
}
