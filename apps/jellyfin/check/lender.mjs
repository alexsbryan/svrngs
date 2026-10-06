// The lender makes each person an account, as `svrngs share 8096` tells
// them to (crates/svrngs/src/well_known.txt, its jellyfin row; u5-4): in
// their own Jellyfin's pages, on their own computer, signed in as
// themselves. Each step is found by what the printed line names, so a
// line that drifts from Jellyfin's pages, or pages that drift from the
// line, stop here naming the step.
//
//   node lender.mjs '<the line share printed>' <jellyfin> <lender> <person>...
//   env: SECRETS (<lender>_pw, and each <person>_pw, written here if absent)
//
// Prints one JSON line: each person made, or where it stopped.
import fs from 'node:fs';
import crypto from 'node:crypto';
import { launch } from '../../../web/tests/page.mjs';

const [said, at, lender, ...people] = process.argv.slice(2);
const { SECRETS } = process.env;
const out = { said, made: [], steps: [] };
const named = (words) => {
  if (!said.includes(words)) throw new Error(`the line share printed does not say ${JSON.stringify(words)}: ${said}`);
  out.steps.push(words);
};
const { context, close } = await launch('chromium');
const page = await context.newPage();
try {
  await page.goto(`${at}/web/`);
  await page.waitForSelector('#txtManualName, .btnManual', { timeout: 60_000 });
  const manual = page.locator('.btnManual:visible');
  if (await manual.count()) await manual.first().click();
  await page.fill('#txtManualName', lender);
  await page.fill('#txtManualPassword', fs.readFileSync(`${SECRETS}/${lender}_pw`, 'utf8').trim());
  await page.locator('.manualLoginForm button[type=submit]').click();
  await page.waitForSelector('.card', { timeout: 30_000 });
  for (const who of people) {
    const file = `${SECRETS}/${who}_pw`;
    if (!fs.existsSync(file)) fs.writeFileSync(file, crypto.randomBytes(18).toString('base64url'), { mode: 0o600 });
    out.steps = [];
    named('your picture at the top right');
    await page.getByRole('button', { name: 'User Menu' }).or(page.locator('header button').last()).first().click();
    named('Dashboard');
    await page.getByRole('menuitem', { name: 'Dashboard' }).click();
    named('Users');
    await page.getByRole('link', { name: 'Users', exact: true }).first().click();
    named('+');
    await page.locator('button[title="Add User"]').click();
    named('a name and a password');
    await page.fill('#txtUsername', who);
    await page.fill('#txtPassword', fs.readFileSync(file, 'utf8').trim());
    named('tick the libraries they may watch');
    // Every library the page lists, each a box of its own beside "Enable
    // access to all libraries"; none ticked until the lender ticks it.
    // The libraries come a moment after the form. The box itself is drawn
    // over, so a person, and this, taps its label.
    await page.waitForFunction(() => [...document.querySelectorAll('label')].some((l) =>
      l.offsetParent && l.querySelector('input[type=checkbox]') && !/^enable access to all/i.test(l.innerText.trim())), null, { timeout: 15_000 });
    const boxes = page.locator('label:visible:has(input[type=checkbox])');
    let n = 0;
    for (let i = 0; i < (await boxes.count()); i++) {
      const box = boxes.nth(i);
      if (/^enable access to all/i.test((await box.innerText()).trim())) continue;
      await box.click();
      if (!(await box.locator('input').isChecked())) throw new Error(`${await box.innerText()} did not tick`);
      n++;
    }
    if (!n) throw new Error('no library to tick');
    named('Save');
    await page.getByRole('button', { name: 'Save' }).click();
    await page.waitForURL((u) => !u.hash.includes('/users/add'), { timeout: 15_000 });
    out.made.push({ who, libraries_ticked: n, steps: out.steps });
    await page.goto(`${at}/web/#/home`);
    await page.waitForSelector('.card', { timeout: 30_000 });
  }
} catch (e) {
  out.stopped = `${String(e.message).split('\n')[0]}; after ${JSON.stringify(out.steps)}`;
  await page.screenshot({ path: `${SECRETS}/../lender-stopped.png` }).catch(() => {});
} finally {
  delete out.steps;
  await close();
  console.log(JSON.stringify(out));
  process.exitCode = out.stopped ? 1 : 0;
}
