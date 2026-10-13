// The story's browsers (run.py): a computer in Chromium or WebKit, a phone
// in WebKit as an iPhone (web/tests/page.mjs). The phone's join is
// demo/runs/c/dry-2026-10-12-rerun/c.mjs's; llama-server's chat page is
// read by what it showed in a probe of it: its box's placeholder, "Type a
// message...", Enter to send, each answer a `.chat-message-assistant`.
//
//   node story.mjs page   <json>   a page's words once it has settled
//   node story.mjs line   <json>   the group's page on a computer: the line it shows
//   node story.mjs chat   <json>   a home page until it lists `lend`; it clicked; the
//                                  question asked in the tab it opens
//   node story.mjs copy   <json>   a home page's "Copy its address for a program"
//                                  pressed, and what Chromium's clipboard holds
//   node story.mjs phones <json>   Cy, Dee and Eve on phones: Cy's chat alone, then
//                                  the three at once with no limit, then with 2
//                                  set on Sam's home page
//
// Prints one JSON line. Every time is ms from `t0` (else from the start); a
// step that did not happen within its time is null with what the page said,
// never a number guessed.
import { spawn } from 'node:child_process';
import { chromium, webkit } from '../../../../web/node_modules/playwright/index.mjs';
import { launch } from '../../../../web/tests/page.mjs';

const [mode, raw] = process.argv.slice(2);
const a = JSON.parse(raw ?? '{}');
const T0 = a.t0 ?? Date.now();
const at = () => Date.now() - T0;
const out = { mode };
const words = (page) => page.evaluate(() => document.body?.innerText.slice(0, 600).replace(/\s+/g, ' ') ?? '').catch(() => '?');
const shot = (page, name) => (a.shots ? page.screenshot({ path: `${a.shots}/${name}.png`, fullPage: true }).catch(() => {}) : null);
const browsers = [];

async function until(fn, ms, every = 250) {
  const end = Date.now() + ms;
  for (;;) {
    const v = await fn().catch(() => null);
    if (v) return v;
    if (Date.now() > end) return null;
    await new Promise((r) => setTimeout(r, every));
  }
}

// Every answer of 400 or more a page was given, and its page errors.
function watched(page) {
  const seen = [];
  page.on('pageerror', (e) => seen.push(`pageerror: ${e}`));
  page.on('response', (r) => {
    if (r.status() >= 400) seen.push(`${at()} ${r.status()} ${r.request().method()} ${r.url().slice(0, 140)} ${r.headers()['x-svrngs-refused'] ?? ''}`);
  });
  return seen;
}

// A computer's browser: Chromium, or WebKit as a desktop; Chromium may read
// the clipboard.
async function computer(engine) {
  const browser = await (engine === 'webkit' ? webkit : chromium).launch();
  browsers.push(browser);
  const context = await browser.newContext({ viewport: { width: 1100, height: 900 } });
  if (engine === 'chromium') await context.grantPermissions(['clipboard-read', 'clipboard-write']).catch(() => {});
  return context;
}

// In llama-server's chat page open in `page`: `question` typed and sent;
// when its answer's first words showed, when the answer stopped growing
// for 3 s, and its words. Before it was sent, what the page said if it has
// no box to type in.
async function ask(page, question, { within = 120_000 } = {}) {
  const r = {};
  const box = page.getByPlaceholder('Type a message...');
  r.box = await box.waitFor({ timeout: 30_000 }).then(() => at(), () => null);
  if (r.box === null) return { ...r, stopped: `no box to type in: ${await words(page)}` };
  const before = await page.locator('.chat-message-assistant').count();
  await box.fill(question);
  await box.press('Enter');
  r.sent = at();
  const answer = () => page.locator('.chat-message-assistant').nth(before).innerText().catch(() => '');
  r.first = await until(async () => (await answer()).trim().length > 0 && at(), within, 100);
  if (r.first === null) {
    r.stopped = `no answer within ${within} ms: ${await words(page)}`;
    return r;
  }
  let last = '', still = Date.now();
  const end = Date.now() + within;
  while (Date.now() < end) {
    const now = await answer();
    if (now !== last) (last = now), (still = Date.now());
    else if (Date.now() - still > 3000) break;
    await new Promise((res) => setTimeout(res, 200));
  }
  r.done = at() - 3000;
  r.words = last.slice(0, 600);
  return r;
}

