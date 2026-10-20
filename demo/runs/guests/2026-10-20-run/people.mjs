// Each person's browser for run.py here (PREDICTIONS.md §1): Chromium for
// each computer's home page, and for each guest a browser that keeps
// nothing of any group, WebKit as an iPhone or Chromium (web/tests/page.mjs).
// One browser each, kept for the whole run, so what a guest's page keeps
// stays. Carried over from demo/runs/canon/2026-10-17-house/people.mjs,
// with a guest's commands. It prints `{"ready": true}`, then reads one
// JSON command a line on stdin and prints one JSON answer a line:
//
//   {"do": "open", "who", "engine"}             the person's browser
//   {"do": "visit", "who", "link", "name"}      a guest: open the link, give
//                                               the name if asked, read what
//                                               the page says and lists
//   {"do": "watch", "who", "lend"}              a guest: press the lend with
//                                               those words in what the page
//                                               lists, and play its film
//   {"do": "forge", "who", "token", "key", "name", "person"}
//                                               a guest: what the page never
//                                               offers, a pass signed with the
//                                               visit's own key for the lend
//                                               `name` on the computer `key`,
//                                               opened at its own address
//   {"do": "steps", "who", "steps": [...]}      each step in turn, on the
//                                               person's one page:
//     {"goto": url}                             open it
//     {"wait": re, "within"?: ms}               until the page's words match
//     {"poll": url, "re", "within"?, "every"?}  open it again until they do
//     {"click": words, "until"?: re}            press the button with those
//                                               words, then wait until the
//                                               words match `until`, else change
//     {"read": true}                            the page's words now
//
// Every answer says how long after the command began it ended and the
// page's words then; a step that fails ends the command, said with the
// words as they were. A page's errors are kept and answered with each
// command, and its console's last 60 lines.
//
//   node people.mjs
import { createInterface } from 'node:readline';
import { launch, play } from '../../../../web/tests/page.mjs';

const people = {};
const words = async (page) => {
  try {
    return await page.evaluate(() => document.body?.innerText ?? '');
  } catch {
    return '';
  }
};

/** Until `page`'s words match `re`, read every 200 ms through any navigation, or `within` ms pass. */
async function until(page, re, within = 30_000) {
  const t0 = Date.now();
  for (;;) {
    const text = await words(page);
    if (new RegExp(re).test(text)) return text;
    if (Date.now() - t0 > within) throw new Error(`the page never said ${re} within ${within} ms`);
    await new Promise((r) => setTimeout(r, 200));
  }
}

async function step(page, s) {
  if (s.goto) {
    await page.goto('about:blank');
    await page.goto(s.goto);
    return;
  }
  if (s.wait) return until(page, s.wait, s.within);
  if (s.poll) {
    const t0 = Date.now();
    for (;;) {
      await page.goto('about:blank');
      await page.goto(s.poll);
      const got = await until(page, s.re, Math.min(10_000, s.every ?? 10_000)).catch(() => null);
      if (got) return got;
      if (Date.now() - t0 > (s.within ?? 60_000)) throw new Error(`${s.poll} never said ${s.re} within ${s.within ?? 60_000} ms`);
    }
  }
  if (s.click) {
    const before = await words(page);
    await page.getByRole('button', { name: s.click, exact: true }).first().click({ timeout: 30_000 });
    if (s.until) return until(page, s.until, s.within);
    const t0 = Date.now();
    while ((await words(page)) === before) {
      if (Date.now() - t0 > 30_000) throw new Error(`nothing changed after pressing ${s.click}`);
      await new Promise((r) => setTimeout(r, 100));
    }
    return;
  }
  if (s.select) return page.locator(s.select).first().selectOption({ label: s.label }, { timeout: 30_000 });
  if (s.options) return page.locator(s.options).first().evaluate((sel) => ({ chosen: sel.selectedOptions[0]?.textContent ?? null, offered: [...sel.options].map((o) => o.textContent) }));
  if (s.text) return page.locator(s.text).first().textContent({ timeout: 30_000 });
  if (s.read) return;
  throw new Error(`no such step: ${JSON.stringify(s)}`);
}

