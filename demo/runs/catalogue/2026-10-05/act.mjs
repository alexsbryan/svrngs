// Bo does one app's first real act, on a phone through the page by the
// link (run.sh `act`). The way in is apps/jellyfin/check/drive.mjs's (u5-4):
// open the link, give a name, open the app from the group's list. Then the
// act, in the app's own pages as a person would, signing in once with the
// account the lender made (lender.mjs):
//   home-assistant  the lender turns the lamp on; Bo's page shows it, live
//   immich          takes a 100 MB video, and plays it, seeking three times
//   nextcloud       takes a file from the browser
//   actual          opens a budget
//   paperless       takes in a PDF
// and checks it from the lender's side, through the app's own API on its
// port: what Bo's page says is not taken as the act.
//
//   node act.mjs '{"app", "way": "wildcard"|"floor", "engine": "chromium"|"webkit",
//                  "link", "name", "shots": "<dir for its screenshots>"}'
//   env: SECRETS (bo_pw, and <app>.json from lender.mjs)
//
// Prints one JSON line: `did` true or false, `why` when not, every step's
// ms, and what the page showed at the end. A step that did not happen
// within its time is reported with what the page showed, never guessed.
import { execFileSync } from 'node:child_process';
import fs from 'node:fs';
import { launch } from '../../../../web/tests/page.mjs';

const a = JSON.parse(process.argv[2]);
const { SECRETS } = process.env;
const boPw = fs.readFileSync(`${SECRETS}/bo_pw`, 'utf8').trim();
const lender = () => JSON.parse(fs.readFileSync(`${SECRETS}/${a.app}.json`, 'utf8'));
const out = { app: a.app, way: a.way, engine: a.engine, steps: {}, did: false };
fs.mkdirSync(a.shots, { recursive: true });
const tag = `${a.app}-${a.way}-${a.engine}`;

