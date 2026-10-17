// The proof (row canon-spike (c)): every canon fixture with a scenario,
// replayed by canon-cli (`canon replay <dir> --json`), by the wasm and by
// the JavaScript wasm2js wrote, each step's answer equal across the three.
//
//   node prove.mjs <canon> <canon's tree> <canon_page.wasm> <canon.js>
//
// `CANON_PLANT=1` alters one step of canon-cli's first answer before the
// comparison, to watch the comparison catch it. Exit 0 only when every
// fixture was judged and passed; a fixture canon-cli could not answer is
// could-not-judge, never a pass (AGENTS.md 6).

import { execFileSync } from 'node:child_process';
import { readdirSync, readFileSync, existsSync } from 'node:fs';
import { isDeepStrictEqual } from 'node:util';
import { join, relative } from 'node:path';

import { bind } from './call.mjs';

const [canon, tree, wasmPath, jsPath] = process.argv.slice(2);
if (!jsPath) {
  console.error('usage: node prove.mjs <canon> <canon tree> <canon_page.wasm> <canon.js>');
  process.exit(2);
}

const { instance } = await WebAssembly.instantiate(readFileSync(wasmPath), {});
const builds = { wasm: bind(instance.exports), js: bind(await import(jsPath)) };

// Every directory under fixtures/ holding a scenario, and fernwood's second.
function scenarios(dir) {
  const here = existsSync(join(dir, 'scenario.jsonl'))
    ? [{ dir, scenario: 'scenario.jsonl', expect: 'expected.json' }]
    : [];
  const below = readdirSync(dir, { withFileTypes: true })
    .filter((e) => e.isDirectory())
    .flatMap((e) => scenarios(join(dir, e.name)));
  return [...here, ...below];
}
const runs = scenarios(join(tree, 'fixtures'));
const fernwood = join(tree, 'fixtures', 'fernwood-commons');
runs.push({ dir: fernwood, scenario: 'kitchen.jsonl', expect: 'kitchen.expected.json' });

// canon-cli prints the steps as one pretty JSON object, then (with an
// expected file beside it) a blank line and whether they were as expected.
// Pretty JSON holds no blank line, so the object ends at the first. A
// scenario of no steps prints `{}` (maple-house/accepted, a seed alone).
function cli({ dir, scenario, expect }) {
  const args = ['replay', dir, '--scenario', join(dir, scenario), '--json'];
  const expected = existsSync(join(dir, expect));
  if (expected) args.push('--expect', join(dir, expect));
  let out;
  let code = 0;
  try {
    out = execFileSync(canon, args, { encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'] });
  } catch (e) {
    out = e.stdout ?? '';
    code = e.status;
  }
  let answer = null;
  try {
    answer = JSON.parse(out.split('\n\n')[0]);
  } catch {}
  return { answer, code, expected };
}

let failed = 0;
let judged = 0;
let plant = process.env.CANON_PLANT === '1';
for (const run of runs) {
  const name = relative(join(tree, 'fixtures'), join(run.dir, run.scenario));
  const { answer, code, expected } = cli(run);
  if (!answer) {
    console.log(`could-not-judge  ${name}: canon-cli gave no answer (exit ${code})`);
    failed = 1;
    continue;
  }
  if (plant) {
    const first = Object.keys(answer)[0];
    answer[first] = { ...answer[first], planted: true };
    console.log(`plant            ${name}: step ${first} altered in canon-cli's answer`);
    plant = false;
  }
  const seed = readFileSync(join(run.dir, 'acts.jsonl'), 'utf8');
  const scenario = readFileSync(join(run.dir, run.scenario), 'utf8');
  const steps = Object.keys(answer).length;
  const verdicts = Object.entries(builds).map(([build, call]) => {
    const got = call({ call: 'replay', seed, scenario });
    const differ = Object.keys({ ...answer, ...got }).filter(
      (k) => !isDeepStrictEqual(answer[k], got[k]),
    );
    return { build, differ };
  });
  const bad = verdicts.filter((v) => v.differ.length);
  judged += 1;
  if (bad.length) failed = 1;
  const bar = !expected ? 'no expected file' : code === 0 ? 'as expected' : `canon-cli exit ${code}`;
  console.log(
    `${bad.length ? 'failed ' : 'passed '}          ${name}: ${steps} steps, ${bar}; ` +
      verdicts
        .map((v) => `${v.build} ${v.differ.length ? `differs at ${v.differ.join(', ')}` : 'equal'}`)
        .join('; '),
  );
}
console.log(`${judged} of ${runs.length} replays judged; ${failed ? 'NOT all equal' : 'all equal'}`);
process.exit(failed);
