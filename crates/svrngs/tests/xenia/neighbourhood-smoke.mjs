// A scripted proof of the real world's mechanics; never a stranger result.
import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { createInterface } from 'node:readline';
import { dirname, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const here = dirname(fileURLToPath(import.meta.url));
const root = resolve(here, '../../../..');
const scenes = resolve(root, 'demo/xenia/permaculture');
const xenia = process.env.XENIA_ROOT ?? resolve(root, '../xenia');
const load = (path) => import(pathToFileURL(resolve(xenia, path)).href);
const [{ parseScene }, { parsePerson }, { launch, reach }] = await Promise.all([
  load('src/core/scene.ts'),
  load('src/core/person.ts'),
  load('src/browser.ts'),
]);
const out = resolve(
  root,
  'target/permaculture-smoke',
  new Date().toISOString().replace(/[:.]/g, '-'),
);
await mkdir(out, { recursive: true });
const world = spawn(resolve(scenes, 'neighbourhood'), [], {
  cwd: scenes,
  stdio: ['pipe', 'pipe', 'inherit'],
});
const exited = new Promise((done) => world.on('exit', (code, signal) => done({ code, signal })));
const lines = createInterface({ input: world.stdout });
let browser;
let evidencePath;
const homes = new Map();
const report = {
  simulated: true,
  kind: 'scripted workflow proof',
  controls: [],
  checks: [],
  gaps: [],
};
try {
  const first = await Promise.race([
    lines[Symbol.asyncIterator]()
      .next()
      .then((r) => {
        assert.ok(!r.done, 'the world prints its scene');
        return r.value;
      }),
    exited.then((r) => {
      throw new Error(`world ended before its scene: ${JSON.stringify(r)}`);
    }),
  ]);
  const scene = parseScene(first);
  assert.equal(scene.scenes.length, 5);
  for (const m of scene.cast)
    parsePerson(await readFile(resolve(scenes, m.person), 'utf8'), m.person);
  await writeFile(resolve(out, 'scene.json'), `${JSON.stringify(JSON.parse(first), null, 2)}\n`);
  const launched = await launch('computer');
  browser = launched.browser;
  const away = [];
  for (const m of scene.cast) {
    const context = await browser.newContext(launched.options);
    away.push(await reach(context, scene.reach));
    const page = await context.newPage();
    homes.set(m.as, page);
    page.on('pageerror', (error) => console.error(`${m.as}'s page: ${error}`));
    page.setDefaultTimeout(60_000);
    await page.goto(m.url);
    // The current home opens on the groups overview. Enter the workspace
    // through its visible link before looking for a lent app.
    await page.getByRole('link', { name: /^Seed swap/ }).click();
    await page
      .getByRole('link', { name: /notebook/i })
      .first()
      .waitFor();
  }
  const open = async (who, name) => {
    const home = homes.get(who);
    const [page] = await Promise.all([
      home.context().waitForEvent('page'),
      home.getByRole('link', { name }).first().click(),
    ]);
    page.setDefaultTimeout(120_000);
    await page.waitForLoadState();
    return page;
  };
  evidencePath = scene.attachments[0].path;
  const latest = async () => {
    const evidence = JSON.parse(await readFile(evidencePath, 'utf8'));
    return evidence.observations.at(-1)?.observed;
  };
  const until = async (read, check, why) => {
    const end = Date.now() + 120_000;
    let last;
    while (Date.now() < end) {
      last = await read();
      if (check(last)) return last;
      await new Promise((done) => setTimeout(done, 500));
    }
    assert.fail(`${why}: ${JSON.stringify(last)}`);
  };
  const providers = (e) => e?.notebooks.map((n) => n.lender);
  const isJo = (e) => assert.deepEqual(providers(e), ['Jo']);
  const note = 'Seed envelopes are in the blue tin.';
  const ren = await open('Ren', /notebook/i);
  await ren
    .getByText('Seed swap: Saturday at 11, outside number 14. Bring spare envelopes.')
    .waitFor();
  const before = await until(
    latest,
    (e) => providers(e)?.[0] === 'Sam',
    'Sam provides the initial notebook',
  );
  assert.throws(() => isJo(before));
  report.controls.push('successor check rejected the original provider');
  assert.throws(() => assert.ok(before.notebooks[0].read.contains_ren_note));
  report.controls.push('retained-note check rejected the real empty notebook');
  await ren.getByLabel('Your note').fill(note);
  await ren.getByRole('button', { name: 'Add your note', exact: true }).click();
  await ren.getByText(note, { exact: true }).waitFor();
  await until(latest, (e) => e?.notebooks[0]?.read.contains_ren_note, 'Ren writes useful state');
  report.checks.push('Ren reads the seed-swap details and adds a note without keeping the app');

  const sam = await open('Sam', /Give/i);
  const jo = await open('Jo', /Give/i);
  const row = (page) =>
    page
      .locator('li')
      .filter({ has: page.getByRole('heading', { name: 'Neighbourhood notebook', exact: true }) });
  const offer = async () => {
    await row(sam).getByRole('button', { name: 'Move it to…', exact: true }).click();
    await row(sam).getByRole('button', { name: 'Jo’s computer', exact: true }).click();
    await row(jo)
      .getByText('Sam wants to move Neighbourhood notebook to your computer.', { exact: true })
      .waitFor();
  };
  await offer();
  await row(jo).getByRole('button', { name: 'Not now', exact: true }).click();
  await row(sam).getByText('Jo said not now.', { exact: true }).waitFor();
  const declined = await until(
    latest,
    (e) => e?.record.ops.some((o) => o.payload?.kind === 'give/declined'),
    'Jo really declined',
  );
  assert.deepEqual(providers(declined), ['Sam']);
  await ren.reload();
  await ren.getByText(note, { exact: true }).waitFor();
  report.checks.push(
    'Jo declines in Give; Sam still provides the notebook and Ren can still read it',
  );
  await offer();
  assert.deepEqual(providers(await latest()), ['Sam'], 'an offer alone does not move provision');
  await row(jo).getByRole('button', { name: 'Keep it here', exact: true }).click();
  await row(jo)
    .getByText('Neighbourhood notebook · kept by your computer', { exact: true })
    .waitFor();
  await row(sam)
    .getByText('Neighbourhood notebook · kept by Jo’s computer', { exact: true })
    .waitFor();
  const moved = await until(
    latest,
    (e) =>
      providers(e)?.length === 1 &&
      providers(e)[0] === 'Jo' &&
      e.notebooks[0].read.contains_ren_note &&
      (e.pods.Sam.exists === false ||
        (e.pods.Sam.exit === 0 &&
          e.pods.Sam.state &&
          !['Running', 'Degraded'].includes(e.pods.Sam.state))),
    'Jo serves the retained note and Sam has stopped',
  );
  isJo(moved);
  assert.ok(
    moved.pods.Sam.exists === false || moved.pods.Sam.state,
    'only explicit absence or an observed stopped state proves the old app is no longer running',
  );
  await homes.get('Ren').reload();
  const after = await open('Ren', /notebook/i);
  await after.getByText(note, { exact: true }).waitFor();
  await after.screenshot({ path: resolve(out, 'note-after-handover.png'), fullPage: true });
  await jo.screenshot({ path: resolve(out, 'jo-keeps-it.png'), fullPage: true });
  report.checks.push('consensual handover serves Ren’s note from Jo, with Sam’s old pod stopped');

  await homes.get('Jo').reload();
  await homes.get('Jo').getByRole('link', { name: 'This computer', exact: true }).click();
  const stop = homes
    .get('Jo')
    .locator('li')
    .filter({ hasText: /notebook:/ })
    .getByRole('button', { name: 'Stop lending', exact: true });
  for (const detail of await stop.locator('xpath=ancestor::details').all()) {
    if ((await detail.getAttribute('open')) === null)
      await detail.locator(':scope > summary').click();
  }
  await stop.click();
  const unshared = await until(
    latest,
    (e) => Array.isArray(providers(e)) && providers(e).length === 0,
    'the lend disappears',
  );
  assert.equal(unshared.pods.Jo.exit, 0);
  assert.ok(['Running', 'Degraded'].includes(unshared.pods.Jo.state));
  report.gaps.push(
    'Stop lending removes access but Jo’s app is still running; Give offers no stop-keeping action',
  );
  assert.deepEqual(away.flat(), [], 'all browser traffic stayed within declared reach');
  report.status = 'passed';
} catch (error) {
  report.status = 'failed';
  report.error = String(error.stack ?? error);
  report.screens = [];
  for (const [who, home] of homes) {
    for (const [i, page] of home.context().pages().entries()) {
      report.screens.push({
        who,
        tab: i,
        url: page.url(),
        text: await page.locator('body').innerText().catch(String),
      });
      await page
        .screenshot({ path: resolve(out, `${who}-${i}-failure.png`), fullPage: true })
        .catch(() => {});
    }
  }
  process.exitCode = 1;
} finally {
  if (evidencePath)
    await readFile(evidencePath)
      .then((bytes) => writeFile(resolve(out, 'evidence.json'), bytes))
      .catch((error) => {
        report.evidence_error = String(error);
      });
  await browser?.close();
  lines.close();
  world.stdin.end();
  const timer = setTimeout(() => world.kill('SIGTERM'), 30_000);
  report.cleanup = await exited;
  clearTimeout(timer);
  await writeFile(resolve(out, 'report.json'), `${JSON.stringify(report, null, 2)}\n`);
  console.log(JSON.stringify({ ...report, artifacts: out }, null, 2));
}
