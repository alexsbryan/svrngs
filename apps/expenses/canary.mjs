// The Expenses canary (DESIGN §5 "The canaries"): each person's computer
// holds one group's record; everyone writes through their own computer's
// loopback API, a correction voids, and every copy folds, with `init` and
// `reduce`, to the same balances — the ones the acts say. It uses only what DESIGN §3 says an
// app gets: the people, and the record over loopback.
//
//   node apps/expenses/canary.mjs <group> <Sam's record> <Alex's> <Bo's> <Cy's>
//
// Each record is a member node's loopback API, `http://127.0.0.1:<port>`.
// The caller (crates/svrngs-node/tests/canaries.rs) stands up one computer
// for each of Sam, Alex, Bo and Cy, since a computer writes only as its own
// person (DESIGN §3, `not_yours`). Exit 0 holds; 1 says what broke, and when
// the copies differ, names each and what it says.

import { isDeepStrictEqual as same } from 'node:util';

import { EXPENSE, SETTLE, SHARE, actsOf, init, money, reduce } from './expenses.js';

const [group, samAt, alexAt, boAt, cyAt] = process.argv.slice(2);
if (!cyAt) {
  console.error("usage: node canary.mjs <group> <Sam's record> <Alex's> <Bo's> <Cy's>");
  process.exit(2);
}
const SAM = { name: "Sam's computer", at: samAt };
const ALEX = { name: "Alex's computer", at: alexAt };
const BO = { name: "Bo's computer", at: boAt };
const CY = { name: "Cy's computer", at: cyAt };
const COMPUTERS = [SAM, ALEX, BO, CY];
const PEOPLE = ['alex', 'bo', 'cy', 'sam'];
/** What the acts below say each person is owed (+) or owes (-), in cents. */
const OWED = { alex: 0, bo: -1500, cy: 0, sam: 1500 };
/** How long one copy is given to hold an act the other wrote: rounds an
 * append wakes carry it in well under a second here, and the round's own
 * interval is 60 s, so only a woken round can carry it inside this. */
const WAIT_MS = 10_000;

async function call(computer, door, body) {
  const url = `${computer.at}/v1/rail/${door}?namespace=${encodeURIComponent(group)}`;
  const sent = body === undefined ? {} : { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify(body) };
  const res = await fetch(url, sent);
  const out = await res.json();
  if (!res.ok) throw new Error(`${computer.name} refused ${door}: ${res.status} ${JSON.stringify(out)}`);
  return out;
}

const written = [];
/** Append `act` on `computer`, as its own person; answers its id. */
async function append(computer, act) {
  const { id } = await call(computer, 'append', act);
  written.push(id);
  return id;
}
const record = (computer, op) => append(computer, { op: 'record', payload: op });
const correct = (computer, id, replacement) => append(computer, { op: 'correct', corrects: id, replacement });

const log = (computer) => call(computer, 'log');

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

const said = (balances) => PEOPLE.map((p) => `${p} ${p in balances ? money(balances[p]) : 'absent'}`).join(', ');

async function main() {
  for (const computer of COMPUTERS) {
    const people = (await call(computer, 'people')).people.map((p) => p.person).sort();
    if (!same(people, PEOPLE)) throw new Error(`${computer.name} lists ${people.join(', ')} in ${group}, not ${PEOPLE.join(', ')}`);
  }

  // Sam pays for pizza; Alex, Cy and Bo each say they had some.
  const pizza = await record(SAM, { kind: EXPENSE, amount_cents: 6000, description: 'pizza' });
  for (const computer of [ALEX, CY, BO]) {
    await arrived(computer, pizza);
    await record(computer, { kind: SHARE, expense: pizza });
  }
  // Cy pays for a cab, and Sam was in it.
  const cab = await record(CY, { kind: EXPENSE, amount_cents: 3000, description: 'cab' });
  await arrived(SAM, cab);
  await record(SAM, { kind: SHARE, expense: cab });
  // Alex pays Sam back $20.00, meaning $15.00, and corrects it.
  const typo = await record(ALEX, { kind: SETTLE, to: 'sam', amount_cents: 2000 });
  await correct(ALEX, typo, { kind: SETTLE, to: 'sam', amount_cents: 1500 });

  const copies = [];
  for (const computer of COMPUTERS) {
    const missing = await unheld(computer, written);
    const book = actsOf(await log(computer)).reduce(reduce, init());
    copies.push({ computer, missing, book });
  }
  const [s] = copies;
  const lines = copies.map(({ computer, missing, book }) =>
    `  ${computer.name}: ${said(book.balances)}` +
    (book.gaps.length ? `; unread: ${JSON.stringify(book.gaps)}` : '') +
    (missing.length ? `; never received ${missing.join(', ')}` : ''));
  if (copies.some((c) => !same(c.book.balances, s.book.balances))) {
    throw new Error(`the copies of ${group} fold to different balances:\n${lines.join('\n')}`);
  }
  if (!same(s.book.balances, OWED) || copies.some((c) => c.book.gaps.length)) {
    throw new Error(`every copy of ${group} agrees, but not with what the acts say (${said(OWED)}):\n${lines.join('\n')}`);
  }
  console.log(`${COMPUTERS.length} computers fold ${written.length} acts of ${group} to the same balances: ${said(OWED)}`);
}

main().catch((e) => {
  console.error(`the Expenses canary failed: ${e.message}`);
  process.exit(1);
});
