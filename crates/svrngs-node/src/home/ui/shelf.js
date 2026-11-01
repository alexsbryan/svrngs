// SPDX-License-Identifier: AGPL-3.0-or-later
// "Bring things to the table", both pages' (page-shelf (a); question.js
// `table`, :190-210, which it moves): the shelf, svrngs's own apps as
// `shelf` lists them, each not on group `g`'s table yet unticked, and one
// button bringing the ticked ones by the page's `bring(g, names)`, which
// answers each `{app, already}`; what happened said beside the button,
// where it is looked for, not over the words above the boxes, which v1
// replaced (page-shelf (b)). Folded as "Bring more to <group>" for someone
// who has just `joined` (question.js:195-198). Its machine, SHELF, is the
// part's own (ui/view.js `useMachine`), its trace `SHELF:`. While it
// brings, the button and the boxes are held and a second bring is refused,
// where v1's second tap brought again; then what it brought is offered no
// more, before the group is read again (page-shelf (b)).
import { html, useMachine } from './view.js?v=__VERSION__';
import { go, machine, no } from './machine.js?v=__VERSION__';
import { Act, Fold } from './parts.js?v=__VERSION__';
import { spoken } from '../question.js?v=__VERSION__';

const done = (name) => (r) => (r.already ? `${spoken(r.app)} is on the table already.` : `You brought ${spoken(r.app)} to ${name}: everyone in it can open it in a few seconds.`);
const BUSY = 'a bring is under way, and is answered first';
export const SHELF = (bring) => ({
  states: { bring: ['idle', 'bringing', 'brought', 'refused'] },
  step: (s, m) => ({
    tick: () => (s.at.bring === 'bringing' ? no(s, BUSY) : go(s, {}, { ticked: m.on ? [...s.ticked, m.app] : s.ticked.filter((a) => a !== m.app) })),
    fold: () => go(s, {}, { open: m.open }),
    bring: () => (s.at.bring === 'bringing' ? no(s, BUSY) : go(s, { bring: 'bringing' }, { name: m.name }, { run: 'bring', g: m.g, apps: m.apps, failed: 'unbrought' })),
    brought: () => ((apps) => go(s, { bring: 'brought' }, { said: m.answers.map(done(s.name)).join(' ') || 'Tick something to bring first.', brought: [...s.brought, ...apps], ticked: s.ticked.filter((a) => !apps.includes(a)) }))(m.answers.map((r) => r.app)),
    unbrought: () => go(s, { bring: 'refused' }, { said: `Nothing was brought: ${m.why}` }),
  })[m.type]?.() ?? no(s, `the shelf takes no ${m.type}`),
  io: { bring: ({ g, apps }) => bring(g, apps).then((answers) => ({ type: 'brought', answers, why: answers.map((r) => `${r.app}${r.already ? ' already' : ''}`).join(', ') || 'none ticked' })) },
});

const FIRST = { at: { bring: 'idle' }, ticked: [], brought: [], open: false, said: '' };
export const Shelf = (props) => drawn(...useMachine(() => machine('SHELF', SHELF(props.bring), FIRST)), props);

// `lent` and `shelf` as the group's last read gives them; `g` its id and name.
export function drawn(s, send, { g, lent, shelf, joined }) {
  const name = g.name ?? 'your group', on = (a) => lent.some((l) => l.group === g.id && l.kind === a.name), offered = shelf.filter((a) => !on(a) && !s.brought.includes(a.name));
  const tick = (a) => (e) => send({ type: 'tick', app: a, on: e.currentTarget.checked });
  const section = html`<section><h3>Bring things to the table</h3>${shelf.filter(on).map((a) => html`<p>${spoken(a.name)} is on the table.</p>`)}<p>${offered.length ? `These come with svrngs. Bring one, and everyone in ${name} opens it on their own computer or phone, while any computer in ${name} is on.` : ''}</p>${offered.map((a) => html`<p><label><input type="checkbox" disabled=${s.at.bring === 'bringing'} checked=${s.ticked.includes(a.name)} onChange=${tick(a.name)} />${` ${spoken(a.name)}: ${a.words}`}</label></p>`)}${(offered.length > 0 || s.said) && html`<p><${Act} answer=${offered.length ? 'beside' : 'instead'} words=${`Bring the ticked ones to ${name}`} doing=${{ state: s.at.bring === 'bringing' ? 'busy' : s.at.bring, said: s.said && ` ${s.said}` }} on=${{ type: 'bring', g, name, apps: offered.map((a) => a.name).filter((a) => s.ticked.includes(a)) }} send=${send} /></p>`}</section>`;
  return joined ? html`<${Fold} summary=${`Bring more to ${name}`} fold="shelf" open=${s.open} send=${send}>${section}<//>` : section;
}
