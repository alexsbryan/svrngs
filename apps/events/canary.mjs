// The Events canary (DESIGN §8 "The canaries"; events-app (c)): two computers
// hold one group's record; everyone answers through the record's loopback
// API, a correction voids, and both copies fold, with events.js, to the same
// event, who is coming and who came — the ones the acts say; and a second
// group, on Sam's computer alone, holds the same event and none of the
// first's answers. It uses only what DESIGN §3 says an app gets: the people,
// and the record over loopback.
//
//   node apps/events/canary.mjs <group> <second group> <Sam's record> <Alex's record>
//
// The caller (crates/svrngs-node/tests/canaries.rs) stands both up with Sam,
// Alex, Bo and Cy in the group — Sam's computer and Alex's, Bo and Cy with
// none here, whose acts an app on one of the two writes for them, as an app
// serving someone the door verified does — and Sam and Di in the second.
// Exit 0 holds; 1 says what broke, and when the copies differ, names both
// and what each says.

import { randomBytes } from 'node:crypto';
import { isDeepStrictEqual as same } from 'node:util';

import { CAME, COMING, EVENT, actsOf, fold } from './events.js';

const [group, second, samAt, alexAt] = process.argv.slice(2);
if (!alexAt) {
  console.error("usage: node canary.mjs <group> <second group> <Sam's record> <Alex's record>");
  process.exit(2);
}
const SAM = { name: "Sam's computer", at: samAt };
const ALEX = { name: "Alex's computer", at: alexAt };
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
/** Append `act` in `at` on `computer`, for `person` when one is named; answers its id. */
async function append(computer, at, act, person) {
  const { id } = await call(computer, 'append', at, { ...act, ...(person && { on_behalf_of: person }) });
  if (at === group) written.push(id);
  return id;
}
const record = (computer, op, person, at = group) => append(computer, at, { op: 'record', payload: op }, person);
const correct = (computer, id, replacement, person) => append(computer, group, { op: 'correct', corrects: id, replacement }, person);
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
    for (const computer of at === group ? [SAM, ALEX] : [SAM]) {
      const listed = (await call(computer, 'people', at)).people.map((p) => p.person).sort();
      if (!same(listed, people)) throw new Error(`${computer.name} lists ${listed.join(', ')} in ${at}, not ${people.join(', ')}`);
    }
  }

  // Sam posts a dinner to both groups, under one id.
  const event = randomBytes(8).toString('hex');
  const dinner = { kind: EVENT, event, what: 'dinner', when: 'Friday 7', where: 'the house' };
  const posted = await record(SAM, dinner);
  await record(SAM, dinner, null, second);
  await arrived(ALEX, posted);
  // Alex is coming; Bo is coming with Maya, from Sam's computer; Cy is
  // coming, from Alex's, and takes it back from Sam's.
  await record(ALEX, { kind: COMING, event });
  const bo = await record(SAM, { kind: COMING, event, guests: ['Maya'] }, 'bo');
  const cy = await record(ALEX, { kind: COMING, event }, 'cy');
  await arrived(SAM, cy);
  const takenBack = await correct(SAM, cy, null, 'cy');
  // Alex tries to take Bo's back: the record voids it, and the fold does not count it.
  await arrived(ALEX, bo);
  await correct(ALEX, bo, null);
  // Bo came, and says Maya came.
  await record(ALEX, { kind: CAME, event }, 'bo');
  await record(ALEX, { kind: CAME, event, guest: 'Maya' }, 'bo');
  // In the second group, Di is coming.
  await record(SAM, { kind: COMING, event }, 'di', second);

  const want = { host: 'sam', what: 'dinner', when: 'Friday 7', where: 'the house', coming: { alex: [], bo: ['Maya'] }, came: ['bo'], guests_came: { bo: ['Maya'] } };
  const copies = [];
  for (const computer of [SAM, ALEX]) {
    const missing = await unheld(computer, written);
    const ops = (await log(computer)).ops;
    const voided = ops.find((o) => o.id === cy)?.voided;
    const { events, gaps } = fold(actsOf({ ops }));
    const { at: _, ...said } = events[event] ?? {};
    copies.push({ computer, missing, voided, said, gaps: gaps.map((g) => g.code) });
  }
  const lines = copies.map((c) =>
    `  ${c.computer.name}: ${JSON.stringify(c.said)}; gaps ${JSON.stringify(c.gaps)}; Cy's coming voided: ${c.voided}` +
    (c.missing.length ? `; never received ${c.missing.join(', ')}` : ''));
  const [s, a] = copies;
  if (!same([s.said, s.gaps], [a.said, a.gaps])) throw new Error(`the two copies of ${group} fold to different answers:\n${lines.join('\n')}`);
  if (!same(s.said, want) || !same(s.gaps, ['not_theirs']) || !s.voided || !a.voided) {
    throw new Error(`both copies of ${group} agree, but not with what the acts say (${JSON.stringify(want)}; Alex's change of Bo's answer not counted; Cy's ${takenBack} voiding hers on both):\n${lines.join('\n')}`);
  }

  const other = await log(SAM, second);
  const strangers = actsOf(other).filter((x) => !PEOPLE[second].includes(x.by)).map((x) => x.by);
  const there = fold(actsOf(other)).events[event];
  if (strangers.length || !same(there?.coming, { di: [] }) || there?.host !== 'sam') {
    throw new Error(`${second} holds ${group}'s answers, or not its own: acts by ${JSON.stringify(strangers)}; it folds to ${JSON.stringify(there)}`);
  }
  console.log(`${SAM.name} and ${ALEX.name} fold ${written.length} acts of ${group} to the same answers: ${JSON.stringify(want)}, Cy's taken back on both and Alex's change of Bo's not counted; ${second} holds the event and Di alone`);
}

main().catch((e) => {
  console.error(`the Events canary failed: ${e.message}`);
  process.exit(1);
});