const { context, close } = await launch(a.engine);
const page = await context.newPage();
const failed = [];
// A lend's label, at the front of its host or after /l/ on the floor, is not kept.
const unlabel = (u) => u.replace(/^(https?:\/\/)[a-z0-9]{20,}\./, '$1<label>.').replace(/\/l\/[a-z0-9]{20,}\//, '/l/<label>/');
page.on('response', (r) => {
  if (r.status() >= 400 && failed.length < 40) failed.push(`${r.status()} ${r.request().method()} ${unlabel(r.url())}`);
});
page.on('requestfailed', (r) => failed.length < 40 && failed.push(`failed ${r.method()} ${unlabel(r.url())}: ${r.failure()?.errorText}`));
const said = [];
page.on('console', (m) => ['error', 'warning'].includes(m.type()) && said.length < 30 && said.push(`${m.type()}: ${m.text().slice(0, 300)}`));
page.on('pageerror', (e) => said.length < 30 && said.push(`pageerror: ${String(e.message).slice(0, 300)}`));
const shown = () => page.evaluate(() => document.body?.innerText.slice(0, 300).replace(/\s+/g, ' ') ?? '').catch((e) => `? ${e.message}`);
async function step(what, f) {
  const t0 = Date.now();
  try {
    const r = await f();
    out.steps[what] = Date.now() - t0;
    return r ?? true;
  } catch (e) {
    out.steps[what] = null;
    out.why = `${what}: ${String(e.message).split('\n')[0]}`;
    throw e;
  }
}

// ── the way in ──────────────────────────────────────────────────────────
async function open() {
  await step('the list', async () => {
    await page.goto(a.link);
    await page.waitForFunction(() => !document.getElementById('join').hidden || /good|bad/.test(document.getElementById('status').className), null, { timeout: 120_000 });
    await page.fill('#name', a.name);
    await page.click('#join button');
    await page.waitForSelector('#status.good', { timeout: 120_000 });
  });
  out.listed = await page.evaluate(() => [...document.querySelectorAll('#lent button')].map((b) => b.textContent));
  await step('the app', async () => {
    const list = page.url();
    await page.locator('#lent button', { hasText: a.app }).click();
    await page.waitForURL((u) => u.href !== list && !u.pathname.includes('open.html'), { timeout: 90_000 });
    await page.waitForLoadState('load', { timeout: 60_000 });
  });
  out.at = unlabel(page.url());
}

// ── each app's act ──────────────────────────────────────────────────────
const fill = async (sel, v) => {
  await page.locator(sel).first().waitFor({ timeout: 60_000 });
  await page.locator(sel).first().fill(v);
};

const acts = {
  // Its sign-in, then the lamp: the lender turns it on through the API, and
  // Bo's page, which keeps every state from its socket, shows it.
  async 'home-assistant'() {
    await step('sign-in', async () => {
      await fill('input[name="username"]', 'bo');
      await fill('input[name="password"]', boPw);
      await page.keyboard.press('Enter');
      await page.waitForFunction(() => document.querySelector('home-assistant')?.hass?.states?.['input_boolean.lamp'], null, { timeout: 90_000 });
    });
    const token = lender().token;
    const before = await page.evaluate(() => document.querySelector('home-assistant').hass.states['input_boolean.lamp'].state);
    const want = before === 'on' ? 'off' : 'on';
    await step('live', async () => {
      await fetch(`http://127.0.0.1:8123/api/services/input_boolean/turn_${want}`, {
        method: 'POST', headers: { authorization: `Bearer ${token}`, 'content-type': 'application/json' }, body: JSON.stringify({ entity_id: 'input_boolean.lamp' }),
      });
      await page.waitForFunction((w) => document.querySelector('home-assistant').hass.states['input_boolean.lamp'].state === w, want, { timeout: 30_000 });
    });
    out.saw = `the lamp ${before}, then ${want} on Bo's page`;
    return true;
  },

  // Its sign-in, an upload by its own button, then the video played and
  // sought; the lender's API reads the asset back.
  async immich() {
    const video = `${SECRETS}/../catalogue-100mb.mp4`;
    if (!fs.existsSync(video)) throw new Error(`no ${video}: run.sh makes it`);
    await step('sign-in', async () => {
      await fill('input[type="email"]', 'bo@house.test');
      await fill('input[type="password"]', boPw);
      await page.keyboard.press('Enter');
      await page.waitForURL((u) => !u.pathname.startsWith('/auth/login'), { timeout: 90_000 });
    });
    // Immich's own welcome for a person new to it, page by page, as they would.
    await step('welcome', async () => {
      for (let i = 0; i < 12 && !new URL(page.url()).pathname.startsWith('/photos'); i++) {
        // Each page's way on is its last button, named for the page after it.
        const next = page.locator('button:visible').last();
        await next.click({ timeout: 15_000 }).catch(() => {});
        await page.waitForTimeout(800);
      }
      await page.waitForURL((u) => u.pathname.startsWith('/photos'), { timeout: 30_000 });
    });
    const name = `catalogue-${a.way}-${a.engine}-${Date.now()}.mp4`;
    const copy = `${SECRETS}/../${name}`;
    fs.copyFileSync(video, copy);
    // A byte of its own, so Immich's duplicate check takes each run's file.
    fs.appendFileSync(copy, Buffer.from(`${tag} ${Date.now()}`));
    const size = fs.statSync(copy).size;
    const id = await step('upload', async () => {
      const chooser = page.waitForEvent('filechooser', { timeout: 30_000 });
      await page.getByRole('button', { name: /upload/i }).first().click();
      await (await chooser).setFiles(copy);
      // Bo's assets are Bo's alone: asked as Bo, on the lender's port.
      const { accessToken: token } = await (await fetch('http://127.0.0.1:2283/api/auth/login', {
        method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ email: 'bo@house.test', password: boPw }),
      })).json();
      for (let i = 0; i < 240; i++) {
        const r = await fetch('http://127.0.0.1:2283/api/search/metadata', {
          method: 'POST', headers: { authorization: `Bearer ${token}`, 'content-type': 'application/json' }, body: JSON.stringify({ originalFileName: name, withExif: true }),
        });
        const hit = (await r.json()).assets?.items?.[0];
        if (hit) {
          out.uploaded = { bytes: size, kept: hit.exifInfo?.fileSizeInByte ?? null };
          return hit.id;
        }
        await new Promise((r) => setTimeout(r, 1000));
      }
      throw new Error('the lender never saw the asset');
    });
    await step('play', async () => {
      await page.goto(new URL(`/photos/${id}`, page.url()).href);
      await page.waitForSelector('video', { timeout: 60_000 });
      const r = await page.evaluate(play, { at: null, within: 30_000 });
      out.played = r;
      if (r.ms === null) throw new Error(`it did not play: ${JSON.stringify(r)}`);
    });
    out.seeks = [];
    for (const at of [60, 20, 85]) {
      const r = await page.evaluate(play, { at, within: 30_000 });
      out.seeks.push(r);
    }
    if (out.seeks.some((s) => s.ms === null)) throw new Error(`a seek did not play again: ${JSON.stringify(out.seeks)}`);
    out.saw = `uploaded ${size} bytes; played in ${out.played.ms} ms; seeks ${out.seeks.map((s) => s.ms).join(', ')} ms`;
    return true;
  },

  // Its sign-in, then a file by the Files app's own upload; the lender's
  // WebDAV reads it back, byte for byte.
  async nextcloud() {
    await step('sign-in', async () => {
      await fill('input[name="user"]', 'bo');
      await fill('input[name="password"]', boPw);
      await page.keyboard.press('Enter');
      await page.waitForURL((u) => !u.pathname.includes('/login'), { timeout: 90_000 });
    });
    const name = `catalogue-${a.way}-${a.engine}-${Date.now()}.bin`;
    const file = `${SECRETS}/../${name}`;
    fs.writeFileSync(file, Buffer.concat([Buffer.alloc(5 * 1024 * 1024, 7), Buffer.from(`${tag} ${Date.now()}`)]));
    const want = fs.readFileSync(file);
    await step('upload', async () => {
      await page.goto(new URL('/apps/files/', page.url()).href);
      await page.locator('input[type="file"]').first().waitFor({ state: 'attached', timeout: 60_000 });
      await page.locator('input[type="file"]').first().setInputFiles(file);
      const basic = `Basic ${Buffer.from(`bo:${boPw}`).toString('base64')}`;
      for (let i = 0; i < 120; i++) {
        const r = await fetch(`http://127.0.0.1:8080/remote.php/dav/files/bo/${name}`, { headers: { authorization: basic } });
        if (r.ok) {
          const got = Buffer.from(await r.arrayBuffer());
          if (got.equals(want)) return;
        }
        await new Promise((r) => setTimeout(r, 1000));
      }
      throw new Error('the lender never read the file back whole');
    });
    out.saw = `${want.length} bytes taken, read back whole by the lender`;
    return true;
  },

  // The server's password, typed once; then a new budget, which opens, and
  // which the lender's server then holds.
  async actual() {
    const { token } = lender();
    const files = async () => ((await (await fetch('http://127.0.0.1:5006/sync/list-user-files', { headers: { 'x-actual-token': token } })).json()).data ?? []).length;
    out.files_before = await files();
    await step('sign-in', async () => {
      await fill('input[type="password"]', boPw);
      await page.keyboard.press('Enter');
      await page.getByRole('button', { name: /start budgeting|create new file/i }).first().waitFor({ timeout: 90_000 });
    });
    // With no budget of its own, its welcome's first button makes one and opens it.
    await step('budget', async () => {
      await page.getByRole('button', { name: /start budgeting|create new file/i }).first().click();
      await page.getByText(/to budget/i).first().waitFor({ timeout: 90_000 });
      for (let i = 0; i < 60; i++) {
        out.files = await files();
        if (out.files > out.files_before) return;
        await new Promise((r) => setTimeout(r, 1000));
      }
      throw new Error("the lender's server holds no new budget");
    });
    out.saw = `a budget open on Bo's page; the lender's server holds ${out.files} budget(s), ${out.files_before} before`;
    return true;
  },

  // Its sign-in, then a PDF by its own upload; the lender's API finds it
  // consumed.
  async paperless() {
    await step('sign-in', async () => {
      await fill('input[name="login"], input[name="username"]', 'bo');
      await fill('input[name="password"]', boPw);
      await page.keyboard.press('Enter');
      await page.waitForURL((u) => !u.pathname.includes('/accounts/login'), { timeout: 90_000 });
      await page.locator('input[type="file"]').first().waitFor({ state: 'attached', timeout: 90_000 });
    });
    const title = `catalogue ${a.way} ${a.engine} ${Date.now()}`;
    const file = `${SECRETS}/../catalogue-${a.way}-${a.engine}.pdf`;
    fs.writeFileSync(file, pdf(title));
    await step('upload', async () => {
      await page.locator('input[type="file"]').first().setInputFiles(file);
      const basic = lender().token;
      const name = `catalogue-${a.way}-${a.engine}`;
      for (let i = 0; i < 240; i++) {
        const r = await fetch(`http://127.0.0.1:8000/api/documents/?title__icontains=${encodeURIComponent(name)}`, { headers: { authorization: basic } });
        const docs = (await r.json()).results ?? [];
        const mine = docs.find((d) => (d.content ?? '').includes(title));
        if (mine) return (out.document = { id: mine.id, title: mine.title });
        await new Promise((r) => setTimeout(r, 1000));
      }
      throw new Error('the lender\'s Paperless never took it in');
    });
    out.saw = `consumed as document ${out.document.id}`;
    return true;
  },
};

