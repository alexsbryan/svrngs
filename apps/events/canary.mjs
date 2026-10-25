// The Events canary (DESIGN §5 "The canaries"; events-app (c)): each
// person's computer holds one group's record; everyone answers through their
// own computer's loopback API, a correction voids, and every copy folds, with
// events.js, to the same event, who is coming and who came — the ones the acts say; and a second
// group, on Sam's computer alone, holds the same event and none of the
// first's answers. It uses only what DESIGN §3 says an app gets: the people,
// and the record over loopback.
//
//   node apps/events/canary.mjs <group> <second group> <Sam's record> <Alex's> <Bo's> <Cy's>
//
// The caller (crates/svrngs-node/tests/canaries.rs) stands up one computer
// for each of Sam, Alex, Bo and Cy in the group, since a computer writes only
// as its own person (DESIGN §3, `not_yours`), and Sam and Di in the second,
// on Sam's alone. Exit 0 holds; 1 says what broke, and when the copies
// differ, names each and what it says.

import { randomBytes } from 'node:crypto';
import { isDeepStrictEqual as same } from 'node:util';

import { CAME, COMING, EVENT, actsOf, fold } from './events.js';

const [group, second, samAt, alexAt, boAt, cyAt] = process.argv.slice(2);
if (!cyAt) {
  console.error("usage: node canary.mjs <group> <second group> <Sam's record> <Alex's> <Bo's> <Cy's>");
  process.exit(2);
}
const SAM = { name: "Sam's computer", at: samAt };
const ALEX = { name: "Alex's computer", at: alexAt };
const BO = { name: "Bo's computer", at: boAt };
const CY = { name: "Cy's computer", at: cyAt };
const COMPUTERS = [SAM, ALEX, BO, CY];
const PEOPLE = { [group]: ['alex', 'bo', 'cy', 'sam'], [second]: ['di', 'sam'] };
/** How long one copy is given to hold an act the other wrote (apps/expenses/canary.mjs, WAIT_MS). */
const WAIT_MS = 10_000;

async function call(computer, door, at, body) {
  const url = `${computer.at}/v1/rail/${door}?namespace=${encodeURIComponent(at)}`;
  const sent = body === undefined ? {} : { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify(body) };
  const res = await fetch(url, sent);
  const out = await res.json();
  if (!res.ok) throw new Error(`${computer.name} refused ${door} in ${at}: ${res.status} ${JSON.stringify(out)}`);
  return out;
}

const written = [];
/** Append `act` in `at` on `computer`, as its own person; answers its id. */
async function append(computer, at, act) {
  const { id } = await call(computer, 'append', at, act);
  if (at === group) written.push(id);
  return id;
}
const record = (computer, op, at = group) => append(computer, at, { op: 'record', payload: op });
const correct = (computer, id, replacement) => append(computer, group, { op: 'correct', corrects: id, replacement });
const log = (computer, at = group) => call(computer, 'log', at);

/** The ids of `ids` that `computer` does not hold within WAIT_MS. */
async function unheld(computer, ids) {
  const deadline = Date.now() + WAIT_MS;
  for (;;) {
    const held = new Set((await log(computer)).ops.map((o) => o.id));
    const missing = ids.filter((id) => !held.has(id));
    if (!missing.length || Date.now() > deadline) return missing;
    await new Promise((r) => setTimeout(r, 50));
  }
}

/** An app writes about what its copy holds: wait for `id` on `computer`. */
async function arrived(computer, id) {
  if ((await unheld(computer, [id])).length) throw new Error(`${computer.name} never held ${id} within ${WAIT_MS} ms`);
}

