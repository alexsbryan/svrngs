// Repro: a phone's doc socket after a 45 s lid. Fedora alone, Di as Chromium Pixel.
import { spawn, spawnSync } from 'node:child_process';
import { mkdirSync } from 'node:fs';
const pw = await import('playwright');
const [BIN, W, DOC, LID_S = '45'] = process.argv.slice(2);
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const env = (who) => ({ ...process.env, HOME: `${W}/${who}`, USER: who, SVRNGS_DIR: `${W}/${who}/.svrngs`, RUST_LOG: 'info,door=debug,dial=debug', NO_COLOR: '1' });
const said = {}; const procs = {};
function run(who, cmd, args, e = env(who)) { mkdirSync(`${W}/${who}`, { recursive: true }); const p = spawn(cmd, args, { env: e, stdio: ['ignore', 'pipe', 'pipe'] }); said[who] = ''; p.stdout.on('data', (d) => (said[who] += d)); p.stderr.on('data', (d) => (said[who] += d)); procs[who] = p; return p; }
async function until(f, ms) { const t = Date.now(); for (;;) { const v = await f(); if (v) return v; if (Date.now() - t > ms) return null; await sleep(200); } }
const log = (o) => console.log(JSON.stringify({ t: new Date().toISOString().slice(11, 23), ...o }));
run('alex', BIN, ['new', 'House', '--as', 'Alex', '--origin', 'https://group.svrnme.sh/']);
const link = await until(async () => said.alex.match(/https:\/\/group\.svrnme\.sh\/#\S+/)?.[0], 60000);
run('server', process.execPath, [`${DOC}/server.mjs`, '--port', '0', '--keep', `${W}/keep`, '--svrngs', `${W}/alex/.svrngs/api`], { ...process.env, DOC_DEBUG: '1' });
const port = await until(async () => said.server.match(/127\.0\.0\.1:(\d+)/)?.[1], 10000);
spawnSync(BIN, ['share', port, '--name', 'doc', '--kind', 'doc'], { env: env('alex'), input: '', encoding: 'utf8' });
const b = await pw.chromium.launch(); const ctx = await b.newContext(pw.devices['Pixel 7']); const page = await ctx.newPage();
await page.goto(link); await page.waitForSelector('#join:not([hidden]), #status.good', { timeout: 60000 });
if (await page.locator('#join').isVisible()) { await page.fill('#name', 'Di'); await page.locator('#join button').first().click(); }
await page.waitForSelector('#status.good', { timeout: 60000 });
await page.locator('#lent button', { hasText: 'doc' }).first().click(); await page.waitForSelector('.tiptap', { timeout: 60000 });
log({ open: page.url().slice(0, 60) });
const night = () => page.locator('#night').isVisible();
const trace = async (n = 25) => (await page.evaluate(async () => (await fetch('/svrngs-trace')).text().catch((e) => String(e)))).split('\n').filter((l) => /RUNTIME|HTTP/.test(l)).slice(-n);
await sleep(3000);
for (const w of ['alex', 'server']) procs[w].kill('SIGSTOP'); log({ lid: 'closed' });
const g = await until(night, 15000); log({ goodnight: Boolean(g) });
await sleep(Number(LID_S) * 1000 - 6000);
for (const w of ['alex', 'server']) procs[w].kill('SIGCONT'); log({ lid: 'open' });
const back = await until(async () => !(await night()), 30000); log({ back_within_30s: back !== null });
if (back === null) {
  log({ trace: await trace().catch((e) => String(e)) });
  // The poke: one page fetch at the lend's origin, which the worker's fetch path answers by dropping a dead dial and dialling again.
  const poked = await page.evaluate(async () => { const t = Date.now(); try { const r = await fetch('/protocol.js', { cache: 'no-store' }); return { status: r.status, ms: Date.now() - t }; } catch (e) { return { error: String(e), ms: Date.now() - t }; } });
  log({ poked });
  const back2 = await until(async () => !(await night()), 30000); log({ back_after_poke_within_30s: back2 !== null });
  log({ trace_after: (await trace(12).catch((e) => [String(e)])) });
}
log({ server_sockets: said.server.split('\n').filter((l) => /socket:/.test(l)).map((l) => JSON.parse(l)).map((o) => `${o.at.slice(11, 23)} ${o.what} ${o.person ?? ''}`) });
await b.close(); for (const p of Object.values(procs)) { p.kill('SIGCONT'); p.kill('SIGTERM'); }
process.exit(0);
