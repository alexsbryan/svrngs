// A cold-start construction proof through Xenia's actual browser transport
// and reach filter. No native newcomer or preaccepted request account.
import assert from 'node:assert/strict';
import { spawn, execFileSync } from 'node:child_process';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { createInterface } from 'node:readline';
import { dirname, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { proxied } from '../../../web/tests/page.mjs';
import { seek } from '../../jellyfin/check/person.mjs';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '../../..');
const xenia = process.env.XENIA_ROOT ?? resolve(root, '../xenia');
const load = (p) => import(pathToFileURL(resolve(xenia, p)).href);
const [{ parseScene }, { launch, reach }] = await Promise.all([
  load('src/core/scene.ts'),
  load('src/browser.ts'),
]);
process.env.XENIA_BROWSER_PROXY = await proxied();
process.env.XENIA_ALLOW_TEST_TLS = 'true';
const out = resolve(root, 'target/newcomer-film', new Date().toISOString().replace(/[:.]/g, '-'));
await mkdir(out, { recursive: true });
const world = spawn(resolve(root, 'demo/xenia/holyarrs/film-newcomer'), [], {
  cwd: root,
  stdio: ['pipe', 'pipe', 'inherit'],
});
const exited = new Promise((done) => world.on('exit', (code, signal) => done({ code, signal })));
const lines = createInterface({ input: world.stdout });
const report = { kind: 'cold-start construction proof', checks: [], controls: [] };
let browser;
let page;
let at = 'world startup';
const CONTROL_DONE = Symbol('the named negative control was observed');
const wait = async (what, read, holds, ms = 90_000) => {
  const end = Date.now() + ms;
  let last;
  do {
    last = await read();
    if (holds(last)) return last;
    await new Promise((r) => setTimeout(r, 250));
  } while (Date.now() < end);
  throw new Error(`${what}: ${JSON.stringify(last)}`);
};
try {
  const first = await lines[Symbol.asyncIterator]().next();
  assert.ok(!first.done, 'the real world prints its scene');
  const scene = parseScene(first.value);
  await writeFile(resolve(out, 'scene-private.json'), first.value);
  const state = JSON.parse(
    await readFile(resolve(root, 'target/scenes/xenia-film-chromium/world-private.json'), 'utf8'),
  );
  const services = JSON.parse(await readFile(state.services_state, 'utf8'));
  report.product_binary = state.product_binary;
  report.plant = process.env.NEWCOMER_PLANT ?? null;
  const viewerPassword = await readFile(resolve(state.jellyfin_secrets, 'bo_pw'), 'utf8');
  const launched = await launch('phone');
  browser = launched.browser;
  const context = await browser.newContext(launched.options);
  const away = await reach(context, scene.reach);
  page = await context.newPage();
  page.setDefaultTimeout(30_000);
  report.navigations = [];
  report.page_errors = [];
  report.media_requests = [];
  const safeUrl = (url) => {
    try {
      const u = new URL(url);
      return `${u.origin}${u.pathname}`;
    } catch {
      return '<unreadable address>';
    }
  };
  page.on('response', (response) => {
    if (/\/Videos\/|PlaybackInfo|authenticatebyname/i.test(response.url())) {
      const entry = {
        at,
        url: safeUrl(response.url()),
        status: response.status(),
        headers: response.headers(),
      };
      report.media_requests.push(entry);
      response
        .finished()
        .then((error) => {
          entry.finished = error ? String(error) : 'complete';
        })
        .catch((error) => {
          entry.finish_unobserved = String(error);
        });
    }
  });
  page.on('requestfailed', (request) => {
    if (/\/Videos\/|PlaybackInfo/i.test(request.url()))
      report.media_requests.push({
        at,
        url: safeUrl(request.url()),
        failed: request.failure()?.errorText,
      });
  });
  page.on('framenavigated', (frame) => {
    if (frame === page.mainFrame())
      report.navigations.push({ at, url: frame.url().replace(/#[^\s"]+/g, '#<fragment>') });
  });
  page.on('pageerror', (error) => report.page_errors.push({ at, error: String(error) }));
  at = 'N1: ordinary invitation';
  const arrival = Date.now();
  let joinedAt;
  await page.goto(state.ordinary);
  await page.waitForSelector('#join:not([hidden]), #status.bad', { timeout: 90_000 });
  if (await page.locator('#join').isVisible()) {
    await page.fill('#name', 'Bo');
    joinedAt = Date.now();
    await page.locator('#join button').click();
  }
  await page.waitForSelector('#status.good, #status.bad', { timeout: 90_000 });
  if (process.env.NEWCOMER_PLANT === 'closed-invitation') {
    assert.ok(await page.locator('#status.bad').isVisible(), 'N1: a closed invitation cannot join');
    report.refusal = await page.locator('body').innerText();
    assert.match(
      report.refusal,
      /ended|stopped|closed|new link|not valid|did not let|lets nobody in.*Ask for a new one/is,
    );
    const api = (await readFile(resolve(state.owner_data, 'api'), 'utf8')).trim();
    const token = (await readFile(resolve(state.owner_data, 'token'), 'utf8')).trim();
    const sample = JSON.parse(await readFile(state.evidence, 'utf8')).observations.at(-1).observed;
    const people = await (
      await fetch(`${api}/v1/rail/people?namespace=${sample.people.namespace}`, {
        headers: { authorization: `Bearer ${token}` },
      })
    ).json();
    assert.ok(
      Array.isArray(people.people) && people.people.every((p) => p.person !== 'Bo'),
      'N1: the authoritative record contains no refused newcomer',
    );
    report.controls.push(
      'N1: closed ordinary invitation visibly refused and added no Bo to the real record',
    );
    throw CONTROL_DONE;
  }
  assert.ok(
    await page.locator('#status.good').isVisible(),
    'N1: the live ordinary invitation joins',
  );
  report.arrival_ms = Date.now() - arrival;
  report.join_ms = joinedAt === undefined ? null : Date.now() - joinedAt;
  report.page_shell = await page.locator('script[type=module]').first().getAttribute('src');
  report.checks.push('N1: fresh phone browser joined as Bo by the ordinary invitation');
  const groupRoot = new URL('/', state.ordinary).href;
  at = 'N2: host-created request invitation';
  try {
    await page.goto(state.app_invitation.invitation);
  } catch (e) {
    if (away.some((url) => url.startsWith(state.app_invitation.origin))) {
      report.controls.push(
        'N2: lender-local invitation rejected by the actual friend reach filter',
      );
      throw new Error(
        'N2: the invitation copied on Ana’s computer names her localhost, not a destination her friend can open',
      );
    }
    throw e;
  }
  await page
    .getByRole('button', { name: 'Join and sign in', exact: true })
    .waitFor({ timeout: 90_000 });
  const ownPassword = 'newcomer-only-7fu28q4pv9';
  await page.getByLabel('Name', { exact: true }).fill('Bo');
  await page.getByLabel('Password', { exact: true }).fill(ownPassword);
  await page.getByLabel('Confirm password').fill(ownPassword);
  await page.getByRole('button', { name: 'Join and sign in', exact: true }).click();
  await page.waitForURL((u) => u.pathname === '/');
  const me = await page.evaluate(async () => (await fetch('/api/v1/auth/me')).json());
  assert.equal(me.name, 'Bo', 'N2: the new app account belongs to Bo');
  await page
    .getByText(/^Good .*, Bo$/)
    .first()
    .waitFor({ timeout: 90_000 });
  report.checks.push(
    'N2: Bo accepted Ana’s actual UI-produced invitation and made his own request account',
  );
  await page.screenshot({ path: resolve(out, 'new-request-account.png') });
  at = 'N4: request as the first-time account';
  const requestsOrigin = new URL(page.url()).origin;
  await page.getByRole('navigation').getByRole('link', { name: 'Search', exact: true }).click();
  const search = page.getByRole('combobox', { name: 'Search for a title, an artist or a person' });
  await search.fill('Copper');
  await search.press('Enter');
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
  const pending = await wait(
    'N4: independent request observation',
    async () => JSON.parse(await readFile(state.evidence, 'utf8')),
    (e) =>
      e.observations.some((o) =>
        o.observed.asks.body?.asks?.some((a) => a.asked_by.name === 'Bo' && a.state === 'asked'),
      ),
  );
  report.checks.push('N4: independent observer saw Bo’s own request waiting for approval');
  report.request_observed = pending.observations.at(-1);
  at = 'N3: Movie Night permission and discovery';
  const finding = Date.now();
  await page.goto(groupRoot);
  await page.locator('#lent button', { hasText: 'Movie Night' }).click();
  await page.getByRole('heading', { name: 'Movie Night' }).waitFor({ timeout: 90_000 });
  await page.fill('#q', 'Test Pattern One');
  await page.getByRole('button', { name: 'Search', exact: true }).click();
  const copy = page.locator('#results a.copy.first').first();
  await copy.waitFor({ timeout: 60_000 });
  const href = await copy.getAttribute('href');
  const item = new URLSearchParams(href.split('#/details?')[1]).get('id');
  assert.ok(item, 'N3: a copy identifies the actual film');
  assert.equal(
    item,
    state.expected_item,
    'N3: the chosen copy is the independently named Test Pattern One item',
  );
  await copy.click();
  await page.waitForSelector('#txtManualName, .btnManual', { timeout: 90_000 });
  const manual = page.locator('.btnManual:visible');
  if (await manual.count()) await manual.first().click();
  await page.fill('#txtManualName', 'bo');
  at = 'N5: wrong password and recovery';
  await page.fill('#txtManualPassword', 'not-the-password');
  const refused = page.waitForResponse(
    (r) => r.url().toLowerCase().includes('authenticatebyname') && r.status() === 401,
  );
  await page.locator('.manualLoginForm button[type=submit]').click();
  await refused;
  await page
    .getByText(/Invalid username or password|Invalid user or password/i)
    .first()
    .waitFor();
  report.controls.push('N5: wrong viewing password refused with visible words');
  await page.fill('#txtManualPassword', viewerPassword.trim());
  await page.locator('.manualLoginForm button[type=submit]').click();
  at = 'N3: actual playback';
  await page.locator('.mainDetailButtons .btnPlay:visible').first().tap();
  // WebKit can present a decoded first frame paused, with its native Play
  // control visible. Count the real second touch, as the existing film driver
  // does; never call play() from evaluate to bypass the browser's gesture rule.
  await wait(
    'the player has a decoded frame',
    () =>
      page.evaluate(() => {
        const v = document.querySelector('video');
        return v && { paused: v.paused, time: v.currentTime, ready: v.readyState };
      }),
    (v) => v && v.ready >= 3,
    30_000,
  );
  const held = await page.evaluate(() => {
    const v = document.querySelector('video');
    return v.paused && v.currentTime < 0.5;
  });
  if (held) {
    report.native_play_touch = true;
    await page.tap('video');
  }
  await page.waitForFunction(
    () => {
      const v = document.querySelector('video');
      return v && !v.paused && v.currentTime > 1;
    },
    null,
    { timeout: 45_000 },
  );
  report.find_and_play_ms = Date.now() - finding;
  const sought = await page.evaluate(seek, 50);
  assert.ok(sought.ms !== null && Math.abs(sought.at - sought.target) < 3, JSON.stringify(sought));
  await wait(
    'N3: independent playing identity and item',
    async () => JSON.parse(await readFile(state.evidence, 'utf8')),
    (e) =>
      e.observations.some((o) =>
        o.observed.playback.sessions?.some(
          (s) => s.user === 'bo' && s.item === item && s.position_ticks > 0 && s.paused === false,
        ),
      ),
  );
  report.checks.push(
    'N3: cold newcomer found Ana’s copy, recovered from a wrong password, played and sought the same item as bo',
  );
  report.seek = sought;
  await page.screenshot({ path: resolve(out, 'newcomer-playing.png') });
  at = 'N5: return without another identity';
  await page.goto(state.ordinary);
  await page.waitForSelector('#status.good', { timeout: 90_000 });
  assert.equal(
    await page.locator('#join').isVisible(),
    false,
    'the same browser does not join again',
  );
  await page.locator('#lent button', { hasText: 'holyarrs' }).click();
  await page.waitForURL((u) => u.origin === requestsOrigin && u.pathname === '/', {
    timeout: 90_000,
  });
  const returned = await page.evaluate(async () => (await fetch('/api/v1/auth/me')).json());
  assert.equal(returned.name, 'Bo');
  await page
    .getByText(/^Good .*, Bo$/)
    .first()
    .waitFor({ timeout: 90_000 });
  await page
    .getByRole('navigation')
    .getByRole('link', { name: /^Requests/ })
    .click();
  await page
    .getByTestId('ask')
    .filter({ hasText: 'Copper Lighthouse' })
    .getByTestId('ask-state')
    .filter({ hasText: 'Requested' })
    .waitFor();
  await page.waitForURL((u) => u.pathname === '/requests');
  report.checks.push(
    'N5: original invitation reopened the same group; request account and waiting request survived',
  );
  at = 'N5: temporarily unavailable app and retry';
  execFileSync('podman', ['stop', `${services.pod}-holyarrs`], { timeout: 30_000 });
  await page.reload({ waitUntil: 'domcontentloaded' });
  const unavailable = await page.locator('body').innerText();
  report.unavailable = unavailable;
  assert.match(
    unavailable,
    /did not answer|not answering|try again|try opening|could not reach/i,
    'N5: the unavailable app gives a next step',
  );
  execFileSync('podman', ['start', `${services.pod}-holyarrs`], { timeout: 30_000 });
  await wait(
    'N5: host app starts again',
    async () => (await fetch(`${services.at}/api/v1/auth/state`)).status,
    (s) => s === 200,
    30_000,
  );
  await page.reload();
  // Reload keeps /requests; the Home greeting is not on this page. Judge
  // recovery by the request the person returned to, then verify its session.
  await page
    .getByTestId('ask')
    .filter({ hasText: 'Copper Lighthouse' })
    .getByTestId('ask-state')
    .filter({ hasText: 'Requested' })
    .waitFor({ timeout: 90_000 });
  const recovered = await page.evaluate(async () => (await fetch('/api/v1/auth/me')).json());
  assert.equal(recovered.name, 'Bo', 'N5: restarting the app preserves Bo’s account/session');
  report.controls.push(
    'N5: actual service outage produced recovery words; retry after restart retained the same app account',
  );
  assert.deepEqual(away, [], 'no private or outside origin was needed');
  report.verdict = 'passed';
} catch (e) {
  if (e === CONTROL_DONE) {
    report.verdict = 'passed';
    report.kind = 'named negative control';
  } else {
    report.verdict = 'failed';
    report.at = at;
    report.error = String(e).replace(/#[^\s"]+/g, '#<fragment>');
    if (page) {
      report.video = await page
        .evaluate(() => {
          const v = document.querySelector('video');
          return v
            ? {
                time: v.currentTime,
                paused: v.paused,
                ready: v.readyState,
                network: v.networkState,
                error: v.error?.message ?? null,
                buffered: Array.from({ length: v.buffered.length }, (_, i) => [
                  v.buffered.start(i),
                  v.buffered.end(i),
                ]),
              }
            : null;
        })
        .catch((error) => ({ could_not_read: String(error) }));
      report.text = (
        await page
          .locator('body')
          .innerText()
          .catch(() => '')
      ).slice(0, 3000);
      await page.screenshot({ path: resolve(out, 'stopped.png') }).catch(() => {});
    }
    process.exitCode = 1;
  }
} finally {
  await browser?.close();
  world.stdin.end();
  lines.close();
  await Promise.race([exited, new Promise((r) => setTimeout(r, 30_000))]);
  await writeFile(resolve(out, 'report.json'), JSON.stringify(report, null, 2));
  console.log(JSON.stringify({ out, verdict: report.verdict, at: report.at, error: report.error }));
}
