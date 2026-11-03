// SPDX-License-Identifier: AGPL-3.0-or-later
// "What acts as you" on the home page (page-acts (c); home.js@19b63b2b:205-207,
// :114-124, :278-286 and `uses`, :118-124): ui/acts.js `Acts`, its parts
// derived where v1 pushed each line in by `add`. Each group's GROUP read is
// told to HOME by its place in the group's section (`Told`; group.js
// PLACES), so the one section draws, group by group, the apps it carries
// and their new versions (ui/changes.js), and who decides for you in it, of
// its people by this computer's key; then what opens friends' things as
// you, the yeses /reaching gives, each Stop the form v1 posts
// (acts.mjs:64), and what apps use of each asking lend, from /choices
// (ui/uses.js, as `p`); knows; and WiFi and Play, page-home-rooms' parts.
// Drawn once every group has been told and /reaching read, as v1 drew the
// page's foot only then (home.js@19b63b2b:326-328), and kept after. Its machine, ACTS, is the section's
// own (ui/view.js `useMachine`), each read counted, read again on each of
// HOME's rounds; a yes to a new version is posted to /runs, kept in
// `acted` by what it acts on, so it stays said once read again; a Stop on
// what apps use reads the groups again, and so the section, where v1
// reloaded the page (home.js@19b63b2b:221; page-acts (d)).
import {
  Acts,
  ask,
  assertNever,
  called,
  carrying,
  Changes,
  deciders,
  go,
  html,
  late,
  machine,
  no,
  once,
  reading,
  useEffect,
  useMachine,
  Uses,
} from '../ui/kit.js?v=__VERSION__';
import { get, post } from './group.js?v=__VERSION__';
import { Reach, label } from './index.js?v=__VERSION__';
import * as wifi from './wifi.js?v=__VERSION__';
import * as play from './play.js?v=__VERSION__';
/** @import { Group, Lends, Lent, PlaceProps } from './group.js?v=__VERSION__' */
/** @import { Lend, PeopleRow } from '../question.js?v=__VERSION__' */
/** @import { Io, Machine, Step } from '../ui/machine.js?v=__VERSION__' */
/** @import { Drawn } from '../ui/view.js?v=__VERSION__' */
/** @import { Asked } from '../ui/uses.js?v=__VERSION__' */
/** @import { Change } from '../ui/changes.js?v=__VERSION__' */
/** @import * as HOME from '../home.js?v=__VERSION__' */

/**
 * The fields of HOME's that `take` sets: whether "What acts as you" is
 * read, and what each group's GROUP last read, by its id.
 * @typedef {object} Fields
 * @property {boolean} [actsRead]
 * @property {Record<string, { people: PeopleRow[] | undefined, lends: Lends | undefined }>} [told]
 */

/**
 * HOME's messages this file takes: "What acts as you" read, or what a
 * group's GROUP read (`Told`).
 * @typedef {{ type: 'told', flow: 'acts', read: true, why: string }
 *   | { type: 'told', flow: 'acts', id: string, people: PeopleRow[] | undefined, lends: Lends | undefined, why: string, read?: undefined }} Taken
 */

/**
 * What a group's GROUP last read, as HOME keeps it (`Fields`' `told`).
 * @typedef {{ people: PeopleRow[] | undefined, lends: Lends | undefined }} Told
 */

/**
 * What the person said of an app reaching their friends' lends as them,
 * and when, in seconds (door/choices.rs `Choice`, :63-68).
 * @typedef {{ said: 'yes' | 'stop', at: number }} Choice
 */

/**
 * /reaching's answer: each app's `Choice`, by its label (home.rs
 * `reached`); or why it could not be read.
 * @typedef {Record<string, Choice> | { why: string }} Yeses
 */

/**
 * A yes to a new version, kept by what it acts on: the change, and, once
 * answered, how, with each refusal said.
 * @typedef {{ c?: Change, state?: 'done' | 'refused', said?: string }} Acted
 */

/**
 * ACTS's state: where its read and its yes are, the yeses and each asking
 * lend's choices as read, which folds are open, the yes under way, by its
 * group and app, each yes kept, and each Stop's refusal, by lend and app.
 * @typedef {object} State
 * @property {{ reaching: 'reading' | 'read' | 'unread', doing: 'idle' | 'busy' | 'done' | 'refused' }} at
 * @property {Record<string, number>} [asked]
 * @property {Yeses} [yeses]
 * @property {Record<string, Asked>} [choices]
 * @property {Record<string, boolean>} [folds]
 * @property {string} [acting]
 * @property {Record<string, Acted>} acted
 * @property {Record<string, string>} said
 */

