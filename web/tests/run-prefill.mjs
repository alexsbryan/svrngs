// Browser half of run_prefill_waits_for_confirmation_in_webkit.
import assert from 'node:assert/strict';
import { launch } from './page.mjs';

const { engine, ask, port, other, runPort, folder, plant } = JSON.parse(process.argv[2]);
const { context, close } = await launch(engine);
try {
  // Hold only the selected answer, after the real request. A task after
  // json() lets us check the form once the page has handled the answer.
  await context.addInitScript(
    ({ plant }) => {
      const fetch = window.fetch.bind(window);
      window.runPosts = 0;
      window.fetch = async (path, init) => {
        if (path === '/run' && init?.method === 'POST') window.runPosts++;
        const hold =
          window.holdPrefill && typeof path === 'string' && path.startsWith('/run?prefill=');
        if (!hold) return fetch(path, init);
        window.holdPrefill = false;
        const released = new Promise((resolve) => (window.releasePrefill = resolve));
        const response = await fetch(path, init);
        await released;
        const json = response.json.bind(response);
        response.json = async () => {
          const body = await json();
          setTimeout(() => (window.prefillDelivered = true), 0);
          return body;
        };
        return response;
      };
      if (plant === 'edit')
        document.addEventListener('input', (event) => event.stopPropagation(), true);
    },
    { plant },
  );
  const page = await context.newPage();
  await page.goto(ask);
  const label = await page.locator('#run-form summary').textContent();
  await page.getByRole('button', { name: 'Look for programs running on this computer' }).click();
  const row = (p) =>
    page.locator('#running li', { hasText: `localhost:${p}.` }).getByRole('link', { name: label });
  const form = page.locator('#run-form form');
  const field = (name) => form.locator(`input[name=${name}]`);
  const hold = () =>
    page.evaluate(() => {
      window.holdPrefill = true;
      window.prefillDelivered = false;
    });
  const release = async () => {
    await page.evaluate(() => window.releasePrefill());
    await page.waitForFunction(() => window.prefillDelivered);
  };
  const askOther = async () => {
    const answered = page.waitForResponse((reply) => reply.url().endsWith(`?prefill=${other}`));
    await row(other).click();
    assert.equal((await (await answered).json()).kind, 'unavailable');
  };

  await hold();
  await row(port).click();
  await field('command').fill('Sam’s own command');
  await release();
  assert.equal(
    await field('command').inputValue(),
    'Sam’s own command',
    'late answer replaced Sam’s edit',
  );

  await hold();
  await row(port).click();
  await askOther();
  await release();
  assert.equal(await field('command').inputValue(), '', 'old answer filled another row');

  await row(port).click();
  await page.waitForFunction(() =>
    document.querySelector('#run-form input[name=command]').value.includes('model.gguf'),
  );
  assert.equal(await field('name').inputValue(), 'llama-server');
  assert.equal(await field('port').inputValue(), String(port));
  assert.equal(await field('folder').inputValue(), folder);
  const command = await field('command').inputValue();
  assert.ok(
    command.startsWith('llama-server --model ') &&
      command.endsWith(` --host 127.0.0.1 --port ${port}`),
    command,
  );
  if (plant === 'submit') await form.evaluate((element) => element.requestSubmit());
  assert.equal(
    await page.evaluate(() => window.runPosts),
    0,
    'Run submitted without a confirming press',
  );

  await field('name').fill('test-page-coder');
  await field('port').fill(String(runPort));
  await field('command').fill(`/usr/bin/python3 -m http.server ${runPort} --bind 127.0.0.1`);
  await form.getByRole('button', { name: label }).click();
  await page
    .locator('#run-form [role=status]', { hasText: 'test-page-coder is running here.' })
    .waitFor();
  assert.equal(await page.evaluate(() => window.runPosts), 1);
  console.log(
    JSON.stringify({
      command,
      folder,
      port,
      lateEdit: 'preserved',
      otherRow: 'blank command',
      confirmedPosts: 1,
    }),
  );
} finally {
  await close();
}