// A one-page PDF holding `text`, by hand: no dependency.
function pdf(text) {
  const objs = [
    '<< /Type /Catalog /Pages 2 0 R >>',
    '<< /Type /Pages /Kids [3 0 R] /Count 1 >>',
    '<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Contents 4 0 R /Resources << /Font << /F1 5 0 R >> >> >>',
    null,
    '<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>',
  ];
  const stream = `BT /F1 18 Tf 72 700 Td (${text}) Tj ET`;
  objs[3] = `<< /Length ${stream.length} >>\nstream\n${stream}\nendstream`;
  let body = '%PDF-1.4\n';
  const at = [];
  objs.forEach((o, i) => {
    at.push(body.length);
    body += `${i + 1} 0 obj\n${o}\nendobj\n`;
  });
  const xref = body.length;
  body += `xref\n0 ${objs.length + 1}\n0000000000 65535 f \n${at.map((n) => `${String(n).padStart(10, '0')} 00000 n \n`).join('')}`;
  body += `trailer\n<< /Size ${objs.length + 1} /Root 1 0 R >>\nstartxref\n${xref}\n%%EOF\n`;
  return Buffer.from(body, 'latin1');
}

// In the page: page.mjs's `play`, inlined, since the act's page is the app's.
async function play({ at, within }) {
  const v = document.querySelector('video');
  if (!v) return { ms: null, error: `no <video>; the page says ${document.body.innerText.slice(0, 160)}` };
  const started = performance.now();
  if (at === null) {
    v.muted = true;
    v.play().catch(() => {});
  } else {
    v.currentTime = at;
  }
  const from = at ?? 0;
  return new Promise((done) => {
    const check = () => {
      const t = performance.now() - started;
      if (!v.seeking && !v.paused && v.currentTime > from + 0.25) done({ at, ms: Math.round(t), ct: +v.currentTime.toFixed(2) });
      else if (t > within) done({ at, ms: null, ct: v.currentTime, seeking: v.seeking, paused: v.paused, rs: v.readyState, error: v.error?.message ?? null });
      else setTimeout(check, 20);
    };
    check();
  });
}

try {
  await open();
  out.did = await acts[a.app]();
} catch (e) {
  out.why ??= String(e.message).split('\n')[0];
}
out.shown = await shown();
out.failed = failed;
out.console = said;
// The lend's worker's own account of what it did (web/lend.js, TRACE).
out.trace = await page.evaluate(async () => (await fetch(`${(await navigator.serviceWorker.getRegistration()).scope}svrngs-trace`)).text()).catch((e) => `? ${e.message}`);
await page.screenshot({ path: `${a.shots}/${tag}.png` }).catch(() => {});
await close();
execFileSync('true');
console.log(JSON.stringify(out));
