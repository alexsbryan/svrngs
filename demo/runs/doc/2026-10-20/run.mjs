// The run (PREDICTIONS.md), driven from the Mac: Bo's phone here, the rest
// on Fedora through agent.mjs over SSH. It writes every command and answer
// to <data>/events.jsonl, what each bar read to summary.json, and the
// verdicts to verdicts.json.
//
//   PLAYWRIGHT=<web/node_modules/playwright/index.mjs> node run.mjs <data dir> [<fedora dir>]
//
// <fedora dir> (~/doc-run) holds agent.mjs, people.mjs, apps/doc at the
// commit under test, bin/svrngs (the published V1.1.2 Linux download) and
// Playwright with its Chromium.
import { spawn, spawnSync } from 'node:child_process';
import { appendFileSync, mkdirSync, writeFileSync } from 'node:fs';
import { loadavg } from 'node:os';
import { createInterface } from 'node:readline';
import { person } from './people.mjs';

const [DATA, FEDORA_DIR = 'doc-run'] = process.argv.slice(2);
const HOST = process.env.FEDORA ?? 'alexbryan@fedora';
mkdirSync(`${DATA}/shots`, { recursive: true });
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const log = (o) => appendFileSync(`${DATA}/events.jsonl`, `${JSON.stringify({ t: Date.now(), ...o })}\n`);

// Fedora's agent: one command a line out, answers matched by id.
const agent = spawn('ssh', ['-o', 'BatchMode=yes', HOST, `cd ${FEDORA_DIR} && rm -rf w && mkdir -p w/shots && exec node agent.mjs bin/svrngs $HOME/${FEDORA_DIR}/w $HOME/${FEDORA_DIR}/apps/doc`], { stdio: ['pipe', 'pipe', 'inherit'] });
const waiting = new Map();
let ready;
const isReady = new Promise((r) => (ready = r));
createInterface({ input: agent.stdout }).on('line', (line) => {
  const a = JSON.parse(line);
  if (a.ready) return ready(a);
  waiting.get(a.id)?.(a);
});
let ids = 0;
function fedora(c) {
  const id = (ids += 1);
  log({ to: 'fedora', ...c, id });
  return new Promise((done) => {
    waiting.set(id, (a) => (log({ from: 'fedora', ...a }), done(a)));
    agent.stdin.write(`${JSON.stringify({ ...c, id })}\n`);
  });
}
const here = async (what, f) => {
  log({ to: 'mac', what });
  const a = await f();
  log({ from: 'mac', what, ...a });
  return a;
};

