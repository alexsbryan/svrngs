// SPDX-License-Identifier: AGPL-3.0-or-later
// "What acts as you" on the phone (page-acts (b); web/app.js@19b63b2b:490-521),
// ui/acts.js `Acts` drawn in #acts once the group is read, each part
// derived from state: the apps the group carries and their new versions
// (ui/changes.js), what opens friends' things as you, by the yes kept with
// the group (`reads`), each with Stop, then what apps use (web/page/uses.js),
// what you open knowing your name, and who decides for you, of the people
// by this phone's key, `me`, as enter reads it with the group (enter.js
// `read`), as page-members needed it too.
// DOING is one act at a time, keyed by what it acts on, `acting`; each
// act's answer kept in `acted` by that key, so a yes said "opens at its
// new version for you now" stays said once the group, read again, lists
// the change no more. A yes is this phone's own (capability-signed): the
// computer it reaches names the act at /runs, and the phone signs it with
// its own key as its chain's next line, kept with the group before it is
// sent, so no seq is signed twice, then carries every line not yet taken
// (app.js@19b63b2b:497-508). A Stop is kept with the group (app.js@19b63b2b:495). After either, enter
// is given the group as kept, and reads it again; the key stays the edge's.
// Lines a carry left unsent are carried again on each read that holds
// them, where v1 carried them only at the next yes (app.js@19b63b2b:506; page-acts (d)):
// refused, that carry asks no read again, so it is tried once a read.
import { Act, Acts, carrying, Changes, deciders, go, html, named, no, once, reading, spoken, useEffect } from '../ui/kit.js?v=__VERSION__';
import * as store from '../store.js?v=__VERSION__';
import { through } from './enter.js?v=__VERSION__';
import { PASS_LIFE } from './index.js?v=__VERSION__';
import { Used } from './uses.js?v=__VERSION__';

export const owns = ['folds', 'acting', 'acted'];
export const states = { doing: ['idle', 'busy', 'done', 'refused'] };
const acting = (s, key, effect, patch = {}) => (s.at.doing === 'busy' ? no(s, `${s.acting} is under way, and is answered first`) : go(s, { doing: 'busy' }, { acting: key, acted: { ...s.acted, [key]: { ...s.acted?.[key], ...patch } } }, effect));
const answered = (s, m, state) => (s.at.doing !== 'busy' ? no(s, 'no act is under way') : go(s, { doing: state }, { acted: { ...s.acted, [s.acting]: { ...s.acted[s.acting], state, said: state === 'refused' ? `${s.acted[s.acting].said ?? ''} ${m.why}` : '' } } }, ...(m.group ? [{ run: 'send', m: { type: 'opened', group: m.group, why: 'kept here, so read again' } }] : [])));
const fold = (k) => (s, m) => go(s, {}, { folds: { ...s.folds, [k]: m.open } });
export const takes = {
  'fold acts': fold('acts'), 'fold changes': fold('changes'),
  yes: (s, { app }) => acting(s, `yes ${app}`, { run: 'runs', group: s.group, app, failed: 'unacted' }, { c: s.changed?.find((c) => c.group === s.group.id && c.app === app) }),
  'stop reads': (s, { label }) => acting(s, `stop ${label}`, { run: 'stopping', group: s.group, label, failed: 'unacted' }),
  acted: (s, m) => answered(s, m, 'done'), unacted: (s, m) => answered(s, m, 'refused'),
  carry: (s) => (s.group?.unsent?.length ? acting(s, 'carry', { run: 'carrying', group: s.group, failed: 'unacted' }) : no(s, 'no line is unsent')),
};

