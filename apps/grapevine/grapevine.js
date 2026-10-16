// Grapevine: an ask that crosses rings through people (svrn-docs
// rings/current/GRAPEVINE.md §4, GRAPEVINE_DEMO.md §3-§4). This file is the
// app's fold, and the one decider (AGENTS.md 8): the page and the loop read
// its four answers and decide nothing of their own. Pure; no I/O, no clock.
//
// Three acts, each a payload in a group's record, every one `app: "grapevine"`:
//
//   ask    {kind, fields, hops, name, expires, seal_to, handle}   the asker's
//   carry  {kind, fields, hops, expires, seal_to, of, handle}     a carrier's, in
//                                                                 another group
//   msg    {handle, dir, from, to, nonce, box}                    one sealed message
//
// A kind's listing (`tool`'s, a tool the person has) is a plain record of
// its own app, LISTINGS, in the person's own group: never one of these, so
// never carried, and given to that kind's `notice` as the person's own.
// and the record's own correction: an asker's withdrawal, or a carrier's of
// the carry whose ask was withdrawn. `fold` takes the person's groups, each
// with the person's name in it (`me`, null where they do not stand) and its
// log as GET /v1/rail/log answers it, their state (their one-ask keys and
// their routes), the kinds there are, and the time, and answers:
//
//   asks       each live ask, folded by the original ask's id, with the
//              groups holding it, its carriers, its kind's `show` and `notice`;
//              each copy's `by`, and `via`, whose computer wrote it when that
//              was someone else's, for the page to say (ralph/FINDING_ON_BEHALF_OF.md)
//   carriable  each live ask paired with each group, and each pairing left
//              out with its reason (GRAPEVINE §4.2's decider)
//   threads    each ask this person asked or answered, its messages opened
//              with their own keys; and each they carried, its messages unopened
//   relay      each message the person's routes must forward and have not,
//              and each carry of theirs whose ask was withdrawn
//
// ── the rules worth reading before you change anything ──
//
// **A carry names its carrier and never the asker.** It copies the ask's
// kind, fields, expiry and seal_to, cites the original ask in `of`, and has
// a handle of its own; never the asker, the group it came from, or its
// handle (`carryAct`). The record names the carrier as its author.
//
// **An ask is one ask however it arrives.** Copies fold by the original
// ask's id, `of` for a carry, so two carriers into one group are one ask
// with two carriers.
//
// **A correction counts only from the person whose act it corrects**
// (§13.4), as in Events: the record voids whoever corrects, so this fold
// reads every act and decides the voids itself. Expiry needs no act: an
// ask or carry past its `expires` is gone.
//
// **Only the person's own groups are read.** A group the person does not
// stand in is never folded, and a kind's `notice` is given only the
// records the person wrote in their own groups.
//
// **Names wait for two yeses.** A `yes` carries its sender's name; a thread
// shows names only once both ends have sent one, and none before.
//
// **The state is JSON and nothing else**: no Set, no Map, no Date.

import { ACT_MOST, open } from './seal.mjs';

export const APP = 'grapevine';
export const ASK = 'ask';
export const CARRY = 'carry';
export const MSG = 'msg';
/** How many rings an ask may cross, at most (GRAPEVINE.md §4.1). */
export const MOST_HOPS = 3;
/** The most characters a text field or a message's text may hold: with MAX_FIELDS, a sealed body stays under BODY_MOST (§13.7; grapevine.test.mjs). */
export const MAX_TEXT = 1000;
/** The most fields a kind's ask, reply or listing may have. */
export const MAX_FIELDS = 6;
export const MAX_NAME = 64;
/** What each act says, exactly: a field beyond these is a word this fold cannot read. */
const KEYS = {
  [ASK]: ['app', 'type', 'kind', 'fields', 'hops', 'name', 'expires', 'seal_to', 'handle'],
  [CARRY]: ['app', 'type', 'kind', 'fields', 'hops', 'expires', 'seal_to', 'of', 'handle'],
  [MSG]: ['app', 'type', 'handle', 'dir', 'from', 'to', 'nonce', 'box'],
};
/** What a sealed message says, and what each saying holds besides `say`. */
const SAYS = { offer: ['fields', 'text'], yes: ['name'], no: [], text: ['text'] };
/** The field types a kind may use: a closed set (GRAPEVINE_DEMO §7). */
export const FIELD_TYPES = ['text', 'date', 'number', 'choice'];

