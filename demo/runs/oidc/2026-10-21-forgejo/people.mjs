// Forgejo through svrngs, as the row's five bars meet it
// (ralph/next/oidc/STATE.md, oidc-forgejo; SIGN_IN_WITH_SVRNGS §5a, §7 1).
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
const out = { at: new Date().toISOString(), people: {}, forgejo: {}, bars: {} };
const T0 = Date.now();
const now = () => Date.now() - T0;
const save = () => fs.writeFileSync(process.argv[3], JSON.stringify(out, null, 2));
const shown = (page) => page.evaluate(() => document.body.innerText.slice(0, 400).replace(/\s+/g, ' ')).catch(() => '?');

async function person(engine, { device } = {}) {
  const { context, close } = await launch(engine, { device });
  const page = await context.newPage();
  const logs = [];
  const navs = [];
  let localLoginPosts = 0;
  page.on('pageerror', (e) => logs.push(`pageerror: ${e}`));
  page.on('console', (m) => { if (m.type() === 'error') logs.push(`console: ${m.text().slice(0, 200)}`); });
  page.on('requestfailed', (r) => logs.push(`failed: ${r.url().slice(0, 150)}: ${r.failure()?.errorText}`));
  page.on('response', (r) => { if (r.status() >= 400) logs.push(`http ${r.status()} ${r.request().method()} ${r.url().slice(0, 160)}`); });
  page.on('framenavigated', (f) => { if (f === page.mainFrame()) navs.push(f.url()); });
  page.on('request', (r) => { if (r.method() === 'POST' && r.url().includes('/user/login')) localLoginPosts++; });
  return { context, close, page, logs, navs, posts: () => localLoginPosts };
}

// Forgejo's own answer for the browser's session: the login it made from
// the claims. A failed read is its status, never a guessed account.
async function me(p) {
  return p.page.evaluate(async () => {
    try {
      const r = await fetch('/api/v1/user', { headers: { accept: 'application/json' } });
      return r.ok ? await r.json() : { status: r.status };
    } catch (e) { return { error: String(e) }; }
  }).catch((e) => ({ error: String(e) }));
}

async function open(p, at, { link, name } = {}) {
  const t = {};
  try {
    if (link) {
      const opened = Date.now();
      await p.page.goto(link, { waitUntil: 'domcontentloaded', timeout: 90_000 });
      await p.page.waitForFunction(
        () => !document.getElementById('join')?.hidden || /good|bad/.test(document.getElementById('status')?.className ?? ''),
        null, { timeout: 120_000 });
      if (name) { await p.page.fill('#name', name); await p.page.click('#join button'); }
      await p.page.waitForSelector('#status.good', { timeout: 120_000 });
      t.joined_ms = Date.now() - opened;
      t.listed = await p.page.evaluate(() => [...document.querySelectorAll('#lent button')].map((b) => b.textContent));
      await p.page.locator('#lent button', { hasText: 'forgejo' }).first().click();
      t.lend_ms = Date.now() - opened;
    } else {
      const opened = Date.now();
      await p.page.goto(at, { waitUntil: 'domcontentloaded', timeout: 90_000 });
      t.lend_ms = Date.now() - opened;
    }
    t.lend_origin = new URL(p.page.url()).origin;
    // Forgejo's login page, with the lender's OIDC source on it.
    const button = p.page.locator('a[href*="/user/oauth2/svrngs"], button[formaction*="/user/oauth2/svrngs"]').first();
    const offered = Date.now();
    await button.waitFor({ timeout: 60_000 });
    t.login_page_ms = Date.now() - offered;
    t.sso_offered = true;
    t.form_visible = await p.page.locator('#user_name:visible, input[name="user_name"]:visible').count() > 0;
    t.typed = false;
    const pressed = Date.now();
    await button.click();
    for (let i = 0; i < 300; i++) {
      const who = await me(p);
      if (who?.login) { t.signed_in_ms = Date.now() - pressed; t.user = who; break; }
      await p.page.waitForTimeout(100);
    }
    if (t.signed_in_ms == null) t.stopped = `no signed-in answer; the page says: ${await shown(p.page)}`;
    t.url = p.page.url();
    t.stayed = new URL(t.url).origin === t.lend_origin;
    t.page = await shown(p.page);
  } catch (e) {
    t.stopped = `${String(e.message).split('\n')[0]}; the page says: ${await shown(p.page)}`;
    t.url = p.page.url();
    t.lend_origin = t.lend_origin ?? (() => { try { return new URL(p.page.url()).origin; } catch { return null; } })();
    t.stayed = t.url && t.lend_origin ? new URL(t.url).origin === t.lend_origin : false;
  }
  t.navigations = p.navs;
  t.local_login_posts = p.posts();
  return t;
}

const adminUsers = async () => {
  const r = await fetch(`${env.fj}/api/v1/admin/users?limit=50`, {
    headers: { authorization: `token ${env.admin_token}`, accept: 'application/json' },
  });
  return r.ok ? (await r.json()).map((u) => ({ id: u.id, login: u.login, email: u.email })) : { status: r.status };
};
const logAnswers = (path) => {
  try { return [...fs.readFileSync(path, 'utf8').matchAll(/oidc: the authorize path answered/g)].length; } catch { return null; }
};

let alex, bo, cy;
try {
  // 1. Alex, another computer, Chromium, at his own computer's address.
  alex = await person('chromium');
  out.people.alex = await open(alex, env.alex_at);
  out.people.alex.engine = `chromium ${alex.context.browser()?.version() ?? ''}`.trim();
  out.people.alex.logs = alex.logs;
  save();

  // 2. Bo, a phone, WebKit as an iPhone, through the group's own page.
  bo = await person('webkit');
  out.people.bo = await open(bo, null, { link: env.bo_link });
  out.people.bo.engine = `webkit ${bo.context.browser()?.version() ?? ''}`.trim();
  out.people.bo.logs = bo.logs;
  save();

  // Who each is, from Forgejo's own admin list, and each session's answer.
  const all = await adminUsers();
  out.forgejo.users = all;
  out.bars.apart = {
    distinct_users: !!out.people.alex.user?.id && out.people.alex.user.id !== out.people.bo.user?.id,
    distinct_logins: !!out.people.alex.user?.login && out.people.alex.user.login !== out.people.bo.user?.login,
    alex: out.people.alex.user?.login ?? null,
    bo: out.people.bo.user?.login ?? null,
  };
  save();

  // 3. Cy, a member Kim removes before Cy ever signs in.
  const before = all;
  const beforeAnswers = logAnswers(env.kim_log);
  out.cy_before = { users: before, kim_authorize_answers: beforeAnswers };
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
  out.people.cy.sso_offered = await cy.page.locator('a[href*="/user/oauth2/svrngs"]').count() > 0;
  out.people.cy.logs = cy.logs;
  const after = await adminUsers();
  out.people.cy.forgejo_users_after = after;
  out.people.cy.new_users = Array.isArray(after) && Array.isArray(before)
    ? after.filter((u) => !before.some((b) => b.id === u.id)).map((u) => u.login)
    : null;
  out.people.cy.kim_authorize_answers_after = logAnswers(env.kim_log);
  save();

  // 4. A planted ask: Bo's browser names Alex in the ask, and the code must
  // still name Bo. The callback is intercepted before the app redeems it.
  const redirect = `http://localhost:${env.fj_port}/planted-callback`;
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
