// Events: a post in a group, and the people who answer it (svrn-docs
// rings/current/EVENTS.md §1). This file is the app's fold; the runtime
// orders its acts and has never heard of an event.
//
// Three acts, each a payload in the group's record:
//
//   event   {kind, event, what, when, where}   written by whoever hosts
//   coming  {kind, event, guests?}             each person's own, guests by name
//   came    {kind, event, guest?}              each person's own, or, with
//                                              `guest`, a guest they brought
//
// `actsOf` reads them from the record a member's node serves over loopback
// (`GET /v1/rail/log`, DESIGN §3), or from the app's own file (`store.mjs`);
// `init` and `reduce` fold them as the record orders them; `answers` says
// what they come to. Tests beside it: `events.test.mjs`, the laws in
// `laws.test.mjs`; `canary.mjs` folds two computers' copies.
//
// ── the rules worth reading before you change anything ──
//
// **The person is the act's speaker, never a name in the payload.** A
// `coming` or `came` has nowhere to put anyone else, so no one answers in
// another's name; a guest is a name tied to the member who wrote it, so
// only that member can say their guest came.
//
// **A correction counts only from the person whose act it corrects.** The
// record voids an act whoever corrects it (svrngs-core admit.rs, `voided`),
// so this fold reads every act, voided ones too, and decides the voids
// itself: Bo changes his own mind, and the host changes the event, which is
// the host's act. Anyone else's correction is a gap, and changes nothing.
//
// **Nothing records "no."** There is no act for it, and an answer carrying
// a field this fold does not read is not counted at all — a `coming` that
// also says `coming: false` is neither a yes nor a no. So no one can be shown
// declining or not showing up: the record never holds it.
//
// **The host is whoever wrote the event first in the record's order.** Its
// id is minted by the host's computer (`server.mjs`), the same in every group
// it is posted to, so the host's view joins them.
//
// **The state is JSON and nothing else**: no Set, no Map, no Date, and
// nothing kept between calls in a module variable.

export const EVENT = 'event';
export const COMING = 'coming';
export const CAME = 'came';
export const MAX_WORDS = 200;
const MAX_NAME = 60;
const MAX_GUESTS = 20;
/** What each kind may say: a field beyond these is a word this fold cannot read. */
const FIELDS = {
  [EVENT]: ['kind', 'event', 'what', 'when', 'where'],
  [COMING]: ['kind', 'event', 'guests'],
  [CAME]: ['kind', 'event', 'guest'],
};

const isObject = (v) => v !== null && typeof v === 'object' && !Array.isArray(v);
// A key of the state's maps. `__proto__` as a key would set a prototype
// rather than store a value.
const isKey = (k) => typeof k === 'string' && k.length > 0 && k.length <= 200 && k !== '__proto__';
const has = (map, k) => typeof k === 'string' && Object.prototype.hasOwnProperty.call(map, k);
const isWords = (s, max) => typeof s === 'string' && s.length <= max;
/** A guest's name as a member wrote it, trimmed; null when it is not one. */
export function guestName(text) {
  if (typeof text !== 'string') return null;
  const name = text.trim();
  return name && name.length <= MAX_NAME && name !== '__proto__' ? name : null;
}

/**
 * Everything wrong with one act, as gaps. **The one validator**, for the
 * server before anything is written and for `reduce` on what was. Every gap
 * is fatal: an answer read in part is not a cautious reading of it.
 */
