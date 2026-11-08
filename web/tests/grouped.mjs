// SPDX-License-Identifier: AGPL-3.0-or-later
// Native authenticated shell; synthetic three-group reads and recorded writes.
// B is what a person sees; A, frozen at `/?ui=a`, is the live control baseline.
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { launch } from './page.mjs';

const args = JSON.parse(process.argv[2]);
const { context, close } = await launch(args.engine, { device: 'Desktop Chrome', b: true });
const logs = [];
context.on('console', (message) => logs.push(message.text()));
try {
  if (process.env.GROUPED_HOME_PLANT === 'missing-wifi') {
    await context.route('**/page/grouped.js*', async (route) => {
      const source = await readFile(
        new URL('../../crates/svrngs-node/src/home/page/grouped.js', import.meta.url),
        'utf8',
      );
      const body = source.replace(
        "draw(['links', 'wifi', 'show', 'leave'])",
        "draw(['links', 'show', 'leave'])",
      );
      await route.fulfill({ contentType: 'text/javascript', body });
    });
  }
  const ids = [args.group, '2'.repeat(64), '3'.repeat(64)];
  await context.addInitScript(
    ({ ids }) => {
      const people = ['Alex', 'Sam', 'Kim', 'Ana'].map((person, i) => ({
        key: `person-${i}`,
        person_key: `person-${i}`,
        person,
        device: 'computer',
        joined: i === 0 ? 'started' : { at: 1793404800 + i },
      }));
      const shelf = [
        { name: 'Money', words: 'Who paid for what and who owes whom.' },
        { name: 'Events' },
      ];
      const groups = ids.map((id, i) => ({
        id,
        name: i === 2 ? 'Welcome room' : 'House',
        person: 'Alex',
        removed: false,
        left: false,
        keeps: true,
        folders: [`/fixture/${id}`],
        fallback: null,
        made: { name: 'House', maker: i === 1 ? 'Kim' : 'Sam', since: 1793404800 },
        removals: [],
        visiting: [
          {
            id: 'visit',
            guest: 'Pat',
            host: 'Alex',
            host_key: 'person-0',
            until: 1893404800,
            yours: true,
          },
        ],
      }));
      const hosted = {
        key: 'person-0',
        person: 'Alex',
        name: 'library',
        kind: 'books',
        origin: 'http://library.localhost:1234',
        limit: 2,
        using: 0,
      };
      const lends = (id) => ({
        lent: [
          {
            key: 'person-0',
            person: 'Alex',
            name: `Money-${id}`,
            kind: 'Money',
            group: id,
            origin: `http://money-${id}.localhost:1234`,
            limit: null,
            using: 0,
          },
          {
            ...hosted,
            key: 'person-1',
            person: 'Sam',
            reads: ['taste'],
            asks: 'May I read your taste?',
          },
          {
            key: 'person-1',
            person: 'Sam',
            name: 'played',
            kind: 'echo',
            raw: 'udp',
            origin: 'http://played.localhost:1234',
            limit: 3,
            using: 0,
          },
        ],
        shelf,
        unanswered: [{ person: 'Ana', person_key: 'person-3' }],
        unheard: [],
        refused: [],
        older: [],
        changed: [{ group: id, app: 'Money', by: 'Kim', by_key: 'person-2', at: 1793404900 }],
      });
      const original = window.fetch;
      const writes = JSON.parse(sessionStorage.getItem('grouped-writes') ?? '[]');
      window.fetch = async (input, init) => {
        const url = new URL(String(input), location.href);
        const path = url.pathname;
        const mode = sessionStorage.getItem('grouped-mode');
        const json = (body) => Response.json(body);
        if (
          init?.method === 'POST' &&
          [
            '/folder',
            '/group',
            '/join',
            '/link',
            '/publication',
            '/publication/close',
            '/lent',
            '/runs',
            '/choices',
            '/visits',
            '/v1/leaves',
            '/v1/handovers',
          ].includes(path)
        ) {
          const body = JSON.parse(init.body);
          writes.push({ path, body });
          sessionStorage.setItem('grouped-writes', JSON.stringify(writes));
          if (path === '/group') return json([body[0], null, []]);
          if (path === '/join') return json(['House', 'Alex', null]);
          if (path === '/link')
            return json({ client: 'svrngs://fixture', https: 'https://group.test/#fixture' });
          if (path === '/publication')
            return json({
              id: 'pub',
              state: 'open',
              address: 'https://join.test/fixture',
              until: 1893404800,
            });
          if (path === '/publication/close')
            return json({ id: 'pub', state: 'closed', address: null, until: 1893404800 });
          if (path === '/lent') return json({});
          return json({});
        }
        if (path === '/v1/groups') {
          if (mode === 'unread') return new Response('fixture unavailable', { status: 503 });
          const rows = mode === 'empty' ? [] : groups.map((g) => ({ ...g }));
          if (mode === 'left') rows[2].left = true;
          if (mode === 'removed') rows[2].removed = true;
          return json({ groups: rows, key: 'person-0' });
        }
        if (path === '/v1/rail/people') return json({ people });
        if (path === '/v1/lends') {
          if (mode === 'lends-unread') return new Response('fixture unavailable', { status: 503 });
          return json(lends(url.searchParams.get('namespace')));
        }
        if (path === '/v1/room' || path === '/v1/play') return json([]);
        if (path === '/reaching') return json({ library: { said: 'yes', at: 1793404800 } });
        if (path === '/choices') return json({ choices: [] });
        if (path === '/lent') return json({ lent: [hosted], signing: [] });
        if (path === '/update') return json({ state: 'current' });
        if (path === '/publication') return json(null);
        if (path === '/group') return json([]);
        if (path === '/join') return json(['House', null, 'fixture']);
        return original(input, init);
      };
    },
    { ids },
  );
  const page = await context.newPage();
  page.on('pageerror', (error) => logs.push(`pageerror: ${error}`));
  await page.goto(args.open);
  await page.getByRole('navigation', { name: 'Your groups' }).waitFor({ timeout: 10_000 });
  assert.equal(await page.getByRole('link', { name: 'All your groups', exact: true }).count(), 1);
  const origin = new URL(page.url()).origin;
  const nav = page.locator('.grouped-nav');
  // Each place's drawn sign: A's foot, B's navigation.
  const visit = async (path) => {
    await page.goto(`${origin}/${path}`);
    await (path.includes('ui=a') ? page.locator('#home p.address') : nav).waitFor();
  };
  const links = () => nav.locator('a').allTextContents();
  // The page's foot, and #intro, where B shows them and nowhere else.
  const shows = async (name, { intro = false, foot = false }) => {
    const button = page.getByRole('button', { name: 'Check the connection now', exact: true });
    assert.equal(await page.locator('#intro').isVisible(), intro, `${name}: intro`);
    assert.equal(await button.count(), foot ? 1 : 0, `${name}: the connection button`);
    assert.equal(await page.locator('#home p.address').count(), foot ? 1 : 0, `${name}: address`);
    assert.equal(await page.locator('#again').isVisible(), foot, `${name}: how it opens again`);
    assert.equal(await page.locator('#terminal').isVisible(), foot, `${name}: the terminal`);
  };
  const TWO = ['All your groups', 'This computer'];
  const inventory = async (selector) => {
    await page.locator(selector).evaluateAll((roots) => {
      for (const root of roots)
        for (const detail of root.querySelectorAll('details')) detail.open = true;
    });
    return page.locator(selector).evaluateAll((roots) =>
      roots
        .flatMap((root) =>
          [...root.querySelectorAll('a,button,input,select')]
            .filter((el) => el.getAttribute('type') === 'hidden' || el.checkVisibility())
            .map((el) => [
              el.tagName,
              el.getAttribute('type'),
              el.getAttribute('name'),
              el.tagName === 'INPUT' ? '' : el.textContent.trim(),
              el.getAttribute('href'),
              el.getAttribute('action'),
            ])
            .map((entry) => JSON.stringify(entry)),
        )
        .sort(),
    );
  };
  assert.equal(await page.locator('#home > section[id^="group-"]:visible').count(), 0);
  await page.locator('.group-list a').nth(2).waitFor();
  assert.equal(await page.locator('.group-list a').count(), 3);
  assert.deepEqual(
    (await links()).filter((name) => !/^(House|Welcome room) /.test(name)),
    [...TWO, 'Join a group', 'Make a group', 'Open a link', 'Groups you left'],
    'the overview offers these and no way back to A',
  );
  assert.equal(await page.locator('#home details:visible').count(), 0, 'nothing to unfold');
  assert.equal(await page.getByText('Choose a group', { exact: false }).count(), 0);
  await shows('overview', { intro: true });
  const overview = await page.screenshot({
    path: `${args.artifacts ?? 'target'}/grouped-overview.png`,
    fullPage: true,
  });
  assert.ok(overview.length > 0);

  // A supplies the control inventory independently of B's layout manifest.
  await visit('?ui=a');
  const baselineGroups = [];
  for (const id of ids) baselineGroups.push(await inventory(`#group-${id}`));
  const baselineHome = {};
  for (const name of ['acts', 'open', 'join', 'make', 'running', 'lent'])
    baselineHome[name] = await inventory(`#${name}`);
  assert.equal(await nav.count(), 0);
  assert.equal(await page.locator('#terminal').isVisible(), true);
  assert.equal(await page.locator('#again').isVisible(), true);
  assert.equal(await page.locator('#intro').isVisible(), true);

  // B has no foot to wait on: a place's controls are read until they are all drawn.
  const drawn = async (selector, want, why) => {
    let seen;
    for (let tries = 0; tries < 100; tries++) {
      seen = await inventory(selector);
      if (JSON.stringify(seen) === JSON.stringify(want)) break;
      await page.waitForTimeout(100);
    }
    assert.deepEqual(seen, want, why);
  };

  for (const [i, id] of ids.entries()) {
    await visit(`?group=${id}`);
    const group = page.locator(`#group-${id}`);
    assert.equal(await page.locator('#home > section[id^="group-"]:visible').count(), 1);
    assert.equal(await group.isVisible(), true);
    await group.locator('table.index').waitFor();
    await group
      .getByRole('button', { name: /^Open shared files in (Finder|your file manager)$/ })
      .waitFor();
    assert.deepEqual(await links(), TWO, 'a group page has no Switch group, no tools row');
    assert.equal(await group.locator('h2').first().isVisible(), true, 'its banner heads the page');
    await shows('a group', {});
    assert.equal(await group.getByRole('button', { name: /^Leave / }).isVisible(), false);
    await drawn(`#group-${id}`, baselineGroups[i], 'all original group controls retained');
    await group
      .getByRole('button', { name: /^Open shared files in (Finder|your file manager)$/ })
      .click();
    await group.getByText('Opened, in a window of its own.', { exact: false }).waitFor();
    const sent = await page.evaluate(() => JSON.parse(sessionStorage.getItem('grouped-writes')));
    assert.deepEqual(sent.at(-1), { path: '/folder', body: { group: id, dir: `/fixture/${id}` } });
    await page.reload();
    await group.waitFor();
    assert.equal(await group.isVisible(), true, 'refresh retains selected stable id');
  }

  // Normal links, Back, and rereads retain the selected group.
  await page.getByRole('link', { name: 'All your groups', exact: true }).click();
  await page.locator(`.group-list a[href$="group=${ids[0]}"]`).click();
  await page.locator(`#group-${ids[0]}`).waitFor();
  await page.goBack();
  await page.goBack();
  await page.locator(`#group-${ids[2]}`).waitFor();
  await page.evaluate(() => document.dispatchEvent(new Event('visibilitychange')));
  assert.equal(await page.locator(`#group-${ids[2]}`).isVisible(), true);

  await page.locator('.grouped-notice a').click();
  await page.locator('#home p.address').waitFor();
  assert.equal(await page.locator('#acts').isVisible(), true);
  assert.equal(await page.locator('#acts details').first().getAttribute('open'), '');
  assert.equal(
    await page
      .getByRole('button', { name: 'Open the new version', exact: true })
      .first()
      .isVisible(),
    true,
  );
  await shows('This computer', { foot: true });
  assert.equal(
    await page.locator('#home p.address').textContent(),
    `Your home page, the same address each time svrngs starts: ${origin}/`,
  );
  for (const [view, names] of Object.entries({
    computer: ['acts', 'running', 'lent'],
    open: ['open'],
    join: ['join'],
    make: ['make'],
  })) {
    await visit(`?view=${view}`);
    assert.deepEqual(await links(), TWO, `${view} has no Switch group, no tools row`);
    for (const name of names) {
      await page.locator(`#${name}`).waitFor();
      await drawn(`#${name}`, baselineHome[name], `${name} controls retained`);
    }
    await shows(view, { foot: view === 'computer' });
    assert.equal(await page.locator('#home > section[id^="group-"]:visible').count(), 0);
  }

  await page.locator('#make input[name="name"]').fill('New group');
  await page.locator('#make input[name="person"]').fill('Alex');
  await page.locator('#make button').click();
  await page.getByText('You made New group.', { exact: false }).waitFor();
  assert.equal(await nav.isVisible(), true);
  await visit('?view=join');
  await page.locator('#join input[name="link"]').fill('fixture');
  await page.locator('#join input[name="person"]').fill('Alex');
  await page.locator('#join button').click();
  await page.getByText('You’re in House', { exact: false }).waitFor();
  assert.equal(await nav.isVisible(), true);

  // The button's answer lands on This computer, beside the button, said once.
  await visit('?view=computer');
  await page.getByRole('button', { name: 'Check the connection now', exact: true }).click();
  await page.waitForURL(`${origin}/?view=computer`);
  await page
    .locator('#home > p')
    .getByText(/^The public way (answers|did not answer)/)
    .waitFor();
  await page.locator('#home p.address').waitFor();
  await shows('the answered button', { foot: true });
  await page.reload();
  await page.locator('#home p.address').waitFor();
  assert.equal(await page.getByText(/^The public way/).count(), 0, 'said once');

  // An old bookmark of B still opens B, and nothing draws a way to A.
  await visit('?ui=b');
  await page.locator('.group-list a').nth(2).waitFor();
  assert.equal(await page.locator('a[href*="ui="]').count(), 0);
  await visit('?ui=a');
  assert.equal(await page.locator('a[href*="ui="]').count(), 0);
  // A's said-once address stays A, and so does its write.
  await visit('?ui=a&made=Old');
  assert.equal(page.url(), `${origin}/?ui=a`);
  await page.locator('#make input[name="name"]').fill('New group');
  await page.locator('#make input[name="person"]').fill('Alex');
  await page.locator('#make button').click();
  await page.getByText('You made New group. Invite someone to it below.').waitFor();
  assert.equal(await nav.count(), 0, 'a write on A stays on A');

  await page.goto(`${origin}/#group-${ids[2]}`);
  await page.getByRole('heading', { name: 'Welcome to Welcome room', exact: true }).waitFor();
  assert.equal(await page.locator('#terminal').isVisible(), false);
  assert.deepEqual(
    await page.locator('#home a').evaluateAll((all) => all.map((a) => a.getAttribute('href'))),
    [`/?group=${ids[2]}`, '/'],
  );
  await page.getByRole('link', { name: 'Continue to Welcome room', exact: true }).click();
  await page.locator(`#group-${ids[2]}`).waitFor();

  for (const view of ['constructor', '__proto__', 'not-a-view']) {
    await visit(`?view=${view}`);
    await page.getByText('This page is not available.', { exact: false }).waitFor();
    assert.equal(await page.locator('#lent').isVisible(), false);
  }
  await visit('?group=missing');
  await page.getByText('This group is not available here.', { exact: false }).waitFor();
  assert.equal(await page.locator('#home > section[id^="group-"]:visible').count(), 0);

  for (const mode of ['left', 'removed', 'lends-unread', 'empty']) {
    await page.evaluate((mode) => sessionStorage.setItem('grouped-mode', mode), mode);
    await page.goto(`${origin}/?group=${ids[2]}`);
    await page.reload();
    const expected = {
      left: 'This group is not available here.',
      removed: 'You are no longer in Welcome room.',
      'lends-unread': 'cannot read what Welcome room lends',
      empty: 'You’re in no group yet.',
    };
    await page.getByText(expected[mode], { exact: false }).waitFor();
    if (mode === 'left') {
      await page.getByRole('link', { name: 'All your groups', exact: true }).click();
      await page.getByRole('link', { name: 'Groups you left', exact: true }).click();
      await page.locator('#left').waitFor();
      assert.ok((await page.locator('#left').textContent()).includes('Welcome room'));
    }
  }
  await page.evaluate(() => sessionStorage.setItem('grouped-mode', 'unread'));
  await page.reload();
  await page.getByText('This computer cannot read your groups:', { exact: false }).waitFor();
  await page.evaluate(() => sessionStorage.removeItem('grouped-mode'));
  await visit('');
  await page.setViewportSize({ width: 390, height: 844 });
  assert.equal(await page.evaluate(() => document.documentElement.scrollWidth > innerWidth), false);
  await page.screenshot({
    path: `${args.artifacts ?? 'target'}/grouped-narrow.png`,
    fullPage: true,
  });
  await visit(`?group=${ids[2]}`);
  await page.locator(`#group-${ids[2]} table.index`).waitFor();
  assert.equal(
    await page.evaluate(() => document.documentElement.scrollWidth > innerWidth),
    false,
    'workspace has no horizontal overflow',
  );
  await page.screenshot({
    path: `${args.artifacts ?? 'target'}/grouped-workspace.png`,
    fullPage: true,
  });
  assert.ok(!logs.some((line) => line.startsWith('pageerror:')), logs.join('\n'));
  console.log(
    JSON.stringify({
      navigation: true,
      groupControls: baselineGroups.map((list) => list.length),
      homeControls: Object.fromEntries(
        Object.entries(baselineHome).map(([key, list]) => [key, list.length]),
      ),
    }),
  );
} catch (error) {
  console.error(`${error}\n${logs.join('\n')}`);
  process.exitCode = 1;
} finally {
  await close();
}
