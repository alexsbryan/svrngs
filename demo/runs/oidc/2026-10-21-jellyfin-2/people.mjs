// Jellyfin through its stock SSO plugin, as the row's five bars meet it
// (ralph/next/oidc/STATE.md, oidc-jellyfin; SIGN_IN_WITH_SVRNGS §5a, §7 2).
//
//   node people.mjs <env.json> <raw.json>
//
// Alex on another computer (Chromium), Bo on a phone (Playwright's WebKit as
// an iPhone), Cy a member Kim removes. Each bar's evidence is written as it
// happens; a step that did not happen within its time is null with what the
// page showed, never a number guessed.
import fs from 'node:fs';
import { execFileSync } from 'node:child_process';
import { launch } from '../../../../web/tests/page.mjs';

const env = JSON.parse(fs.readFileSync(process.argv[2], 'utf8'));
const out = { at: new Date().toISOString(), people: {}, jellyfin: {}, bars: {} };
const T0 = Date.now();
const now = () => Date.now() - T0;
const save = () => fs.writeFileSync(process.argv[3], JSON.stringify(out, null, 2));
const shown = (page) => page.evaluate(() => document.body.innerText.slice(0, 400).replace(/\s+/g, ' ')).catch(() => '?');
const stored = (page) => page.evaluate(() => {
  try { return JSON.parse(localStorage.getItem('jellyfin_credentials') || '{}'); } catch { return {}; }
});

async function person(engine, { device } = {}) {
  const { context, close } = await launch(engine, { device });
  const page = await context.newPage();
  const logs = [];
  page.on('pageerror', (e) => logs.push(`pageerror: ${e}`));
  page.on('console', (m) => { if (m.type() === 'error') logs.push(`console: ${m.text().slice(0, 200)}`); });
  page.on('requestfailed', (r) => logs.push(`failed: ${r.url().slice(0, 150)}: ${r.failure()?.errorText}`));
  page.on('response', (r) => { if (r.status() >= 400) logs.push(`http ${r.status()} ${r.request().method()} ${r.url().slice(0, 160)}`); });
  return { context, close, page, logs };
}