/** Why a pairing of an ask and a group is not carriable: a closed set, each said. */
export const REFUSED = Object.freeze({
  unread: (g) => `${g} can't be read on this computer right now`,
  not_in_both: (g) => `you're not in ${g}`,
  expired: () => 'it has ended',
  already_held: (g) => `${g} already has it`,
  no_hops: () => 'the person who asked wanted it to go no further',
  would_name_asker: () => "it's your own ask, and carrying it would say so",
});

const isObject = (v) => v !== null && typeof v === 'object' && !Array.isArray(v);
const has = (map, k) => typeof k === 'string' && Object.prototype.hasOwnProperty.call(map, k);
const isKey = (k) => typeof k === 'string' && k.length > 0 && k.length <= 200 && k !== '__proto__';
// Base64 of exactly 16, 24 and 32 bytes, in its one spelling.
const HANDLE = /^[A-Za-z0-9+/]{21}[AQgw]==$/;
const NONCE = /^[A-Za-z0-9+/]{32}$/;
const PUBLIC = /^[A-Za-z0-9+/]{42}[AEIMQUYcgkosw048]=$/;
const isHandle = (s) => typeof s === 'string' && HANDLE.test(s);
const isPublic = (s) => typeof s === 'string' && PUBLIC.test(s);
const canonical = (v) => JSON.stringify(v, (_, x) => (isObject(x) ? Object.fromEntries(Object.entries(x).sort(([a], [b]) => (a < b ? -1 : a > b ? 1 : 0))) : x));

/** A field's type, from a kind's spec of it: `'text'`, `'date'`, `'number'` or `{choice: [...]}`. */
export function typeOf(spec) {
  if (FIELD_TYPES.includes(spec) && spec !== 'choice') return spec;
  if (isObject(spec) && Object.keys(spec).length === 1 && Array.isArray(spec.choice) && spec.choice.length && spec.choice.every((c) => typeof c === 'string' && c)) return 'choice';
  return null;
}

function isDate(s) {
  const m = typeof s === 'string' && /^(\d{4})-(\d{2})-(\d{2})$/.exec(s);
  if (!m) return false;
  const [y, mo, d] = [Number(m[1]), Number(m[2]), Number(m[3])];
  const leap = (y % 4 === 0 && y % 100 !== 0) || y % 400 === 0;
  const days = [31, leap ? 29 : 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31][mo - 1];
  return days !== undefined && d >= 1 && d <= days;
}

/**
 * What is wrong with `values` as `spec`'s fields (a kind's `ask`, `reply` or
 * `listing` fields), each said in the page's words, or []. **The one check**
 * of a field, for the form before anything is written and for the fold on
 * what was. A text is refused past MAX_TEXT, never cut.
 */
export function fieldGaps(spec, values) {
  if (!isObject(values)) return ['nothing filled in'];
  const out = Object.keys(values).filter((k) => !has(spec, k)).map((k) => `${k} is not asked for here`);
  for (const [k, s] of Object.entries(spec)) {
    const v = has(values, k) ? values[k] : undefined;
    const type = typeOf(s);
    if (type === 'text' && !(typeof v === 'string' && v.trim())) out.push(`${k} is empty`);
    else if (type === 'text' && v.length > MAX_TEXT) out.push(`${k} is ${v.length} characters, and the most is ${MAX_TEXT}`);
    else if (type === 'date' && !isDate(v)) out.push(`${k} is not a date`);
    else if (type === 'number' && !Number.isSafeInteger(v)) out.push(`${k} is not a whole number`);
    else if (type === 'choice' && !s.choice.includes(v)) out.push(`${k} is not one of ${s.choice.join(', ')}`);
    else if (type === null) out.push(`${k} has no type this app knows`);
  }
  return out;
}

