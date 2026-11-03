// SPDX-License-Identifier: AGPL-3.0-or-later
// "Bring things to the table", both pages' (page-shelf (a); question.js
// `table`, :190-210, which it moves): the shelf, svrngs's own apps as
// `shelf` lists them, each not on group `g`'s table yet unticked, and one
// button bringing the ticked ones by the page's `bring(g, names)`, which
// answers each `{app, already}`; what happened said beside the button,
// where it is looked for, not over the words above the boxes, which v1
// replaced (page-shelf (b)). Folded as "Bring more to <group>" for someone
// who has just `joined` (question.js@19b63b2b:195-198). Its machine, SHELF, is the
// part's own (ui/view.js `useMachine`), its trace `SHELF:`. While it
// brings, the button and the boxes are held, "Bringing…" said beside it,
// and a second bring is refused, where v1's second tap brought again; then what it brought is offered no
// more, and the page's `after()` reads the group again, so it is said on
// the table from the group's own answer, never a reload (page-shelf (b)).
// The group lists an app brought once its folder has placed the files in
// its record (carried.rs `carried`), a moment after the bring is answered,
// so each read that does not list it yet is followed by another, a second,
// then two, four and eight on, at the edge, and then no more (`listed`).
import { html, useEffect, useMachine } from './view.js?v=__VERSION__';
import { assertNever, go, machine, no } from './machine.js?v=__VERSION__';
import { Act, Fold } from './parts.js?v=__VERSION__';
import { spoken } from '../question.js?v=__VERSION__';
/** @import { Machine, Store } from './machine.js?v=__VERSION__' */
/** @import { Drawn } from './view.js?v=__VERSION__' */
/** @import { Shelved } from '../question.js?v=__VERSION__' */

/**
 * A group as the shelf reads it: its id, and its name, `null` when this
 * computer cannot read it. A page gives its own group, which the shelf
 * gives back to the page's `bring`.
 * @typedef {{ id: string, name: string | null }} ShelfGroup
 */

/**
 * A lend as the shelf reads it: the group carrying it, if any, and its kind.
 * @typedef {{ group?: string | null, kind: string }} ShelfLent
 */

/**
 * /shelf's answer for each app brought: whether it was on the table already.
 * @typedef {{ app: string, already: boolean }} Brought
 */

/**
 * SHELF's state: where its bring is; the apps ticked, and those brought,
 * by kind; whether its fold is open; what its bring said; the group's
 * name, as the bring under way names it; and how often the group was read
 * again without listing all it brought.
 * @typedef {object} ShelfState
 * @property {{ bring: 'idle' | 'bringing' | 'brought' | 'refused' }} at
 * @property {Record<string, number>} [asked]
 * @property {string[]} ticked
 * @property {string[]} brought
 * @property {boolean} open
 * @property {string} said
 * @property {string} [name]
 * @property {number} [tries]
 */

/**
 * The messages SHELF takes: a box ticked or not; its fold; a bring to the
 * group `g`; its answers, or why none; the group read again; and what that
 * read does not list yet.
 * @template {ShelfGroup} G
 * @typedef {{ type: 'tick', app: string, on: boolean }
 *   | { type: 'fold', fold: 'shelf', open: boolean }
 *   | { type: 'bring', g: G, name: string, apps: string[] }
 *   | { type: 'brought', answers: Brought[], why: string }
 *   | { type: 'told', why: string }
 *   | { type: 'listed', missing: string[], why: string }
 *   | { type: 'unbrought', why: string }} ShelfMessage
 */

/**
 * The effects SHELF's step answers: the apps brought to `g`, and the group
 * read again `ms` on.
 * @template {ShelfGroup} G
 * @typedef {{ run: 'bring', g: G, apps: string[], failed: 'unbrought' }
 *   | { run: 'after', ms: number, failed: 'told' }} ShelfEffect
 */

/** @param {string} name */
const done = (name) => (/** @type {Brought} */ r) =>
  r.already
    ? `${spoken(r.app)} is on the table already.`
    : `You brought ${spoken(r.app)} to ${name}: everyone in it can open it in a few seconds.`;
const BUSY = 'a bring is under way, and is answered first',
  TRIES = 4;
// Whether app `a` is on group `g`'s table, as its read `lent` says.
/**
 * @param {ShelfGroup} g
 * @param {ShelfLent[]} lent
 */
const on = (g, lent) => (/** @type {string} */ a) =>
  lent.some((l) => l.group === g.id && l.kind === a);
/**
 * The shelf's machine, bringing by the page's `bring` and reading the
 * group again by its `after`.
 * @template {ShelfGroup} G
 * @param {(g: G, apps: string[]) => Promise<Brought[]>} bring
 * @param {() => void} after
 * @returns {Machine<ShelfState, ShelfMessage<G>, ShelfEffect<G>>}
 */
