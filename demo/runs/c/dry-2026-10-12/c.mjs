// Demo C's dry run in a browser (run.py): each newcomer's computer in
// Chromium, Cy's phone in WebKit as an iPhone (web/tests/page.mjs).
//
//   node c.mjs line   <link>        the group's page on a computer: the line it shows
//   node c.mjs person <json>        a newcomer's home page until it lists the group, its
//                                   folder and both lends; Sam's film played, Ana's album
//                                   played, a trip photo opened from their own disk
//   node c.mjs phone  <json>        Cy: the link on a phone, a film and an album through it
//   node c.mjs read   <json>        a home page's words, loaded again until they hold `until`
//   node c.mjs music  <json>        an album played at `origin` alone (with `header`, the
//                                   person's header sent as the door sends it: a probe of
//                                   Navidrome's pages, never a judged step)
//
// Prints one JSON line. Every time is ms from `t0` (else from the start); a
// step that did not happen within its time is null with what the page said,
// never a number guessed. Jellyfin's steps are apps/jellyfin/check/person.mjs's
// (signed in or asked, then the film's page by its id and its play button).
import { createHash } from 'node:crypto';
import { existsSync, readFileSync, statSync } from 'node:fs';
import { launch } from '../../../../web/tests/page.mjs';

const [mode, raw] = process.argv.slice(2);
const a = raw?.startsWith('{') ? JSON.parse(raw) : { link: raw };
const T0 = a.t0 ?? Date.now();
const at = () => Date.now() - T0;
const out = { mode };
const words = (page) => page.evaluate(() => document.body?.innerText.slice(0, 300).replace(/\s+/g, ' ') ?? '').catch(() => '?');

// Until `fn` holds or `ms` passes: its value, or null.
async function until(fn, ms, every = 250) {
  const end = Date.now() + ms;
  for (;;) {
    const v = await fn().catch(() => null);
    if (v) return v;
    if (Date.now() > end) return null;
    await new Promise((r) => setTimeout(r, every));
  }
}

// Jellyfin through the lend open in `page`: what it showed someone new to
// it, then the film `film` playing. Nothing is typed: a sign-in asked is
// where it stops.
async function film(page, { server, film }) {
  const r = {};
  const t = Date.now();
  r.shown = await page.waitForSelector('#txtManualName, .manualLoginForm, .btnManual, .card', { timeout: 60_000 }).then(() => Date.now() - t, () => null);
  if (r.shown === null) return { ...r, stopped: `no sign-in and no home: ${await words(page)}` };
  await page.waitForTimeout(1500);
  r.asked = (await page.locator('#txtManualName:visible, .manualLoginForm:visible, .btnManual:visible').count()) > 0;
  if (r.asked) return { ...r, stopped: `asked to sign in: ${await words(page)}` };
  await page.evaluate(([id, s]) => { location.hash = `#/details?id=${id}&serverId=${s}`; }, [film, server]);
  const p0 = Date.now();
  const play = page.locator('.mainDetailButtons .btnPlay:visible');
  try {
    await play.first().waitFor({ timeout: 30_000 });
    await play.first().click({ force: true });
  } catch (e) {
    return { ...r, stopped: `no play button: ${String(e).split('\n')[0]}; ${await words(page)}` };
  }
  r.playing = await until(() => page.evaluate(() => { const v = document.querySelector('video'); return v && v.currentTime > 1 && !v.paused; }), 30_000)
    .then((ok) => (ok ? Date.now() - p0 : null));
  if (r.playing === null) r.stopped = `not playing 30 s after play: ${await words(page)}`;
  return r;
}