/**
 * Everything wrong with one grapevine act's payload, as gaps; [] when it
 * reads. **The one validator**, for the server before anything is written
 * and for `fold` on what was.
 */
export function validate(op, kinds) {
  const gap = (code, message) => [{ code, message }];
  if (!isObject(op) || op.app !== APP || !has(KEYS, op.type)) return gap('not_grapevine', 'not one of grapevine\'s acts');
  const extra = Object.keys(op).filter((k) => !KEYS[op.type].includes(k));
  const missing = KEYS[op.type].filter((k) => !has(op, k));
  if (extra.length || missing.length) {
    return gap('unread_field', `a ${op.type} saying ${extra.length ? `more than this version reads (${extra.join(', ')})` : `too little (no ${missing.join(', ')})`}, so not counted`);
  }
  if (!isHandle(op.handle)) return gap('bad_handle', `a ${op.type} with no handle of 16 bytes`);
  if (op.type === MSG) {
    if (op.dir !== 'up' && op.dir !== 'down') return gap('bad_dir', 'a message going neither up nor down');
    if (!isPublic(op.from) || !isPublic(op.to)) return gap('bad_key', 'a message from or to no key');
    if (typeof op.nonce !== 'string' || !NONCE.test(op.nonce) || typeof op.box !== 'string' || !op.box || op.box.length > ACT_MOST) {
      return gap('bad_box', 'a message with nothing sealed in it');
    }
    return [];
  }
  if (!has(kinds, op.kind)) return gap('unknown_kind', `an ask of a kind this app does not have (${String(op.kind).slice(0, 40)})`);
  if (!Number.isSafeInteger(op.hops) || op.hops < 0 || op.hops > MOST_HOPS) return gap('bad_hops', `a ${op.type} going a number of rings that is not 0 to ${MOST_HOPS}`);
  if (!Number.isSafeInteger(op.expires)) return gap('bad_expires', `a ${op.type} with no end`);
  if (!isPublic(op.seal_to)) return gap('bad_key', `a ${op.type} with no key to answer it to`);
  if (op.type === ASK && op.name !== 'withheld') {
    // GRAPEVINE.md §4.1 allows `shown`; here a carry never names its asker
    // and names wait for two yeses, so `shown` would mean nothing. It is
    // refused, never read as withheld.
    return gap('name_shown', 'an ask that shows its asker\'s name, which this version does not carry');
  }
  if (op.type === CARRY && !isKey(op.of)) return gap('no_ask', 'a carry of no ask');
  const bad = fieldGaps(kinds[op.kind].ask.fields, op.fields);
  return bad.length ? gap('bad_fields', `a ${op.type} whose fields do not read: ${bad.join('; ')}`) : [];
}

/** What a sealed body says wrong for `kind`, or []. */
function bodyGaps(body, kind) {
  if (!isObject(body) || !has(SAYS, body.say)) return ['it says nothing this app reads'];
  const extra = Object.keys(body).filter((k) => k !== 'say' && !SAYS[body.say].includes(k));
  if (extra.length) return [`it says more than this version reads (${extra.join(', ')})`];
  const out = [];
  if (body.say === 'offer') out.push(...fieldGaps(kind.reply.fields, body.fields));
  if (body.say === 'text' || (body.say === 'offer' && body.text !== undefined)) {
    if (typeof body.text !== 'string' || !body.text.trim()) out.push('its words are empty');
    else if (body.text.length > MAX_TEXT) out.push(`its words are ${body.text.length} characters, and the most is ${MAX_TEXT}`);
  }
  if (body.say === 'yes' && !(typeof body.name === 'string' && body.name.trim() && body.name.length <= MAX_NAME)) out.push('a yes with no name');
  return out;
}

