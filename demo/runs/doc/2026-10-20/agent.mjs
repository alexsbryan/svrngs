// Fedora's half of the run (PREDICTIONS.md §1): Alex's computer and the
// doc's server, Cy's computer and its browser, and Di's phone. It reads one
// JSON command a line on stdin, `{id, do, ...}`, runs each as it comes (so
// waits on two pages overlap), and answers `{id, ...}` a line on stdout.
// Every computer has a HOME and SVRNGS_DIR of its own under <work>, so the
// machine's own ~/.svrngs and ~/svrngs are never touched.
//
//   node agent.mjs <svrngs binary> <work dir> <apps/doc dir>
//
// Run with no arguments, it is browsers alone: `lend`, `cy` and `lid`
// need all three.
import { spawn, spawnSync } from 'node:child_process';
import { existsSync, mkdirSync, readFileSync } from 'node:fs';
import { createInterface } from 'node:readline';
import { person } from './people.mjs';

const [BIN, W, DOC] = process.argv.slice(2);
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const env = (who) => ({ ...process.env, HOME: `${W}/${who}`, USER: who, SVRNGS_DIR: `${W}/${who}/.svrngs`, RUST_LOG: 'info,door=debug,dial=debug,redeem=debug', NO_COLOR: '1' });
const said = {};
const procs = {};
const people = {};
let link = null;
let docPort = null;
let origin = null;

function run(who, cmd, args, e = env(who)) {
  mkdirSync(`${W}/${who}`, { recursive: true });
  const p = spawn(cmd, args, { env: e, stdio: ['ignore', 'pipe', 'pipe'] });
  said[who] = '';
  p.stdout.on('data', (d) => (said[who] += d));
  p.stderr.on('data', (d) => (said[who] += d));
  procs[who] = p;
  return p;
}
const api = (who) => (existsSync(`${W}/${who}/.svrngs/api`) ? readFileSync(`${W}/${who}/.svrngs/api`, 'utf8').trim() : null);
async function until(f, ms) {
  const t = Date.now();
  for (;;) {
    const v = await f().catch(() => null);
    if (v) return v;
    if (Date.now() - t > ms) return null;
    await sleep(200);
  }
}
const json = async (url) => (await fetch(url)).json();

const commands = {
  now: async () => ({ now: Date.now() }),
  async lend() {
    const t0 = Date.now();
    run('alex', BIN, ['new', 'House', '--as', 'Alex', '--origin', 'https://group.svrnme.sh/']);
    link = await until(async () => said.alex.match(/https:\/\/group\.svrnme\.sh\/#\S+/)?.[0], 60_000);
    if (!link) return { failed: 'no link from `svrngs new`', said: said.alex.slice(-600) };
    run('server', process.execPath, [`${DOC}/server.mjs`, '--port', '0', '--keep', `${W}/keep`, '--svrngs', `${W}/alex/.svrngs/api`], { ...process.env, DOC_DEBUG: '1' });
    docPort = await until(async () => said.server.match(/127\.0\.0\.1:(\d+)/)?.[1], 10_000);
    const share = spawnSync(BIN, ['share', docPort, '--name', 'doc', '--kind', 'doc'], { env: env('alex'), input: '', encoding: 'utf8', timeout: 30_000 });
    return { ms: Date.now() - t0, link, port: Number(docPort), share: share.status, share_said: (share.stdout + share.stderr).trim().slice(-400) };
  },
  async cy() {
    const t0 = Date.now();
    run('cy', BIN, ['join', link, '--as', 'Cy']);
    const lend = await until(async () => {
      const a = api('cy');
      const g = a && (await json(`${a}/v1/groups`)).groups?.[0];
      return g && (await json(`${a}/v1/lends?namespace=${g.id}`)).lent?.find((l) => l.name === 'doc');
    }, 90_000);
    if (!lend) return { failed: 'Cy never listed the doc', said: said.cy?.slice(-600) };
    const joined = Date.now() - t0;
    origin = lend.origin;
    people.cy = await person({ who: 'cy', engine: 'chromium' });
    return { joined_ms: joined, lender: lend.person, origin: lend.origin, opened: await people.cy.open(lend.origin) };
  },
  async phone({ who, name, device }) {
    people[who] = await person({ who, engine: 'chromium', device });
    const joined = await people[who].join(link, name);
    return { joined, pressed: await people[who].press('doc') };
  },
  // A page opened afresh on Cy's computer, after everything: what the
  // server kept is what it reads.
  async fresh({ texts }) {
    const p = await person({ who: 'cy2', engine: 'chromium' });
    const opened = await p.open(origin);
    const has = await p.has(texts, 30_000);
    const read = await p.read();
    await p.close();
    return { opened, has, paragraphs: read.paragraphs, note_lines: read.note_lines };
  },
  // A browser alone, for a driver that runs its svrngs itself (the
  // scenes, crates/svrngs/tests/scenes/doc.rs): `person` opens one, `open`,
  // `join` and `press` are people.mjs's.
  person: async ({ who, engine, device }) => ((people[who] = await person({ who, engine, device })), {}),
  open: ({ who, url }) => people[who].open(url),
  join: ({ who, link, name }) => people[who].join(link, name),
  press: ({ who, name }) => people[who].press(name),
  type: ({ who, text }) => people[who].type(text),
  has: ({ who, texts, within }) => people[who].has(texts, within),
  shows: ({ who, sel, shown, within }) => people[who].shows(sel, shown, within),
  says: ({ who, sel, re, within }) => people[who].says(sel, re, within),
  read: ({ who }) => people[who].read(),
  shot: async ({ who, path }) => (await people[who].shot(path), { path }),
  // The lid: Alex's svrngs and the doc's server stopped together, as a
  // closed lid stops every process on the laptop.
  lid: async ({ open }) => {
    for (const who of ['alex', 'server']) procs[who].kill(open ? 'SIGCONT' : 'SIGSTOP');
    return { at: Date.now(), open: Boolean(open) };
  },
  acts: async () => ({
    acts: readFileSync(`${W}/keep/acts.jsonl`, 'utf8').split('\n').filter(Boolean).map((l) => {
      const a = JSON.parse(l);
      return { seq: a.seq, person: a.person, ts: a.ts };
    }),
  }),
  async end() {
    for (const p of Object.values(people)) await p.close().catch(() => {});
    for (const [who, p] of Object.entries(procs)) {
      p.kill('SIGCONT');
      p.kill('SIGTERM');
    }
    await sleep(1500);
    return { logs: Object.fromEntries(Object.entries(said).map(([k, v]) => [k, v.length])) };
  },
};

const out = (o) => process.stdout.write(`${JSON.stringify(o)}\n`);
out({ ready: true, load: (await import('node:os')).loadavg()[0] });
for await (const line of createInterface({ input: process.stdin })) {
  const c = JSON.parse(line);
  (async () => {
    try {
      out({ id: c.id, ...(await commands[c.do](c)) });
    } catch (e) {
      out({ id: c.id, failed: String(e).slice(0, 600) });
    }
    if (c.do === 'end') {
      const { writeFileSync } = await import('node:fs');
      for (const [who, s] of Object.entries(said)) writeFileSync(`${W}/${who}.log`, s);
      process.exit(0);
    }
  })();
}
