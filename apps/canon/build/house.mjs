// The measure (row canon-spike (a)): a house of 300 acts, each minted by
// canon (`act`), then `derive` timed in the wasm and the JavaScript, median
// of 20 warm calls, JSON in and out as the page will call it. Writes the
// house's acts to <out> for the native timer (examples/time.rs).
//
//   node house.mjs <canon_page.wasm> <canon.js> <out.jsonl>

import { readFileSync, writeFileSync } from 'node:fs';
import { performance } from 'node:perf_hooks';

import { bind } from './call.mjs';

const [wasmPath, jsPath, out] = process.argv.slice(2);
const { instance } = await WebAssembly.instantiate(readFileSync(wasmPath), {});
const wasm = bind(instance.exports);
const js = bind(await import(jsPath));

const DAY = 86_400;
const people = Array.from({ length: 12 }, (_, i) => `human:p${String(i + 1).padStart(2, '0')}`);
const lines = [];
let ts = 1_767_225_600; // 2026-01-01
const write = (actor, kind) => {
  const line = wasm({ call: 'act', kind, ts_unix: ts, actor });
  lines.push(line);
  ts += 600;
  return JSON.parse(line).id;
};

for (const p of people) write(people[0], { op: 'grant', holder: p, scope: 'house' });
ts += DAY;
write(people[0], {
  op: 'ratification',
  text: 'A rule holds after two days unless someone objects.',
  rule: { rule: 'consent', days: 2 },
  scope: 'house',
});
const rules = [];
for (let round = 0; lines.length < 300; round += 1) {
  ts += DAY / 2;
  const by = people[round % people.length];
  const id = write(by, { op: 'assert', text: `House rule ${round}: something about the house.` });
  if (lines.length < 300) write(by, { op: 'scoped', commitment: id, scope: 'house' });
  for (let k = 1; k <= 3 && lines.length < 300; k += 1) {
    const who = people[(round + k) % people.length];
    write(who, { op: 'position', about: id, pull: 'toward', because: 'yes, for the house' });
  }
  if (round % 4 === 3 && lines.length < 300) {
    const who = people[(round + 5) % people.length];
    write(who, { op: 'position', about: id, pull: 'against', because: 'not like this' });
  }
  if (round % 5 === 4 && rules.length && lines.length < 300) {
    write(by, { op: 'supersede', text: `House rule ${round}, said better.`, old: [rules.shift()] });
  }
  if (round % 7 === 6 && rules.length && lines.length < 300) {
    write(by, { op: 'retract', target: rules.shift(), rationale: 'we stopped doing it' });
  }
  rules.push(id);
}
const acts = lines.join('');
writeFileSync(out, acts);

// Three days past the last act, so the last proposals' two days have run.
const now = JSON.parse(lines.at(-1)).ts_unix + 3 * DAY;
const request = { call: 'derive', acts, now };
const time = (call) => {
  call(request);
  const ms = [];
  for (let i = 0; i < 20; i += 1) {
    const t = performance.now();
    call(request);
    ms.push(performance.now() - t);
  }
  ms.sort((a, b) => a - b);
  return (ms[9] + ms[10]) / 2;
};
const a = wasm(request);
const b = js(request);
const same = JSON.stringify(a) === JSON.stringify(b);
const counts = Object.fromEntries(
  ['active', 'proposed', 'refused', 'superseded', 'retracted'].map((s) => [
    s,
    a.commitments.filter((c) => c.status.status === s).length,
  ]),
);
console.log(`house: ${lines.length} acts, 12 people; commitments ${JSON.stringify(counts)}`);
console.log(`derive, the wasm and the JavaScript ${same ? 'equal' : 'DIFFER'}`);
console.log(`derive median of 20: wasm ${time(wasm).toFixed(2)} ms, JavaScript ${time(js).toFixed(2)} ms`);
process.exit(same ? 0 : 1);
