// For the tests: a world of groups held in memory, each person's state, and
// every act written through the app's own functions (`app`, grapevine.js
// and seal.mjs, or a planted copy of either). `tick` is one pass of each
// person's relay as relay.mjs makes it: each `relay` entry the fold
// answers, done. GRAPEVINE_DEMO §10's scene is `scene`.

import { randomBytes } from 'node:crypto';

import { kindsOf } from './grapevine.js';
import ask from './kinds/ask.js';
import tool from './kinds/tool.js';

/** A new handle: 16 random bytes, base64. */
export const newHandle = () => randomBytes(16).toString('base64');

/** Sam asks for a tile saw in 1400 Page, with one ring to go. */
export const TILE_SAW = { what: 'tile saw', when: 'Saturday' };
export const NOW = 1_800_000_000;
export const WEEK = 7 * 24 * 3600;

/**
 * `groups` {<id>: {name, people}}. Every act is timed one second after the
 * last; `sealed` keeps every body sealed, so a law can look for its words.
 */
export function world(app, kinds, groups, now = NOW) {
  const logs = Object.fromEntries(Object.keys(groups).map((id) => [id, []]));
  const states = {};
  const sealed = [];
  let n = 0;
  let ts = now - 3600;
  const stateOf = (who) => (states[who] ??= { keys: {}, routes: [] });
  const write = (group, who, op) => {
    if (!groups[group].people.includes(who)) throw new Error(`${who} is not in ${group}`);
    const id = `a${++n}`;
    logs[group].push({ id, ts_unix: ++ts, person: who, ...op });
    return id;
  };
  const w = {
    logs,
    states,
    sealed,
    append: (group, who, payload) => write(group, who, { payload }),
    correct: (group, who, corrects) => write(group, who, { corrects }),
    view(who, at = now) {
      const gs = Object.entries(groups).map(([id, g]) => ({ id, name: g.name, me: g.people.includes(who) ? who : null, ops: structuredClone(logs[id]) }));
      return app.fold({ groups: gs, state: structuredClone(stateOf(who)), kinds, now: at });
    },
    ask(who, group, kind, fields, { hops = 1, expires = now + WEEK } = {}) {
      const key = app.keyPair();
      stateOf(who).keys[key.public] = key.secret;
      return w.append(group, who, app.askAct({ kind, fields, hops, expires, seal_to: key.public, handle: newHandle() }));
    },
    /** `who`'s carry of `ask` into `group`, decided on `v`, their view (now, unless given an older one). */
    carry(who, ask, group, v = w.view(who)) {
      const can = v.carriable.can.find((c) => c.ask === ask && c.group === group);
      if (!can) throw new Error(`${who} cannot carry ${ask} to ${group}: ${JSON.stringify(v.carriable.not.find((c) => c.ask === ask && c.group === group))}`);
      const { payload, route } = app.carryAct(v.asks.find((a) => a.id === ask), can, newHandle());
      stateOf(who).routes.push(route);
      return w.append(group, who, payload);
    },
    /** A replier's first message, through the copy in `group`. */
    reply(who, ask, group, body) {
      const a = w.view(who).asks.find((x) => x.id === ask);
      const copy = a.in.find((c) => c.group === group);
      const key = app.keyPair();
      stateOf(who).keys[key.public] = key.secret;
      sealed.push(body);
      return w.append(group, who, app.msgAct({ handle: copy.handle, dir: 'up', from: key.public, to: a.seal_to }, app.seal(body, a.seal_to, key.secret)));
    },
    /** A message in `who`'s thread on `ask`. */
    say(who, ask, body) {
      const t = w.view(who).threads.find((x) => x.ask === ask && x.role !== 'carrier');
      sealed.push(body);
      return w.append(t.send.group, who, app.msgAct(t.send, app.seal(body, t.send.to, stateOf(who).keys[t.send.from])));
    },
    withdraw(who, ask) {
      const group = Object.keys(logs).find((g) => logs[g].some((o) => o.id === ask));
      return w.correct(group, who, ask);
    },
    /** One pass of every person's relay. */
    tick() {
      for (const who of [...new Set(Object.values(groups).flatMap((g) => g.people))].sort()) {
        for (const r of w.view(who).relay) {
          if (r.do === 'forward') w.append(r.group, who, r.payload);
          else w.correct(r.group, who, r.corrects);
        }
      }
    },
  };
  return w;
}

/** GRAPEVINE_DEMO §10's two groups, and a third Kim is in that Jo is not. */
export const GROUPS = {
  page: { name: '1400 Page', people: ['sam', 'jo'] },
  waller: { name: 'Waller block', people: ['jo', 'kim'] },
  yard: { name: 'The yard', people: ['kim', 'lee'] },
  club: { name: 'Book club', people: ['sam', 'lee'] },
};

/** The kinds the app ships. */
export const KINDS = kindsOf([ask, tool]);

/**
 * §10's scene, through to both yeses: Sam asks; Jo carries it to Waller;
 * Kim, who listed a tile saw, offers; Sam and Kim say yes. Withdrawal is
 * the caller's.
 */
export function scene(app, kinds) {
  const w = world(app, kinds, GROUPS);
  w.append('waller', 'kim', app.listingAct('tool', { what: 'tile saw' }));
  const ask = w.ask('sam', 'page', 'tool', TILE_SAW);
  const carry = w.carry('jo', ask, 'waller');
  w.reply('kim', ask, 'waller', { say: 'offer', fields: { when: 'Saturday morning', where: 'Waller and Pierce' } });
  w.tick();
  w.say('sam', ask, { say: 'yes', name: 'Sam Reyes' });
  w.tick();
  w.say('kim', ask, { say: 'yes', name: 'Kim Okafor' });
  w.tick();
  return { w, ask, carry };
}