export const SHELF = (bring, after) => ({
  states: { bring: ['idle', 'bringing', 'brought', 'refused'] },
  step: (s, m) => {
    switch (m.type) {
      case 'tick':
        return s.at.bring === 'bringing'
          ? no(s, BUSY)
          : go(
              s,
              {},
              { ticked: m.on ? [...s.ticked, m.app] : s.ticked.filter((a) => a !== m.app) },
            );
      case 'fold':
        return go(s, {}, { open: m.open });
      case 'bring':
        return s.at.bring === 'bringing'
          ? no(s, BUSY)
          : go(
              s,
              { bring: 'bringing' },
              { name: m.name },
              { run: 'bring', g: m.g, apps: m.apps, failed: 'unbrought' },
            );
      case 'brought':
        // `name` is the bring's, which this answers.
        return ((apps) =>
          go(
            s,
            { bring: 'brought' },
            {
              said:
                m.answers.map(done(/** @type {string} */ (s.name))).join(' ') ||
                'Tick something to bring first.',
              brought: [...s.brought, ...apps],
              ticked: s.ticked.filter((a) => !apps.includes(a)),
              tries: 0,
            },
            { run: 'after', ms: 0, failed: 'told' },
          ))(m.answers.map((r) => r.app));
      case 'told':
        return go(s, {});
      case 'listed': {
        // `brought` sets `tries` to 0, and it is the one way to `brought`.
        const tries = /** @type {number} */ (s.tries);
        return !m.missing.length || s.at.bring !== 'brought'
          ? go(s, {})
          : tries >= TRIES
            ? no(s, `${m.why} not listed after ${TRIES} reads again; the next read says it`)
            : go(
                s,
                {},
                { tries: tries + 1 },
                { run: 'after', ms: 1000 * 2 ** tries, failed: 'told' },
              );
      }
      case 'unbrought':
        return go(s, { bring: 'refused' }, { said: `Nothing was brought: ${m.why}` });
      default:
        return assertNever(s, m, 'the shelf');
    }
  },
  io: {
    after: ({ ms }) =>
      new Promise((r) => setTimeout(r, ms)).then(
        () => (after(), { type: 'told', why: `the group is read again, ${ms} ms on` }),
      ),
    bring: ({ g, apps }) =>
      bring(g, apps).then((answers) => ({
        type: 'brought',
        answers,
        why:
          answers.map((r) => `${r.app}${r.already ? ' already' : ''}`).join(', ') || 'none ticked',
      })),
  },
});

/** @type {ShelfState} */
const FIRST = { at: { bring: 'idle' }, ticked: [], brought: [], open: false, said: '' };
/**
 * The shelf's props: the group, its lends and shelf as its last read
 * gives them, whether it was just joined here, and the page's `bring` and
 * `after`.
 * @template {ShelfGroup} G
 * @typedef {object} ShelfProps
 * @property {G} g
 * @property {ShelfLent[]} lent
 * @property {Shelved[]} shelf
 * @property {boolean} [joined]
 * @property {(g: G, apps: string[]) => Promise<Brought[]>} bring
 * @property {() => void} after
 */

/**
 * @template {ShelfGroup} G
 * @param {ShelfProps<G>} props
 */
export function Shelf(props) {
  const [s, send] = useMachine(() => machine('SHELF', SHELF(props.bring, props.after), FIRST));
  // Each read the page gives after a bring: what it brought and the read does not list yet.
  useEffect(
    () =>
      void (
        s.at.bring === 'brought' &&
        ((missing) => send({ type: 'listed', missing, why: missing.join(', ') || 'all listed' }))(
          s.brought.filter((a) => !on(props.g, props.lent)(a)),
        )
      ),
    [props.lent],
  );
  return drawn(s, send, props);
}

// `lent` and `shelf` as the group's last read gives them; `g` its id and name.
/**
 * The shelf drawn from its state.
 * @template {ShelfGroup} G
 * @param {ShelfState} s
 * @param {Store<ShelfState, ShelfMessage<G>>['dispatch']} send
 * @param {Pick<ShelfProps<G>, 'g' | 'lent' | 'shelf' | 'joined'>} props
 * @returns {Drawn}
 */
export function drawn(s, send, { g, lent, shelf, joined }) {
  const name = g.name ?? 'your group',
    listed = (/** @type {Shelved} */ a) => on(g, lent)(a.name),
    offered = shelf.filter((a) => !listed(a) && !s.brought.includes(a.name));
  const tick =
    (/** @type {string} */ a) => (/** @type {Event & { currentTarget: HTMLInputElement }} */ e) =>
      send({ type: 'tick', app: a, on: e.currentTarget.checked });
  const section = html`<section><h3>Bring things to the table</h3>${shelf.filter(listed).map((a) => html`<p>${spoken(a.name)} is on the table.</p>`)}<p>${offered.length ? `These come with svrngs. Bring one, and everyone in ${name} opens it on their own computer or phone, while any computer in ${name} is on.` : ''}</p>${offered.map((a) => html`<p><label><input type="checkbox" disabled=${s.at.bring === 'bringing'} checked=${s.ticked.includes(a.name)} onChange=${tick(a.name)} />${` ${spoken(a.name)}: ${a.words}`}</label></p>`)}${(offered.length > 0 || s.said) && html`<p><${Act} answer=${offered.length ? 'beside' : 'instead'} words=${`Bring the ticked ones to ${name}`} doing=${{ state: s.at.bring === 'bringing' ? 'busy' : s.at.bring, said: s.said && ` ${s.said}` }} on=${{ type: 'bring', g, name, apps: offered.map((a) => a.name).filter((a) => s.ticked.includes(a)) }} send=${send} />${s.at.bring === 'bringing' && ' Bringing…'}</p>`}</section>`;
  return joined
    ? html`<${Fold} summary=${`Bring more to ${name}`} fold="shelf" open=${s.open} send=${send}>${section}<//>`
    : section;
}