// Navidrome through the lend open in `page`: whom it signed in, or the
// sign-in it asked for, then the album `album` playing.
async function music(page, { album }) {
  const r = {};
  const t = Date.now();
  r.shown = await page.waitForSelector('input[name="username"], header', { timeout: 60_000 }).then(() => Date.now() - t, () => null);
  if (r.shown === null) return { ...r, stopped: `no sign-in and no app: ${await words(page)}` };
  await page.waitForTimeout(1500);
  r.asked = (await page.locator('input[name="username"]:visible').count()) > 0;
  if (r.asked) return { ...r, stopped: `asked to sign in: ${await words(page)}` };
  r.as = await page.evaluate(() => localStorage.getItem('username'));
  await page.evaluate((id) => { location.hash = `#/album/${id}/show`; }, album);
  const p0 = Date.now();
  const play = page.getByRole('button', { name: /^play$/i });
  try {
    await play.first().waitFor({ timeout: 30_000 });
    await play.first().click();
  } catch (e) {
    return { ...r, stopped: `no play button: ${String(e).split('\n')[0]}; ${await words(page)}` };
  }
  r.playing = await until(() => page.evaluate(() => { const v = document.querySelector('audio'); return v && v.currentTime > 1 && !v.paused; }), 30_000)
    .then((ok) => (ok ? Date.now() - p0 : null));
  if (r.playing === null) r.stopped = `not playing 30 s after play: ${await words(page)}`;
  else r.track = await page.evaluate(() => document.querySelector('.music-player-panel .audio-title, .react-jinke-music-player-mobile-header-title, .audio-title')?.textContent ?? null);
  return r;
}

// The home page at `url`, loaded again as a person would until it lists
// `group`, a folder at `dir` and a link for each of `lends` (the start of
// its words): when, and what it says.
async function home(page, { url, group, dir, lends, deadline }) {
  for (;;) {
    await page.goto(url).catch((e) => (out.home_error = String(e).split('\n')[0]));
    const ok = await page.waitForFunction(([g, d, ls]) => {
      const h = [...document.querySelectorAll('h2')].some((e) => e.textContent === g);
      const f = !d || [...document.querySelectorAll('p.folder')].some((e) => e.textContent.includes(d));
      const links = [...document.querySelectorAll('a')].map((e) => e.textContent);
      return h && f && ls.filter(Boolean).every((l) => links.some((t) => t.startsWith(l)));
    }, [group, dir, lends], { timeout: 5_000 }).then(() => true, () => false);
    if (ok || Date.now() > deadline) {
      return {
        listed: ok ? at() : null,
        said: await page.locator('main').innerText().catch(() => ''),
        links: await page.evaluate(() => Object.fromEntries([...document.querySelectorAll('a')].map((e) => [e.textContent, e.href]))),
      };
    }
  }
}

// The photo at `path` on this disk, the bytes `sha` names, opened as an
// image: when it was there, and its size as the browser decoded it.
async function photo(page, { path, sha, deadline }) {
  const there = await until(async () => {
    if (!existsSync(path) || statSync(path).size === 0) return false;
    return createHash('sha256').update(readFileSync(path)).digest('hex') === sha;
  }, Math.max(0, deadline - Date.now()), 200);
  if (!there) return { on_disk: null, stopped: `${path} not on this disk, as Ana's, by ${deadline - T0} ms` };
  const r = { on_disk: at() };
  await page.goto(`file://${path}`);
  r.decoded = await page.evaluate(() => { const i = document.querySelector('img'); return i && { w: i.naturalWidth, h: i.naturalHeight }; });
  r.opened = r.decoded?.w > 0 ? at() : null;
  return r;
}