// In the guest's page, at the page's own address: the pass app.js `open`
// signs, here with the visit's key for a lend the page never lists, handed
// to that lend's address as app.js hands it.
async function forged({ token, key: computer, name, person }) {
  const app = document.querySelector('script[type=module]').src;
  const v = new URL(app).search;
  const wasm = await import(new URL(`./wasm/svrngs_web.js${v}`, app).href);
  await wasm.default();
  const store = await import(new URL(`./store.js${v}`, app).href);
  const { key, until } = await store.visitOf(token);
  const label = wasm.label(computer, name);
  const holder = crypto.getRandomValues(new Uint8Array(32));
  const hex = (bytes) => [...bytes].map((b) => b.toString(16).padStart(2, '0')).join('');
  const relay = wasm.relay_of(token) ?? null;
  const lends = [{ label, key: computer, relay, name, kind: name, person }];
  const handoff = { label, lends, pass: wasm.pass(key, holder, label, [], until), holder: hex(holder), relay, page: `${location.origin}${location.pathname}`, unanswered: [] };
  const json = new TextEncoder().encode(JSON.stringify(handoff));
  const fragment = btoa(String.fromCharCode(...json)).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
  return `${store.lendOrigin(location.protocol, label, location.host)}/open.html${v}#${fragment}`;
}

async function act(c) {
  const t0 = Date.now();
  if (c.do === 'open') {
    const { context, close } = await launch(c.engine);
    const page = await context.newPage();
    const errors = [];
    const logs = [];
    page.on('pageerror', (e) => errors.push(String(e).slice(0, 400)));
    context.on('console', (m) => logs.push(`${m.type()}: ${m.text()}`.slice(0, 400)));
    people[c.who] = { close, page, errors, logs };
    return { ms: Date.now() - t0 };
  }
  const { page, errors, logs } = people[c.who];
  const answer = (got) => ({ ms: Date.now() - t0, ...got, url: page.url(), errors: errors.splice(0), logs: logs.splice(0).slice(-60) });
  if (c.do === 'visit') {
    await page.goto('about:blank');
    await page.goto(c.link);
    await page.waitForSelector('#status.good, #status.bad, #join:not([hidden])', { timeout: 120_000 });
    const asked = await page.locator('#join').isVisible();
    if (asked) {
      await page.locator('#name').fill(c.name);
      await page.locator('#join button').first().click();
    }
    await page.waitForSelector('#status.good, #status.bad', { timeout: 120_000 });
    const askedAgain = await page.locator('#join').isVisible();
    return answer({ asked, askedAgain, status: await page.locator('#status').textContent(), listed: await page.locator('#lent button').allTextContents(), text: await words(page) });
  }
  if (c.do === 'watch') {
    const away = (url) => url.hostname.endsWith('.localhost');
    await Promise.all([page.waitForURL(away, { timeout: 120_000 }), page.locator('#lent button', { hasText: c.lend }).first().click({ timeout: 30_000 })]);
    await page.waitForSelector('video', { timeout: 120_000 });
    const opened = Date.now() - t0;
    const played = await page.evaluate(play, { at: null, within: 60_000 });
    return answer({ opened, played, text: await words(page) });
  }
  if (c.do === 'forge') {
    const to = await page.evaluate(forged, c);
    await page.goto(to);
    // The lend's start page hands the pass to its worker and opens `/`,
    // which the lender's door answers, or refuses in words.
    await page.waitForURL((u) => !u.pathname.endsWith('open.html'), { timeout: 120_000 });
    const text = await until(page, '\\S', 120_000).catch(() => '');
    await new Promise((r) => setTimeout(r, 2_000));
    return answer({ text: await words(page), video: await page.locator('video').count() });
  }
  if (c.do === 'steps') {
    const done = [];
    for (const s of c.steps) {
      let value;
      try {
        value = await step(page, s);
      } catch (e) {
        done.push({ step: s, ms: Date.now() - t0, error: String(e).slice(0, 600), text: await words(page), url: page.url() });
        return answer({ done, failed: true });
      }
      done.push({ step: s, ms: Date.now() - t0, text: await words(page), url: page.url(), ...(value !== undefined && typeof value !== 'string' ? { value } : s.text ? { value } : {}) });
    }
    return answer({ done });
  }
  return { error: `no such command: ${c.do}` };
}

console.log(JSON.stringify({ ready: true }));
const input = createInterface({ input: process.stdin });
for await (const line of input) {
  if (!line.trim()) continue;
  const c = JSON.parse(line);
  let answer;
  try {
    answer = await act(c);
  } catch (e) {
    answer = { error: String(e).slice(0, 600), text: people[c.who] ? await words(people[c.who].page) : '', url: people[c.who]?.page.url() };
  }
  console.log(JSON.stringify(answer));
}
for (const p of Object.values(people)) await p.close();
