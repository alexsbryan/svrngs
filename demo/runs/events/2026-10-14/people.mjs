// Each person's browser for run.py here (PREDICTIONS.md §1): a computer's
// is Chromium, a phone's WebKit as an iPhone (web/tests/page.mjs), one
// browser each, kept for the whole run so what a phone keeps stays. It
// prints `{"ready": true}`, then reads one JSON command a line on stdin and
// prints one JSON answer a line:
//
//   {"do": "open", "who", "engine"}          the person's browser
//   {"do": "join", "who", "link", "name"}    a phone opens its link, and
//                                            types its name if asked
//   {"do": "goto", "who", "url"}             open an address; on a phone at
//                                            `#open=`, wait for the lend's page
//   {"do": "press", "who", "button", "fill"} fill the fields `fill` names,
//                                            press the button named so;
//                                            `absent` when there is none
//
// Every answer about a page is `view`: its status, address, the words on
// it, each group's section as lists, and the buttons it offers.
//
//   node people.mjs
import { createInterface } from 'node:readline';
import { launch } from '../../../../web/tests/page.mjs';

const people = {};

async function view(who, res, t0) {
  const { page } = people[who];
  const shown = await page.evaluate(() => ({
    text: (document.body?.innerText ?? '').slice(0, 3000),
    title: document.title,
    sections: [...document.querySelectorAll('section[data-group]')].map((s) => ({
      group: s.dataset.group,
      name: s.querySelector('h2')?.textContent ?? null,
      invited: [...s.querySelectorAll('.invited li')].map((l) => l.textContent),
      coming: [...s.querySelectorAll('.coming li')].map((l) => l.textContent),
      came: [...s.querySelectorAll('.came li')].map((l) => l.textContent),
      unread: s.querySelector('.unread')?.textContent ?? null,
    })),
    buttons: [...document.querySelectorAll('button')].map((b) => b.textContent),
  }));
  return {
    ms: Date.now() - t0,
    status: res ? res.status() : null,
    refused: res ? (res.headers()['x-svrngs-refused'] ?? null) : null,
    url: page.url(),
    ...shown,
  };
}

// A page shown: an event's or the app's own words, or svrngs's in its place.
const shown = () => {
  const t = (document.body?.innerText ?? '').trim();
  return Boolean(document.querySelector('h1, p.unread')) || (t.length > 0 && !/^Opening/.test(t) && !document.getElementById('status'));
};

async function act(c) {
  const t0 = Date.now();
  if (c.do === 'open') {
    const { context, close } = await launch(c.engine);
    const page = await context.newPage();
    const logs = [];
    page.on('console', (m) => logs.push(`${m.type()}: ${m.text()}`.slice(0, 300)));
    people[c.who] = { context, close, page, logs, engine: c.engine };
    return { ms: Date.now() - t0 };
  }
  const { page } = people[c.who];
  if (c.do === 'join') {
    await page.goto(c.link);
    await page.waitForFunction(
      () => !document.getElementById('join').hidden || /good|bad/.test(document.getElementById('status').className),
      null, { timeout: 90_000 });
    const asked = await page.evaluate(() => !document.getElementById('join').hidden);
    if (asked) {
      await page.fill('#name', c.name);
      await page.click('#join button');
    }
    await page.waitForSelector('#status.good, #status.bad', { timeout: 90_000 });
    return { ms: Date.now() - t0, asked, status: await page.locator('#status').textContent() };
  }
  if (c.do === 'goto') {
    if (/#open=/.test(c.url)) {
      // The group's page signs a pass and steps aside to the lend's origin.
      const home = new URL(c.url).host;
      await page.goto('about:blank');
      await page.goto(c.url);
      await page.waitForURL((u) => u.hostname.endsWith('.localhost') && u.host !== home, { timeout: 90_000 });
      await page.waitForFunction(shown, null, { timeout: 90_000 });
      return view(c.who, null, t0);
    }
    const res = await page.goto(c.url, { timeout: 90_000 });
    if (people[c.who].engine === 'webkit') await page.waitForFunction(shown, null, { timeout: 90_000 }).catch(() => {});
    return view(c.who, res, t0);
  }
  if (c.do === 'press') {
    for (const [sel, value] of Object.entries(c.fill ?? {})) await page.fill(sel, value);
    for (const sel of c.check ?? []) await page.check(sel);
    const button = page.getByRole('button', { name: c.button, exact: true });
    if (!(await button.count())) return { absent: true, ms: Date.now() - t0, buttons: await page.$$eval('button', (bs) => bs.map((b) => b.textContent)) };
    const [res] = await Promise.all([page.waitForNavigation({ timeout: 90_000 }), button.first().click()]);
    await page.waitForFunction(shown, null, { timeout: 90_000 }).catch(() => {});
    return view(c.who, res, t0);
  }
  return { error: `no such command: ${c.do}` };
}

console.log(JSON.stringify({ ready: true }));
const input = createInterface({ input: process.stdin });
for await (const line of input) {
  if (!line.trim()) continue;
  let answer;
  const c = JSON.parse(line);
  try {
    answer = await act(c);
  } catch (e) {
    answer = { error: String(e).slice(0, 600), logs: (people[c.who]?.logs ?? []).slice(-15) };
  }
  console.log(JSON.stringify(answer));
}
for (const p of Object.values(people)) await p.close();
