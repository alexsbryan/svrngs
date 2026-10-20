// SPDX-License-Identifier: AGPL-3.0-or-later
// House rules' glue (row canon-house; svrn-docs rings/current/CANON_IN_SVRNGS.md
// §3): what the group's record holds, handed to canon as canon's own acts,
// and canon's answers handed back to the page. It decides nothing about a
// house's rules. Every verdict — what is a rule, what is proposed, who has a
// say — is canon-core's, through `call`, the vendored build's one function
// (vendor/call.mjs over vendor/canon.js). This file only builds canon's input
// and reads its output.
//
// ── the rules worth reading before you change anything ──
//
// **Membership is standing** (CANON_IN_SVRNGS §8; the operator, 2026-10-17,
// "Yes, reuse"). svrngs-core decides who is in the house and when; it tells
// the page as `spans`, `{person, joined, left}` (crates/svrngs-node/src/
// carried.rs `acts`). `membership` turns that into ordinary canon acts by a
// `house:<group>` actor, in memory on every computer and never written to the
// record: the house grants itself ROOT, then grants each person ROOT at their
// `joined` and withdraws them at their `left`. A grant takes on its writer's
// seat and never waits on a rule (canon fold.rs:739-776, :1597-1641), and only
// `human:` actors count in a tally (ratify.rs:1031-1033), so the house gives its
// people a say and has none itself.
//
// **The house's acts come first.** The house's own grant is at HOUSE_AT and a
// founder's at FOUNDED_AT, strictly before every recorded act and every
// joiner's grant, so the house's grant is the only one canon takes on an open
// seat (ratify.rs:896-908: only grants strictly before an act count for it).
// A record dated at or before FOUNDED_AT is not counted. The times are
// constants, never a computer's clock or a span's `joined`, so a person
// joining later never moves when the house began, and a past rule keeps the
// agreement it had.
//
// **Who said it is the record's person, never the act's text.** A recorded
// act is `{kind: 'act', app: 'canon', act}`, `act` canon's own op and its
// fields. canon's actor is `human:<the person the record speaks for>`, its
// time the record's; an `act` naming anyone else is not counted, and its own
// id, time, signature and refs are never read. Whose computer wrote it, when
// it was someone else's, is kept beside it as `via`.
//
// **What only membership gives, nobody writes.** A recorded `key` (a key
// bound under a name makes that name's unsigned acts uncounted, canon
// sign.rs:385-402), a grant or withdrawal over ROOT, and an undoing of the
// house's own acts are each dropped before the fold, and said.
//
// **The house is never said as a person.** Who decides is asked of canon for
// each person in the house, never read off canon's grants; what canon lists
// as `unattended`, `bootstrap` or `ungoverned` reaches the page only for
// acts a person recorded.

/** The one scope a house has: membership grants it, and nothing else does. */
export const ROOT = 'house';
/** When the house grants itself ROOT, and when it grants a founder theirs. */
const HOUSE_AT = 0;
const FOUNDED_AT = 1;
/** The only kinds of canon act the page writes (OFFERS): a rule proposed, a
 * position on one, how the house decides. No draw, rotation or subsidiarity,
 * whose pools canon 0.5.0 counts the house among (draw.rs:179-183,
 * allot.rs:200-205, policy.rs:537-552), and no grant, which membership alone
 * gives. */
export const OFFERS = Object.freeze(['assert', 'position', 'ratification']);
/** What an act's envelope holds, which canon builds and the record never gives. */
const ENVELOPE = ['id', 'v', 'ts_unix', 'actor', 'refs', 'sig'];

export const house = (group) => `house:${group}`;
export const human = (person) => `human:${person}`;

/** The house's membership as canon acts, one wire line each, in one order
 * whatever order `spans` came in: ROOT granted to the house, then to each
 * person at their `joined` (a founder at FOUNDED_AT), and withdrawn at their
 * `left`. */