/**
 * The acts of a log, as GET /v1/rail/log answers it: every op carrying an
 * app's act or a correction, in the record's order, voided ones included.
 * `by` is `on_behalf_of` when the op names one, else its `person`; `via` is
 * the `person` when `on_behalf_of` names another, else null. Null when the
 * log holds no list of acts.
 */
export function actsOf(log) {
  const ops = Array.isArray(log) ? log : log?.ops;
  if (!Array.isArray(ops)) return null;
  return ops
    .filter((o) => isObject(o) && (o.payload != null || o.corrects != null))
    .map((o) => ({ id: o.id, ts: o.ts_unix, by: o.on_behalf_of ?? o.person, via: o.on_behalf_of != null && o.on_behalf_of !== o.person ? o.person : null, op: o.payload ?? null, corrects: o.corrects ?? null }));
}

/**
 * One group's acts, read: its asks and carries (`copies`, withdrawn ones
 * flagged), its messages, and the other apps' records the person wrote,
 * each act that could not be read a gap.
 */
function readGroup(g, kinds, gap) {
  const acts = {};
  const order = [];
  const all = actsOf(g.ops);
  if (all === null) gap('unread', g, null, `${g.name ?? g.id} answered no list of acts`);
  for (const a of all ?? []) {
    if (!isKey(a.id) || !Number.isSafeInteger(a.ts) || !isKey(a.by) || has(acts, a.id)) {
      gap('not_an_act', g, a?.id, 'an entry with no one or nothing in it');
      continue;
    }
    if (a.corrects != null && !isKey(a.corrects)) {
      gap('not_an_act', g, a.id, 'a change of nothing');
      continue;
    }
    acts[a.id] = a;
    order.push(a.id);
  }
  // A correction counts only from the person whose act it corrects (§13.4).
  const voided = {};
  for (const id of order) {
    const c = acts[id];
    if (c.corrects == null) continue;
    const was = has(acts, c.corrects) ? acts[c.corrects] : null;
    if (was?.by === c.by) voided[c.corrects] = true;
    else if (was) gap('not_theirs', g, id, `${c.by} tried to take back ${was.by}'s, which only ${was.by} can`);
    else gap('changes_unheld', g, id, `${c.by} changed something this computer does not hold yet`);
  }
  const copies = [];
  const msgs = [];
  const records = [];
  for (const id of order) {
    const { ts, by, via, op } = acts[id];
    if (op == null) continue;
    if (!isObject(op) || op.app !== APP) {
      if (!voided[id] && by === g.me && isObject(op)) records.push(structuredClone(op));
      continue;
    }
    const why = validate(op, kinds);
    if (why.length) {
      gap(why[0].code, g, id, why[0].message);
      continue;
    }
    const at = { id, group: g.id, by, via, ts, voided: Boolean(voided[id]) };
    if (op.type === MSG) msgs.push({ ...at, op: structuredClone(op) });
    else copies.push({ ...at, type: op.type, of: op.type === ASK ? id : op.of, kind: op.kind, fields: structuredClone(op.fields), hops: op.hops, expires: op.expires, seal_to: op.seal_to, handle: op.handle });
  }
  return { copies, msgs, records };
}

const byTime = (a, b) => a.ts - b.ts || (a.group < b.group ? -1 : a.group > b.group ? 1 : 0) || (a.id < b.id ? -1 : a.id > b.id ? 1 : 0);

/** Each ask, from its copies, folded by the original ask's id: [{id, copies}], in the order first seen. */
function byAsk(copies, gap, groupName) {
  const out = {};
  const order = [];
  for (const c of [...copies].sort(byTime)) {
    if (!has(out, c.of)) {
      out[c.of] = [c];
      order.push(c.of);
      continue;
    }
    const first = out[c.of][0];
    if (c.kind !== first.kind || c.seal_to !== first.seal_to || c.expires !== first.expires || canonical(c.fields) !== canonical(first.fields)) {
      gap('copy_differs', { id: c.group, name: groupName(c.group) }, c.id, `${c.by}'s copy of an ask says something other than the ask, so not counted`);
    } else {
      out[c.of].push(c);
    }
  }
  return order.map((id) => ({ id, copies: out[id] }));
}

