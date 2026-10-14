// Events' page in a browser (events-app (b)): Chromium as a desktop and
// WebKit as an iPhone (web/tests/page.mjs, `launch`), the server run alone on
// a file of its own, each person named by the header a proxy would set.
// Sam posts a dinner to The House and dWeb; Bo is coming with Maya, came, and
// says Maya came; Ama says she's coming and takes it back; Cy, in dWeb, is
// coming. Each group's people but its host are said as invited, Ama, who
// took hers back, there and nowhere else. Each member's page holds only
// their own group's answers, the host's holds both, and a member of one
// group opening the other's event page is refused. Without Playwright or its browser this fails naming the
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

async function serve() {
  const dir = mkdtempSync(join(tmpdir(), 'events-page-'));
  writeFileSync(join(dir, 'store.json'), JSON.stringify({
    groups: { house: { name: 'The House', people: ['sam', 'bo', 'ama'] }, dweb: { name: 'dWeb', people: ['sam', 'cy'] } },
  }));
  writeFileSync(join(dir, 'config.json'), JSON.stringify({ file: join(dir, 'store.json'), who: WHO }));
  const child = spawn(process.execPath, [new URL('./server.mjs', import.meta.url).pathname, join(dir, 'config.json')], { stdio: ['ignore', 'pipe', 'inherit'] });
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
    try {
      const page = await context.newPage();
      const said = [];
      const as = async (person, path) => {
        await context.setExtraHTTPHeaders({ [WHO]: person });
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
      await as('sam', house);
      assert.deepEqual(await lists(), [
        { group: 'house', invited: ['ama', 'bo'], coming: ['bo + Maya'], came: ['bo', "Maya, bo's guest"] },
        { group: 'dweb', invited: ['cy'], coming: ['cy'], came: [] },
      ], "Sam's page: both groups' answers");
      for (const text of said) assert.doesNotMatch(text, NOT_SAID);
    } finally {
      await close();
      server.stop();
    }
  });
}