export function membership(call, group, spans) {
  const actor = house(group);
  const mint = (kind, ts) => call({ call: 'act', kind, ts_unix: ts, actor }).trim();
  const lines = [mint({ op: 'grant', holder: actor, scope: ROOT }, HOUSE_AT)];
  for (const s of spans) {
    lines.push(mint({ op: 'grant', holder: human(s.person), scope: ROOT }, s.joined ?? FOUNDED_AT));
    if (s.left != null) lines.push(mint({ op: 'withdraw', holder: human(s.person), scope: ROOT }, s.left));
  }
  const key = (l) => {
    const a = JSON.parse(l);
    return [a.ts_unix, a.id];
  };
  return lines.sort((a, b) => {
    const [[ta, ia], [tb, ib]] = [key(a), key(b)];
    return ta - tb || (ia < ib ? -1 : ia > ib ? 1 : 0);
  });
}

/** Why a recorded act is not counted, or null when it is: the record's
 * person `by`, its time `ts`, its `act`, and the ids of the house's acts. */
function refused(by, ts, act, ours) {
  if (typeof by !== 'string' || !by) return 'nobody is named as having written it';
  if (act === null || typeof act !== 'object' || Array.isArray(act) || typeof act.op !== 'string') return 'this version of House rules can’t read it';
  if ('actor' in act && act.actor !== human(by)) return `it speaks as ${act.actor}, not as ${by}, who wrote it`;
  if (!(ts > FOUNDED_AT)) return 'it is dated before your house began';
  if (act.op === 'key') return 'it binds a key to a name, which House rules never does';
  if ((act.op === 'grant' || act.op === 'withdraw') && act.scope === ROOT) return 'it gives or takes a say in the house, which only joining or leaving it does';
  if (act.op === 'revert' && (act.targets ?? []).some((t) => ours.has(t))) return 'it undoes who is in your house, which only joining or leaving it does';
  return null;
}

/** The record's House rules acts, each `{id, ts, by, via, act}`, `by` the
 * person it speaks for and `via` whose computer wrote it when that was
 * someone else's; and those not counted, each with why. */
export function recorded(read, ours = new Set()) {
  const kept = [];
  const dropped = [];
  for (const o of read.ops ?? []) {
    if (o.voided || o.payload == null) continue;
    const by = o.on_behalf_of ?? o.person;
    const via = o.on_behalf_of != null && o.on_behalf_of !== o.person ? o.person : null;
    const p = o.payload;
    const why = p.kind !== 'act' || p.app !== 'canon' ? 'this version of House rules can’t read it' : refused(by, o.ts_unix, p.act, ours);
    if (why) dropped.push({ id: o.id, by, via, why });
    else kept.push({ id: o.id, ts: o.ts_unix, by, via, act: Object.fromEntries(Object.entries(p.act).filter(([k]) => !ENVELOPE.includes(k))) });
  }
  return { kept, dropped };
}

/** The first question, "How does your house decide its rules?" (CANON_IN_SVRNGS
 * §3), each answer a canon rule. "Most of you" and "everyone" are canon's
 * share of the house, more than half of it and all of it, counted against
 * whoever had a say when each proposal was written: a later joiner never
 * moves the count, and once the share can no longer be reached, because
 * enough of them objected or left, the proposal is refused, so a house that
 * shrinks still decides, a change of its rule included (canon
 * ratify.rs:1291-1303). canon spells those two shares `majority` and
 * `unanimous`, its one spelling of `share:>1/2` and `share:100%`
 * (ratify.rs:345-350, :525-528). Never a count of the house on the day it
 * chose, which outlives the house when it shrinks. */
export const CHOICES = Object.freeze({
  consent: { words: 'Anyone proposes a rule, and it holds after two days unless someone objects', rule: { rule: 'consent', days: 2 } },
  most: { words: 'Most of you agree', rule: { rule: 'majority' } },
  everyone: { words: 'Everyone agrees', rule: { rule: 'unanimous' } },
});

/** The act that says how the house decides, chosen as `choice`: the whole
 * house's rule, written with no scope, since a rule for a scope never
 * reaches a rule proposed with none (ratify.rs:812-816), and itself a rule
 * the house can change, judged by the rule it replaces. There is no
 * default: until someone answers, the page asks. */
export function decide(choice) {
  const c = CHOICES[choice];
  if (!c) throw new Error(`no way to decide called ${choice}`);
  return { op: 'ratification', text: `${c.words}.`, rule: { ...c.rule } };
}

