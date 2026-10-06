// Control (u5-11): the canary's seeks straight at it, no svrngs, through the
// same link the conformance test gives a phone: 150 ms each way, what the
// canary sends paced to 16,666,666 bit/s with at most a second's bytes
// queued, shared by every connection the browser opens. Each Range asked
// and each connection closed is logged with the run.
//
//   cd web && node ../demo/runs/u5/2026-10-06-seek-stall/control.mjs <engines,> <runs each> >out
import { spawn } from 'node:child_process';
import net from 'node:net';
import { launch } from '../../../../web/tests/page.mjs';
const RATE = 16_666_666 / 8, DELAY = 150, QUEUE = RATE;
const child = spawn('node', [new URL('../../../../apps/web-conformance/serve.mjs', import.meta.url).pathname, 'control']);
const port = await new Promise((r) => child.stdout.once('data', (d) => r(JSON.parse(d).port)));
// One shared downstream link.
let conns = 0; const log = [];
const queue = []; let queued = 0; let busy = false; const paused = new Set();
function pump() {
  if (busy) return;
  const item = queue[0];
  if (!item) return;
  const wait = item.due - Date.now();
  busy = true;
  setTimeout(() => {
    queue.shift();
    if (!item.to.destroyed) item.to.write(item.data);
    setTimeout(() => {
      queued -= item.data.length; busy = false;
      if (queued < QUEUE) for (const s of paused) { paused.delete(s); s.resume(); }
      pump();
    }, (item.data.length * 1000) / RATE);
  }, Math.max(0, wait));
}
const proxy = net.createServer((client) => {
  const up = net.connect(port, '127.0.0.1');
  const id = ++conns;
  client.on('data', (d) => {
    for (const m of d.toString('latin1').matchAll(/^range: (.*)$/gim)) log.push(`${new Date().toISOString()} c${id} asked ${m[1].trim()}`);
    setTimeout(() => up.write(d), DELAY);
  });
  client.on('close', () => log.push(`${new Date().toISOString()} c${id} closed`));
  up.on('data', (d) => {
    queue.push({ due: Date.now() + DELAY, data: d, to: client }); queued += d.length;
    if (queued >= QUEUE) { up.pause(); paused.add(up); }
    pump();
  });
  const end = () => { client.destroy(); up.destroy(); };
  client.on('close', end); up.on('close', end); client.on('error', end); up.on('error', end);
});
await new Promise((r) => proxy.listen(0, '127.0.0.1', r));
const at = proxy.address().port;
const out = [];
for (const engine of (process.argv[2] ?? 'webkit,chromium').split(',')) {
  for (let i = 0; i < Number(process.argv[3] ?? 3); i++) {
    const { context, close } = await launch(engine);
    const page = await context.newPage();
    await page.goto(`http://localhost:${at}/`);
    await page.waitForFunction(() => window.conformance);
    const r = await page.evaluate(() => window.conformance.seeks());
    console.log(engine, i, r.again, r.ok);
    out.push({ engine, again: r.again, ok: r.ok, saw: r.saw, log: log.splice(0) });
    await close();
  }
}
console.log(JSON.stringify(out));
child.kill(); proxy.close(); process.exit(0);