/**
 * A yes to a new version of `app` in the group `g`, with its change.
 * @typedef {{ type: 'yes', app: string, g: string, c: Change | undefined }} Yes
 */

/**
 * The messages ACTS takes: a read of the yeses and of each asking lend's
 * choices; that read, or why not; a fold opened or closed; a yes, its
 * answer, or why not; a Stop, its answer, or why not; HOME told.
 *
 * `unstopped` carries no `lend` and no `app`: it is only the `stop`
 * effect's `failed` message, which `machine` makes as `{ type, n, why }`
 * (ui/machine.js `run`), so its step files the refusal under
 * "undefined undefined", where `Uses` never reads it. Kept as it runs: a
 * refused Stop said beside it is a change a person would see (the
 * phone's web/page/uses.js names it in its io's own refusal).
 * @typedef {{ type: 'read', asking: string[] }
 *   | { type: 'reached', n: number, yeses: Yeses, choices: Record<string, Asked>, why: string }
 *   | { type: 'unreached', why: string }
 *   | { type: 'fold', fold: string, open: boolean }
 *   | Yes
 *   | { type: 'acted', why: string }
 *   | { type: 'unacted', why: string }
 *   | { type: 'stop', lend: string, app: string }
 *   | { type: 'stopped', why: string }
 *   | { type: 'unstopped', why: string, lend?: undefined, app?: undefined }
 *   | { type: 'told', why: string }} Message
 */

/**
 * The effects its step answers: read `n` of the yeses and of `asking`'s
 * choices; a yes to `app`'s new version in the group `g`; HOME told to
 * read the groups again; and `app` stopped using `lend`.
 * @typedef {{ run: 'read', n: number, asking: string[], failed: 'unreached' }
 *   | { run: 'yes', g: string, app: string, failed: 'unacted' }
 *   | { run: 'again', failed: 'told' }
 *   | { run: 'stop', lend: string, app: string, failed: 'unstopped' }} Effect
 */

// HOME's: what each group's GROUP last read, by its id.
/**
 * @param {HOME.State} state
 * @param {Taken} message
 * @returns {Step<HOME.State, HOME.Effect>}
 */
export const take = (state, message) =>
  message.read
    ? go(state, {}, { actsRead: true })
    : go(
        state,
        {},
        { told: { ...state.told, [message.id]: { people: message.people, lends: message.lends } } },
      );
// The page's foot held until the section has read (home.js `Home`).
// HOME asks once the groups are read (home.js `Home`).
/** @param {HOME.State} state */
export const holds = (state) => /** @type {Group[]} */ (state.groups).length > 0 && !state.actsRead;
/** @param {PlaceProps} props */
const Told = ({ g: group, s: state, home }) => {
  useEffect(
    () =>
      void (
        state.at.people !== 'reading' &&
        state.at.lends !== 'reading' &&
        home({
          type: 'told',
          flow: 'acts',
          id: group.id,
          people: state.people?.people,
          lends: state.lends,
          why: `${group.name}: people ${state.at.people}, lends ${state.at.lends}`,
        })
      ),
    [state.people, state.lends, state.at.people, state.at.lends],
  );
  return null;
};

/**
 * ACTS but for `again`, which `Acting` adds, as only it holds HOME's
 * `dispatch`.
 * @typedef {Omit<Machine<State, Message, Effect>, 'io'> & { io: Omit<Io<Effect, Message>, 'again'> }} Acts
 */