const engine = mode === 'phone' ? 'webkit' : 'chromium';
const { context, close } = await launch(engine);
const page = await context.newPage();
const logs = [];
page.on('pageerror', (e) => logs.push(`pageerror: ${e}`));
page.on('response', (r) => { if (r.status() >= 400) logs.push(`${r.status()} ${r.request().method()} ${r.url().slice(0, 120)}`); });
try {
  if (mode === 'line') {
    await page.goto(a.link);
    await page.waitForFunction(() => document.getElementById('line')?.textContent?.startsWith('curl '), null, { timeout: 60_000 });
    out.shown = at();
    out.line = await page.locator('#line').textContent();
    out.title = await page.title();
  } else if (mode === 'person') {
    const deadline = T0 + 300_000;
    out.home = await home(page, { url: a.home, group: a.group, dir: a.dir, lends: [a.film_lend, a.music_lend], deadline });
    if (out.home.listed !== null) {
      out.film_opened = at();
      await page.getByRole('link', { name: a.film_lend }).first().click();
      out.film = await film(page, a);
      out.film.done = out.film.playing == null ? null : at();
      if (a.music_lend) {
        await page.goto(a.home);
        await page.getByRole('link', { name: a.music_lend }).first().click({ timeout: 30_000 }).catch((e) => (out.music_link = String(e).split('\n')[0]));
        out.music_opened = at();
        out.music = await music(page, a);
        out.music.done = out.music.playing == null ? null : at();
      }
    }
    out.photo = await photo(page, { path: a.photo, sha: a.sha, deadline: T0 + 300_000 });
    const all = [out.film?.done, ...(a.music_lend ? [out.music?.done] : []), out.photo?.opened];
    out.done = all.every((t) => t != null) ? Math.max(...all) : null;
  } else if (mode === 'phone') {
    await page.goto(a.link);
    await page.waitForFunction(
      () => !document.getElementById('join').hidden || /good|bad/.test(document.getElementById('status').className),
      null, { timeout: 90_000 });
    out.asked_name = await page.evaluate(() => !document.getElementById('join').hidden);
    if (out.asked_name) {
      out.name_asked_at = at();
      await page.fill('#name', a.name);
      await page.click('#join button');
    }
    await page.waitForSelector('#status.good, #status.bad', { timeout: 90_000 });
    out.status = { at: at(), said: await page.locator('#status').textContent() };
    const listed = async (want) => until(async () => {
      const lent = await page.evaluate(() => [...document.querySelectorAll('#lent button')].map((b) => b.textContent));
      return want.every((w) => lent.some((l) => l.startsWith(w))) && lent;
    }, 90_000, 500);
    out.lent = await listed([a.film_lend, a.music_lend]);
    out.lent_at = out.lent ? at() : null;
    if (out.lent) {
      out.film_opened = at();
      await page.locator('#lent button', { hasText: a.film_lend }).first().click();
      await page.waitForURL((u) => u.pathname.startsWith('/web/'), { timeout: 90_000 }).catch(() => {});
      out.film = await film(page, a);
      out.film.done = out.film.playing == null ? null : at();
      await page.goto('about:blank');
      await page.goto(new URL(a.link).origin + '/');
      await page.waitForSelector('#status.good, #status.bad', { timeout: 90_000 }).catch(() => {});
      if (await listed([a.music_lend])) {
        out.music_opened = at();
        await page.locator('#lent button', { hasText: a.music_lend }).first().click();
        await page.waitForURL((u) => u.hostname !== new URL(a.link).hostname, { timeout: 90_000 }).catch(() => {});
        out.music = await music(page, a);
        out.music.done = out.music.playing == null ? null : at();
      } else out.music = { stopped: `the group's page, opened again, did not list ${a.music_lend}: ${await words(page)}` };
    }
    const all = [out.film?.done, out.music?.done];
    out.done = all.every((t) => t != null) ? Math.max(...all) : null;
  } else if (mode === 'read') {
    const end = Date.now() + (a.within ?? 0);
    for (;;) {
      await page.goto(a.url).catch(() => {});
      await page.waitForFunction(() => !/Reading your groups/.test(document.body.innerText), null, { timeout: 10_000 }).catch(() => {});
      out.said = await page.locator('main').innerText().catch(() => '');
      if (!a.until || out.said.includes(a.until)) { out.at = at(); break; }
      if (Date.now() > end) { out.at = null; break; }
      await page.waitForTimeout(2000);
    }
  } else if (mode === 'music') {
    if (a.header) await context.setExtraHTTPHeaders({ 'X-Svrngs-Person': a.header });
    await page.goto(a.origin);
    out.music = await music(page, a);
  } else {
    throw new Error(`no mode ${mode}`);
  }
} catch (e) {
  out.stopped = `${String(e).split('\n')[0]}; the page says: ${await words(page)}`;
} finally {
  out.logs = logs.slice(-30);
  await close();
  console.log(JSON.stringify(out));
}