export function validate(act) {
  const gap = (code, message) => ({ code, fatal: true, message });
  if (!isObject(act) || !isKey(act.id) || !Number.isSafeInteger(act.ts)) {
    return [gap('not_an_act', 'an entry with nothing in it')];
  }
  if (!isKey(act.by)) return [gap('no_one', 'an entry that no one in the group wrote')];
  if (act.corrects != null && !isKey(act.corrects)) return [gap('not_an_act', 'a change of nothing')];
  if (act.op == null) {
    return act.corrects == null ? [gap('not_an_act', 'an entry with nothing in it')] : [];
  }
  const { op } = act;
  if (!isObject(op) || !has(FIELDS, op.kind)) {
    return [gap('unknown_kind', 'an entry a newer version of Events may know how to read')];
  }
  const extra = Object.keys(op).filter((k) => !FIELDS[op.kind].includes(k));
  if (extra.length) {
    return [gap('unread_field', `${op.kind} saying more than this version reads (${extra.join(', ')}), so not counted`)];
  }
  if (!isKey(op.event)) return [gap('no_event', `${op.kind} for no event`)];
  if (op.kind === EVENT) {
    const gaps = [];
    if (typeof op.what !== 'string' || !op.what.trim() || op.what.length > MAX_WORDS) {
      gaps.push(gap('no_what', `an event for nothing in particular, or over ${MAX_WORDS} characters`));
    }
    if (!isWords(op.when, MAX_WORDS) || !isWords(op.where, MAX_WORDS)) {
      gaps.push(gap('bad_when_where', `a when or a where that is not words, or over ${MAX_WORDS} characters`));
    }
    return gaps;
  }
  if (op.kind === COMING) {
    if (op.guests === undefined) return [];
    const ok = Array.isArray(op.guests) && op.guests.length <= MAX_GUESTS && op.guests.every((g) => guestName(g) !== null);
    return ok ? [] : [gap('bad_guests', `guests that are not up to ${MAX_GUESTS} names of up to ${MAX_NAME} characters`)];
  }
  return op.guest === undefined || guestName(op.guest) !== null ? [] : [gap('bad_guest', 'a guest with no name')];
}

/**
 * The acts `reduce` folds, from the record's answer to `GET /v1/rail/log`:
 * every op that carries an app's act or a correction, in the record's
 * order, **voided ones included** — the record voids whoever corrects, and
 * this fold decides whose correction counts. `by` is `on_behalf_of` when
 * the op names one, else its `person`; `via` is the `person` when
 * `on_behalf_of` names another — whose computer wrote it for them
 * (ralph/FINDING_ON_BEHALF_OF.md) — else null. A log with no list of ops is
 * an error, never a group with no answers.
 */
export function actsOf(log) {
  if (!Array.isArray(log?.ops)) throw new Error(`the record answered no list of acts: ${JSON.stringify(log)}`);
  return log.ops
    .filter((o) => o.payload != null || o.corrects != null)
    .map((o) => ({ id: o.id, ts: o.ts_unix, by: o.on_behalf_of ?? o.person, via: o.on_behalf_of != null && o.on_behalf_of !== o.person ? o.person : null, op: o.payload ?? null, corrects: o.corrects ?? null }));
}

/** No acts yet. */
export function init() {
  return { acts: {}, order: [], gaps: [] };
}

/** Take one act in the record's order. One that cannot be read is a gap, never a throw (the totality law). */
export function reduce(state, act) {
  const gaps = validate(act);
  for (const g of gaps) state.gaps.push({ ...g, id: isObject(act) && typeof act.id === 'string' ? act.id : null });
  if (gaps.length || has(state.acts, act.id)) return state;
  state.acts[act.id] = { by: act.by, via: act.via ?? null, ts: act.ts, op: act.op ?? null, corrects: act.corrects ?? null };
  state.order.push(act.id);
  return state;
}

const add = (list, item) => (list.includes(item) ? list : [...list, item].sort());
/** `via` added under `k` in `map`, when there is one: the computers an answer came through. */
const through = (map, k, via) => {
  if (via) map[k] = add(has(map, k) ? map[k] : [], via);
};

/**
 * What the acts come to: each event with its host, who is coming (with the
 * guests each brings) and who came (with the guests each says came), and
 * every act not counted, as a gap saying why. Only what someone said is
 * here: no one appears who did not answer. `via` says whose computer wrote
 * the post (`host`) and each answer, where it was someone else's: under
 * `coming`, `came` and `guests_came`, as they are keyed, the computers each
 * came through.
 */