// A home page at `url`, loaded again until it has a link whose words are
// `link`: when.
async function listed(page, url, link, ms = 60_000) {
  const end = Date.now() + ms;
  for (;;) {
    await page.goto(url).catch(() => {});
    const ok = await page.getByRole('link', { name: link }).first().waitFor({ timeout: 5_000 }).then(() => true, () => false);
    if (ok) return at();
    if (Date.now() > end) return null;
  }
}

// A phone: the link opened, the name asked typed, in; when each happened.
async function phone(name, link) {
  const { context, close } = await launch('webkit');
  browsers.push({ close });
  const page = await context.newPage();
  const r = { name, seen: watched(page) };
  r.opened = at();
  await page.goto(link);
  await page.waitForFunction(
    () => !document.getElementById('join').hidden || /good|bad/.test(document.getElementById('status').className),
    null, { timeout: 90_000 });
  if (await page.evaluate(() => !document.getElementById('join').hidden)) {
    r.asked = at();
    await page.fill('#name', name);
    await page.click('#join button');
  }
  await page.waitForSelector('#status.good, #status.bad', { timeout: 90_000 });
  r.status = { at: at(), said: await page.locator('#status').textContent() };
  return { page, r };
}

// On the group's page open in `page`: the lend whose button starts with
// `lend` pressed once it is listed, and the chat page through it.
async function openLend(page, lend, r) {
  r.lent = await until(async () => {
    const lent = await page.evaluate(() => [...document.querySelectorAll('#lent button')].map((b) => b.textContent));
    return lent.some((l) => l.startsWith(lend)) && lent;
  }, 90_000, 500);
  r.listed = r.lent ? at() : null;
  if (!r.lent) return (r.stopped = `${lend} not listed: ${await words(page)}`);
  const origin = new URL(page.url()).hostname;
  r.pressed = at();
  await page.locator('#lent button', { hasText: lend }).first().click();
  await page.waitForURL((u) => u.hostname !== origin && !u.pathname.endsWith('open.html'), { timeout: 90_000 }).catch(() => {});
  r.at_lend = { at: at(), url: page.url() };
  await page.waitForLoadState('networkidle', { timeout: 30_000 }).catch(() => {});
  r.page = { at: at(), title: await page.title().catch(() => null), said: await words(page) };
}

// Bo's program, once, as run.py runs it: started here so it asks while the
// phones' chats run.
function program(args) {
  return new Promise((done) => {
    const p = spawn(a.py, [a.program, ...args], { env: { ...process.env, OPENAI_API_KEY: '' } });
    let text = '';
    p.stdout.on('data', (d) => (text += d));
    p.on('close', () => {
      try { done(JSON.parse(text.trim().split('\n').pop())); } catch { done({ stopped: `printed nothing: ${text.slice(-300)}` }); }
    });
  });
}