/** @type {Acts} */
const ACTS = {
  states: { reaching: ['reading', 'read', 'unread'], doing: ['idle', 'busy', 'done', 'refused'] },
  io: {
    read: ({ n: serial, asking }) =>
      Promise.all([
        get('/reaching').catch((error) => ({ why: error.message })),
        ...asking.map((asked) =>
          get(`/choices?lend=${asked}`).then(
            /** @returns {[string, Asked]} */ ({ choices }) => [asked, { choices }],
            /** @returns {[string, Asked]} */ (error) => [asked, { why: error.message }],
          ),
        ),
      ]).then(([yeses, ...choices]) => ({
        type: 'reached',
        n: serial,
        yeses,
        choices: Object.fromEntries(choices),
        why: `${choices.length} asking`,
      })),
    yes: ({ g: group, app }) =>
      post('/runs', [group, app]).then(() => ({ type: 'acted', why: `${app} at its new version` })),
    stop: ({ lend, app }) =>
      post('/choices', { lend, app, said: 'stop' }).then(() => ({
        type: 'stopped',
        why: `${app} stopped using ${lend}`,
      })),
  },
  step: (state, message) => {
    switch (message.type) {
      case 'read':
        return ((asked, serial) =>
          go(
            state,
            {},
            { asked },
            { run: 'read', n: serial, asking: message.asking, failed: 'unreached' },
          ))(...ask(state, 'reaching'));
      case 'reached':
        return late(state, 'reaching', message.n)
          ? no(state, late(state, 'reaching', message.n))
          : go(state, { reaching: 'read' }, { yeses: message.yeses, choices: message.choices });
      case 'unreached':
        return go(state, { reaching: 'unread' }, { yeses: { why: message.why }, choices: {} });
      case 'fold':
        return go(state, {}, { folds: { ...state.folds, [message.fold]: message.open } });
      case 'yes':
        return state.at.doing === 'busy'
          ? no(state, `${state.acting} is under way, and is answered first`)
          : go(
              state,
              { doing: 'busy' },
              {
                acting: `${message.g} ${message.app}`,
                acted: {
                  ...state.acted,
                  [`${message.g} ${message.app}`]: {
                    ...state.acted[`${message.g} ${message.app}`],
                    c: message.c,
                  },
                },
              },
              { run: 'yes', g: message.g, app: message.app, failed: 'unacted' },
            );
      // A yes's answer, to the yes that set `acting` and kept it in `acted`:
      // `acting` read once, cast, where it was read at each use.
      case 'acted': {
        const acting = /** @type {string} */ (state.acting);
        return go(
          state,
          { doing: 'done' },
          { acted: { ...state.acted, [acting]: { ...state.acted[acting], state: 'done' } } },
          { run: 'again', failed: 'told' },
        );
      }
      case 'unacted': {
        const acting = /** @type {string} */ (state.acting);
        return go(
          state,
          { doing: 'refused' },
          {
            acted: {
              ...state.acted,
              [acting]: {
                ...state.acted[acting],
                state: 'refused',
                said: `${/** @type {Acted} */ (state.acted[acting]).said ?? ''} ${message.why}`,
              },
            },
          },
        );
      }
      case 'stop':
        return go(
          state,
          {},
          {},
          { run: 'stop', lend: message.lend, app: message.app, failed: 'unstopped' },
        );
      case 'stopped':
        return go(state, {}, {}, { run: 'again', failed: 'told' });
      case 'unstopped':
        return go(
          state,
          {},
          { said: { ...state.said, [`${message.lend} ${message.app}`]: message.why } },
        );
      case 'told':
        return go(state, {});
      default:
        return assertNever(state, message, 'what acts as you');
    }
  },
};
const NOTES = {
  opens: 'Each Stop holds from the app’s next request.',
  play: 'Any program on this computer reaches these as you while Play is on.',
};

