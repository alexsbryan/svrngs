// Each person's browser for run.py here (PREDICTIONS.md §1): Chromium for
// each computer, WebKit as an iPhone for the phone (web/tests/page.mjs), one
// browser each, kept for the whole run so what a phone keeps stays. It
// prints `{"ready": true}`, then reads one JSON command a line on stdin and
// prints one JSON answer a line:
//
//   {"do": "open", "who", "engine"}             the person's browser
//   {"do": "join", "who", "link", "name"}       the phone: open its link,
//                                               type the name if asked
//   {"do": "steps", "who", "steps": [...]}      each step in turn, on the
//                                               person's one page:
//     {"goto": url}                             open it
//     {"wait": re, "within"?: ms}               until the page's words match
//     {"poll": url, "re", "within"?, "every"?}  open it again until they do
//     {"click": words, "in"?: words, "until"?: re}
//                                               press the button with those
//                                               words (in the item of the
//                                               page's list holding `in`),
//                                               then wait until the words
//                                               match `until`, else change
//     {"summary": words}                        open a <details> by its words
//     {"check" | "uncheck": selector}           tick or untick
//     {"fill": selector, "value"}               type into it
//     {"lent": words, "until": re}              the phone: open the app with
//                                               those words from the group's
//                                               list, until its words match
//     {"count": selector}                       how many match, as `value`
//     {"read": true}                            the page's words now
//
// Every step's answer is how long after the command began it ended and the
// page's words then; a step that fails ends the command, said with the
// words as they were. A page's errors are kept and answered with each
// command, and its console's last 60 lines.
//
//   node people.mjs
import { createInterface } from 'node:readline';
import { launch } from '../../../../web/tests/page.mjs';

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
    const at = s.in ? page.locator('main li, #home section, #shelf p', { hasText: s.in }).last() : page;
    await at.getByRole('button', { name: s.click, exact: true }).first().click({ timeout: 30_000 });
    if (s.until) return until(page, s.until, s.within);
    const t0 = Date.now();
    while ((await words(page)) === before) {
      if (Date.now() - t0 > 30_000) throw new Error(`nothing changed after pressing ${s.click}`);
      await new Promise((r) => setTimeout(r, 100));
    }
    return;
  }
  if (s.summary) return page.locator('summary', { hasText: s.summary }).first().click({ timeout: 30_000 });
  if (s.check) return page.locator(s.check).first().check({ timeout: 30_000 });
  if (s.uncheck) return page.locator(s.uncheck).first().uncheck({ timeout: 30_000 });
  if (s.fill) return page.locator(s.fill).first().fill(s.value, { timeout: 30_000 });
  if (s.lent) {
    await page.locator('#lent button', { hasText: s.lent }).first().click({ timeout: 90_000 });
    return until(page, s.until, s.within ?? 120_000);
  }
  if (s.count) return page.locator(s.count).count();
  if (s.read) return;
  throw new Error(`no such step: ${JSON.stringify(s)}`);
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
    return { ms: Date.now() - t0, asked, status: await page.textContent('#status'), errors: errors.splice(0), logs: logs.splice(0).slice(-60) };
  }
  if (c.do === 'steps') {
    const done = [];
    for (const s of c.steps) {
      let value;
      try {
        value = await step(page, s);
      } catch (e) {
        done.push({ step: s, ms: Date.now() - t0, error: String(e).slice(0, 600), text: await words(page), url: page.url() });
        return { ms: Date.now() - t0, done, failed: true, errors: errors.splice(0), logs: logs.splice(0).slice(-60) };
      }
      done.push({ step: s, ms: Date.now() - t0, text: await words(page), url: page.url(), ...(s.count ? { value } : {}) });
    }
    return { ms: Date.now() - t0, done, errors: errors.splice(0), logs: logs.splice(0).slice(-60) };
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
    answer = { error: String(e).slice(0, 600) };
  }
  console.log(JSON.stringify(answer));
}
for (const p of Object.values(people)) await p.close();