/** The person's state, read: their one-ask keys by public key, and their routes. Anything else in it is a gap. */
function stateOf(state, gap) {
  const keys = {};
  for (const [pub, secret] of Object.entries(isObject(state?.keys) ? state.keys : {})) {
    if (isPublic(pub) && typeof secret === 'string') keys[pub] = secret;
    else gap('bad_state', null, null, 'a key in the state file that is not one');
  }
  const routes = [];
  for (const r of Array.isArray(state?.routes) ? state.routes : []) {
    if (isObject(r) && isKey(r.down_group) && isKey(r.up_group) && isHandle(r.down_handle) && isHandle(r.up_handle)) {
      routes.push({ down_group: r.down_group, down_handle: r.down_handle, up_group: r.up_group, up_handle: r.up_handle });
    } else {
      gap('bad_state', null, null, 'a route in the state file that is not one');
    }
  }
  return { keys, routes };
}

/**
 * The person's view: their asks, what they may carry, their threads and what
 * their routes must relay, and every act not counted, as a gap saying why.
 *
 *   groups  [{id, name, me, ops} | {id, name, me, unread}]: every group this
 *           computer holds; `me` the person's name in it, null where they
 *           do not stand; `unread` why its record could not be read
 *   state   {keys: {<public>: <secret>}, routes: [{down_group, down_handle,
 *           up_group, up_handle}]}: the person's own state file
 *   kinds   {<kind>: kind}: the kinds there are
 *   now     the time, in unix seconds
 */