/** @param {HOME.PlaceProps} props */
function Acting({ s: homeState, home }) {
  const [state, send] = useMachine(() =>
    machine(
      'ACTS',
      {
        ...ACTS,
        io: {
          ...ACTS.io,
          again: /** @returns {Message} */ () => {
            home({ type: 'again' });
            return { type: 'told', why: 'the groups read again' };
          },
        },
      },
      { at: { reaching: 'reading', doing: 'idle' }, acted: {}, said: {} },
    ),
  );
  // HOME draws the section once the groups are read (home.js `Home`).
  const groups = /** @type {Group[]} */ (homeState.groups).filter(
    (group) => !group.left && group.removed === false,
  );
  const told = /** @param {Group} group */ (group) => homeState.told?.[group.id];
  const rows = /** @param {Group} group */ (group) => told(group)?.people ?? [];
  const lent = groups.flatMap((group) =>
    (told(group)?.lends?.lent ?? []).map((lend) => ({
      ...lend,
      called: called(rows(group), rows(group).find((row) => row.key === lend.key)?.person_key),
    })),
  );
  const asking = lent.filter((lend) => lend.asks).map(label);
  const ready = groups.every(told);
  useEffect(
    () => void (ready && send({ type: 'read', asking })),
    [ready, homeState.round, asking.join()],
  );
  useEffect(
    () =>
      void (
        ready &&
        state.at.reaching !== 'reading' &&
        !homeState.actsRead &&
        home({ type: 'told', flow: 'acts', read: true, why: 'what acts as you is read' })
      ),
    [ready, state.at.reaching],
  );
  // Drawn once read; a group new in a round (one made or joined) is said
  // read until it is told, the rest kept, where the section dropped out
  // and came back after each write (home_group.rs:219 on v2).
  if (state.at.reaching === 'reading') return null;
  // Each group `shown` is told; the casts of `told(g)` below say so, and
  // that a part drawn for its lends or its people has them, as it
  // filtered by them.
  const shown = groups.filter(told);
  /**
   * Changes' messages, a yes given its group and its change.
   * @param {Group} group
   */
  const yes = (group) => /** @param {Omit<Yes, 'g' | 'c'> | Message} message */ (message) =>
    message.type === 'yes'
      ? send({
          ...message,
          g: group.id,
          c: /** @type {{ lends: Lends }} */ (told(group)).lends.changed?.find(
            (change) => change.group === group.id && change.app === message.app,
          ),
        })
      : send(message);
  /** @param {Group} group */
  const doing = (group) =>
    Object.fromEntries(
      Object.entries(state.acted)
        .filter(([key]) => key.startsWith(`${group.id} `))
        .map(([key, act]) => [
          key.slice(group.id.length + 1),
          state.acting === key && state.at.doing === 'busy' ? { state: 'busy' } : act,
        ]),
    );
  // A change kept by its yes, once done, which the record no longer
  // lists: `act.c` and `lends.changed` are read where the filter has found
  // them, as tsc does not narrow them inside `some`'s callback, nor what
  // the filter keeps.
  /**
   * @param {Group} group
   * @param {Lends} [lends]
   */
  const changed = (group, lends = /** @type {{ lends: Lends }} */ (told(group)).lends) =>
    lends.changed && [
      ...lends.changed,
      ...Object.values(state.acted)
        .filter(
          (act) =>
            act.c?.group === group.id &&
            act.state === 'done' &&
            !(
              /** @type {Change[]} */ (lends.changed).some(
                (change) => change.app === /** @type {Change} */ (act.c).app,
              )
            ),
        )
        .map((act) => /** @type {Change} */ (act.c)),
    ];
  const parts = {
    reads: shown
      .filter((group) => /** @type {Told} */ (told(group)).lends)
      .flatMap((group) => [
        ...once(/** @type {{ lends: Lends }} */ (told(group)).lends.lent)
          .filter((lend) => lend.group)
          .map(
            (shared) =>
              html`<p>${carrying(shared.kind, /** @type {{ lends: Lends }} */ (told(group)).lends.shelf)}</p>`,
          ),
        ...Changes({
          g: group,
          changed: changed(group),
          rows: rows(group),
          me: homeState.key,
          doing: doing(group),
          open: state.folds?.changes,
          send: yes(group),
        }),
      ]),
    opens: [
      // The yeses are read, or why not, once ACTS is past 'reading'.
      ...(/** @type {Yeses} */ (state.yeses).why !== undefined
        ? [
            html`<p>This computer cannot say which apps open your friends’ things as you: ${/** @type {Yeses} */ (state.yeses).why}</p>`,
          ]
        : Object.entries(/** @type {Record<string, Choice>} */ (state.yeses)).map(
            ([app, change]) =>
              html`<p>${reading(
                /** @type {(Lend & { reads: string[] }) | undefined} */ (
                  lent.find((one) => label(one) === app && one.reads?.length)
                ),
                change,
              )}${change.said === 'yes' && html`<${Reach} app=${app} words="Stop" said="stop" />`}</p>`,
          )),
      // Read past `reading`, the choices are set: the read's, or none.
      ...Uses({
        lent,
        labelOf: label,
        choices: /** @type {Record<string, Asked>} */ (state.choices),
        tag: 'p',
        said: state.said,
        send,
      }),
    ],
    knows: [html`<p>What you open knows your name.</p>`],
    wifi: wifi.part(),
    play: play.part(),
    decides: shown
      .filter((group) => /** @type {Told} */ (told(group)).people)
      .map(
        (group) =>
          html`<p>${`${group.name ?? 'Your group'}: ${deciders(rows(group), /** @type {string} */ (homeState.key), 'computer')}.`}</p>`,
      ),
  };
  return html`<${Acts} parts=${parts} notes=${NOTES} open=${state.folds?.acts} send=${send} />${!ready && html`<p class="said">Reading your groups…</p>`}`;
}
/** @type {{ acts: (props: HOME.PlaceProps) => Drawn, told: (props: PlaceProps) => Drawn }} */
export const view = {
  acts: ({ s: state, home }) =>
    /** @type {Group[]} */ (state.groups).length > 0 && html`<${Acting} s=${state} home=${home} />`,
  told: Told,
};
