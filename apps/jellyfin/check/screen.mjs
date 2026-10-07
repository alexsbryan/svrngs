// u6-2 (DESIGN §3, "The room", call 4; demo/runs/u6/2026-10-06-jellyfin-screen/
// README.md, its parts and how each is read, committed before this):
// Jellyfin's own client for a screen through the room door. The screen is
// jellyfin-web in Playwright's Chromium at the room address, a profile
// kept on disk as a TV's app keeps what it stored; "Jellyfin for WebOS"
// loads the same client from the server once its address is typed. Every
// request it makes off the room address is kept and refused, as Sam's
// Jellyfin across town would not answer a TV at Dee's. Phones come by the
// group's link through the page (person.mjs, way `phone`).
//
//   node screen.mjs a|b|c|d|control '<json: {room, link, server, film, profile, as?, phone_as?, direct?}>'
//   env: SECRETS (the passwords), JF (the lender's Jellyfin), ADMIN (the
//   lender's token, to read its sessions and nothing else), DUMP (where the
//   phones' responses are written, and each screen's requests)
//
//   a        the address typed once: sign in typing `dee`'s, play `film`, seek three times
//   b        a screen new to it signs in by Quick Connect, Dee's phone approving the code
//   c        a phone signed in as `phone_as` plays `film` on the screen of `profile` by "Play on",
//            then pauses and seeks it from jellyfin-web's "Remote Control"; with `direct`, the
//            screen and the phone (a computer's browser) both straight at Sam's Jellyfin
//   d        the screen of `profile` opened again, as the next evening, and left at home 30 s;
//            with BOUNCE (screen.sh's), the room door stopped and started again meanwhile
//   control  the lender ticks "Allow remote control of other users" for `as`, in Jellyfin's pages
//
// Prints one JSON line. A step that did not happen within its time is
// null with what the page showed: never a number guessed.
import fs from 'node:fs';
import { execSync } from 'node:child_process';
import { enter, lender, seek, timed, watch } from './person.mjs';
import { kept, open as openAt, playOn, screen as screenAt, session as sessionOf, until, video } from './tv.mjs';
import { launch } from '../../../web/tests/page.mjs';

const [part, raw] = process.argv.slice(2);
const args = JSON.parse(raw);
const { SECRETS, JF, ADMIN, DUMP } = process.env;
const T0 = Date.now();
const jf = lender(JF, ADMIN);
const out = { part, room: args.room };
const pw = (who) => fs.readFileSync(`${SECRETS}/${who}_pw`, 'utf8').trim();
const screen = (profile) => screenAt(profile, { room: args.room, dir: DUMP, log: `screen-${part}.jsonl`, T0 });
const session = (device) => sessionOf(jf, device);
const open = (s, who) => openAt(s, who, args.room, pw);
const phone = (name, user) => enter({ name, user }, { args: { way: 'phone', engine: 'webkit', at: args.link, server: args.server }, SECRETS, DUMP, T0 });
const done = async (who) => { if (who?.close) await who.close().catch(() => {}); };
const people = [];

