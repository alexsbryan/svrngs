// Each person's browser for run.py here (PREDICTIONS.md): a computer's is
// Chromium keeping no group, its clipboard allowed; a phone's WebKit as an
// iPhone (web/tests/page.mjs). One browser each for the whole take, so what
// a phone keeps, its worker and its held dial, stays. events-run's
// people.mjs (../2026-10-14/) with the link's three steps added. Prints
// `{"ready": true}`, then reads one JSON command a line and prints one JSON
// answer a line, every page saved as SHOTS/<shot>.png:
//
//   {"do": "open", "who", "engine"}         the person's browser
//   {"do": "join", "who", "link", "name"}   a phone opens its link, and types
//                                           its name if asked
//   {"do": "goto", "who", "url"}            open an address on a lend's origin
//   {"do": "home", "who", "url", "words"}   open a home page; wait for any of
//                                           `words`
//   {"do": "press", "who", "button", "fill", "check"}
//                                           fill, tick, press the button named
//                                           so; `absent` when there is none
//   {"do": "copy", "who", "group"}          in `group`'s part, press "Copy the
//                                           link to this event"; what it said,
//                                           the box, the clipboard
//   {"do": "link", "who", "url"}            open a link as from a chat: until
//                                           the lend's page is shown, or the
//                                           group's page says what it says
//   {"do": "paste", "who", "home", "link"}  on the home page, paste `link`
//                                           under "Open a link from your
//                                           group" and press "Open it"; the
//                                           tab it opens, from then the
//                                           person's page
//
// `ms` is from the command's start, or for paste from the press, to the
// page shown or the words saying it was not.
//
//   node people.mjs
import { createInterface } from 'node:readline';
import { launch } from '../../../../web/tests/page.mjs';

const people = {};
const COPY = 'Copy the link to this event';

// A page shown: an event's or the app's own words, or svrngs's in its place.
const shown = () => {
  const t = (document.body?.innerText ?? '').trim();
  return Boolean(document.querySelector('h1, p.unread')) || (t.length > 0 && !/^Opening/.test(t) && !document.getElementById('status'));
};

async function view(who, extra) {
  const { page, responses } = people[who];
  const url = page.url();
  const res = responses.get(url.split('#')[0]);
  const shownNow = await page.evaluate(() => ({
    text: (document.body?.innerText ?? '').slice(0, 3000),
    title: document.title,
    sections: [...document.querySelectorAll('section[data-group]')].map((s) => ({
      group: s.dataset.group,
      name: s.querySelector('h2')?.textContent ?? null,
      invited: [...s.querySelectorAll('.invited li')].map((l) => l.textContent),
      coming: [...s.querySelectorAll('.coming li')].map((l) => l.textContent),
      came: [...s.querySelectorAll('.came li')].map((l) => l.textContent),
      link: s.querySelector('input.link')?.value ?? null,
      nolink: s.querySelector('.nolink')?.textContent ?? null,
    })),
    buttons: [...document.querySelectorAll('button')].map((b) => b.textContent),
  })).catch((e) => ({ text: null, error: String(e).slice(0, 300) }));
  return { status: res?.status ?? null, refused: res?.refused ?? null, url, ...shownNow, ...extra };
}

// Poll `done` in whatever document the page holds, across its navigations.
async function until(page, done, arg = null, bound = 120_000) {
  const t0 = Date.now();
  while (Date.now() - t0 < bound) {
    if (await page.evaluate(done, arg).catch(() => false)) return true;
    await page.waitForTimeout(50);
  }
  return false;
}