const NOTES = { opens: `A Stop here holds from the next pass this phone signs for the app, within ${PASS_LIFE / 3600} hours.` };
const doing = (s, key) => (s.acting === key && s.at.doing === 'busy' ? { state: 'busy' } : (s.acted?.[key] ?? {}));
// Each reading lend's yes or stop as kept, the yes with Stop, its line said
// in its place once stopped (app.js@19b63b2b:512-518).
const read = (s, send, l, c = s.group.reads[l.label], d = doing(s, `stop ${l.label}`)) => (d.state === 'done' ? html`<p>${named(l)} stops opening your friends’ ${l.reads.map(spoken).join(' and ')} as you within ${PASS_LIFE / 3600} hours, when the pass it holds ends.</p>`
  : html`<p>${reading(l, c)}${c.said === 'yes' && html`<${Act} words="Stop" doing=${d} on=${{ type: 'stop reads', label: l.label }} send=${send} />`}</p>`);
function Drawn({ s, send }) {
  useEffect(() => void (s.at.list === 'read' && s.group.unsent?.length && send({ type: 'carry', why: `${s.group.unsent.length} line(s) unsent` })), [s.lent]);
  if (!s.lent) return html`<section id="acts"></section>`;
  const done = Object.values(s.acted ?? {}).filter((a) => a.c && a.state === 'done' && !s.changed?.some((c) => c.app === a.c.app && c.group === a.c.group)).map((a) => a.c);
  const parts = {
    reads: [...once(s.lent).filter((o) => o.group).map((l) => html`<p>${carrying(l.kind, s.shelf)}</p>`),
      ...Changes({ g: s.group, changed: s.changed && [...s.changed, ...done], rows: s.people, me: s.me, doing: Object.fromEntries(Object.keys(s.acted ?? {}).filter((k) => k.startsWith('yes ')).map((k) => [k.slice(4), doing(s, k)])), open: s.folds?.changes, send })],
    opens: [...s.lent.filter((o) => o.reads?.length && s.group.reads?.[o.label]).map((l) => read(s, send, l)), ...(s.lent.some((l) => l.asks) ? [html`<${Used} s=${s} send=${send} />`] : [])],
    knows: [html`<p>What you open knows your name.</p>`], decides: [html`<p>${deciders(s.people, s.me, 'browser')}.</p>`],
  };
  return html`<${Acts} parts=${parts} notes=${NOTES} open=${s.folds?.acts} send=${send} />`;
}
export const view = { acts: ({ s, send }) => (s.group ? html`<${Drawn} s=${s} send=${send} />` : html`<section id="acts"></section>`) };

// The edge: the key unsealed here, never in state, and the wasm imported
// only once app.js has run its init.
const wasm = () => import('../wasm/svrngs_web.js?v=__VERSION__');
// Every line not yet taken carried, in order, then none kept as unsent; or
// why not, with `kept`, the group as kept, its unsent lines in it.
const carry = (g, why, kept = g) => g.unsent.reduce((p, l) => p.then(() => through(g, '/carry', l)), Promise.resolve()).then(() => store.opened({ ...g, unsent: [] })).then((g) => ({ type: 'acted', group: g, why: `${why}, and carried` }), (e) => ({ type: 'unacted', group: kept, why: e.message }));
export const io = {
  runs: ({ group, app }) => Promise.all([wasm(), store.unseal(group), through(group, '/runs', app)]).then(([w, key, { sign }], at = group.line ?? { seq: -1, id: null }) => {
    const line = JSON.parse(w.sign_line(key, group.id, Math.floor(Date.now() / 1000), at.seq + 1, at.id, JSON.stringify(sign)));
    return store.opened({ ...group, line: { seq: line.seq, id: line.id }, unsent: [...(group.unsent ?? []), line] }).then((g) => carry(g, `line ${line.seq} signed`));
  }),
  carrying: ({ group }) => carry(group, `${group.unsent.length} line(s) unsent`, null),
  stopping: ({ group, label }) => store.opened({ ...group, reads: { ...group.reads, [label]: { said: 'stop', at: Math.floor(Date.now() / 1000) } } }).then((g) => ({ type: 'acted', group: g, why: `${label}'s reads stopped here` })),
};
