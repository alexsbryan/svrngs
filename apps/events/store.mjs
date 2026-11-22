// Where Events' acts are kept: the group's record, or a file of the app's
// own for running alone (EVENTS.md §5, "It runs without svrngs too"). Both
// answer the same four questions in the same shapes, so one fold
// (events.js) reads either:
//
//   groups()                  [{id, name}]: the groups there are
//   people(group)             [{name, id, joined?}]: who stands in the
//                             group, `id` what the person header names them
//                             by, and the fold too (events.js `who`), and
//                             `joined`, when they did, seconds, or null for
//                             whoever started it: two of one name told apart
//   log(group)                {ops, complete}: `ops` as GET /v1/rail/log
//                             answers them
//   append(group, act, name)  the act written as `name`'s: {op: 'record',
//                             payload} or {op: 'correct', corrects, replacement};
//                             the file's alone: under svrngs the server names
//                             the act in its answer (server.mjs `naming`)
//
// Every failure throws, saying whose: a group whose record cannot be read is
// said as that by the server, never shown as no one answering.

import { randomBytes } from 'node:crypto';
import { readFileSync, renameSync, writeFileSync } from 'node:fs';
import { homedir } from 'node:os';
import { join } from 'node:path';

import { svrngs } from './svrngs.mjs';

/**
 * svrngs on this computer, as the config's `record` names its data dir
 * (`~/.svrngs`), asked through svrngs.mjs, which never sends its token.
 * A config that names svrngs's address or its `token` file, as before
 * svrngs V1.13.3, is refused, saying what to write instead.
 */
export function svrngsOf(config) {
  if (config.token !== undefined || /^https?:/.test(String(config.record)))
    throw new Error(
      `\`record\` names svrngs's data dir now, as {"record": "${join(homedir(), '.svrngs')}"}, and no \`token\`: svrngs.mjs asks svrngs as you without ever sending its token`,
    );
  if (typeof config.record !== 'string')
    throw new Error("the config names no `record`, svrngs's data dir, and no `file` of its own");
  return svrngs(config.record.replace(/^~(?=$|\/)/, homedir()));
}

/**
 * The group's record, through `here`, svrngs on the computer this app
 * runs on (svrngs.mjs's, `svrngsOf`). A person is named by their first key,
 * `person_key`, which the door tells the app as `x-svrngs-key`
 * (crates/svrngs-node/src/door.rs:123-127).
 */
export function record(here) {
  async function call(path, body) {
    try {
      return await here.ask(path, body);
    } catch (e) {
      if (e.status === undefined)
        throw new Error(`this computer's svrngs did not answer: ${e.message}`);
      throw new Error(
        `this computer's svrngs refused ${path.split('?')[0]}: ${e.status} ${e.answer?.error ?? JSON.stringify(e.answer)}`,
      );
    }
  }
  const at = (door, group) => `/v1/rail/${door}?namespace=${encodeURIComponent(group)}`;
  return {
    kind: 'record',
    async groups() {
      return (await call('/v1/groups')).groups.map((g) => ({ id: g.id, name: g.name }));
    },
    async people(group) {
      const out = [];
      for (const p of (await call(at('people', group))).people) {
        if (!out.some((o) => o.id === p.person_key))
          out.push({ name: p.person, id: p.person_key, joined: p.joined?.at ?? null });
      }
      return out;
    },
    // Events' own part of the record, its kind `events/` taken off (svrngs
    // `grant::app_act`), and the acts it wrote before it had one.
    async log(group) {
      const log = await call(at('log', group));
      const own = (o) =>
        !o.payload || !o.payload.kind?.includes('/') || o.payload.kind.startsWith('events/');
      const ops = log.ops
        .filter(own)
        .map((o) =>
          o.payload?.kind?.startsWith('events/')
            ? { ...o, payload: { ...o.payload, kind: o.payload.kind.slice(7) } }
            : o,
        );
      return { ops, complete: log.complete };
    },
  };
}

/**
 * A file of the app's own: its groups, each with its name and people, and
 * each group's acts, in the order written. A person is named by their name,
 * as the proxy in front sets it. Written whole through a file renamed into
 * place, one write at a time.
 *
 *   {"groups": {"<id>": {"name": "The House", "people": ["sam", "bo"]}},
 *    "ops": {"<id>": [{id, ts_unix, person, payload?, corrects?}]}}
 */
export function file(path) {
  const kept = JSON.parse(readFileSync(path, 'utf8'));
  if (!kept.groups || typeof kept.groups !== 'object') throw new Error(`${path} names no groups`);
  kept.ops ??= {};
  const group = (id) => {
    if (!Object.prototype.hasOwnProperty.call(kept.groups, id))
      throw new Error(`there is no group ${id} in ${path}`);
    return kept.groups[id];
  };
  return {
    kind: 'file',
    async groups() {
      return Object.entries(kept.groups).map(([id, g]) => ({ id, name: g.name ?? id }));
    },
    async people(id) {
      return group(id).people.map((name) => ({ name, id: name }));
    },
    async log(id) {
      group(id);
      return { ops: structuredClone(kept.ops[id] ?? []), complete: true };
    },
    async append(id, act, name) {
      if (!group(id).people.includes(name)) throw new Error(`${name} is not in ${id}`);
      const op = {
        id: randomBytes(8).toString('hex'),
        ts_unix: Math.floor(Date.now() / 1000),
        person: name,
      };
      if (act.op === 'record') op.payload = act.payload;
      else if (act.op === 'correct')
        Object.assign(
          op,
          { corrects: act.corrects },
          act.replacement && { payload: act.replacement },
        );
      else throw new Error(`an app writes records and corrections, not ${act.op}`);
      (kept.ops[id] ??= []).push(op);
      writeFileSync(`${path}.new`, JSON.stringify(kept, null, 1));
      renameSync(`${path}.new`, path);
      return op.id;
    },
  };
}
