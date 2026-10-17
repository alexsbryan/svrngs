// Every person's browser for run.py here (PREDICTIONS.md): a computer is
// Chromium as a desktop, a phone WebKit as an iPhone (web/tests/page.mjs),
// a browser each. Prints `{"ready": true}`, then reads one JSON command a
// line on stdin and prints one JSON answer a line. Each answer says what
// the person was shown — the page's words, and whether it held a password
// box or Jellyfin's sign-in — and how long each step took, in ms; a step
// that did not happen in time says where it stopped, never a number.
//
//   {"do": "open", "who", "as": "computer"|"phone"}   their browser
//   {"do": "link", "who", "url"}       a computer: the chat's link, its line read
//   {"do": "home", "who", "url"}       a computer: the home page, the door and index once judged
//   {"do": "bring", "who"}             the home page's "Bring the ticked ones"
//   {"do": "lend", "who", "kind"}      the home page's Look, then "Lend <kind>"
//   {"do": "join", "who", "url", "name"}  a phone: the link, the name typed, Join
//   {"do": "group", "who", "url"}      a phone: the group's page, the door and index
//   {"do": "money", "who", "add"?, "read"?}  Money from the index: added, read, or in on Gas bill and paid until square
//   {"do": "library", "who", "film"}   Priya's jellyfin from the index: the film's card, Play, playing
//   {"do": "shot", "who", "name"}      the page in front now, to SHOTS/<name>.png
//
//   SHOTS=<dir> node people.mjs
import { createInterface } from 'node:readline';
import { launch } from '../../../../web/tests/page.mjs';

const people = {};
const SIGN_IN = '#txtManualName, .manualLoginForm, .btnManual';

// In a page: what it shows, and whether it asks for a password or a
// Jellyfin sign-in, visible.
function seen(sel) {
  const visible = (e) => !!(e.offsetWidth || e.offsetHeight || e.getClientRects().length);
  return {
    url: location.href,
    words: (document.body?.innerText ?? '').trim(),
    password: [...document.querySelectorAll('input[type=password]')].filter(visible).length,
    sign_in: [...document.querySelectorAll(sel)].filter(visible).length,
  };
}
const look = (page) => page.evaluate(seen, SIGN_IN).catch((e) => ({ error: String(e).split('\n')[0] }));

async function timed(fn) {
  const t0 = Date.now();
  try {
    return { ms: (await fn()) ?? Date.now() - t0 };
  } catch (e) {
    return { ms: null, stopped: String(e.message ?? e).split('\n')[0] };
  }
}

// The door's cells and the index's rows as drawn, on the home page or
// the group's page.
const drawn = (page) =>
  page.evaluate(() => ({
    door: [...document.querySelectorAll('ul.door li')].map((li) => ({ name: li.querySelector('strong')?.textContent, drawn: li.className, says: li.querySelector('small')?.textContent })),
    index: [...document.querySelectorAll('table.index tr')].map((tr) => [...tr.children].map((td) => td.innerText.trim())),
  }));

// The index's row whose "What it is" cell is `kind` (and whose "Whose" is
// `person`, when given): its link on a computer, its button on a phone.
const rowOf = (page, kind, person) =>
  page.locator('table.index tr').filter({ has: page.locator('td:nth-child(2)', { hasText: new RegExp(`^${kind}$`) }) })
    .filter(person ? { has: page.locator('td:nth-child(3)', { hasText: new RegExp(`^${person}$`) }) } : {}).first();

// Press what opens `kind` in the index: a computer's opens a tab, whose
// page is answered; a phone's opens in this tab.
async function press(p, kind, person) {
  const at = rowOf(p.page, kind, person).locator('a, button').first();
  await at.waitFor({ timeout: 30_000 });
  if (p.as === 'phone') {
    await at.click();
    return p.page;
  }
  const [tab] = await Promise.all([p.context.waitForEvent('page', { timeout: 30_000 }), at.click()]);
  p.tabs.push(tab);
  return tab;
}