export function answers(state) {
  const { voided, uncounted } = voids(state);
  const gaps = [...state.gaps, ...Object.values(uncounted)];
  const gap = (code, id, message) => gaps.push({ code, fatal: true, id, message });
  const events = {};
  const live = state.order.filter((id) => !has(voided, id) && !has(uncounted, id) && state.acts[id].op);
  for (const id of live) {
    const { by, via, ts, op } = state.acts[id];
    if (op.kind !== EVENT) continue;
    if (!has(events, op.event)) {
      const vias = { host: via, coming: {}, came: {}, guests_came: {} };
      events[op.event] = { host: by, what: op.what.trim(), when: op.when, where: op.where, at: ts, coming: {}, came: [], guests_came: {}, via: vias };
    } else {
      gap(events[op.event].host === by ? 'posted_twice' : 'not_the_host', id, `${by} posted ${op.event} again; ${events[op.event].host} hosts it`);
    }
  }
  for (const id of live) {
    const { by, via, op } = state.acts[id];
    if (op.kind === EVENT) continue;
    if (!has(events, op.event)) {
      gap('no_event', id, `${by} answered an event this copy does not hold`);
      continue;
    }
    const e = events[op.event];
    if (op.kind === COMING) {
      // Saying it twice is saying it once; a guest named twice is one guest.
      let guests = has(e.coming, by) ? e.coming[by] : [];
      for (const g of op.guests ?? []) guests = add(guests, guestName(g));
      e.coming[by] = guests;
      through(e.via.coming, by, via);
    } else if (op.guest === undefined) {
      e.came = add(e.came, by);
      through(e.via.came, by, via);
    } else {
      e.guests_came[by] = add(has(e.guests_came, by) ? e.guests_came[by] : [], guestName(op.guest));
      if (via) through((e.via.guests_came[by] ??= {}), guestName(op.guest), via);
    }
  }
  return { events, gaps };
}

/** Fold a group's acts from scratch: what one group's record says. */
export function fold(acts) {
  return answers(acts.reduce(reduce, init()));
}

/**
 * The acts that change `person`'s own answers to `event`, newest first: what
 * a correction names when they change their mind. Their live `coming` acts,
 * or their live `came` acts for themselves (`guest` null) or for one guest.
 */
export function ownActs(state, person, event, kind, guest = null) {
  const { voided } = voids(state);
  return state.order
    .filter((id) => !has(voided, id))
    .filter((id) => {
      const a = state.acts[id];
      return a.by === person && a.op && a.op.event === event && a.op.kind === kind &&
        (kind !== CAME || (a.op.guest === undefined ? guest === null : guestName(a.op.guest) === guest));
    })
    .reverse();
}

/**
 * The acts voided, each by its own person's correction, and the corrections
 * that count for nothing, each with its gap: one by someone else, or of an
 * act this copy does not hold yet.
 */
function voids(state) {
  const voided = {};
  const uncounted = {};
  for (const id of state.order) {
    const c = state.acts[id];
    if (c.corrects == null) continue;
    const was = has(state.acts, c.corrects) ? state.acts[c.corrects] : null;
    if (was?.by === c.by) {
      voided[c.corrects] = true;
    } else {
      uncounted[id] = was
        ? { code: 'not_theirs', fatal: true, id, message: `${c.by} tried to change ${was.by}'s answer, which only ${was.by} can change` }
        : { code: 'changes_unheld', fatal: true, id, message: `${c.by} changed an answer this copy does not hold yet` };
    }
  }
  return { voided, uncounted };
}

/**
 * The host's view of `event`: each group's answers to it, from the groups
 * the host's computer holds, where the one who asks hosts it there.
 * `groups` is `[{group, answers, me}]`, `me` the asker's name in that
 * group; a group whose answers could not be read is `{group, unread}`, and
 * is said as that, never as nobody answering. A group where someone else
 * hosts an event of that id, or holds none, adds nothing.
 */
export function hosted(event, groups) {
  return groups
    .filter((g) => g.unread || (has(g.answers.events, event) && g.answers.events[event].host === g.me))
    .map((g) => (g.unread ? { group: g.group, unread: g.unread } : { group: g.group, event: g.answers.events[event] }));
}