try {
  if (mode === 'page') {
    const page = await (await computer('chromium')).newPage();
    out.seen = watched(page);
    await page.goto(a.url);
    await page.waitForFunction(() => !/Opening/.test(document.getElementById('status')?.textContent ?? ''), null, { timeout: 30_000 }).catch(() => {});
    out.at = at();
    out.said = await words(page);
    out.line = await page.locator('#line').textContent().catch(() => null);
    await shot(page, a.name);
  } else if (mode === 'line') {
    const page = await (await computer('chromium')).newPage();
    await page.goto(a.link);
    await page.waitForFunction(() => document.getElementById('line')?.textContent?.startsWith('curl '), null, { timeout: 60_000 });
    out.shown = at();
    out.line = await page.locator('#line').textContent();
    out.said = await words(page);
    await shot(page, 'bo-line');
  } else if (mode === 'chat') {
    const context = await computer(a.engine);
    const page = await context.newPage();
    out.seen = watched(page);
    out.listed = await listed(page, a.home, a.link);
    if (out.listed === null) throw new Error(`the home page never listed ${a.link}: ${await words(page)}`);
    await shot(page, `bo-home-${a.engine}`);
    const tab = context.waitForEvent('page', { timeout: 15_000 });
    out.clicked = at();
    await page.getByRole('link', { name: a.link }).first().click();
    const chat = await tab;
    out.tab_seen = watched(chat);
    await chat.waitForLoadState('domcontentloaded');
    out.tab = { at: at(), url: chat.url(), title: await chat.title().catch(() => null) };
    out.ask = await ask(chat, a.question);
    await shot(chat, `bo-chat-${a.engine}`);
  } else if (mode === 'copy') {
    const context = await computer('chromium');
    const page = await context.newPage();
    out.listed = await listed(page, a.home, a.link);
    const item = page.locator('li', { has: page.getByRole('link', { name: a.link }) }).first();
    out.button = await item.getByRole('button').first().innerText();
    out.pressed = at();
    await item.getByRole('button').first().click();
    await page.waitForTimeout(500);
    out.said = await item.innerText();
    out.clipboard = await page.evaluate(() => navigator.clipboard.readText()).catch((e) => `unread: ${e}`);
    await shot(page, 'bo-copy');
  } else if (mode === 'phones') {
    // Cy alone: in, the chat page through the group's page, the question.
    const cy = await phone('Cy', a.link);
    await openLend(cy.page, a.lend, cy.r);
    cy.r.ask = await ask(cy.page, a.question);
    await shot(cy.page, 'cy-chat');
    out.cy = cy.r;
    // Dee and Eve, in the same way, each on the chat page.
    const phones = [cy];
    for (const name of ['Dee', 'Eve']) {
      const p = await phone(name, a.link);
      await openLend(p.page, a.lend, p.r);
      phones.push(p);
    }
    out.dee = phones[1].r;
    out.eve = phones[2].r;
    // The three at once, no limit.
    out.none = { sent: at() };
    const asks = await Promise.all(phones.map((p) => ask(p.page, a.long, { within: 180_000 })));
    out.none.each = Object.fromEntries(phones.map((p, i) => [p.r.name, asks[i]]));
    await Promise.all(phones.map((p) => shot(p.page, `none-${p.r.name}`)));
    // Eve's chat page closed; the door's window let pass.
    await phones[2].page.goto('about:blank');
    out.window = { from: at() };
    await new Promise((r) => setTimeout(r, a.window_ms));
    // Sam sets the limit on Sam's home page.
    const sam = await (await computer('chromium')).newPage();
    await sam.goto(a.sam_home);
    const item = sam.locator('#lent li', { hasText: a.lend_name });
    await item.waitFor({ timeout: 30_000 });
    out.limit = { before: await item.innerText() };
    await item.locator('input[type=number]').fill('2');
    out.limit.pressed = at();
    await item.getByRole('button', { name: 'Let in this many' }).click();
    await sam.waitForLoadState('load');
    await sam.locator('#lent li', { hasText: a.lend_name }).waitFor({ timeout: 30_000 });
    await sam.waitForTimeout(500);
    out.limit.after = await sam.locator('#lent li', { hasText: a.lend_name }).innerText();
    out.limit.at = at();
    await shot(sam, 'sam-limit');
    // Cy and Dee ask; Bo's program beside them; then Eve opens it again.
    out.two = { sent: at() };
    const two = Promise.all(phones.slice(0, 2).map((p) => ask(p.page, a.long, { within: 180_000 })));
    const bo = program([a.address, 'once']);
    await new Promise((r) => setTimeout(r, 1500));
    const eve = phones[2];
    const r = (out.two.eve = { opened: at(), seen_from: eve.r.seen.length });
    await eve.page.goto(new URL(a.link).origin + '/');
    await eve.page.waitForSelector('#status.good, #status.bad', { timeout: 90_000 }).catch(() => {});
    await openLend(eve.page, a.lend, r);
    r.box = await eve.page.getByPlaceholder('Type a message...').count();
    if (r.box) r.ask = await ask(eve.page, a.long, { within: 60_000 });
    r.said = await words(eve.page);
    r.seen = eve.r.seen.slice(r.seen_from);
    await shot(eve.page, 'two-Eve');
    const [cyTwo, deeTwo] = await two;
    out.two.Cy = cyTwo;
    out.two.Dee = deeTwo;
    out.two.bo = await bo;
    await Promise.all(phones.slice(0, 2).map((p) => shot(p.page, `two-${p.r.name}`)));
    for (const p of phones) p.r.seen = p.r.seen.slice(-40);
  } else {
    throw new Error(`no mode ${mode}`);
  }
} catch (e) {
  out.stopped = `${String(e).split('\n')[0]}`;
} finally {
  for (const b of browsers) await b.close().catch(() => {});
  console.log(JSON.stringify(out));
}
