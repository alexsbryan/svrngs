// The grapevine canary (GRAPEVINE_DEMO §8, §10, §13.8; gv-page (d)): §10's
// scene end to end on demo.json, in one process acting for Sam, Jo and
// Kim, through the same relay.mjs the server runs and a file of the app's
// own (store.mjs `file`): no svrngs, no second computer. Each step is timed
// from Sam's ask, and each read of a group's log is timed, so a reader sees
// what the loop's polling costs (§13.8). demo.json itself is never written:
// the scene runs on a copy in a scratch folder.
//
//   node apps/grapevine/canary.mjs [demo.json]
//
// Then every other kind in kinds.mjs's list, a developer's too, is asked by
// Sam and carried one ring by Jo, and Kim sees it carried by Jo alone.
//
// A turn of the loop is each person's pass once, as relay.mjs's `loop`
// makes it every RELAY_EVERY. Exit 0 holds; 1 says the step that broke and
// what each person's page would have read.

import { copyFileSync, mkdtempSync, readFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { performance } from 'node:perf_hooks';

import { KINDS as kinds } from './kinds.mjs';
import { RELAY_EVERY, heldIn, person, stateFile } from './relay.mjs';
import { file } from './store.mjs';

const demo = process.argv[2] ?? new URL('./demo.json', import.meta.url).pathname;
const dir = mkdtempSync(join(tmpdir(), 'grapevine-canary-'));
const path = join(dir, 'demo.json');
copyFileSync(demo, path);
const store = file(path);
const reads = [];
const p = {};
for (const id of ['Sam', 'Jo', 'Kim']) p[id] = person({ id, store, held: heldIn(store, id), kinds, state: stateFile(join(dir, `${id}.state.json`)), timed: (r) => reads.push(r) });
const OFFER = { when: 'Saturday morning', where: 'Waller and Pierce' };
const SEALED = [...Object.values(OFFER), 'Sam Reyes', 'Kim Okafor'];

let t0;
const steps = [];
const view = (id) => p[id].view();
/** One turn of the loop: each person's pass. */
async function turn() {
  for (const id of ['Sam', 'Jo', 'Kim']) await p[id].pass();
}
function step(n, what, ok, seen) {
  const ms = performance.now() - t0;
  if (!ok) throw new Error(`step ${n}, ${what}, did not hold:\n${seen}`);
  steps.push(`  ${n}. ${(ms / 1000).toFixed(3).padStart(7)} s  ${what}`);
}
const has = (v, s) => JSON.stringify(v).includes(s);
const carries = [];
/** A value for each of a kind's fields, by its type: what someone might type. */
const example = (fields) => Object.fromEntries(Object.entries(fields).map(([name, spec]) => [name,
  spec === 'date' ? new Date(Date.now() + 3 * 86_400_000).toISOString().slice(0, 10) : spec === 'number' ? 2 : spec.choice?.[0] ?? `a ${name}`]));

async function main() {
  t0 = performance.now();
  // 1. Sam asks in 1400 Page for a tile saw on Saturday: one ring, name withheld.
  const id = await p.Sam.ask({ kind: 'tool', fields: { what: 'tile saw', when: 'Saturday' }, group: 'page', hops: 1, expires: Math.floor(Date.now() / 1000) + 7 * 86_400 });
  step(1, 'Sam asks in 1400 Page', (await view('Sam')).asks.some((a) => a.id === id && a.asked), '');

  // 2. Jo's page shows the ask and "carry to Waller block". Jo taps it.
  const jo = await view('Jo');
  step(2, 'Jo sees it, and may carry it to Waller block', jo.carriable.can.some((c) => c.ask === id && c.group === 'waller'), JSON.stringify(jo.carriable));
  const carry = await p.Jo.carry({ ask: id, group: 'waller' });

  // 3. Kim's page: carried by Jo, never Sam; the tool kind's notice. Kim offers.
  const kim = (await view('Kim')).asks.find((a) => a.id === id);
  step(3, 'Kim sees it carried by Jo, told she might have one, never who asked', kim?.carriers.join() === 'Jo' && kim.notice === true && !has(kim, 'Sam'), JSON.stringify(kim));
  await p.Kim.reply({ ask: id, group: 'waller', fields: OFFER });

  // 4. One turn: Sam's page opens the offer; Jo's says a sealed message passed, and nothing of it.
  await turn();
  const sam = (await view('Sam')).threads.find((t) => t.role === 'asker');
  const joNow = await view('Jo');
  const passed = joNow.threads.find((t) => t.role === 'carrier')?.passed ?? [];
  step(4, 'Sam reads the offer; Jo sees a sealed message pass, and none of it',
    sam?.messages[0]?.fields?.where === OFFER.where && passed.length === 1 && !SEALED.some((s) => has(joNow, s)), JSON.stringify({ sam, passed }));

  // 5. Sam says yes, then Kim; each sees the other's name, and only then.
  await p.Sam.yes({ ask: id, them: sam.them, name: 'Sam Reyes' });
  await turn();
  const before = (await view('Kim')).threads.find((t) => t.role === 'replier');
  await p.Kim.yes({ ask: id, them: before.them, name: 'Kim Okafor' });
  await turn();
  const names = [(await view('Sam')).threads.find((t) => t.role === 'asker')?.names, (await view('Kim')).threads.find((t) => t.role === 'replier')?.names];
  step(5, 'two yeses, then each sees the other\'s name',
    before.names === null && names[0]?.them === 'Kim Okafor' && names[1]?.them === 'Sam Reyes', JSON.stringify({ before: before.names, names }));

  // 6. Sam withdraws; within one turn of the loop the carry in Waller is gone.
  await p.Sam.withdraw({ ask: id });
  await turn();
  const left = (await view('Kim')).asks;
  const corrected = (await store.log('waller')).ops.some((o) => o.corrects === carry && o.person === 'Jo');
  step(6, 'Sam takes it back; within one turn the carry in Waller is gone', !left.length && corrected, `Kim still sees ${JSON.stringify(left.map((a) => a.show))}; Jo's carry corrected: ${corrected}`);

  // 7. Every other kind in kinds.mjs's list, a developer's too: Sam asks, Jo carries it one ring, Kim sees it carried by Jo.
  const others = Object.values(kinds).filter((k) => k.kind !== 'tool');
  for (const k of others) {
    const other = await p.Sam.ask({ kind: k.kind, fields: example(k.ask.fields), group: 'page', hops: 1, expires: Math.floor(Date.now() / 1000) + 7 * 86_400 });
    if (!(await view('Jo')).carriable.can.some((c) => c.ask === other && c.group === 'waller')) throw new Error(`step 7, Jo may not carry Sam's ${k.kind} to Waller block`);
    carries.push(await p.Jo.carry({ ask: other, group: 'waller' }));
    const seen = (await view('Kim')).asks.find((a) => a.id === other);
    if (!(seen?.carriers.join() === 'Jo' && !has(seen, 'Sam'))) throw new Error(`step 7, Kim does not see Sam's ${k.kind} carried by Jo alone:\n${JSON.stringify(seen)}`);
  }
  step(7, `every other kind's ask carried one ring, by Jo: ${others.map((k) => k.kind).join(', ')}`, true, '');

  // Every record read after: no plaintext; the carry names Jo and never Sam.
  const kept = readFileSync(path, 'utf8');
  const waller = (await store.log('waller')).ops;
  const leak = SEALED.find((s) => kept.includes(s));
  const by = [carry, ...carries].map((c) => waller.find((o) => o.id === c)?.person);
  if (leak || by.some((b) => b !== 'Jo') || has(waller, 'Sam')) throw new Error(`the records after: ${leak ? `hold ${JSON.stringify(leak)}` : `the carries are ${by.join(', ')}'s, Waller's log ${has(waller, 'Sam') ? 'names Sam' : 'is clean'}`}`);

  const ms = reads.map((r) => r.ms).sort((a, b) => a - b);
  console.log(`§10's scene on demo.json, one process acting for Sam, Jo and Kim, each step's time from Sam's ask:\n${steps.join('\n')}`);
  console.log(`  ${reads.length} reads of a group's log, every one the scene made, each the whole log (${Math.max(...reads.map((r) => r.ops))} entries at most): median ${ms[ms.length >> 1].toFixed(2)} ms, slowest ${ms.at(-1).toFixed(2)} ms; the loop reads each person's groups every ${RELAY_EVERY / 1000} s`);
  console.log('  no record holds a sealed word; every carry is written by Jo and Waller block names Sam nowhere');
}

main().catch((e) => {
  console.error(`the grapevine canary failed: ${e.message}`);
  if (steps.length) console.error(`it got through:\n${steps.join('\n')}`);
  process.exit(1);
});
