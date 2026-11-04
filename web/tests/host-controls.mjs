// The onboarding host controls against real native routes and a fake resolver.
import assert from 'node:assert/strict';
import { launch } from './page.mjs';

const args = JSON.parse(process.argv[2]);
const { context, close } = await launch(args.engine);
try {
  await context.grantPermissions(['clipboard-read', 'clipboard-write']);
  const page = await context.newPage();
  const sent = [];
  page.on('request', (r) => sent.push([r.method(), r.url()]));
  await page.goto(args.ask);
  const controls = page.locator('.publication');
  const words = {
    pending: 'Invite people here',
    open: 'Retry joining',
    closing: 'Close joining',
    closed: 'Retry closing',
    inspect: 'Joining is closed',
  };
  await controls.getByText(words[args.phase], { exact: true }).waitFor();
  if (args.phase === 'closing') {
    assert.equal(await controls.locator('code').textContent(), args.address);
    await controls.getByRole('button', { name: 'Copy the address', exact: true }).click();
    assert.equal(await page.evaluate(() => navigator.clipboard.readText()), args.address);
    assert.ok(
      await controls
        .locator('code')
        .evaluate((e) => parseFloat(getComputedStyle(e).fontSize) >= 20),
    );
    assert.match(await controls.textContent(), /Ends .+This computer must remain reachable/);
  }
  if (args.phase !== 'inspect') {
    await controls.getByRole('button', { name: words[args.phase], exact: true }).click();
    const expected = ['pending', 'closing'].includes(args.phase)
      ? 'joining is temporarily unavailable'
      : args.phase === 'open'
        ? args.address
        : 'Joining is closed';
    await controls.getByText(expected, { exact: false }).first().waitFor();
    if (['pending', 'closing'].includes(args.phase)) {
      assert.doesNotMatch(await controls.textContent(), /invitation ended|Joining is closed/);
      await controls
        .getByRole('button', {
          name: args.phase === 'closing' ? 'Retry closing' : 'Retry joining',
          exact: true,
        })
        .waitFor();
    }
  }
  const text = await controls.textContent();
  await page.reload();
  const after =
    args.phase === 'pending'
      ? 'Retry joining'
      : args.phase === 'closing'
        ? 'Retry closing'
        : args.phase === 'open'
          ? args.address
          : 'Joining is closed';
  await controls.getByText(after, { exact: false }).first().waitFor();
  assert.ok(
    sent.every(([, url]) => new URL(url).origin === new URL(args.ask).origin),
    'browser only asks its native origin',
  );
  console.log(
    JSON.stringify({
      text,
      reloaded: await controls.textContent(),
      sent: sent.filter(([, url]) => new URL(url).pathname.startsWith('/publication')),
    }),
  );
} finally {
  await close();
}
