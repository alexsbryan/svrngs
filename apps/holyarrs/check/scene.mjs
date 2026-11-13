// Browsers through each person's lent origin; direct calls only observe.
import assert from 'node:assert/strict';
import { mkdirSync, readFileSync, readdirSync, writeFileSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { setTimeout as pause } from 'node:timers/promises';
import { arrived, launch } from '../../../web/tests/page.mjs';
import { record, streamed } from '../../jellyfin/check/person.mjs';

const args = JSON.parse(process.argv[2]);
const services = JSON.parse(readFileSync(args.state));
mkdirSync(args.out, { recursive: true });
const seen = { steps: [], errors: [] };
const step = (at) => {
  seen.at = at;
  writeFileSync(`${args.out}/progress.json`, JSON.stringify(seen, null, 2));
};
const bo = await launch(args.engine);
const ana = await launch('chromium');
let page = await bo.context.newPage();
record(bo.context, page, 'bo', { DUMP: args.dump, T0: Date.now() });
let own = await ana.context.newPage();
const watched = new WeakSet();
function watchErrors(p) {
  if (watched.has(p)) return;
  watched.add(p);
  p.on('pageerror', (e) =>
    seen.errors.push({
      who: p === own ? 'ana' : 'bo',
      at: seen.at ?? 'setup',
      message: String(e),
      stack: e.stack?.slice(0, 1500),
    }),
  );
}
for (const p of [page, own]) watchErrors(p);
bo.context.on('console', (m) => {
  if (/^(OPEN|LEND|SIGN)/.test(m.text())) {
    seen.logs ??= [];
    seen.logs.push(m.text().replace(/#[^\s"]+/g, '#<fragment>'));
  }
});
const passed = (bar, evidence) => seen.steps.push({ bar, verdict: 'passed', ...evidence });
async function wait(what, f, ms = 120_000) {
  const end = Date.now() + ms;
  let last;
  do {
    last = await f();
    if (last) return last;
    await pause(500);
  } while (Date.now() < end);
  throw new Error(`${what} not observed in ${ms}ms; last: ${JSON.stringify(last)}`);
}
async function api(p, method, path, body) {
  return p.evaluate(
    async ([method, path, body]) => {
      const me = await (await fetch('/api/v1/auth/me')).json();
      const r = await fetch(`/api/v1${path}`, {
        method,
        headers: { 'content-type': 'application/json', 'x-csrf-token': me.csrf },
        ...(body === undefined ? {} : { body: JSON.stringify(body) }),
      });
      return { status: r.status, body: await r.json() };
    },
    [method, path, body],
  );
}
async function nightSearch(p) {
  await p.fill('#q', services.film.title);
  await p.getByRole('button', { name: 'Search', exact: true }).click();
  await p.waitForFunction(
    () =>
      document.querySelector('#said li') &&
      [...document.querySelectorAll('#said li')].every((l) => l.dataset.state !== 'asking'),
  );
  return {
    cards: await p.locator('#results > li').allTextContents(),
    said: await p.locator('#said').innerText(),
  };
}
async function login(p, who) {
  await p.waitForLoadState('domcontentloaded');
  await arrived(p, 90_000);
  await p.getByLabel('Name', { exact: true }).fill(who.name);
  await p.getByLabel('Password', { exact: true }).fill(who.password);
  await p.getByRole('button', { name: 'Sign in', exact: true }).click();
  await p.waitForURL((u) => u.pathname === '/', { timeout: 30_000 });
  await p
    .getByText(/^Good .*, /)
    .filter({ hasText: who.name })
    .first()
    .waitFor({ timeout: 90_000 });
}
async function phoneOpen(p, word, heading) {
  const group = new URL('/', args.phone_link);
  await p.goto(group.href);
  await p.waitForSelector('#status.good', { timeout: 90_000 });
  await Promise.all([
    p.waitForURL(
      (u) => u.hostname !== group.hostname && !['/open.html', '/svrngs-open'].includes(u.pathname),
      {
        waitUntil: 'domcontentloaded',
        timeout: 90_000,
      },
    ),
    p.locator('#lent button', { hasText: word }).click(),
  ]);
  if (heading) {
    const ready =
      word === 'holyarrs'
        ? p.getByText(/^Good .*, Bo$/)
        : p.getByRole('heading', { name: heading });
    await ready.first().waitFor({ timeout: 90_000 });
  }
}
try {
  const system = await (await fetch(`${args.jf}/System/Info/Public`)).json();
  seen.runtime = {
    jellyfin: system.Version,
    browser: bo.context.browser().version(),
    source_head: execFileSync('git', ['rev-parse', 'HEAD'], { encoding: 'utf8' }).trim(),
  };
  if (args.phone_link) {
    await page.goto(args.phone_link);
    await page.waitForSelector('#status.good', { timeout: 90_000 });
  } else {
    await page.goto(args.bo_home);
    await arrived(page, 90_000);
  }
  await own.goto(args.ana_home);
  await arrived(own, 90_000);
  const home = page;
  if (args.phone_link) {
    await phoneOpen(page, 'holyarrs');
  } else {
    [page] = await Promise.all([
      bo.context.waitForEvent('page'),
      home.getByRole('link', { name: /^Open .*holyarrs$/ }).click(),
    ]);
  }
  [own] = await Promise.all([
    ana.context.waitForEvent('page'),
    own.getByRole('link', { name: /^Open .*holyarrs$/ }).click(),
  ]);
  for (const p of [page, own]) watchErrors(p);
  await login(page, services.bo);
  await login(own, services.ana);
  const boAt = page.url();
  const me = await page.evaluate(async () => (await fetch('/api/v1/auth/me')).json());
  assert.equal(me.name, 'Bo', `H1: ${JSON.stringify(me)}`);
  seen.steps.push({ bar: 'H1', verdict: 'passed', name: me.name, url: page.url() });
  await page.screenshot({ path: `${args.out}/bo-holyarrs.png` });
  step('Movie Night, before acquisition');
  let night;
  if (args.phone_link) {
    night = page;
    await phoneOpen(night, 'Movie Night', 'Movie Night');
  } else {
    [night] = await Promise.all([
      bo.context.waitForEvent('page'),
      home.getByRole('button', { name: 'Open Movie Night', exact: true }).click(),
    ]);
  }
  await arrived(night, 90_000);
  await night.getByRole('heading', { name: 'Movie Night' }).waitFor();
  const before = await nightSearch(night);
  assert.equal(
    before.cards.length,
    0,
    `H4: title absent before request: ${JSON.stringify(before)}`,
  );
  seen.before = before;

  if (args.phone_link) {
    await phoneOpen(page, 'holyarrs', /Bo/);
  }
  step('Bo searches HolyArrs');
  await page.goto(new URL('/search?q=Copper&kind=movie', boAt).href);
  await page
    .getByRole('main')
    .getByRole('link', { name: /Copper Lighthouse,/ })
    .first()
    .click();
  await page.getByRole('button', { name: /^Ask for Copper Lighthouse( too)?$/ }).click();
  await page
    .getByRole('dialog', { name: /^Ask for Copper Lighthouse/ })
    .getByRole('button', { name: 'Ask for Copper Lighthouse', exact: true })
    .click();
  step('Bo asks');
  const asked = await wait('H2: Bo request recorded', async () => {
    const r = await api(page, 'GET', '/asks');
    return r.body.asks?.find((a) => a.name === services.film.title);
  });
  assert.equal(asked.state, 'asked', JSON.stringify(asked));
  const denied = await api(page, 'POST', `/asks/${asked.id}/approve`, {});
  assert.equal(denied.status, 403, `H2: Bo cannot approve: ${JSON.stringify(denied)}`);
  const empty = await (await fetch(`${services.downloader}/api/v2/torrents/info`)).json();
  assert.equal(empty.length, 0, 'H2: no download before approval');
  step('Ana sees the request');
  await own.goto(new URL('/requests', args.ana_at).href);
  const card = own
    .getByRole('list', { name: 'Requests' })
    .getByRole('listitem')
    .filter({ hasText: 'Copper Lighthouse' });
  await card.getByRole('button', { name: 'Open Copper Lighthouse, asked by Bo' }).click();
  const win = own.getByRole('dialog', { name: 'Copper Lighthouse', exact: true });
  await win.getByTestId('ask-banner').filter({ hasText: 'Bo asked' }).waitFor();
  await win
    .getByRole('region', { name: 'What to add' })
    .getByRole('button', { name: 'Approve', exact: true })
    .click({ trial: true });
  await own.screenshot({ path: `${args.out}/ana-approval.png`, animations: 'disabled' });
  passed('H2', {
    request_id: asked.id,
    asker: 'Bo',
    approve_as_bo: denied.status,
    downloads_before_approval: empty.length,
  });
  if (args.plant === 'holyarrs:no-seed')
    execFileSync('podman', ['stop', `${services.pod}-seed`], { timeout: 30_000 });
  if (args.plant !== 'holyarrs:no-approval')
    await win
      .getByRole('region', { name: 'What to add' })
      .getByRole('button', { name: 'Approve', exact: true })
      .click();
  step('HolyArrs downloads and imports');
  const imported = await wait(
    'H3: HolyArrs imports the downloaded film',
    async () => {
      const files = readdirSync(services.library, { recursive: true }).filter((p) =>
        /Copper.*\.mp4$/i.test(p),
      );
      return files.length === 1 ? files[0] : false;
    },
    ['holyarrs:no-approval', 'holyarrs:no-seed'].includes(args.plant) ? 15_000 : 120_000,
  );
  const expected = JSON.parse(readFileSync(`${services.dir}/data/release.json`));
  const bytes = readFileSync(`${services.library}/${imported}`);
  assert.equal(
    createHash('sha256').update(bytes).digest('hex'),
    expected.sha256,
    'H3: imported bytes match the seed',
  );
  const torrents = await (await fetch(`${services.downloader}/api/v2/torrents/info`)).json();
  assert.equal(torrents.length, 1, 'H3: exactly one download');
  assert.equal(torrents[0].hash, expected.hash, 'H3: the requested release');
  assert.equal(torrents[0].progress, 1, 'H3: downloader complete');
  assert.ok(
    torrents[0].downloaded_session >= expected.bytes,
    `H3: actual peer transfer: ${JSON.stringify(torrents)}`,
  );
  passed('H3', {
    file: imported,
    bytes: bytes.length,
    sha256: expected.sha256,
    downloaded_session: torrents[0].downloaded_session,
  });
  step('Movie Night finds the imported film');
  if (args.phone_link) {
    await phoneOpen(night, 'Movie Night', 'Movie Night');
  }
  const found = await wait(
    'H4: Movie Night finds Ana’s newly imported copy',
    async () => {
      const result = await nightSearch(night);
      seen.search = result;
      return result.cards.some(
        (c) => c.includes('Copper Lighthouse') && c.includes('Watch from Ana’s library'),
      )
        ? result
        : false;
    },
    args.plant === 'holyarrs:no-readers' ? 10_000 : 90_000,
  );
  await night.screenshot({ path: `${args.out}/movie-night.png` });
  passed('H4', { before, after: found });
  step('Bo follows the copy and plays in Jellyfin');
  const copy = night.locator('#results a.copy.first').first();
  seen.copy = await copy.getAttribute('href');
  const itemId = new URLSearchParams(seen.copy.split('#/details?')[1]).get('id');
  assert.ok(itemId, 'H5: the copy names one Jellyfin item');
  await copy.click();
  await night.waitForSelector('#txtManualName, .btnManual', { timeout: 60_000 });
  const manual = night.locator('.btnManual:visible');
  if (await manual.count()) await manual.first().click();
  await night.fill('#txtManualName', 'bo');
  await night.fill('#txtManualPassword', readFileSync(`${args.jf_secrets}/bo_pw`, 'utf8'));
  await night.locator('.manualLoginForm button[type=submit]').click();
  const play = night.locator('.mainDetailButtons .btnPlay:visible').first();
  await play.waitFor({ timeout: 30_000 });
  // What H5 holds is the film's bytes reaching Bo through Bo's lend, by
  // its worker on a phone (person.mjs `streamed`); that a browser decodes
  // them is the playback step's (crates/svrngs-node/tests/browser/playback.rs).
  const lend = new URL(night.url()).origin;
  const film = streamed(bo.context, itemId, 60_000);
  await play.click();
  const got = await film;
  assert.ok(got.ms !== null, `H5: Bo's browser receives the film: ${JSON.stringify(got)}`);
  assert.equal(got.origin, lend, `H5: through Bo's lend: ${JSON.stringify(got)}`);
  assert.notEqual(got.origin, new URL(args.jf).origin, 'H5: not from Jellyfin itself');
  if (args.phone_link)
    assert.ok(got.from_worker, `H5: by the lend's worker: ${JSON.stringify(got)}`);
  const auth = `MediaBrowser Token="${readFileSync(`${args.jf_secrets}/admin_token`, 'utf8')}"`;
  const sessions = await (
    await fetch(`${args.jf}/Sessions`, { headers: { authorization: auth } })
  ).json();
  // Jellyfin's own word that Bo is in it as Bo; which film a session plays
  // it hears only once the <video> has started, so it is not asked here.
  const session = sessions.find((s) => s.UserName === 'bo');
  assert.ok(session, `H5: Jellyfin independently sees Bo: ${JSON.stringify(sessions)}`);
  await night.screenshot({ path: `${args.out}/bo-playing.png` });
  passed('H5', { user: session.UserName, item: itemId, stream: got });

  step('The group’s access is withdrawn');
  const stranger = await launch('chromium');
  try {
    const q = await stranger.context.newPage();
    const r = await q.goto(new URL('/api/v1/auth/me', args.bo_at).href);
    assert.ok([401, 403].includes(r.status()), `H6: stranger refused: ${r.status()}`);
  } finally {
    await stranger.close();
  }
  if (args.phone_link) {
    step('Leaving Jellyfin for HolyArrs');
    await phoneOpen(page, 'holyarrs', /Bo/);
  }
  step('Holding HolyArrs’s live stream before removal');
  await page.evaluate(async () => {
    window.__held = { began: false, cut: null };
    const r = await fetch('/live');
    if (!r.ok) throw new Error(`live answered ${r.status}`);
    const reader = r.body.getReader();
    window.__held.reader = reader;
    const first = await reader.read();
    if (first.done) throw new Error('live ended before removal');
    window.__held.began = true;
    void (async () => {
      try {
        while (!(await reader.read()).done) {
          /* drain the live stream */
        }
      } catch {
        /* removal breaks the body */
      } finally {
        window.__held.cut = Date.now();
      }
    })();
  });
  step('Removing Bo with a live stream open');
  if (args.plant === 'holyarrs:early-stream')
    await page.evaluate(() => window.__held.reader.cancel());
  assert.equal(
    await page.evaluate(() => window.__held.cut),
    null,
    'H6: stream must still be open before removal',
  );
  const removedAt = Date.now();
  execFileSync(args.binary, ['remove', 'Bo', '--group', 'Friends'], {
    input: '\n',
    encoding: 'utf8',
    timeout: 30_000,
    env: { HOME: args.ana_dir, USER: 'ana', PATH: process.env.PATH, NO_COLOR: '1' },
  });
  step('Observing the removed member');
  await page.waitForFunction(() => window.__held.cut !== null, null, { timeout: 5000 });
  const cut = await page.evaluate(() => window.__held.cut);
  assert.ok(
    cut >= removedAt && cut - removedAt < 5000,
    `H6: already-open live stream cut: ${cut - removedAt}ms`,
  );
  const refused = await page.evaluate(async () => {
    const r = await fetch('/api/v1/asks');
    return { status: r.status, refused: r.headers.get('x-svrngs-refused') };
  });
  assert.ok(
    [401, 403].includes(refused.status),
    `H6: fresh request refused after removal: ${JSON.stringify(refused)}`,
  );
  assert.equal((await api(own, 'GET', '/auth/me')).status, 200, 'H6: Ana still reaches her lend');
  passed('H6', { stranger: 'refused', removed: refused, stream_cut_ms: cut - removedAt });
  if (args.plant === 'holyarrs:credential')
    writeFileSync(
      `${args.dump}/bo/planted-credential.bin`,
      readFileSync(`${args.jf_secrets}/admin_token`),
    );
  seen.fixture = await (await fetch(`${services.fixture}/evidence`)).json();
  assert.deepEqual(seen.fixture.unexpected, [], 'H8: every fixture request has an explicit answer');
  // WebKit reports cancelling Jellyfin's in-flight fetch on navigation and
  // the deliberately cut SSE body as page errors (recorded with their phase).
  // Only those observed messages, at those actions, are expected. Since H5
  // no longer waits for the film to play, Jellyfin's report of the play,
  // `/Sessions/Playing`, can still be in flight as Bo leaves, and WebKit
  // says its cancelling so (the Mac, 2026-11-12, demo/runs/flakes/2026-11-12).
  // The player's own code can be in flight too: webpack's loader rejects with
  // a ChunkLoadError when the navigation cancels its <script> for a /web/
  // chunk (twice at load ~4, svrngs-46's scenes-rerun.log, 2026-11-12). So
  // leaving cancels each way a page loads: fetch (two wordings) and script.
  const cancellation = (e) =>
    args.phone_link &&
    e.who === 'bo' &&
    ((e.at === 'Leaving Jellyfin for HolyArrs' &&
      (e.message === 'Unhandled Promise Rejection: TypeError: Load failed' ||
        /^Fetch API cannot load .+\/Sessions\/Playing(\/\w+)? due to access control checks\.$/.test(
          e.message,
        ) ||
        /^Unhandled Promise Rejection: ChunkLoadError: Loading chunk \d+ failed\.\n\(error: https:\/\/[^/]+\/web\/[\w.-]+\.chunk\.js\)$/.test(
          e.message,
        ))) ||
      (['Removing Bo with a live stream open', 'Observing the removed member'].includes(e.at) &&
        e.message ===
          'FetchEvent.respondWith received an error: Error: error reading a body from connection'));
  seen.cancellations = seen.errors.filter(cancellation);
  assert.deepEqual(
    seen.errors.filter((e) => !cancellation(e)),
    [],
    'H8: no unexpected browser script errors',
  );
} catch (e) {
  seen.observed = await api(own, 'GET', '/downloads').catch((e) => String(e));
  seen.asks = await api(own, 'GET', '/asks').catch((e) => String(e));
  await own.screenshot({ path: `${args.out}/ana-stopped.png` }).catch(() => {});
  seen.stopped = {
    error: String(e).replace(/#[^\s"]+/g, '#<fragment>'),
    url: page.url(),
    text: (await page.locator('body').innerText()).slice(0, 3000),
  };
  await page.screenshot({ path: `${args.out}/stopped.png` }).catch(() => {});
} finally {
  await new Promise((done) => setTimeout(done, 1000));
  await bo.close();
  await ana.close();
  writeFileSync(`${args.out}/result.json`, JSON.stringify(seen, null, 2));
  console.log(JSON.stringify(seen));
}
