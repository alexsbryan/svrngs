// Each person's browser for this run (PREDICTIONS.md §1), on whichever
// machine runs it: a phone joins on the published page and presses "doc",
// a computer opens the doc at its own svrngs's origin. Every wait is timed
// on this machine's clock and answered with `Date.now()` when it ended, so
// the driver can set two machines' times side by side.
const pw = await import(process.env.PLAYWRIGHT ?? 'playwright');

const words = (page, sel) => page.evaluate((s) => document.querySelector(s)?.innerText ?? '', sel).catch(() => '');

/** A browser for `who`: `engine` chromium or webkit, `device` a Playwright device name or none. */
export async function person({ who, engine, device }) {
  const browser = await pw[engine].launch();
  const context = await browser.newContext(device ? pw.devices[device] : {});
  const page = await context.newPage();
  const errors = [];
  page.on('pageerror', (e) => errors.push(String(e).slice(0, 300)));
  const until = async (test, within) => {
    const t0 = Date.now();
    for (;;) {
      if (await test()) return { ms: Date.now() - t0, at: Date.now() };
      if (Date.now() - t0 > within) return { ms: null, at: null, within };
      await new Promise((r) => setTimeout(r, 50));
    }
  };
  return {
    who,
    errors,
    /** Open `link` on the group's page and join as `name`, as a phone does. */
    async join(link, name) {
      const t0 = Date.now();
      await page.goto(link);
      await page.waitForSelector('#status.good, #status.bad, #join:not([hidden])', { timeout: 120_000 });
      if (await page.locator('#join').isVisible()) {
        await page.locator('#name').fill(name);
        await page.locator('#join button').first().click();
      }
      await page.waitForSelector('#status.good, #status.bad', { timeout: 120_000 });
      return { ms: Date.now() - t0, status: await words(page, '#status'), listed: await page.locator('#lent button').allTextContents() };
    },
    /** Press the lend `name` on the group's page, until its editor is ready. */
    async press(name) {
      const t0 = Date.now();
      await page.locator('#lent button', { hasText: name }).first().click({ timeout: 60_000 });
      await page.waitForSelector('.tiptap', { timeout: 120_000 });
      return { ms: Date.now() - t0, at: Date.now(), url: page.url().replace(/#.*/, '#…') };
    },
    /** Open the doc at `url`, as a computer opens it from its home page. */
    async open(url) {
      const t0 = Date.now();
      await page.goto(url);
      await page.waitForSelector('.tiptap', { timeout: 60_000 });
      return { ms: Date.now() - t0, at: Date.now() };
    },
    /** A new paragraph at the doc's end, typed. The caret is put at the
     * last paragraph's end by the page's own selection, which the editor
     * follows: End and its chords differ by engine and platform. */
    async type(text) {
      await page.locator('.tiptap').focus();
      await page.evaluate(() => {
        const last = document.querySelector('.tiptap').lastElementChild;
        const range = document.createRange();
        range.selectNodeContents(last);
        range.collapse(false);
        const s = getSelection();
        s.removeAllRanges();
        s.addRange(range);
      });
      await new Promise((r) => setTimeout(r, 50));
      await page.keyboard.press('Enter');
      const started = Date.now();
      await page.keyboard.type(text);
      return { started, at: Date.now() };
    },
    /** Until the editor holds every one of `texts`. */
    has: (texts, within) => until(async () => {
      const t = await words(page, '.tiptap');
      return texts.every((x) => t.includes(x));
    }, within),
    /** Until `sel` is shown (`shown`) or hidden. */
    shows: (sel, shown, within) => until(async () => (await page.locator(sel).isVisible().catch(() => false)) === shown, within),
    /** Until `sel`'s words match `re`. */
    says: (sel, re, within) => until(async () => new RegExp(re).test(await words(page, sel)), within),
    async read() {
      const r = {};
      for (const [k, s] of Object.entries({ title: '#title', you: '#you', here: '#here', night: '#night', kept: '#kept', back: '#back', editor: '.tiptap', notes: '#attribution' })) r[k] = await words(page, s);
      r.night_shown = await page.locator('#night').isVisible().catch(() => false);
      // Each paragraph's own words: the carets drawn in it, and their names, left out.
      r.paragraphs = await page.evaluate(() => [...document.querySelectorAll('.tiptap > *')].map((p) => {
        const c = p.cloneNode(true);
        c.querySelectorAll('.collaboration-carets__caret').forEach((x) => x.remove());
        return c.textContent;
      })).catch(() => []);
      r.note_lines = await page.evaluate(() => [...document.querySelectorAll('#attribution .note')].map((n) => n.innerText)).catch(() => []);
      r.carets = await page.evaluate(() => [...document.querySelectorAll('.collaboration-carets__label')].map((n) => n.innerText)).catch(() => []);
      r.errors = errors.splice(0);
      return r;
    },
    shot: (path) => page.screenshot({ path, fullPage: true }),
    close: () => browser.close(),
  };
}