export function fold({ groups, state, kinds, now }) {
  const gaps = [];
  const gap = (code, g, id, message) => gaps.push({ code, group: g?.id ?? null, id: typeof id === 'string' ? id : null, message });
  const nameOf = {};
  for (const g of groups) nameOf[g.id] = typeof g.name === 'string' ? g.name : g.id;
  const groupName = (id) => (has(nameOf, id) ? nameOf[id] : id);
  for (const g of groups) if (g.unread != null && g.me != null) gap('unread', g, null, `${groupName(g.id)} can't be read on this computer right now`);
  // Only the person's own groups are read (the law `notice` sees only its person's own).
  const own = groups.filter((g) => g.me != null && g.unread == null);
  const ownIds = own.map((g) => g.id);
  const { keys, routes } = stateOf(state, gap);
  const read = own.map((g) => readGroup(g, kinds, gap));
  const copies = read.flatMap((r) => r.copies);
  const msgs = read.flatMap((r) => r.msgs);
  const mine = { records: read.flatMap((r) => r.records) };

  // ── asks ──
  const standing = byAsk(copies.filter((c) => !c.voided), gap, groupName);
  const asks = [];
  for (const { id, copies: cs } of standing) {
    const first = cs[0];
    const kind = kinds[first.kind];
    const live = cs.filter((c) => c.expires > now);
    const entry = {
      id,
      kind: first.kind,
      fields: structuredClone(first.fields),
      expires: first.expires,
      seal_to: first.seal_to,
      hops: Math.max(...cs.map((c) => c.hops)),
      asked: has(keys, first.seal_to),
      in: cs.map((c) => ({ group: c.group, name: groupName(c.group), act: c.id, by: c.by, via: c.via, handle: c.handle, hops: c.hops, carried: c.type === CARRY, live: c.expires > now })),
      carriers: [...new Set(cs.filter((c) => c.type === CARRY).map((c) => c.by))].sort(),
      live: live.length > 0,
    };
    try {
      entry.show = String(kind.show(structuredClone(first.fields)));
    } catch (e) {
      gap('unshown', { id: first.group }, first.id, `the ${first.kind} kind could not show an ask: ${e?.message ?? e}`);
      continue;
    }
    entry.notice = false;
    if (!entry.asked && typeof kind.notice === 'function') {
      try {
        entry.notice = kind.notice({ kind: first.kind, fields: structuredClone(first.fields) }, structuredClone(mine)) === true;
      } catch (e) {
        gap('notice_failed', { id: first.group }, first.id, `the ${first.kind} kind could not say whether you might help: ${e?.message ?? e}`);
      }
    }
    asks.push(entry);
  }

  // ── carriable: GRAPEVINE §4.2's decider, each pairing left out said ──
  const carriable = { can: [], not: [] };
  for (const a of asks) {
    for (const g of groups) {
      const name = groupName(g.id);
      const why =
        g.unread != null ? 'unread'
        : g.me == null ? 'not_in_both'
        : !a.live ? 'expired'
        : a.in.some((c) => c.group === g.id) ? 'already_held'
        : a.hops < 1 ? 'no_hops'
        : a.asked ? 'would_name_asker'
        : null;
      if (why) {
        carriable.not.push({ ask: a.id, group: g.id, why, words: REFUSED[why](name) });
        continue;
      }
      const up = a.in.filter((c) => c.live).reduce((best, c) => (c.hops > best.hops ? c : best));
      carriable.can.push({ ask: a.id, group: g.id, name, hops: up.hops - 1, up: { group: up.group, handle: up.handle } });
    }
  }

  // ── threads ──
  const seen = {};
  const held = [];
  for (const m of [...msgs].sort(byTime)) {
    if (m.voided || has(seen, m.op.nonce)) continue;
    seen[m.op.nonce] = true;
    held.push(m);
  }
  const askBy = {};
  for (const { id, copies: cs } of byAsk(copies, () => {}, groupName)) askBy[cs[0].seal_to] ??= { id, copy: cs.find((c) => c.type === ASK && ownIds.includes(c.group)) ?? cs[0] };
  const liveAsk = {};
  for (const a of asks) if (a.live) liveAsk[a.id] = a;
  const threads = [];
  const thread = {};
  for (const m of held) {
    const { op } = m;
    const sent = !has(keys, op.to) && has(keys, op.from);
    if (!sent && !has(keys, op.to)) continue;
    const [me, them] = sent ? [op.from, op.to] : [op.to, op.from];
    const ask = askBy[me] ?? askBy[them];
    if (!ask) {
      gap('no_ask', { id: m.group }, m.id, 'a message about an ask this computer does not hold');
      continue;
    }
    const kind = kinds[ask.copy.kind];
    const body = open(op, them, keys[me]);
    const bad = body === null ? ['it does not open with your key'] : bodyGaps(body, kind);
    if (bad.length) {
      gap('unopened', { id: m.group }, m.id, `a message that does not read: ${bad.join('; ')}`);
      continue;
    }
    const key = `${me} ${them}`;
    if (!has(thread, key)) {
      const asker = askBy[me] !== undefined;
      thread[key] = {
        ask: ask.id,
        kind: ask.copy.kind,
        show: has(liveAsk, ask.id) ? liveAsk[ask.id].show : null,
        live: has(liveAsk, ask.id),
        role: asker ? 'asker' : 'replier',
        me,
        them,
        messages: [],
        names: null,
        yes: {},
        // Where this end writes: the asker down under their ask's handle;
        // a replier up under the handle they first wrote to.
        send: asker ? { group: ask.copy.group, handle: ask.copy.handle, dir: 'down', from: me, to: them } : sent ? { group: m.group, handle: op.handle, dir: 'up', from: me, to: them } : null,
      };
      threads.push(thread[key]);
    }
    const t = thread[key];
    if (!t.send && sent) t.send = { group: m.group, handle: op.handle, dir: 'up', from: me, to: them };
    const said = { at: m.ts, mine: sent, say: body.say };
    if (body.fields !== undefined) said.fields = body.fields;
    if (body.text !== undefined) said.text = body.text;
    t.messages.push(said);
    if (body.say === 'yes') t.yes[sent ? 'me' : 'them'] ??= body.name.trim();
  }
  for (const t of threads) {
    // Names wait for two yeses.
    if (t.yes.me !== undefined && t.yes.them !== undefined) t.names = { me: t.yes.me, them: t.yes.them };
    delete t.yes;
  }

  // ── what passed through the person's routes, and what they must relay ──
  const relay = [];
  // Every live message, its forwarded copies too: what a route has relayed.
  const passing = msgs.filter((m) => !m.voided).sort(byTime);
  const under = (g, handle, dir) => passing.filter((m) => m.group === g && m.op.handle === handle && (!dir || m.op.dir === dir));
  for (const r of routes) {
    const [down, up] = [own.find((g) => g.id === r.down_group), own.find((g) => g.id === r.up_group)];
    if (!down || !up) {
      gap('route_closed', { id: (down ? r.up_group : r.down_group) }, null, `a route through ${groupName(down ? r.up_group : r.down_group)}, where you're not now`);
      continue;
    }
    const upstream = copies.find((c) => c.group === r.up_group && c.handle === r.up_handle);
    const passed = [...under(r.down_group, r.down_handle), ...under(r.up_group, r.up_handle)]
      .filter((m, i, all) => all.findIndex((n) => n.op.nonce === m.op.nonce) === i)
      .sort(byTime)
      .map((m) => ({ at: m.ts, dir: m.op.dir, size: Math.floor((m.op.box.length * 3) / 4) }));
    if (upstream) {
      threads.push({ ask: upstream.of, kind: upstream.kind, show: has(liveAsk, upstream.of) ? liveAsk[upstream.of].show : null, live: has(liveAsk, upstream.of), role: 'carrier', from: groupName(r.up_group), to: groupName(r.down_group), passed });
    }
    const there = (g, handle, nonce) => passing.some((m) => m.group === g && m.op.handle === handle && m.op.nonce === nonce);
    for (const m of under(r.down_group, r.down_handle, 'up')) {
      if (!there(r.up_group, r.up_handle, m.op.nonce)) relay.push({ do: 'forward', group: r.up_group, payload: { ...m.op, handle: r.up_handle } });
    }
    // Down only to a replier whose own message came up this way.
    const repliers = under(r.down_group, r.down_handle, 'up').map((m) => m.op.from);
    for (const m of under(r.up_group, r.up_handle, 'down')) {
      if (repliers.includes(m.op.to) && !there(r.down_group, r.down_handle, m.op.nonce)) relay.push({ do: 'forward', group: r.down_group, payload: { ...m.op, handle: r.down_handle } });
    }
    const carried = copies.find((c) => c.group === r.down_group && c.handle === r.down_handle && c.type === CARRY && c.by === down.me && !c.voided);
    if (upstream?.voided && carried) relay.push({ do: 'correct', group: r.down_group, corrects: carried.id });
  }

  return { asks: asks.filter((a) => a.live), carriable, threads, relay, gaps };
}

