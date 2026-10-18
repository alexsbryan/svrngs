// The clean take's step 8 and step 10, reproduced without svrngs (README,
// "What it found", 2): House rules' fold (apps/canon/house.js) over the
// vendored build, asked at the second before the agreement that passes a
// change and at its own second. svrngs times a new act after every act it
// holds (crates/svrngs-core/src/command.rs:243-250), so an agreement written
// in the second after the proposal is timed ahead of the page's clock
// (apps/canon/index.js:53, `Date.now()`).
//
//   node demo/runs/canon/2026-10-17-house/same_second.mjs
import { bind } from '../../../../apps/canon/vendor/call.mjs';
import * as build from '../../../../apps/canon/vendor/canon.js';
import { agree, decide, fold, human, payload } from '../../../../apps/canon/house.js';

const call = bind(build);
let n = 0;
const rec = (ts, person, act) => ({ id: `r${++n}`, ts_unix: ts, person, on_behalf_of: person, voided: false, payload: payload(act) });
const spans = ['ana', 'bo', 'cy'].map((person) => ({ person, joined: null, left: null }));
const change = decide('everyone');
const id = JSON.parse(call({ call: 'act', kind: change, ts_unix: 2000, actor: human('ana') })).id;
const ops = [rec(1000, 'ana', decide('most')), rec(2000, 'ana', change), rec(2100, 'bo', agree(id))];
for (const now of [2099, 2100]) {
  const view = fold(call, { you: 'ana', group: 'g', people: ['ana', 'bo', 'cy'], spans, ops }, now);
  const asked = view.changes.find((c) => c.id === id).verdict;
  console.log(`asked at ${now}: the change ${asked.verdict} since ${asked.since}; the rule in force ${view.rule.rule}`);
}
