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
/** @import { Drawn, Targeted } from './view.js?v=__VERSION__' */
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
 * again without naming all it brought.
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
const done = (name) => (/** @type {Brought} */ answer) =>
  answer.already
    ? `${spoken(answer.app)} is on the table already.`
    : `You brought ${spoken(answer.app)} to ${name}: everyone in it can open it in a few seconds.`;
const BUSY = 'a bring is under way, and is answered first',
  TRIES = 4;
// Whether `app` is on `group`'s table, as its read `lent` says.
/**
 * @param {ShelfGroup} group
 * @param {ShelfLent[]} lent
 */
const on = (group, lent) => (/** @type {string} */ app) =>
  lent.some((lend) => lend.group === group.id && lend.kind === app);
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
  step: (state, message) => {
    switch (message.type) {
      case 'tick':
        return state.at.bring === 'bringing'
          ? no(state, BUSY)
          : go(
              state,
              {},
              {
                ticked: message.on
                  ? [...state.ticked, message.app]
                  : state.ticked.filter((app) => app !== message.app),
              },
            );
      case 'fold':
        return go(state, {}, { open: message.open });
      case 'bring':
        return state.at.bring === 'bringing'
          ? no(state, BUSY)
          : go(
              state,
              { bring: 'bringing' },
              { name: message.name },
              { run: 'bring', g: message.g, apps: message.apps, failed: 'unbrought' },
            );
      case 'brought':
        // `name` is the bring's, which this answers.
        return ((apps) =>
          go(
            state,
            { bring: 'brought' },
            {
              said:
                message.answers.map(done(/** @type {string} */ (state.name))).join(' ') ||
                'Tick something to bring first.',
              brought: [...state.brought, ...apps],
              ticked: state.ticked.filter((app) => !apps.includes(app)),
              tries: 0,
            },
            { run: 'after', ms: 0, failed: 'told' },
          ))(message.answers.map((answer) => answer.app));
      case 'told':
        return go(state, {});
      case 'listed': {
        // `brought` sets `tries` to 0, and it is the one way to `brought`.
        const tries = /** @type {number} */ (state.tries);
        return !message.missing.length || state.at.bring !== 'brought'
          ? go(state, {})
          : tries >= TRIES
            ? no(
                state,
                `${message.why} not listed after ${TRIES} reads again; the next read says it`,
              )
            : go(
                state,
                {},
                { tries: tries + 1 },
                { run: 'after', ms: 1000 * 2 ** tries, failed: 'told' },
              );
      }
      case 'unbrought':
        return go(state, { bring: 'refused' }, { said: `Nothing was brought: ${message.why}` });
      default:
        return assertNever(state, message, 'the shelf');
    }
  },
  io: {
    after: ({ ms }) =>
      new Promise((resolve) => setTimeout(resolve, ms)).then(
        () => (after(), { type: 'told', why: `the group is read again, ${ms} ms on` }),
      ),
    bring: ({ g: group, apps }) =>
      bring(group, apps).then((answers) => ({
        type: 'brought',
        answers,
        why:
          answers.map((answer) => `${answer.app}${answer.already ? ' already' : ''}`).join(', ') ||
          'none ticked',
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
  const [state, send] = useMachine(() => machine('SHELF', SHELF(props.bring, props.after), FIRST));
  // Each read the page gives after a bring: what it brought and the read does not list yet.
  useEffect(
    () =>
      void (
        state.at.bring === 'brought' &&
        ((missing) => send({ type: 'listed', missing, why: missing.join(', ') || 'all listed' }))(
          state.brought.filter((app) => !on(props.g, props.lent)(app)),
        )
      ),
    [props.lent],
  );
  return drawn(state, send, props);
}

// `lent` and `shelf` as the group's last read gives them; `group` its id and name.
/**
 * The shelf drawn from its state.
 * @template {ShelfGroup} G
 * @param {ShelfState} state
 * @param {Store<ShelfState, ShelfMessage<G>>['dispatch']} send
 * @param {Pick<ShelfProps<G>, 'g' | 'lent' | 'shelf' | 'joined'>} props
 * @returns {Drawn}
 */
export function drawn(state, send, { g: group, lent, shelf, joined }) {
  const name = group.name ?? 'your group',
    listed = (/** @type {Shelved} */ app) => on(group, lent)(app.name),
    offered = shelf.filter((app) => !listed(app) && !state.brought.includes(app.name));
  const tick = (/** @type {string} */ app) => (/** @type {Targeted<HTMLInputElement>} */ ticked) =>
    send({ type: 'tick', app: app, on: ticked.currentTarget.checked });
  const section = html`<section><h3>Bring things to the table</h3>${shelf.filter(listed).map((app) => html`<p>${spoken(app.name)} is on the table.</p>`)}<p>${offered.length ? `These come with svrngs. Bring one, and everyone in ${name} opens it on their own computer or phone, while any computer in ${name} is on.` : ''}</p>${offered.map((app) => html`<p><label><input type="checkbox" disabled=${state.at.bring === 'bringing'} checked=${state.ticked.includes(app.name)} onChange=${tick(app.name)} />${` ${spoken(app.name)}: ${app.words}`}</label></p>`)}${(offered.length > 0 || state.said) && html`<p><${Act} answer=${offered.length ? 'beside' : 'instead'} words=${`Bring the ticked ones to ${name}`} doing=${{ state: state.at.bring === 'bringing' ? 'busy' : state.at.bring, said: state.said && ` ${state.said}` }} on=${{ type: 'bring', g: group, name, apps: offered.map((app) => app.name).filter((app) => state.ticked.includes(app)) }} send=${send} />${state.at.bring === 'bringing' && ' Bringing…'}</p>`}</section>`;
  return joined
    ? html`<${Fold} summary=${`Bring more to ${name}`} fold="shelf" open=${state.open} send=${send}>${section}<//>`
    : section;
}