async function main() {
  for (const [at, people] of Object.entries(PEOPLE)) {
    for (const computer of at === group ? COMPUTERS : [SAM]) {
      const listed = (await call(computer, 'people', at)).people.map((p) => p.person).sort();
      if (!same(listed, people)) throw new Error(`${computer.name} lists ${listed.join(', ')} in ${at}, not ${people.join(', ')}`);
    }
  }

  // Sam posts a dinner to both groups, under one id.
  const event = randomBytes(8).toString('hex');
  const dinner = { kind: EVENT, event, what: 'dinner', when: 'Friday 7', where: 'the house' };
  const posted = await record(SAM, dinner);
  await record(SAM, dinner, second);
  for (const computer of [ALEX, BO, CY]) await arrived(computer, posted);
  // Alex is coming; Bo is coming with Maya; Cy is coming, and takes it back.
  await record(ALEX, { kind: COMING, event });
  const bo = await record(BO, { kind: COMING, event, guests: ['Maya'] });
  const cy = await record(CY, { kind: COMING, event });
  const takenBack = await correct(CY, cy, null);
  // Alex tries to take Bo's back: the record voids it, and the fold does not count it.
  await arrived(ALEX, bo);
  await correct(ALEX, bo, null);
  // Bo came, and says Maya came.
  await record(BO, { kind: CAME, event });
  await record(BO, { kind: CAME, event, guest: 'Maya' });
  // In the second group, Sam is coming.
  await record(SAM, { kind: COMING, event }, second);

  // `via`, whose computer wrote an answer for someone else (gv-via): none,
  // since each wrote on their own.
  const via = { host: null, coming: {}, came: {}, guests_came: {} };
  // An event by name: the fold keeps people by first key (events.js `who`).
  const named = (e, names) => {
    const n = (k) => names[k] ?? k;
    const keys = (m) => Object.fromEntries(Object.entries(m).map(([k, v]) => [n(k), v]));
    const vias = { ...e.via, coming: keys(e.via.coming), came: keys(e.via.came), guests_came: keys(e.via.guests_came) };
    return { ...e, host: n(e.host), coming: keys(e.coming), came: e.came.map(n), guests_came: keys(e.guests_came), via: vias };
  };
  const want = { host: 'sam', what: 'dinner', when: 'Friday 7', where: 'the house', coming: { alex: [], bo: ['Maya'] }, came: ['bo'], guests_came: { bo: ['Maya'] }, via };
  const copies = [];
  for (const computer of COMPUTERS) {
    const missing = await unheld(computer, written);
    const ops = (await log(computer)).ops;
    const voided = ops.find((o) => o.id === cy)?.voided;
    const { events, gaps, names } = fold(actsOf({ ops }));
    const { at: _, ...said } = events[event] ? named(events[event], names) : {};
    copies.push({ computer, missing, voided, said, gaps: gaps.map((g) => g.code) });
  }
  const lines = copies.map((c) =>
    `  ${c.computer.name}: ${JSON.stringify(c.said)}; gaps ${JSON.stringify(c.gaps)}; Cy's coming voided: ${c.voided}` +
    (c.missing.length ? `; never received ${c.missing.join(', ')}` : ''));
  const [s] = copies;
  if (copies.some((c) => !same([c.said, c.gaps], [s.said, s.gaps]))) throw new Error(`the copies of ${group} fold to different answers:\n${lines.join('\n')}`);
  if (!same(s.said, want) || !same(s.gaps, ['not_theirs']) || copies.some((c) => !c.voided)) {
    throw new Error(`every copy of ${group} agrees, but not with what the acts say (${JSON.stringify(want)}; Alex's change of Bo's answer not counted; Cy's ${takenBack} voiding hers on each):\n${lines.join('\n')}`);
  }

  const other = await log(SAM, second);
  const strangers = actsOf(other).filter((x) => !PEOPLE[second].includes(x.by)).map((x) => x.by);
  const theirs = fold(actsOf(other));
  const there = theirs.events[event] && named(theirs.events[event], theirs.names);
  if (strangers.length || !same(there?.coming, { sam: [] }) || there?.host !== 'sam') {
    throw new Error(`${second} holds ${group}'s answers, or not its own: acts by ${JSON.stringify(strangers)}; it folds to ${JSON.stringify(there)}`);
  }
  console.log(`${COMPUTERS.length} computers fold ${written.length} acts of ${group} to the same answers: ${JSON.stringify(want)}, Cy's taken back on each and Alex's change of Bo's not counted; ${second} holds the event and Sam's answer alone`);
}

main().catch((e) => {
  console.error(`the Events canary failed: ${e.message}`);
  process.exit(1);
});
