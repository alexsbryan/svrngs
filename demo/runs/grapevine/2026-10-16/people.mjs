// Each person's browser for run.py here (PREDICTIONS.md §1), after the
// Events run's (demo/runs/events/2026-10-14/people.mjs): Sam, Jo and Kim,
// each a Chromium of their own (web/tests/page.mjs), each on the page their
// own start line printed. It prints `{"ready": true}`, then reads one JSON
// command a line on stdin and prints one JSON answer a line:
//
//   {"do": "open", "who", "engine"?}          the person's browser, Chromium
//                                             unless `engine` names another
//   {"do": "goto", "who", "url"}              open their page
//   {"do": "press", "who", "within", "fill", "select", "button"}
//       within the part of the page `within` names, a list of
//       [selector, heading or words] each inside the last, fill each field
//       `fill` names by its label, choose in each list `select` names by
//       its label, and press the button named so; `absent` when the page
//       shows none. It waits for the page to read itself again, or for the
//       words it says in place of a refusal.
//   {"do": "await", "who", "words", "gone", "bound"}
//       reload every half second, as someone refreshing would, until the
//       page shows every one of `words` and none of `gone`; `met` false
//       when `bound` seconds pass first
//
// Every answer about a page is `view`: the words it shows, what its closed
// parts hold, its buttons, and how long the command took.
//
//   node people.mjs
import { createInterface } from 'node:readline';
import { launch } from '../../../../web/tests/page.mjs';

const people = {};

async function view(who, t0) {
  const { page } = people[who];
  const shown = await page.evaluate(() => {
    const all = document.body.cloneNode(true);
    for (const el of all.querySelectorAll('script, style')) el.remove();
    return {
      text: document.body.innerText,
      all: all.textContent.replace(/\s+/g, ' ').trim(),
      not: [...document.querySelectorAll('details.not li')].map((l) => l.textContent),
      asks: [...document.querySelectorAll('li.ask')].map((l) => ({ ask: l.dataset.ask, text: l.innerText })),
      threads: [...document.querySelectorAll('li.thread')].map((l) => ({ role: l.className.replace('thread ', ''), text: l.innerText })),
      groups: [...document.querySelectorAll('section[data-group] h2')].map((h) => h.textContent),
      buttons: [...document.querySelectorAll('button')].map((b) => b.textContent),
    };
  });
  return { ms: Date.now() - t0, url: page.url(), ...shown };
}

const has = (text, words, gone) => words.every((w) => text.includes(w)) && !gone.some((w) => text.includes(w));

async function act(c) {
  const t0 = Date.now();
  if (c.do === 'open') {
    const { context, close } = await launch(c.engine ?? 'chromium');
    const page = await context.newPage();
    const logs = [];
    page.on('console', (m) => logs.push(`${m.type()}: ${m.text()}`.slice(0, 300)));
    people[c.who] = { context, close, page, logs };
    return { ms: Date.now() - t0 };
  }
  const { page } = people[c.who];
  if (c.do === 'goto') {
    const res = await page.goto(c.url, { timeout: 30_000 });
    return { status: res?.status() ?? null, ...(await view(c.who, t0)) };
  }
  if (c.do === 'press') {
    let scope = page.locator('body');
    for (const [sel, words] of c.within ?? []) {
      const heading = scope.locator(sel).filter({ has: page.locator('h2, h3').filter({ hasText: new RegExp(`^${words}$`) }) });
      scope = (await heading.count()) ? heading.first() : scope.locator(sel).filter({ hasText: words }).first();
    }
    const button = scope.getByRole('button', { name: c.button, exact: true });
    if (!(await button.count())) return { absent: true, ...(await view(c.who, t0)) };
    for (const [label, value] of Object.entries(c.fill ?? {})) await scope.getByLabel(label, { exact: true }).fill(value);
    // A list's label reads its options too ("In 1400 Page Waller block"), so it is found by its first words.
    for (const [label, option] of Object.entries(c.select ?? {})) await scope.locator('label').filter({ hasText: new RegExp(`^\\s*${label}\\s`) }).locator('select').first().selectOption({ label: option });
    const said = button.first().locator('xpath=ancestor::form').locator('.said');
    const t1 = Date.now();
    const reloaded = page.waitForEvent('load', { timeout: 30_000 }).then(() => null);
    const refused = said.filter({ hasText: /\S/ }).waitFor({ timeout: 30_000 }).then(() => said.textContent());
    await button.first().click();
    const words = await Promise.race([reloaded, refused]).catch((e) => `no answer: ${String(e).slice(0, 200)}`);
    return { pressed_ms: Date.now() - t1, said: words, ...(await view(c.who, t0)) };
  }
  if (c.do === 'await') {
    const words = c.words ?? [];
    const gone = c.gone ?? [];
    let reloads = 0;
    for (;;) {
      const text = await page.evaluate(() => document.body.innerText);
      if (has(text, words, gone)) return { met: true, reloads, ...(await view(c.who, t0)) };
      if (Date.now() - t0 > (c.bound ?? 60) * 1000) return { met: false, reloads, ...(await view(c.who, t0)) };
      await new Promise((r) => setTimeout(r, 500));
      await page.reload({ timeout: 30_000 });
      reloads++;
    }
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
    answer = { error: String(e).slice(0, 600), logs: (people[c.who]?.logs ?? []).slice(-15) };
  }
  console.log(JSON.stringify(answer));
}
for (const p of Object.values(people)) await p.close();