async function act(c) {
  const t0 = Date.now();
  if (c.do === 'open') {
    const { context, close } = await launch(c.engine);
    if (c.engine === 'chromium') await context.grantPermissions(['clipboard-read', 'clipboard-write']);
    const responses = new Map();
    context.on('response', (r) => responses.set(r.url().split('#')[0], { status: r.status(), refused: r.headers()['x-svrngs-refused'] ?? null }));
    const page = await context.newPage();
    people[c.who] = { context, close, page, responses, engine: c.engine };
    return { ms: Date.now() - t0 };
  }
  const person = people[c.who];
  const { page } = person;
  if (c.do === 'join') {
    await page.goto(c.link);
    await page.waitForFunction(
      () => !document.getElementById('join').hidden || /good|bad/.test(document.getElementById('status').className),
      null, { timeout: 90_000 });
    if (await page.evaluate(() => !document.getElementById('join').hidden)) {
      await page.fill('#name', c.name);
      await page.click('#join button');
    }
    await page.waitForSelector('#status.good, #status.bad', { timeout: 90_000 });
    return { ms: Date.now() - t0, status: await page.locator('#status').textContent() };
  }
  if (c.do === 'goto') {
    await page.goto('about:blank');
    const res = await page.goto(c.url, { timeout: 120_000 }).catch(() => null);
    await until(page, shown);
    return view(c.who, { ms: Date.now() - t0, ...(res && { status: res.status(), refused: res.headers()['x-svrngs-refused'] ?? null }) });
  }
  if (c.do === 'home') {
    await page.goto(c.url, { timeout: 120_000 });
    const met = await until(page, (words) => words.some((w) => (document.body?.innerText ?? '').includes(w)), c.words, 60_000);
    return view(c.who, { ms: Date.now() - t0, met });
  }
  if (c.do === 'press') {
    for (const [sel, value] of Object.entries(c.fill ?? {})) await page.fill(sel, value);
    for (const sel of c.check ?? []) await page.check(sel);
    const button = page.getByRole('button', { name: c.button, exact: true });
    if (!(await button.count())) return { absent: true, ms: Date.now() - t0, buttons: await page.$$eval('button', (bs) => bs.map((b) => b.textContent)) };
    await Promise.all([page.waitForNavigation({ timeout: 90_000 }), button.first().click()]);
    await until(page, shown, null, 90_000);
    return view(c.who, { ms: Date.now() - t0 });
  }
  if (c.do === 'copy') {
    const part = page.locator(`section[data-group="${c.group}"]`);
    await part.getByRole('button', { name: COPY, exact: true }).click();
    await until(page, ([g, w]) => document.querySelector(`section[data-group="${g}"] input.link + button`)?.textContent !== w, [c.group, COPY], 10_000);
    const box = await part.locator('input.link').inputValue();
    const said = await part.locator('input.link + button').textContent();
    const clipboard = person.engine === 'chromium' ? await page.evaluate(() => navigator.clipboard.readText()).catch((e) => `unread: ${e}`) : null;
    return { ms: Date.now() - t0, box, said, clipboard };
  }
  if (c.do === 'link') {
    const home = new URL(c.url).host;
    await page.goto('about:blank');
    await page.goto(c.url);
    const met = await until(page, (h) => {
      if (location.host !== h) return location.hostname.endsWith('.localhost') && !location.pathname.endsWith('/open.html') &&
        (Boolean(document.querySelector('h1, p.unread')) || ((document.body?.innerText ?? '').trim().length > 0 && !document.getElementById('status')));
      const s = document.getElementById('status');
      return s?.className === 'bad' || document.getElementById('computer')?.hidden === false;
    }, home);
    const there = new URL(page.url()).host === home;
    const said = there ? await page.evaluate(() => ({
      status: document.getElementById('status')?.textContent ?? null,
      line: document.getElementById('computer')?.hidden === false ? document.getElementById('line')?.textContent ?? null : null,
    })) : null;
    return view(c.who, { ms: Date.now() - t0, met, landed: !there, said });
  }
  if (c.do === 'paste') {
    await page.goto(c.home, { timeout: 120_000 });
    const box = page.locator('#open input');
    await box.waitFor({ timeout: 30_000 });
    await box.fill(c.link);
    const t1 = Date.now();
    const [tab] = await Promise.all([person.context.waitForEvent('page'), page.locator('#open button').click()]);
    await tab.waitForLoadState('domcontentloaded', { timeout: 120_000 }).catch(() => {});
    const met = await until(tab, shown);
    const ms = Date.now() - t1;
    await page.close();
    person.page = tab;
    return view(c.who, { ms, met });
  }
  return { error: `no such command: ${c.do}` };
}

console.log(JSON.stringify({ ready: true }));
for await (const line of createInterface({ input: process.stdin })) {
  if (!line.trim()) continue;
  let answer;
  const c = JSON.parse(line);
  try {
    answer = await act(c);
    if (process.env.SHOTS && c.shot && people[c.who]) {
      await people[c.who].page.screenshot({ path: `${process.env.SHOTS}/${c.shot}.png`, fullPage: true }).catch(() => {});
    }
  } catch (e) {
    answer = { error: String(e).slice(0, 600) };
  }
  console.log(JSON.stringify(answer));
}
for (const p of Object.values(people)) await p.close();