/** A rule proposed, `text` its words. */
export const propose = (text) => ({ op: 'assert', text });
/** Agreeing to `about`, a proposal's id; objecting to it, `why` required. */
export const agree = (about) => ({ op: 'position', about, pull: 'toward', because: '' });
export function object(about, why) {
  if (!String(why ?? '').trim()) throw new Error('say why you object');
  return { op: 'position', about, pull: 'against', because: String(why).trim() };
}

/** The record's payload for `act`, which must be one the page offers. */
export function payload(act) {
  if (!OFFERS.includes(act?.op)) throw new Error(`House rules never writes ${act?.op}`);
  return { kind: 'act', app: 'canon', act };
}

/** Everything the page shows, canon's answers read from what `svrngs/acts`
 * answered (`read`) at `asked`, or at the newest act handed to canon when
 * that is later: svrngs times a new act after every act it holds
 * (crates/svrngs-core/src/command.rs:243-250), so one written in the second
 * of another is ahead of the browser's clock, and canon asked before it
 * says it passed a change yet keeps the old rule (#206). One answer, one
 * clock. `{unsettled}` when svrngs cannot say who stood when (#124), its
 * `spans` null, never an empty house; an answer with no `spans` at all is
 * not one svrngs gives, and is said as unread, never as unsettled (#209).
 * Each decision is handed to `trace`, `(what, fields)`, at debug (AGENTS.md
 * 1; #210): what isn't counted and why, canon's own error beside the page's
 * words when canon would not build an act, the time canon is asked at. */
export function fold(call, read, asked, trace = () => {}) {
  if (read.spans === null) return (trace('unsettled', { group: read.group }), { unsettled: true });
  if (!Array.isArray(read.spans)) throw new Error('this computer’s answer doesn’t say who is in your house');
  const members = membership(call, read.group, read.spans);
  const ours = new Set(members.map((l) => JSON.parse(l).id));
  const { kept, dropped } = recorded(read, ours);
  const lines = [];
  const who = {};
  for (const r of kept) {
    let line;
    try {
      line = call({ call: 'act', kind: r.act, ts_unix: r.ts, actor: human(r.by) }).trim();
    } catch (e) {
      dropped.push({ id: r.id, by: r.by, via: r.via, why: 'this version of House rules can’t read it', canon: e.message });
      continue;
    }
    who[JSON.parse(line).id] = { by: r.by, via: r.via };
    lines.push(line);
  }
  const acts = [...members, ...lines].join('\n');
  const newest = Math.max(...[...members, ...lines].map((l) => JSON.parse(l).ts_unix));
  const now = Math.max(asked, newest);
  for (const d of dropped) trace('not counted', d);
  trace('canon asked', { asked, at: now, newest });
  // canon leaves an empty list out of its answer.
  const canon = { commitments: [], ratifications: [], positions: [], unattended: [], bootstrap: [], ungoverned: [], ...call({ call: 'derive', acts, now }) };
  const whose = (id) => who[id] ?? null;
  const people = (pairs) => pairs.filter((id) => whose(Array.isArray(id) ? id[0] : id));
  const decided = canon.ratifications.some((r) => r.scope == null && r.verdict.verdict === 'ratified');
  const standing = read.spans.filter((s) => s.left == null).map((s) => s.person);
  const said = (id) => canon.positions.filter((p) => p.about === id && whose(p.act)).map((p) => ({ ...whose(p.act), agrees: p.position.pull === 'toward', why: p.position.because, at: p.at }));
  const entry = (c) => ({ id: c.id, text: c.text, status: c.status, at: c.asserted_at, ...whose(c.id), said: said(c.id) });
  const changes = canon.ratifications.filter((r) => r.scope == null && whose(r.act)).map((r) => ({ id: r.act, rule: r.rule, verdict: r.verdict, at: r.at, ...whose(r.act), said: said(r.act) }));
  const rule = decided ? call({ call: 'rule', acts, now }).rule : null;
  trace('how the house decides', { rule });
  return {
    rule,
    rules: canon.commitments.filter((c) => whose(c.id)).map(entry),
    changes,
    deciding: standing.filter((p) => call({ call: 'seat', acts, actor: human(p), now }) === 'held'),
    spans: read.spans,
    unattended: people(canon.unattended),
    bootstrap: people(canon.bootstrap),
    ungoverned: people(canon.ungoverned),
    dropped,
  };
}