try {
  if (part === 'a') {
    const s = await screen(args.profile);
    people.push(s);
    out.screen = await open(s, 'dee');
    if (out.screen.signed_in_ms != null) {
      out.kept = await kept(s.page);
      await s.page.evaluate(([id, sv]) => { location.hash = `#/details?id=${id}&serverId=${sv}`; }, [args.film, args.server]);
      out.play_ms = await timed(s, 'playing', async () => {
        const play = s.page.locator('.mainDetailButtons .btnPlay:visible');
        await play.first().waitFor({ timeout: 30_000 });
        await play.first().click({ force: true });
        const r = await until(s.page, (v) => v && v.ct > 0.5 && !v.paused, 30_000);
        if (r.ms === null) throw new Error(`not playing 30 s after play: ${JSON.stringify(r.video)}`);
      });
      if (out.play_ms !== null) {
        await s.page.evaluate(watch);
        out.seeks = [];
        for (const pct of [50, 15, 80]) {
          out.seeks.push(await s.page.evaluate(seek, pct));
          await s.page.waitForTimeout(3000);
        }
        out.within_5s = out.seeks.filter((x) => x.ms !== null && x.ms <= 5000).length;
        out.session = await session(out.kept.device);
      }
    }
  } else if (part === 'b') {
    const s = await screen(args.profile);
    people.push(s);
    out.screen = await open(s, null);
    out.quick_connect_enabled = await jf('GET', '/QuickConnect/Enabled');
    if (out.screen.quick_connect_offered) {
      await s.page.locator('.btnQuick:visible').click();
      const text = await s.page.waitForFunction(() => {
        const t = document.body.innerText.match(/Enter code (\d[\d ]{5,})/);
        return t && t[1];
      }, null, { timeout: 15_000 }).then((h) => h.jsonValue(), () => null);
      out.code_shown = text !== null;
      if (text !== null) {
        const dee = await phone('Dee', 'dee');
        people.push(dee);
        out.phone = { listed: dee.listed, at: dee.at, signed_in_ms: dee.signed_in_ms, stopped: dee.stopped ?? null };
        if (dee.signed_in_ms != null) {
          out.approved_ms = await timed(dee, 'approved', async () => {
            await dee.page.evaluate(() => { location.hash = '#/quickconnect'; });
            await dee.page.fill('#txtQuickConnectCode', text.replace(/\s/g, ''));
            await dee.page.locator('button[type=submit]:visible').first().click();
            await dee.page.waitForFunction(() => document.body.innerText.includes('successfully authenticated'), null, { timeout: 15_000 });
          });
          const t0 = Date.now();
          out.screen_signed_in_ms = await s.page.waitForSelector('.card', { timeout: 30_000 }).then(() => Date.now() - t0, () => null);
          out.typed_on_screen = 0;
          out.kept = await kept(s.page);
          out.session = out.kept.device ? await session(out.kept.device) : null;
        }
      }
    }
  } else if (part === 'c') {
    // The screen, then a phone in the room as `phone_as` — Ana, another
    // user than the screen's, unless a run says otherwise. With `direct`,
    // the baseline with no svrngs anywhere: the screen and the phone both
    // straight at Sam's Jellyfin, the phone a computer's browser there.
    const s = await screen(args.profile);
    people.push(s);
    out.phone_as = args.phone_as;
    out.direct = !!args.direct;
    out.screen = await open(s, 'dee');
    out.kept = await kept(s.page);
    out.session_before = await session(out.kept.device);
    const ana = args.direct
      ? await enter({ name: 'Ana', user: args.phone_as }, { args: { way: 'computer', engine: 'chromium', at: `${JF}/web/`, server: args.server }, SECRETS, DUMP, T0 })
      : await phone('Ana', args.phone_as);
    people.push(ana);
    out.phone = { at: ana.at, signed_in_ms: ana.signed_in_ms, stopped: ana.stopped ?? null };
    if (ana.signed_in_ms != null) {
      const p = ana.page;
      // Which player jellyfin-web makes the phone's, as it says it.
      out.phone_player = [];
      p.on('console', (m) => { if (/^Active player/.test(m.text())) out.phone_player.push(`${Date.now() - T0} ${m.text().replace(/"id":"[^"]*",/, '').slice(0, 120)}`); });
      // jellyfin-web's own "Play on": "Cast to Device" at the top — the
      // icon, or the player's name once one is picked — then the screen by
      // its name and its user's, as the menu writes them.
      const { cast, header, menu } = playOn(p);
      out.play_on_offered = await cast.first().waitFor({ timeout: 15_000 }).then(() => true, () => false);
      if (out.play_on_offered) {
        out.play_on_lists = await menu();
        const b = out.session_before;
        const pick = b && p.locator('[role=menuitem]:visible', { hasText: b.device_name }).filter({ hasText: b.user });
        out.screen_listed = !!pick && (await pick.count()) > 0;
        if (out.screen_listed) {
          out.picked_at = Date.now() - T0;
          await pick.first().click();
          await p.waitForTimeout(3000);
          out.header_after_pick = await header();
          await p.evaluate(([id, sv]) => { location.hash = `#/details?id=${id}&serverId=${sv}`; }, [args.film, args.server]);
          const play = p.locator('.mainDetailButtons .btnPlay:visible');
          await play.first().waitFor({ timeout: 30_000 });
          out.play_at = Date.now() - T0;
          await play.first().click({ force: true });
          out.started = await until(s.page, (v) => v && v.ct > 0.5 && !v.paused, 30_000);
          out.phone_video = await video(p);
          await p.waitForTimeout(3000);
          // Pause and seek: from jellyfin-web's "Remote Control", which the
          // same button's menu offers while the screen is the phone's player.
          out.header_after_play = await header();
          out.menu_after_play = await menu();
          const rc = p.getByRole('menuitem', { name: 'Remote Control' });
          out.remote_control_offered = (await rc.count()) > 0;
          if (out.remote_control_offered) {
            await rc.first().click();
            const pp = p.locator('.btnPlayPause:visible');
            await pp.first().waitFor({ timeout: 15_000 });
            await pp.first().click();
            out.paused = await until(s.page, (v) => v && v.paused, 30_000);
            const at = (await video(s.page))?.dur;
            out.seek_to_s = at ? +(at / 2).toFixed(1) : null;
            await p.evaluate(() => {
              const sl = document.querySelector('.nowPlayingPositionSlider');
              sl.value = '50';
              sl.dispatchEvent(new Event('input', { bubbles: true }));
              sl.dispatchEvent(new Event('change', { bubbles: true }));
            });
            out.sought = await until(s.page, (v) => v && at && Math.abs(v.ct - at / 2) < 5, 30_000);
            await pp.first().click();
            out.played_on = await until(s.page, (v) => v && !v.paused && at && v.ct > at / 2 - 5, 30_000);
          } else {
            await p.keyboard.press('Escape');
          }
          out.session_after = await session(out.kept.device);
        }
      }
    }
  } else if (part === 'd') {
    const s = await screen(args.profile);
    people.push(s);
    out.screen = await open(s, null);
    await s.page.waitForTimeout(10_000);
    // With BOUNCE, the room door stops and starts again while the screen
    // is open: where does jellyfin-web go to find its server again?
    if (process.env.BOUNCE) {
      const before = s.asked.sockets.length;
      out.bounced_at = Date.now() - T0;
      execSync(process.env.BOUNCE, { stdio: ['ignore', 'ignore', 'inherit'] });
      out.door_back_at = Date.now() - T0;
      await s.page.waitForTimeout(30_000);
      out.sockets_after_bounce = s.asked.sockets.length - before;
    } else {
      await s.page.waitForTimeout(20_000);
    }
    out.kept = await kept(s.page);
    out.public_info = await s.page.evaluate(() => fetch('/System/Info/Public').then((r) => r.json())).catch((e) => String(e));
    out.session = out.kept.device ? await session(out.kept.device) : null;
  } else if (part === 'control') {
    // As the lender ticks it: their own Jellyfin's pages, signed in as themselves.
    const { context, close } = await launch('chromium');
    people.push({ close });
    const page = await context.newPage();
    await page.goto(`${JF}/web/`);
    await page.waitForSelector('#txtManualName, .btnManual', { timeout: 60_000 });
    const manual = page.locator('.btnManual:visible');
    if (await manual.count()) await manual.first().click();
    await page.fill('#txtManualName', 'sam');
    await page.fill('#txtManualPassword', pw('sam'));
    await page.locator('.manualLoginForm button[type=submit]').click();
    await page.waitForSelector('.card', { timeout: 30_000 });
    const id = (await jf('GET', '/Users')).find((u) => u.Name === args.as).Id;
    out.before = (await jf('GET', `/Users/${id}`)).Policy.EnableRemoteControlOfOtherUsers;
    // Dashboard, Users, the person's picture, as lender.mjs finds them.
    await page.getByRole('button', { name: 'User Menu' }).click();
    await page.getByRole('menuitem', { name: 'Dashboard' }).click();
    await page.getByRole('link', { name: 'Users', exact: true }).first().click();
    await page.locator('.card', { hasText: args.as }).locator('a').first().click();
    const box = page.locator('label:has(.chkEnableRemoteControlOtherUsers)');
    await box.waitFor({ timeout: 15_000 });
    out.label = (await box.innerText()).trim();
    if (!(await page.locator('.chkEnableRemoteControlOtherUsers').isChecked())) await box.click();
    await page.getByRole('button', { name: 'Save' }).first().click();
    await page.waitForTimeout(2000);
    out.after = (await jf('GET', `/Users/${id}`)).Policy.EnableRemoteControlOfOtherUsers;
  } else {
    throw new Error(`no part ${part}: a, b, c, d or control`);
  }
} catch (e) {
  out.stopped = String(e).split('\n')[0];
} finally {
  const s = people[0];
  if (s?.asked) out.asked = s.asked;
  if (s?.logs) out.logs = s.logs.slice(-30);
  for (const p of people.slice(1)) if (p.logs) out[`${p.user}_logs`] = p.logs.slice(-20);
  for (const p of people) await done(p);
  console.log(JSON.stringify(out));
}