/** The words a kind may define, and nothing else: no routing, scope, handle, sealing, carrying or name. */
const KIND_KEYS = ['kind', 'ask', 'show', 'reply', 'listing', 'notice', 'hops', 'name'];
const FIELD_NAME = /^[a-z][a-z0-9_]{0,39}$/;

/** What is wrong with a kind's `ask`, `reply` or `listing` (`part`), or []. */
function partGaps(part, v) {
  if (!isObject(v) || Object.keys(v).join() !== 'fields' || !isObject(v.fields)) return [`its ${part} is not {fields: {...}}`];
  const names = Object.keys(v.fields);
  const out = [];
  if (names.length < 1 || names.length > MAX_FIELDS) out.push(`its ${part} has ${names.length} fields, and a kind has 1 to ${MAX_FIELDS}`);
  for (const n of names) {
    if (!FIELD_NAME.test(n)) out.push(`its ${part} field ${JSON.stringify(n)} is not a short lower-case name`);
    else if (typeOf(v.fields[n]) === null) out.push(`its ${part} field ${n} is not one of ${FIELD_TYPES.join(', ')} (a choice is {choice: [...]})`);
  }
  return out;
}

/**
 * The kinds there are, from a list of kind files' default exports, by
 * their word (GRAPEVINE_DEMO §7). A kind that breaks the interface throws,
 * saying how, when the app starts: it is a developer's to fix, never
 * something to run around.
 */
