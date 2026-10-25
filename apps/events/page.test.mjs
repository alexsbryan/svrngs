// Events' page in a browser (events-app (b)): Chromium as a desktop and
// WebKit as an iPhone (web/tests/page.mjs, `launch`), the server run alone on
// a file of its own, each person named by the header a proxy would set.
// Sam posts a dinner to The House and dWeb; Bo is coming with Maya, came, and
// says Maya came; Ama says she's coming and takes it back; Cy, in dWeb, is
// coming. Each group's people but its host are said as invited, Ama, who
// took hers back, there and nowhere else. Each member's page holds only
// their own group's answers, the host's holds both, and a member of one
// group opening the other's event page is refused. Told its label and its
// groups' pages as svrngs's door tells them (events-links (b)), Sam's page
// offers a link to send in each group's chat, or says why there is none;
// no one else's does. Without Playwright or its browser this fails naming the
// install (`launch` exits 3); it never skips.

import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { mkdtempSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { test } from 'node:test';

import { launch } from '../../web/tests/page.mjs';

const WHO = 'remote-user';
/** Words the page never says (DESIGN §1; the queue's fence 6). */
const NOT_SAID = /\b(key|member|record|act|namespace|origin)\b/i;

/** The server on a file of its own, holding `ops` ({<group>: [op]}) already written. */
async function serve(ops = {}) {
  const dir = mkdtempSync(join(tmpdir(), 'events-page-'));
  writeFileSync(join(dir, 'store.json'), JSON.stringify({
    groups: { house: { name: 'The House', people: ['sam', 'bo', 'ama'] }, dweb: { name: 'dWeb', people: ['sam', 'cy'] } },
    ops,
  }));
  writeFileSync(join(dir, 'config.json'), JSON.stringify({ file: join(dir, 'store.json'), who: WHO }));
  const child = spawn(process.execPath, [new URL('./server.mjs', import.meta.url).pathname, join(dir, 'config.json')], { stdio: ['ignore', 'pipe', 'inherit'] });
  // `launch` exits 3 when a browser cannot start, past this test's `finally`:
  // the server ends with the process, or it outlives it, and node --test,
  // holding its stderr, never exits (capability-step-limit (a)).
  process.once('exit', () => child.kill());
  const line = await new Promise((done, fail) => {
    child.stdout.once('data', (b) => done(String(b)));
    child.once('exit', (code) => fail(new Error(`the server exited ${code} before it listened`)));
  });
  return { at: `http://127.0.0.1:${JSON.parse(line).port}`, stop: () => child.kill() };
}

for (const engine of ['chromium', 'webkit']) {
  test(`each group's people see only their own group's answers, the host every group's (${engine})`, async () => {
    const server = await serve();
    const { context, close } = await launch(engine);
    if (engine === 'chromium') await context.grantPermissions(['clipboard-read', 'clipboard-write']);
    try {
      const page = await context.newPage();
      const said = [];
      const as = async (person, path, told = {}) => {
        await context.setExtraHTTPHeaders({ [WHO]: person, ...told });
        const res = await page.goto(`${server.at}${path}`);
        said.push(await page.innerText('body'));
        return res;
      };
      const press = async (name) => {
        await Promise.all([page.waitForNavigation(), page.getByRole('button', { name, exact: true }).click()]);
        said.push(await page.innerText('body'));
      };
      const lists = async () => page.$$eval('section[data-group]', (ss) => ss.map((s) => ({
        group: s.dataset.group,
        invited: [...s.querySelectorAll('.invited li')].map((l) => l.textContent),
        coming: [...s.querySelectorAll('.coming li')].map((l) => l.textContent),
        came: [...s.querySelectorAll('.came li')].map((l) => l.textContent),
      })));

      await as('sam', '/');
      await page.fill('input[name=what]', 'dinner');
      await page.fill('input[name=when]', 'Friday 7');
      await page.fill('input[name=where]', 'the house');
      await page.check('input[value=house]');
      await page.check('input[value=dweb]');
      await press('Post');
      const house = new URL(page.url()).pathname;
      assert.match(house, /^\/g\/house\/e\/[0-9a-f]{16}$/);
      const dweb = house.replace('/g/house/', '/g/dweb/');

      await as('bo', house);
      await page.fill('input[name=guests]', 'Maya');
      await press("I'm coming");
      await press('I came');
      await press('Maya came');
      await as('ama', house);
      await press("I'm coming");
      await press('Take my name off');
      await as('cy', dweb);
      await press("I'm coming");

      assert.deepEqual(await lists(), [{ group: 'dweb', invited: ['cy'], coming: ['cy'], came: [] }], "Cy's page: dWeb's answers alone");
      await as('bo', house);
      assert.deepEqual(await lists(), [{ group: 'house', invited: ['ama', 'bo'], coming: ['bo + Maya'], came: ['bo', "Maya, bo's guest"] }], "Bo's page: The House's answers alone");
      const refused = await as('cy', house);
      assert.equal(refused.status(), 403);
      assert.match(await page.innerText('body'), /You're not in The House/);
      // What svrngs's door tells the app (events-links (a)): its label, and
      // the page of each of Sam's groups, dWeb keeping none.
      await as('sam', house, { 'x-svrngs-label': 'lbl', 'x-svrngs-pages': 'house=https://group.example/ dweb=none' });
      assert.deepEqual(await lists(), [
        { group: 'house', invited: ['ama', 'bo'], coming: ['bo + Maya'], came: ['bo', "Maya, bo's guest"] },
        { group: 'dweb', invited: ['cy'], coming: ['cy'], came: [] },
      ], "Sam's page: both groups' answers");
      const links = await page.$$eval('section[data-group]', (ss) => ss.map((s) => ({
        group: s.dataset.group,
        link: s.querySelector('input.link')?.value ?? null,
        said: s.querySelector('.nolink')?.textContent ?? null,
      })));
      assert.deepEqual(links, [
        { group: 'house', link: `https://group.example/#open=lbl&at=${house}`, said: null },
        { group: 'dweb', link: null, said: "dWeb keeps no page for links, so there's no link to send." },
      ], "Sam's page: a link to send in each group's chat, or why not");
      const copy = page.locator('section[data-group=house] input.link + button');
      assert.equal(await copy.textContent(), 'Copy the link to this event');
      await copy.click();
      await page.waitForFunction(() => document.querySelector('section[data-group=house] input.link + button').textContent !== 'Copy the link to this event');
      const told = await copy.textContent();
      assert.ok(['Copied', 'Copy it from the box'].includes(told), `the Copy button says what it did: ${told}`);
      if (engine === 'chromium') assert.equal(await page.evaluate(() => navigator.clipboard.readText()), links[0].link);
      for (const [person, path] of [['bo', house], ['cy', dweb]]) {
        await as(person, path, { 'x-svrngs-label': 'lbl', 'x-svrngs-pages': 'house=https://group.example/' });
        assert.equal(await page.locator('input.link, .nolink').count(), 0, `${person} is not offered the host's link`);
      }
      for (const text of said) assert.doesNotMatch(text, NOT_SAID);
    } finally {
      await close();
      server.stop();
    }
  });
}

for (const engine of ['chromium', 'webkit']) {
  test(`an answer another person's computer wrote says whose (${engine})`, async () => {
    // ralph/FINDING_ON_BEHALF_OF.md's scene: Sam's computer writes Bo's
    // "coming" in Bo's name; Bo's "came" is his own computer's.
    const event = { kind: 'event', event: '0123456789abcdef', what: 'dinner', when: 'Friday 7', where: 'the house' };
    const server = await serve({
      house: [
        { id: 'a', ts_unix: 1000, person: 'sam', payload: event },
        { id: 'b', ts_unix: 1100, person: 'sam', on_behalf_of: 'bo', payload: { kind: 'coming', event: event.event, guests: ['Maya'] } },
        { id: 'c', ts_unix: 1200, person: 'bo', on_behalf_of: 'bo', payload: { kind: 'came', event: event.event } },
      ],
    });
    const { context, close } = await launch(engine);
    try {
      const page = await context.newPage();
      await context.setExtraHTTPHeaders({ [WHO]: 'bo' });
      await page.goto(`${server.at}/g/house/e/${event.event}`);
      const lists = await page.$$eval('section[data-group]', (ss) => ss.map((s) => ({
        coming: [...s.querySelectorAll('.coming li')].map((l) => l.textContent),
        came: [...s.querySelectorAll('.came li')].map((l) => l.textContent),
      })));
      assert.deepEqual(lists, [{ coming: ["bo + Maya, through sam's computer"], came: ['bo'] }]);
      assert.match(await page.innerText('body'), /^sam is hosting\.$/m, "Sam's own post says nothing more");
      assert.doesNotMatch(await page.innerText('body'), NOT_SAID);
    } finally {
      await close();
      server.stop();
    }
  });
}