async function act(c) {
  const p = people[c.who];
  if (c.do === 'open') {
    const { context, close } = await launch(c.as === 'phone' ? 'webkit' : 'chromium');
    people[c.who] = { as: c.as, context, close, page: await context.newPage(), tabs: [] };
    return { ok: true };
  }
  if (c.do === 'link') {
    const r = await timed(async () => {
      await p.page.goto(c.url);
      await p.page.waitForFunction(() => document.getElementById('line')?.textContent.startsWith('curl'), null, { timeout: 30_000 });
    });
    return { ...r, line: await p.page.locator('#line').textContent().catch(() => null), seen: await look(p.page) };
  }
  if (c.do === 'home') {
    const r = await timed(async () => {
      await p.page.goto(c.url);
      // Judged: no cell still asking, and the index no longer asking.
      await p.page.waitForFunction(() => document.querySelector('ul.door') && !document.body.innerText.includes('asking…') && !document.body.innerText.includes('Asking each computer'), null, { timeout: 60_000 });
    });
    return { ...r, ...(await drawn(p.page)), seen: await look(p.page) };
  }
  if (c.do === 'bring') {
    const r = await timed(async () => {
      await p.page.getByRole('button', { name: /^Bring the ticked ones/ }).click();
      await p.page.waitForFunction(() => /You brought|on the table already|Nothing was brought/.test(document.body.innerText), null, { timeout: 30_000 });
    });
    return { ...r, seen: await look(p.page) };
  }
  if (c.do === 'lend') {
    // The row's words before Lend is pressed (v1ui-polish (d)).
    let before = null;
    const r = await timed(async () => {
      await p.page.getByRole('button', { name: 'Look for programs running on this computer' }).click();
      const lend = p.page.getByRole('button', { name: `Lend ${c.kind} to everyone in your groups` });
      await lend.waitFor({ timeout: 60_000 });
      before = await p.page.locator('#running li', { has: lend }).innerText();
      await lend.click();
      await p.page.waitForFunction((k) => document.body.innerText.includes(`You lent ${k}`), c.kind, { timeout: 30_000 });
    });
    return { ...r, before, seen: await look(p.page) };
  }
  if (c.do === 'join') {
    const asked = await timed(async () => {
      await p.page.goto(c.url);
      await p.page.waitForSelector('#join:not([hidden])', { timeout: 60_000 });
    });
    if (asked.ms === null) return { asked, seen: await look(p.page) };
    const shown = await look(p.page);
    await p.page.fill('#name', c.name);
    const joined = await timed(async () => {
      await p.page.click('#join button');
      await p.page.waitForSelector('#status.good, #status.bad', { timeout: 90_000 });
    });
    return { asked, shown, joined, status: await p.page.locator('#status').textContent(), seen: await look(p.page) };
  }
  if (c.do === 'group') {
    const r = await timed(async () => {
      await p.page.goto('about:blank');
      await p.page.goto(c.url);
      await p.page.waitForSelector('#status.good, #status.bad', { timeout: 90_000 });
      await p.page.waitForFunction(() => !document.body.innerText.includes('asking…'), null, { timeout: 30_000 });
    });
    const status = await p.page.locator('#status').textContent();
    return { ...r, status, good: await p.page.evaluate(() => document.getElementById('status').className === 'good'), ...(await drawn(p.page)), seen: await look(p.page) };
  }
  if (c.do === 'money') {
    const out = { presses: [], shown: [] };
    let tab;
    out.open = await timed(async () => {
      tab = await press(p, 'Money');
      await tab.waitForFunction(() => document.querySelector('h2') && !document.body.innerText.includes('Reading your group’s money'), null, { timeout: 60_000 });
    });
    if (out.open.ms === null) return { ...out, seen: tab ? await look(tab) : await look(p.page) };
    out.shown.push(await look(tab));
    // Read only: Money as drawn, nothing pressed.
    if (c.read) return out;
    const redrawn = (before) => tab.waitForFunction((b) => document.body.innerText !== b, before, { timeout: 30_000 });
    const words = () => tab.evaluate(() => document.body.innerText);
    if (c.add) {
      out.added = await timed(async () => {
        await tab.fill('#what', c.add.what);
        await tab.fill('#amount', c.add.amount);
        const before = await words();
        await tab.click('#add');
        await redrawn(before);
      });
      out.shown.push(await look(tab));
      return out;
    }
    // In on Gas bill, then each "Say you paid …" until square: a person
    // pressing what the page offers, at most 8 presses.
    out.square = await timed(async () => {
      const t0 = Date.now();
      for (let i = 0; i < 8; i++) {
        const w = await words();
        const inOn = tab.locator('li', { hasText: 'for Gas bill' }).getByRole('button', { name: 'Say you were in on it' });
        // Square before saying you were in is square with nothing owed:
        // not done until Gas bill is yours too.
        if (w.includes('You’re square with everyone.') && !(await inOn.count())) return Date.now() - t0;
        const button = (await inOn.count()) ? inOn : tab.getByRole('button', { name: /^Say you paid / }).first();
        if (!(await button.count())) throw new Error(`nothing to press, not square: ${w.slice(0, 300)}`);
        const label = await button.textContent();
        const at = Date.now();
        await button.click();
        await redrawn(w);
        out.presses.push({ pressed: label, ms: Date.now() - at, at: at - t0 });
        out.shown.push(await look(tab));
      }
      throw new Error('not square after 8 presses');
    });
    out.seen = await look(tab);
    return out;
  }
  if (c.do === 'library') {
    const out = {};
    let tab;
    out.open = await timed(async () => {
      tab = await press(p, 'jellyfin', 'Priya');
      await tab.waitForSelector(`${SIGN_IN}, .card`, { timeout: 60_000 });
      await tab.waitForTimeout(1500);
    });
    if (out.open.ms === null) return { ...out, seen: tab ? await look(tab) : await look(p.page) };
    out.shown = await look(tab);
    if (out.shown.sign_in || out.shown.password) return { ...out, stopped: 'asked to sign in, with nothing of theirs to type', seen: out.shown };
    out.card = await timed(async () => {
      // Its title, a link beneath the poster, as a person clicks it; its
      // poster when the title takes no click (a phone's), said which.
      const card = tab.locator('.card:visible', { hasText: c.film }).first();
      await card.waitFor({ timeout: 30_000 });
      out.clicked = await card.locator('.cardText', { hasText: c.film }).first().click({ timeout: 5_000 }).then(() => 'title', () => null);
      if (!out.clicked) out.clicked = await card.locator('.cardImageContainer').first().click({ timeout: 5_000 }).then(() => 'poster', (e) => `neither: ${String(e).split('\n')[0]}`);
      await tab.waitForURL(/details/, { timeout: 30_000 });
    });
    if (out.card.ms === null) return { ...out, seen: await look(tab) };
    out.playing = await timed(async () => {
      const play = tab.locator('.mainDetailButtons .btnPlay:visible').first();
      await play.waitFor({ timeout: 30_000 });
      await play.click({ force: true });
      await tab.waitForFunction(() => { const v = document.querySelector('video'); return v && v.currentTime > 0.5 && !v.paused; }, null, { timeout: 60_000 });
    });
    out.video = await tab.evaluate(() => { const v = document.querySelector('video'); return v && { ct: +v.currentTime.toFixed(2), paused: v.paused, w: v.videoWidth, h: v.videoHeight }; }).catch(() => null);
    out.seen = await look(tab);
    return out;
  }
  if (c.do === 'shot') {
    const page = p.as === 'phone' ? p.page : p.tabs.at(-1) && !c.home ? p.tabs.at(-1) : p.page;
    await page.screenshot({ path: `${process.env.SHOTS}/${c.name}.png` }).catch(() => {});
    return { ok: true };
  }
  if (c.do === 'back') {
    // A phone back on the group's page; a computer's tabs closed.
    for (const t of p.tabs.splice(0)) await t.close().catch(() => {});
    return { ok: true };
  }
  return { error: `no such command: ${c.do}` };
}

console.log(JSON.stringify({ ready: true }));
for await (const line of createInterface({ input: process.stdin })) {
  if (!line.trim()) continue;
  let answer;
  try {
    answer = await act(JSON.parse(line));
  } catch (e) {
    answer = { error: String(e).slice(0, 600) };
  }
  console.log(JSON.stringify(answer));
}
for (const p of Object.values(people)) await p.close().catch(() => {});
