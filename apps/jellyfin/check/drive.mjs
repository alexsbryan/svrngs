// Two people through one lend (check.sh `pair`; forked from u5-3's, at
// demo/runs/u5/2026-10-05-jellyfin/drive.mjs, its bars (a)-(c), u5-4's
// row's check). Each person is a browser of their own, Playwright's headless
// Chromium or its WebKit as an iPhone (web/tests/page.mjs): on `computer`
// they open the lend's origin on Alex's loopback, as the bare command
// prints it; on `phone` they open the group's link, give a name, and open
// the lend from the list. Each signs in to jellyfin-web with the account
// the lender made, typing it, and plays their film. Then the first seeks
// through the player's own seek bar while the second is watched, and the
// other way round; then the lender's Jellyfin sends each person's session
// a message, which only the live socket carries; then each asks who they
// are signed in as.
//
//   node drive.mjs '{"way": "computer"|"phone", "engine": "chromium"|"webkit",
//     "at": "<the lend's origin, or the group's link>", "server": "<Jellyfin's id>",
//     "people": [{"name", "user", "film", "seeks": [percent, ...]}, {...}]}'
//   env: SECRETS (the passwords), JF (the lender's Jellyfin, for its
//   message to each session), ADMIN (the lender's token, for that alone),
//   DUMP (where every response each browser receives is written, for scan.py)
//   A person may sign in with an `"account"` not their own (`user`, by
//   which they are keyed): check.sh's one-device plant, one account for
//   everyone, as U3's dry run's lend had. With `"seed": "<a Jellyfin user
//   id>"`, the web client's stored sign-in names that user with the
//   placeholder token `door` before the page loads, as U3's dry run's
//   driver wrote it (demo/runs/a/dry-2026-10-04/drive.mjs:41-47): what no
//   person can write, and what a lend keeping a token needs to play at all
//   (u5-3's probe); the plants alone use it.
//
// Prints one JSON line. A step that did not happen within its time is
// null with what the page showed: never a number guessed.
import { enter, judge, lender, seek, socket } from './person.mjs';

const args = JSON.parse(process.argv[2]);
const { SECRETS, JF, ADMIN, DUMP } = process.env;
const out = { way: args.way, engine: args.engine, people: {} };
// Every time this run reports is in ms from here.
const T0 = Date.now();
const jf = lender(JF, ADMIN);

// One seeks through `seeks` while the other plays, watched.
// With nobody else, the seeks alone.
async function phase(seeker, watched, seeks) {
  const since = watched && (await watched.page.evaluate(() => performance.now()));
  const done = [];
  for (const pct of seeks) {
    done.push(await seeker.page.evaluate(seek, pct));
    await seeker.page.waitForTimeout(3000);
  }
  return { seeker: seeker.user, seeks: done, within_5s: done.filter((s) => s.ms !== null && s.ms <= 5000).length,
    watched: watched?.user ?? null, watch: watched ? await watched.page.evaluate(judge, since) : null };
}

const people = [];
try {
  for (const p of args.people) people.push(await enter(p, { args, SECRETS, DUMP, T0 }));
  const [a, b] = people;
  const playing = people.every((p) => p.play_ms != null);
  if (playing) {
    out.first_seeks = await phase(a, b, args.people[0].seeks);
    if (b) out.then_seeks = await phase(b, a, args.people[1].seeks);
  }
  const all = await jf('GET', '/Sessions');
  out.devices_apart = new Set(people.map((p) => p.device)).size === people.length;
  for (const p of people) {
    if (p.signed_in_ms == null || p.stopped) continue;
    p.socket = await socket(jf, p).catch((e) => ({ error: String(e) }));
    p.signed_in_after = await p.page.evaluate(async () => {
      try { const u = await window.ApiClient.getJSON(window.ApiClient.getUrl('Users/Me')); return u.Name; } catch (e) { return `refused: ${e?.status ?? e}`; }
    }).catch((e) => `could not ask: ${e}`);
    p.sessions = all.filter((s) => s.DeviceId === p.device).map((s) => ({ user: s.UserName, playing: !!s.NowPlayingItem, client: s.Client }));
  }
} catch (e) {
  out.stopped = String(e).split('\n')[0];
} finally {
  for (const p of people) {
    const { page, close, token, tally, ...rest } = p;
    out.people[p.user] = { ...rest, token: token ? 'held' : null, received: { ...tally }, logs: p.logs.slice(-30) };
    await close();
  }
  console.log(JSON.stringify(out));
}
