// The person's own process, but its page (GRAPEVINE_DEMO §5, §12.1): their
// state file, the loop that relays for them, and each thing they do, written
// through the store as them. Every one reads the fold's four answers
// (grapevine.js) and decides nothing of its own (AGENTS.md 8).
//
//   person({id, store, held, kinds, state})
//     view()          the fold's answers, with the groups as read and each read's time
//     pass()          one turn of the loop: each `relay` entry done
//     ask, carry, reply, yes, no, withdraw, list
//                     what the page's routes do, each refused in its words
//   heldIn(store, name)       file mode: the groups `name` stands in
//   heldBy(api, store)        record mode: each group's "me" as svrngs answers it
//   loop(people)              each person's pass every RELAY_EVERY
//
// **The loop** (§5), every RELAY_EVERY for each person: read each of their
// groups' logs and fold; append each message the fold says their routes owe,
// unchanged, in the next group with that group's handle swapped in (the
// fold's `forward`); correct each of their carries whose ask was withdrawn
// (the fold's `correct`). No standing carry (§12.1): every carry is a tap.
//
// **The state file** is the person's own and the one file that holds a
// secret: each one-ask secret key, by its public key, and each route. It is
// written whole through a file renamed into place, before the act that
// needs it, so an act is never written whose key or route is lost.
//
// **Sealed means sealed.** Every branch traces under GRAPEVINE_DEBUG, as
// Events does with EVENTS_DEBUG, and what it traces is ids, groups, counts,
// sizes and times: never a message's words, a field, or a key
// (relay.test.mjs).

import { randomBytes } from 'node:crypto';
import { readFileSync, renameSync, writeFileSync } from 'node:fs';
import { performance } from 'node:perf_hooks';

import { MAX_NAME, MOST_HOPS, askAct, carryAct, fold, formGaps, listingAct, msgAct, validate } from './grapevine.js';
import { keyPair, seal } from './seal.mjs';

/** How often each person's loop reads and relays, in ms (GRAPEVINE_DEMO §5). */
export const RELAY_EVERY = 5_000;

/** Every decision branch, at debug (AGENTS.md 1), when GRAPEVINE_DEBUG is set. */
export const debug = process.env.GRAPEVINE_DEBUG ? (what, fields) => console.error(JSON.stringify({ at: new Date().toISOString(), what, ...fields })) : () => {};

/** A new handle: 16 random bytes, base64. */
export const newHandle = () => randomBytes(16).toString('base64');
const unix = () => Math.floor(Date.now() / 1000);

/** A thing the person tried, refused, in the page's words. */
export class Refused extends Error {
  constructor(words, status = 400) {
    super(words);
    this.status = status;
  }
}

/**
 * The person's state file at `path`: read, or new when there is none. One
 * that is there and does not read is said, never replaced: it holds keys.
 */
export function stateFile(path) {
  let state;
  try {
    state = JSON.parse(readFileSync(path, 'utf8'));
  } catch (e) {
    if (e.code !== 'ENOENT') throw new Error(`${path} does not read, and it holds this person's keys, so nothing was replaced: ${e.message}`);
    state = { keys: {}, routes: [] };
  }
  return {
    state,
    save() {
      writeFileSync(`${path}.new`, JSON.stringify(state), { mode: 0o600 });
      renameSync(`${path}.new`, path);
    },
  };
}

/** File mode: the groups the file's `name` stands in, each with them as "me". */
export function heldIn(store, name) {
  return async () => {
    const out = [];
    for (const g of await store.groups()) {
      if ((await store.people(g.id)).some((p) => p.name === name)) out.push({ id: g.id, name: g.name, me: name });
    }
    return out;
  };
}

/**
 * Record mode: every group this computer holds, with this computer's
 * person in it as svrngs answers it (`GET /v1/groups`'s `person`), checked
 * against the group's people; null where it answers none (§13.3). Never a
 * name from the config.
 */
export function heldBy(api, store, trace = debug) {
  return async () => {
    let groups;
    try {
      const res = await fetch(`${api}/v1/groups`);
      if (!res.ok) throw new Error(`${res.status}`);
      ({ groups } = await res.json());
    } catch (e) {
      trace('refused: svrngs did not say who is here', { error: e.message });
      throw new Refused("svrngs on this computer isn't answering, so this page can't say who you are.", 502);
    }
    const out = [];
    for (const g of groups) {
      let me = typeof g.person === 'string' ? g.person : null;
      if (me !== null) {
        const people = await store.people(g.id).catch(() => null);
        if (people && !people.some((p) => p.name === me)) {
          trace("this computer's person is not among the group's people", { group: g.id });
          me = null;
        }
      }
      trace('me', { group: g.id, standing: me !== null });
      out.push({ id: g.id, name: g.name ?? g.id, me });
    }
    return out;
  };
}

/**
 * One person: `id` names them to the process (the file's name, or `me`),
 * `held` answers their groups, `state` is their stateFile, and `timed` is
 * told each read of a group's log, {group, ms, ops}.
 */