const NAMES = { bo: 'Bo', cy: 'Cy', di: 'Di' };
const S = { start: Date.now(), mac_load: loadavg()[0] };
try {
  S.fedora_load = (await isReady).load;
  // Fedora's clock against the Mac's: the round trip with the least time,
  // its middle taken as when Fedora read its clock.
  const pings = [];
  for (let i = 0; i < 5; i += 1) {
    const t0 = Date.now();
    const { now } = await fedora({ do: 'now' });
    const t1 = Date.now();
    pings.push({ rtt: t1 - t0, offset: now - (t0 + t1) / 2 });
  }
  const best = pings.sort((a, b) => a.rtt - b.rtt)[0];
  S.clock = { fedora_minus_mac_ms: Math.round(best.offset), rtt_ms: best.rtt };
  const mac = (fedoraAt) => fedoraAt - best.offset;

  S.page = spawnSync('sh', ['-c', 'curl -fsSL https://group.svrnme.sh/ | shasum -a 256; curl -fsSL https://group.svrnme.sh/ | grep -o "app.js?v=[^\\"]*" | head -1'], { encoding: 'utf8' }).stdout.trim();
  S.lend = await fedora({ do: 'lend' });
  if (!S.lend.link) throw new Error(`no link: ${JSON.stringify(S.lend)}`);

  // Bar 1: everyone opens the doc.
  const bo = await person({ who: 'bo', engine: 'webkit', device: 'iPhone 13' });
  const [boOpen, cyOpen, diOpen] = await Promise.all([
    here('bo opens', async () => ({ joined: await bo.join(S.lend.link, 'Bo'), pressed: await bo.press('doc') })),
    fedora({ do: 'cy' }),
    fedora({ do: 'phone', who: 'di', name: 'Di', device: 'Pixel 7' }),
  ]);
  S.open = { bo: boOpen, cy: cyOpen, di: diOpen };

  // Bar 2: named, and everyone here.
  const ask = (who, c) => (who === 'bo' ? here(`${who} ${c.do}`, () => bo[c.do](...c.args)) : fedora({ do: c.do, who, ...c.named }));
  const says = (who, sel, re, within) => ask(who, { do: 'says', args: [sel, re, within], named: { sel, re, within } });
  const has = (who, texts, within) => ask(who, { do: 'has', args: [texts, within], named: { texts, within } });
  const shows = (who, sel, shown, within) => ask(who, { do: 'shows', args: [sel, shown, within], named: { sel, shown, within } });
  const read = (who) => (who === 'bo' ? here('bo read', () => bo.read()) : fedora({ do: 'read', who }));
  const type = (who, text) => (who === 'bo' ? here('bo type', () => bo.type(text)) : fedora({ do: 'type', who, text }));
  const at = (who, a) => (who === 'bo' ? a.at : a.at && mac(a.at));
  const all = (f) => Promise.all(['bo', 'cy', 'di'].map(f));

  S.named = await all((w) => says(w, '#you', `You’re ${NAMES[w]}\\.`, 5_000));
  S.everyone = await all((w) => says(w, '#here', 'Bo, Cy, Di', 10_000));
  S.read_open = await all(read);
  await here('shot', () => bo.shot(`${DATA}/shots/bo-open.png`));

  // Bar 3: three edits at once.
  const T = { bo: 'Bo brings chips', cy: 'Cy brings drinks', di: 'Di brings a speaker' };
  const typed = await all((w) => type(w, T[w]));
  S.typed = typed;
  const seen = await all((w) => has(w, Object.values(T), 10_000));
  S.live = ['bo', 'cy', 'di'].map((w, i) => {
    const others = ['bo', 'cy', 'di'].filter((o) => o !== w);
    const lastOther = Math.max(...others.map((o) => at(o, typed[['bo', 'cy', 'di'].indexOf(o)])));
    const got = at(w, seen[i]);
    return { reader: w, ms_after_last_other_keystroke: got ? Math.round(got - lastOther) : null };
  });

  // Bar 4: the notes.
  S.notes_wait = await all((w) => says(w, '#attribution', '^(?=[\\s\\S]*Bo)(?=[\\s\\S]*Cy)(?=[\\s\\S]*Di)', 3_000));
  await sleep(2_000);
  S.read_typed = await all(read);
  await here('shot', () => bo.shot(`${DATA}/shots/bo-written.png`));
  await fedora({ do: 'shot', who: 'di', path: `w/shots/di-written.png` });
  await fedora({ do: 'shot', who: 'cy', path: `w/shots/cy-written.png` });

  // Bar 5: the lid.
  const lid = await fedora({ do: 'lid', open: false });
  S.lid = { closed: mac(lid.at) };
  const night = await all((w) => shows(w, '#night', true, 15_000));
  S.goodnight = ['bo', 'cy', 'di'].map((w, i) => ({ who: w, ms_after_lid: at(w, night[i]) ? Math.round(at(w, night[i]) - S.lid.closed) : null }));
  S.read_night = await all(read);

  // Bar 6: writing under the goodnight.
  const A = { bo: 'Bo, away: ice', cy: 'Cy, away: cups', di: 'Di, away: a playlist' };
  S.typed_away = await all((w) => type(w, A[w]));
  S.kept_own = await all((w) => has(w, [A[w]], 3_000));
  S.kept_said = await all((w) => says(w, '#kept', 'kept here', 3_000));
  S.read_away = await all(read);
  await here('shot', () => bo.shot(`${DATA}/shots/bo-goodnight.png`));
  await fedora({ do: 'shot', who: 'di', path: `w/shots/di-goodnight.png` });

  // Bar 7: the lid opens 45 s after it closed.
  await sleep(Math.max(0, S.lid.closed + 45_000 - Date.now()));
  const opened = await fedora({ do: 'lid', open: true });
  S.lid.opened = mac(opened.at);
  const SIX = [...Object.values(T), ...Object.values(A)];
  const [gone, whole] = await Promise.all([all((w) => shows(w, '#night', false, 60_000)), all((w) => has(w, SIX, 60_000))]);
  S.joined = ['bo', 'cy', 'di'].map((w, i) => ({
    who: w,
    goodnight_gone_ms: at(w, gone[i]) ? Math.round(at(w, gone[i]) - S.lid.opened) : null,
    all_six_ms: at(w, whole[i]) ? Math.round(at(w, whole[i]) - S.lid.opened) : null,
  }));
  await sleep(1_500);
  S.read_joined = await all(read);
  await here('shot', () => bo.shot(`${DATA}/shots/bo-joined.png`));

  // Bar 8: what the server kept.
  S.fresh = await fedora({ do: 'fresh', texts: SIX });
  S.acts = await fedora({ do: 'acts' });
  S.six = SIX;
  S.bo_errors = bo.errors;
  await bo.close();
} catch (e) {
  S.failed = String(e.stack ?? e).slice(0, 2000);
} finally {
  S.end = Date.now();
  S.end_mac_load = loadavg()[0];
  S.ended = await Promise.race([fedora({ do: 'end' }), sleep(20_000).then(() => ({ failed: 'the agent did not end' }))]);
  writeFileSync(`${DATA}/summary.json`, JSON.stringify(S, null, 1));
  agent.stdin.end();
  process.exit(0);
}
