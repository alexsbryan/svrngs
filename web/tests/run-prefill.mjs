// Browser half of run_prefill_waits_for_confirmation_in_webkit: a stand-in
// llama-server, started by hand, is found and lent, its fields filled with
// the name svrngs's table knows it by; its lend's row offers to keep it
// running, and Keep it running shows the command it was started with and
// its folder, the keeper's suggestion, behind a Change fold too; nothing
// is started until Start, and Start, with the hand-started copy still on
// the port, is refused, the copy left alone.
import assert from 'node:assert/strict';
import { launch } from './page.mjs';

const { engine, ask, port, folder, command: started, plant } = JSON.parse(process.argv[2]);
const { context, close } = await launch(engine);
try {
  await context.addInitScript(() => {
    const fetch = window.fetch.bind(window);
    window.runPosts = 0;
    window.fetch = (path, init) => {
      if (path === '/run' && init?.method === 'POST') window.runPosts++;
      return fetch(path, init);
    };
  });
  const page = await context.newPage();
  await page.goto(ask);
  await page.getByRole('button', { name: 'Look for programs running on this computer' }).click();
  // A program found is a line, its port's digits last, its form folded under it.
  const row = page.locator('#running li', { hasText: new RegExp(`localhost:${port}(?!\\d)`) });
  await row.locator('summary').click();
  for (const field of ['name', 'kind'])
    assert.equal(await row.locator(`input[name=${field}]`).inputValue(), 'llama-server', field);
  await row.locator('input[type=checkbox]').check();
  // Its review first, then lent: the row then says to whom, its address gone.
  const lit = page.locator('#running p.lit');
  for (let press = 0; press < 3 && (await lit.count()) === 0; press++) {
    const before = await row.textContent();
    await row.getByRole('button').click();
    await page.waitForFunction(
      ([at, was]) => {
        const now = [...document.querySelectorAll('#running li')].find((li) =>
          li.textContent.includes(at),
        );
        return !now || (now.textContent !== was && !now.textContent.includes('…'));
      },
      [`localhost:${port}`, before],
    );
  }
  assert.match(await lit.textContent(), /^Lent to /);

  const offer = page.locator("#lent [role=group][aria-label='llama-server']");
  await offer.getByRole('button', { name: 'Keep it running' }).click();
  const shown = offer.locator('code');
  await shown.first().waitFor();
  const [command, at] = await shown.allTextContents();
  assert.equal(command, started);
  assert.equal(at, folder);
  await offer.locator('summary', { hasText: 'Change' }).click();
  assert.equal(await offer.locator('input[name=command]').inputValue(), command);
  assert.equal(await offer.locator('input[name=folder]').inputValue(), folder);
  if (plant === 'start') await offer.getByRole('button', { name: 'Start' }).click();
  assert.equal(await page.evaluate(() => window.runPosts), 0, 'started without a press of Start');

  await offer.getByRole('button', { name: 'Start' }).click();
  await offer.locator('[role=status]', { hasText: 'is in use by' }).waitFor();
  assert.equal(await page.evaluate(() => window.runPosts), 1);
  console.log(
    JSON.stringify({
      command,
      folder,
      port,
      refused: await offer.locator('[role=status]').textContent(),
      confirmedPosts: 1,
    }),
  );
} finally {
  await close();
}