export function person({ id, store, held, kinds, state, trace = debug, timed = () => {} }) {
  let busy = Promise.resolve();
  /** One thing at a time per person: the loop and the page never interleave. */
  const serial = (fn) => {
    const run = busy.then(fn, fn);
    busy = run.catch(() => {});
    return run;
  };

  async function view() {
    const groups = [];
    const reads = [];
    for (const g of await held()) {
      if (g.me == null) {
        groups.push(g);
        continue;
      }
      const t0 = performance.now();
      try {
        const { ops, complete } = await store.log(g.id);
        reads.push({ group: g.id, ms: performance.now() - t0, ops: ops.length });
        timed(reads.at(-1));
        groups.push({ ...g, ops, complete });
      } catch (e) {
        trace("a group's log could not be read", { person: id, group: g.id, error: e.message });
        groups.push({ ...g, unread: e.message });
      }
    }
    const v = fold({ groups, state: state.state, kinds, now: unix() });
    trace('folded', { person: id, groups: groups.length, asks: v.asks.length, can: v.carriable.can.length, threads: v.threads.length, relay: v.relay.length, gaps: v.gaps.map((g) => g.code) });
    return { ...v, groups, reads };
  }

  /** The group `group` as the person stands in it, or refused. */
  const standing = (v, group) => {
    const g = v.groups.find((x) => x.id === group && x.me != null && x.unread == null);
    if (!g) throw new Refused(v.groups.some((x) => x.id === group && x.unread != null) ? "That group can't be read on this computer right now." : "You're not in that group.", 403);
    return g;
  };

  /** Write `act` in `group` as the person, a grapevine payload checked by the one validator first. */
  async function write(v, group, act, what) {
    const me = standing(v, group).me;
    if (act.payload?.app === 'grapevine') {
      const gaps = validate(act.payload, kinds);
      if (gaps.length) {
        trace('refused: it would not read', { person: id, doing: what, group, gap: gaps[0].code });
        throw new Refused(gaps[0].message);
      }
    }
    try {
      const at = await store.append(group, act, me);
      trace('written', { person: id, doing: what, group, id: at });
      return at;
    } catch (e) {
      trace('could not write', { person: id, doing: what, group, error: e.message });
      throw new Refused("This computer didn't save that just now. Try again in a moment.", 502);
    }
  }

  const fields = (kind, part, values) => {
    const gaps = formGaps(kind, part, values);
    if (gaps.length) {
      trace('refused: the form', { person: id, part, gaps: gaps.length });
      throw new Refused(gaps.join('; '));
    }
  };
  const kindOf = (word) => {
    if (typeof word !== 'string' || !Object.prototype.hasOwnProperty.call(kinds, word)) throw new Refused('This app has no ask of that kind.');
    return kinds[word];
  };
  const askOf = (v, ask) => {
    const a = v.asks.find((x) => x.id === ask);
    if (!a) {
      trace('refused: no such ask', { person: id, ask });
      throw new Refused("That ask isn't here any more: it was taken back, or it ended.", 404);
    }
    return a;
  };
  const threadOf = (v, ask, them) => {
    const t = v.threads.find((x) => x.role !== 'carrier' && x.ask === ask && x.them === them);
    if (!t?.send) {
      trace('refused: no such conversation', { person: id, ask, found: Boolean(t) });
      throw new Refused("That conversation isn't here.", 404);
    }
    return t;
  };
  /** A sealed message in thread `t`, from this end's own key. */
  const say = (v, t, body, what) => write(v, t.send.group, { op: 'record', payload: msgAct(t.send, seal(body, t.send.to, state.state.keys[t.send.from])) }, what);

  return {
    id,
    view: () => serial(view),

    /** One turn of the loop (§5): step 1 the fold, step 2 each forward, step 3 each correction. */
    pass: () =>
      serial(async () => {
        const v = await view();
        const done = { forwarded: 0, corrected: 0, failed: 0 };
        for (const r of v.relay) {
          const g = v.groups.find((x) => x.id === r.group);
          if (!g?.me) {
            trace('relay: not in the next group', { person: id, group: r.group, do: r.do });
            done.failed++;
            continue;
          }
          try {
            if (r.do === 'forward') {
              await store.append(r.group, { op: 'record', payload: r.payload }, g.me);
              trace('relay: forwarded', { person: id, group: r.group, dir: r.payload.dir, size: Math.floor((r.payload.box.length * 3) / 4) });
              done.forwarded++;
            } else if (r.do === 'correct') {
              await store.append(r.group, { op: 'correct', corrects: r.corrects, replacement: null }, g.me);
              trace('relay: corrected a carry whose ask was withdrawn', { person: id, group: r.group, carry: r.corrects });
              done.corrected++;
            } else {
              trace('relay: an entry this loop does not do', { person: id, do: r.do });
              done.failed++;
            }
          } catch (e) {
            trace('relay: could not write, again next turn', { person: id, group: r.group, do: r.do, error: e.message });
            done.failed++;
          }
        }
        trace('a pass', { person: id, ...done, reads: v.reads.map((x) => ({ group: x.group, ms: Math.round(x.ms), ops: x.ops })) });
        return { ...done, reads: v.reads };
      }),

    /** A new ask in `group`: its own key pair kept first, then the ask written. */
    ask: ({ kind, fields: values, group, hops, expires }) =>
      serial(async () => {
        const k = kindOf(kind);
        const v = await view();
        standing(v, group);
        fields(k, 'ask', values);
        if (!Number.isSafeInteger(hops) || hops < 0 || hops > MOST_HOPS) throw new Refused(`How far it may go is 0 to ${MOST_HOPS} rings away.`);
        if (!Number.isSafeInteger(expires) || expires <= unix()) throw new Refused('It has to end after now.');
        const key = keyPair();
        state.state.keys[key.public] = key.secret;
        state.save();
        trace('asking', { person: id, kind, group, hops });
        return write(v, group, { op: 'record', payload: askAct({ kind, fields: values, hops, expires, seal_to: key.public, handle: newHandle() }) }, 'ask');
      }),

    /** One tap, one carry: what `carriable` allows, or its refusal said. */
    carry: ({ ask, group }) =>
      serial(async () => {
        const v = await view();
        const a = askOf(v, ask);
        const can = v.carriable.can.find((c) => c.ask === ask && c.group === group);
        if (!can) {
          const not = v.carriable.not.find((c) => c.ask === ask && c.group === group);
          trace('refused: not carriable', { person: id, ask, group, why: not?.why ?? 'no_such_group' });
          throw new Refused(not ? `It can't go there: ${not.words}.` : "You're not in that group.");
        }
        const { payload, route } = carryAct(a, can, newHandle());
        state.state.routes.push(route);
        state.save();
        trace('carrying', { person: id, ask, from: can.up.group, to: group, hops: can.hops });
        return write(v, group, { op: 'record', payload }, 'carry');
      }),

    /** An offer, the kind's reply fields sealed to the ask, through its copy in `group`. */
    reply: ({ ask, group, fields: values }) =>
      serial(async () => {
        const v = await view();
        const a = askOf(v, ask);
        if (a.asked) throw new Refused("It's your own ask.");
        fields(kindOf(a.kind), 'reply', values);
        const body = { say: 'offer', fields: values };
        const t = v.threads.find((x) => x.role === 'replier' && x.ask === ask && x.send);
        if (t) {
          trace('replying again', { person: id, ask });
          return say(v, t, body, 'reply');
        }
        const copy = a.in.find((c) => c.group === group && c.live);
        if (!copy) throw new Refused("That ask isn't in that group.");
        const key = keyPair();
        state.state.keys[key.public] = key.secret;
        state.save();
        trace('replying', { person: id, ask, group });
        return write(v, group, { op: 'record', payload: msgAct({ handle: copy.handle, dir: 'up', from: key.public, to: a.seal_to }, seal(body, a.seal_to, key.secret)) }, 'reply');
      }),

    /** Yes, with the name the other end sees once both have said it. */
    yes: ({ ask, them, name }) =>
      serial(async () => {
        if (typeof name !== 'string' || !name.trim() || name.length > MAX_NAME) throw new Refused(`Say the name they'll see, up to ${MAX_NAME} letters.`);
        const v = await view();
        return say(v, threadOf(v, ask, them), { say: 'yes', name: name.trim() }, 'yes');
      }),

    no: ({ ask, them }) =>
      serial(async () => {
        const v = await view();
        return say(v, threadOf(v, ask, them), { say: 'no' }, 'no');
      }),

    /** The person's own ask, taken back where they wrote it. */
    withdraw: ({ ask }) =>
      serial(async () => {
        const v = await view();
        const a = askOf(v, ask);
        const own = a.asked && a.in.find((c) => !c.carried && v.groups.some((g) => g.id === c.group && g.me === c.by));
        if (!own) throw new Refused('Only the person who asked can take it back.', 403);
        trace('withdrawing', { person: id, ask, group: own.group });
        return write(v, own.group, { op: 'correct', corrects: own.act, replacement: null }, 'withdraw');
      }),

    /** A listing: a plain record in the person's own group, never carried. */
    list: ({ kind, group, fields: values }) =>
      serial(async () => {
        const k = kindOf(kind);
        if (!k.listing) throw new Refused(`A ${kind} has nothing to list.`);
        fields(k, 'listing', values);
        const v = await view();
        return write(v, group, { op: 'record', payload: listingAct(kind, values) }, 'list');
      }),
  };
}

/** Each person's pass, every RELAY_EVERY, one after another; never two at once. */
export function loop(people, trace = debug) {
  let stopped = false;
  let timer;
  const turn = async () => {
    for (const p of people) {
      try {
        await p.pass();
      } catch (e) {
        trace('a pass failed, again next turn', { person: p.id, error: e.message });
      }
    }
    if (!stopped) timer = setTimeout(turn, RELAY_EVERY);
  };
  timer = setTimeout(turn, RELAY_EVERY);
  return () => {
    stopped = true;
    clearTimeout(timer);
  };
}