export function kindsOf(list) {
  const out = {};
  for (const k of list) {
    const word = isObject(k) && typeof k.kind === 'string' ? k.kind : '(unnamed)';
    const bad = [];
    if (!isObject(k)) bad.push('is not an object');
    else {
      const extra = Object.keys(k).filter((x) => !KIND_KEYS.includes(x));
      if (extra.length) bad.push(`sets ${extra.join(', ')}; a kind writes no routing, scope, handle, sealing, carrying or name, which the app does`);
      if (typeof k.kind !== 'string' || !/^[a-z][a-z0-9-]{0,30}$/.test(k.kind)) bad.push('has no short lower-case word for `kind`');
      bad.push(...partGaps('ask', k.ask), ...partGaps('reply', k.reply));
      if (k.listing !== undefined) bad.push(...partGaps('listing', k.listing));
      if (typeof k.show !== 'function') bad.push('has no `show`');
      if (k.notice !== undefined && typeof k.notice !== 'function') bad.push('has a `notice` that is not a function');
      if (k.hops !== undefined && !(Number.isSafeInteger(k.hops) && k.hops >= 0 && k.hops <= MOST_HOPS)) bad.push(`has hops that are not 0 to ${MOST_HOPS}`);
      if (k.name !== undefined && k.name !== 'withheld') bad.push('has a name other than withheld, the one this version carries');
    }
    if (!bad.length && has(out, k.kind)) bad.push('is a second kind of that word');
    if (bad.length) throw new Error(`the ${word} kind ${bad.join('; ')}`);
    out[k.kind] = k;
  }
  return out;
}

/**
 * What is wrong with what the person filled in, for `kind`'s `part` (`ask`,
 * `reply` or `listing`), or for a message's own words (`part` 'text'),
 * each said with the limit: the form refuses it, and nothing is cut short.
 */
export function formGaps(kind, part, values) {
  if (part === 'text') {
    if (typeof values !== 'string' || !values.trim()) return ['your message is empty'];
    return values.length > MAX_TEXT ? [`your message is ${values.length} characters, and the most is ${MAX_TEXT}`] : [];
  }
  if (!isObject(kind?.[part])) return [`a ${kind?.kind ?? ''} has no ${part}`];
  return fieldGaps(kind[part].fields, values);
}

/** The app other kinds' listings are written as: plain records, never carried. */
export const LISTINGS = 'grapevine-listings';

/** A listing's payload, a plain record in the person's own group: never an ask, so never carried. */
export function listingAct(kind, fields) {
  return { app: LISTINGS, kind, listing: fields };
}

/** A new ask's payload. `seal_to` is the ask's own public key, `handle` 16 random bytes in base64. */
export function askAct({ kind, fields, hops, expires, seal_to, handle }) {
  return { app: APP, type: ASK, kind, fields, hops, name: 'withheld', expires, seal_to, handle };
}

/**
 * The carry of `ask` (an entry of `asks`) that `can` (an entry of
 * `carriable.can`) allows, with a handle of its own: the payload, the group
 * it is written in, and the route the carrier keeps. It copies the ask's
 * kind, fields, expiry and seal_to and cites its id; never the asker, the
 * group it came from or that group's handle.
 */
export function carryAct(ask, can, handle) {
  return {
    group: can.group,
    payload: { app: APP, type: CARRY, kind: ask.kind, fields: ask.fields, hops: can.hops, expires: ask.expires, seal_to: ask.seal_to, of: ask.id, handle },
    route: { down_group: can.group, down_handle: handle, up_group: can.up.group, up_handle: can.up.handle },
  };
}

/** A message's payload: `sealed` is seal's {nonce, box}, `to` the key it is sealed to. */
export function msgAct({ handle, dir, from, to }, sealed) {
  return { app: APP, type: MSG, handle, dir, from, to, nonce: sealed.nonce, box: sealed.box };
}