// The SSO press, then the app's own home: the plugin's callback page writes
// the app's stored sign-in and replaces the page at its `/web/`.
async function jfSignIn(p, at, { link, name } = {}) {
  const t = {};
  try {
    if (link) {
      const opened = Date.now();
      await p.page.goto(link, { waitUntil: 'domcontentloaded', timeout: 90_000 });
      await p.page.waitForFunction(
        () => !document.getElementById('join').hidden || /good|bad/.test(document.getElementById('status').className),
        null, { timeout: 120_000 });
      if (name) { await p.page.fill('#name', name); await p.page.click('#join button'); }
      await p.page.waitForSelector('#status.good', { timeout: 120_000 });
      t.joined_ms = Date.now() - opened;
      t.listed = await p.page.evaluate(() => [...document.querySelectorAll('#lent button')].map((b) => b.textContent));
      await p.page.locator('#lent button', { hasText: 'jellyfin' }).first().click();
      await p.page.waitForURL((u) => u.pathname.startsWith('/web/'), { timeout: 90_000 });
      t.lend_ms = Date.now() - opened;
    } else {
      const opened = Date.now();
      await p.page.goto(at, { waitUntil: 'domcontentloaded', timeout: 90_000 });
      t.lend_ms = Date.now() - opened;
    }
    // Jellyfin's login page, with the lender's SSO button in its disclaimer.
    const button = p.page.locator('text=Sign in with SSO').first();
    const offered = Date.now();
    await button.waitFor({ timeout: 60_000 });
    t.login_page_ms = Date.now() - offered;
    t.sso_offered = true;
    t.typed = await p.page.locator('#txtManualName:visible, .manualLoginForm:visible').count() > 0;
    const pressed = Date.now();
    await button.click();
    await p.page.waitForFunction(() => {
      try {
        const c = JSON.parse(localStorage.getItem('jellyfin_credentials') || '{}');
        return c.Servers?.[0]?.AccessToken && c.Servers?.[0]?.UserId;
      } catch { return false; }
    }, null, { timeout: 60_000 });
    t.signed_in_ms = Date.now() - pressed;
    // The callback page writes the app's stored sign-in, then replaces the
    // page at its `/web/`. Read it there, after that navigation: reading it
    // on the callback page can land in a context the replace has destroyed
    // (the second take's first attempt: "Execution context was destroyed,
    // most likely because of a navigation", with the film playing).
    await p.page.waitForURL((u) => u.pathname.startsWith('/web/'), { timeout: 30_000 }).catch(() => {});
    let s = {};
    for (let i = 0; i < 50 && !s.AccessToken; i++) {
      s = (await stored(p.page).catch(() => ({}))).Servers?.[0] ?? {};
      if (!s.AccessToken) await p.page.waitForTimeout(100);
    }
    t.user_id = s.UserId ?? null;
    t.token = s.AccessToken ?? null;
    t.address = s.ManualAddress ?? null;
    t.url = p.page.url();
    await p.page.waitForSelector('.card', { timeout: 90_000 });
    t.home = await shown(p.page);
  } catch (e) {
    t.stopped = `${String(e.message).split('\n')[0]}; the page says: ${await shown(p.page)}`;
    t.url = p.page.url();
    t.callback = await p.page.evaluate(() => {
      const frame = document.querySelector('iframe');
      const html = document.documentElement.outerHTML;
      const base = html.match(/var data =[^;]*|location\.replace\('([^']+)'|iframe-main'\)\.src = '([^']+)'/g) ?? [];
      return {
        url: location.href, origin: location.origin,
        iframe: frame?.src ?? null,
        stored: Object.keys(localStorage),
        credentials: localStorage.getItem('jellyfin_credentials'),
        absolute: [...new Set((html.match(/http:\/\/localhost:[0-9]+[^'" ]*/g) ?? []).slice(0, 8))],
        base: base.slice(0, 4),
      };
    }).catch((why) => ({ error: String(why) }));
  }
  return t;
}

async function play(p, film, server) {
  try {
    const t0 = Date.now();
    await p.page.evaluate(([id, s]) => { location.hash = `#/details?id=${id}&serverId=${s}`; }, [film, server]);
    const play = p.page.locator('.mainDetailButtons .btnPlay:visible');
    await play.first().waitFor({ timeout: 30_000 });
    await play.first().click({ force: true });
    for (let i = 0; i < 120; i++) {
      const v = await p.page.evaluate(() => { const v = document.querySelector('video'); return v && { ct: v.currentTime, paused: v.paused }; });
      if (v && v.ct > 0.5 && !v.paused) return { played_ms: Date.now() - t0 };
      await p.page.waitForTimeout(250);
    }
    return { played_ms: null, why: `not playing; the page says: ${await shown(p.page)}` };
  } catch (e) {
    return { played_ms: null, why: `${String(e.message).split('\n')[0]}; the page says: ${await shown(p.page)}` };
  }
}

const jf = async (path, { method = 'GET', token, body } = {}) => {
  const r = await fetch(`${env.jf}${path}`, {
    method,
    headers: { 'content-type': 'application/json', authorization: `MediaBrowser Client="oidc-jellyfin run", Device="Kim computer", DeviceId="oidc-jellyfin-driver", Version="1"${token ? `, Token="${token}"` : ''}` },
    body: body && JSON.stringify(body),
  });
  return { status: r.status, body: r.status === 204 ? null : await r.json().catch(() => null) };
};
const users = async () => (await jf('/Users', { token: env.admin })).body ?? [];
const logLines = (path) => { try { return fs.readFileSync(path, 'utf8').split('\n').length; } catch { return 0; } };

let alex, bo, cy;
try {
  // 1. Alex, another computer, Chromium, at his own computer's address.
  alex = await person('chromium');
  out.people.alex = await jfSignIn(alex, env.alex_at);
  out.people.alex.engine = `chromium ${alex.context.browser()?.version() ?? ''}`.trim();
  out.people.alex.logs = alex.logs;
  if (out.people.alex.signed_in_ms != null) Object.assign(out.people.alex, await play(alex, env.film, env.server));
  save();

  // 2. Bo, a phone, WebKit as an iPhone, through the group's own page.
  bo = await person('webkit');
  out.people.bo = await jfSignIn(bo, null, { link: env.bo_link });
  out.people.bo.engine = `webkit ${bo.context.browser()?.version() ?? ''}`.trim();
  out.people.bo.logs = bo.logs;
  if (out.people.bo.signed_in_ms != null) Object.assign(out.people.bo, await play(bo, env.film, env.server));
  save();

  // Who each is, from the lender's Jellyfin, and each token's own account.
  const all = await users();
  out.jellyfin.users = all.map((u) => ({ id: u.Id, name: u.Name }));
  const nameOf = (id) => all.find((u) => u.Id === id)?.Name ?? null;
  for (const who of ['alex', 'bo']) {
    const t = out.people[who];
    t.jellyfin_name = nameOf(t.user_id);
    if (t.token) {
      const me = await fetch(`${env.jf}/Users/Me`, {
        headers: { authorization: `MediaBrowser Client="oidc-jellyfin run", Device="oidc-jellyfin-driver", DeviceId="oidc-jellyfin-driver", Version="1", Token="${t.token}"` },
      });
      t.me = me.ok ? (await me.json()).Name ?? null : null;
    }
  }
  out.bars.apart = {
    distinct_users: !!out.people.alex.user_id && out.people.alex.user_id !== out.people.bo.user_id,
    distinct_tokens: !!out.people.alex.token && out.people.alex.token !== out.people.bo.token,
    alex: out.people.alex.jellyfin_name,
    bo: out.people.bo.jellyfin_name,
    alex_me: out.people.alex.me,
    bo_me: out.people.bo.me,
  };
  save();

  // 3. Cy, a member Kim removes before Cy ever signs in.
  const before = logLines(env.kim_log);
  const beforeNames = all.map((u) => u.Name);
  const beforeAnswers = (() => {
    try { return [...fs.readFileSync(env.kim_log, 'utf8').matchAll(/oidc: the authorize path answered/g)].length; } catch { return null; }
  })();
  out.cy_before = { kim_log_lines: before, users: beforeNames, kim_authorize_answers: beforeAnswers };
  execFileSync(env.bin, ['remove', 'Cy', '--group', 'House'], { env: { ...process.env, SVRNGS_DIR: env.kim_dir, NO_COLOR: '1' } });
  out.cy_removed = true;
  cy = await person('chromium');
  const cyOpen = Date.now();
  try {
    await cy.page.goto(env.cy_at, { waitUntil: 'domcontentloaded', timeout: 60_000 });
  } catch (e) {
    out.people.cy = { stopped: `${String(e.message).split('\n')[0]}` };
  }
  out.people.cy = out.people.cy ?? {};
  out.people.cy.open_ms = Date.now() - cyOpen;
  await cy.page.waitForTimeout(1500);
  out.people.cy.words = await shown(cy.page);
  out.people.cy.url = cy.page.url();
  out.people.cy.sso_offered = await cy.page.locator('text=Sign in with SSO').count() > 0;
  out.people.cy.logs = cy.logs;
  const after = await users();
  out.people.cy.jellyfin_users_after = after.map((u) => u.Name);
  out.people.cy.new_users = after.map((u) => u.Name).filter((n) => !beforeNames.includes(n));
  out.people.cy.kim_log_lines_before = before;
  out.people.cy.kim_log_lines_after = logLines(env.kim_log);
  out.people.cy.kim_authorize_answers_after = (() => {
    try { return [...fs.readFileSync(env.kim_log, 'utf8').matchAll(/oidc: the authorize path answered/g)].length; } catch { return null; }
  })();
  save();

  // 4. A planted ask: Bo's browser names Alex in the ask, and the code must
  // still name Bo. The callback is intercepted before the app redeems it.
  const redirect = `http://localhost:${env.jf_port}/planted-callback`;
  const ask = `${env.bo_at}.svrngs/oidc/authorize?response_type=code&client_id=${encodeURIComponent(env.client_id)}` +
    `&redirect_uri=${encodeURIComponent(redirect)}&scope=openid%20profile&state=planted&nonce=n-planted&sub=alex`;
  const planted = { ask, forged: { key: 'a'.repeat(64), person: 'alex' } };
  let captured = null;
  await bo.context.route('**/planted-callback*', async (route) => {
    captured = route.request().url();
    await route.fulfill({ status: 200, contentType: 'text/html', body: '<!doctype html><title>planted</title>' });
  });
  await bo.context.setExtraHTTPHeaders({ 'x-svrngs-key': planted.forged.key, 'x-svrngs-person': planted.forged.person });
  try {
    const answer = await bo.page.goto(ask, { waitUntil: 'domcontentloaded', timeout: 60_000 });
    planted.status = answer?.status() ?? null;
    planted.url = bo.page.url();
    planted.body = (await bo.page.evaluate(() => document.body.innerText.slice(0, 300))).replace(/\s+/g, ' ');
  } catch (e) {
    planted.stopped = `${String(e.message).split('\n')[0]}`;
    planted.url = bo.page.url();
  }
  await bo.context.setExtraHTTPHeaders({});
  await bo.context.unroute('**/planted-callback*');
  planted.captured = captured;
  const callback = captured ?? planted.url;
  if (callback) {
    const code = new URL(callback).searchParams.get('code');
    planted.code = code;
    const redeem = async () => {
      const form = new URLSearchParams({ grant_type: 'authorization_code', code, redirect_uri: redirect,
        client_id: env.client_id, client_secret: env.client_secret });
      const r = await fetch(`${env.issuer}/token`, { method: 'POST', headers: { 'content-type': 'application/x-www-form-urlencoded' }, body: form });
      return { status: r.status, answer: await r.json().catch(() => null) };
    };
    const first = await redeem();
    planted.first_status = first.status;
    if (first.answer?.id_token) {
      const part = first.answer.id_token.split('.')[1];
      planted.claims = JSON.parse(Buffer.from(part.replace(/-/g, '+').replace(/_/g, '/'), 'base64').toString('utf8'));
    }
    const second = await redeem();
    planted.second_status = second.status;
    planted.second_error = second.answer?.error ?? null;
  }
  out.bars.planted = planted;
  out.people.bo.logs = bo.logs;
  save();
} catch (e) {
  out.stopped = `${String(e.message).split('\n')[0]}`;
  save();
} finally {
  for (const p of [alex, bo, cy]) if (p) await p.close().catch(() => {});
  save();
}
