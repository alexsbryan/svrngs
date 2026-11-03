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
 * @param {HOME.State} s
 * @param {Taken} m
 * @returns {Step<HOME.State, HOME.Effect>}
 */
export const take = (s, m) =>
  m.read
    ? go(s, {}, { actsRead: true })
    : go(s, {}, { told: { ...s.told, [m.id]: { people: m.people, lends: m.lends } } });
// The page's foot held until the section has read (home.js `Home`).
// HOME asks once the groups are read (home.js `Home`).
/** @param {HOME.State} s */
export const holds = (s) => /** @type {Group[]} */ (s.groups).length > 0 && !s.actsRead;
/** @param {PlaceProps} props */
const Told = ({ g, s, home }) => (
  useEffect(
    () =>
      void (
        s.at.people !== 'reading' &&
        s.at.lends !== 'reading' &&
        home({
          type: 'told',
          flow: 'acts',
          id: g.id,
          people: s.people?.people,
          lends: s.lends,
          why: `${g.name}: people ${s.at.people}, lends ${s.at.lends}`,
        })
      ),
    [s.people, s.lends, s.at.people, s.at.lends],
  ),
  null
);

/**
 * ACTS but for `again`, which `Acting` adds, as only it holds HOME's
 * `dispatch`.
 * @typedef {Omit<Machine<State, Message, Effect>, 'io'> & { io: Omit<Io<Effect, Message>, 'again'> }} Acts
 */

/** @type {Acts} */
const ACTS = {
  states: { reaching: ['reading', 'read', 'unread'], doing: ['idle', 'busy', 'done', 'refused'] },
  io: {
    read: ({ n, asking }) =>
      Promise.all([
        get('/reaching').catch((e) => ({ why: e.message })),
        ...asking.map((l) =>
          get(`/choices?lend=${l}`).then(
            /** @returns {[string, Asked]} */ ({ choices }) => [l, { choices }],
            /** @returns {[string, Asked]} */ (e) => [l, { why: e.message }],
          ),
        ),
      ]).then(([yeses, ...choices]) => ({
        type: 'reached',
        n,
        yeses,
        choices: Object.fromEntries(choices),
        why: `${choices.length} asking`,
      })),
    yes: ({ g, app }) =>
      post('/runs', [g, app]).then(() => ({ type: 'acted', why: `${app} at its new version` })),
    stop: ({ lend, app }) =>
      post('/choices', { lend, app, said: 'stop' }).then(() => ({
        type: 'stopped',
        why: `${app} stopped using ${lend}`,
      })),
  },
  step: (s, m) => {
    switch (m.type) {
      case 'read':
        return ((asked, n) =>
          go(s, {}, { asked }, { run: 'read', n, asking: m.asking, failed: 'unreached' }))(
          ...ask(s, 'reaching'),
        );
      case 'reached':
        return late(s, 'reaching', m.n)
          ? no(s, late(s, 'reaching', m.n))
          : go(s, { reaching: 'read' }, { yeses: m.yeses, choices: m.choices });
      case 'unreached':
        return go(s, { reaching: 'unread' }, { yeses: { why: m.why }, choices: {} });
      case 'fold':
        return go(s, {}, { folds: { ...s.folds, [m.fold]: m.open } });
      case 'yes':
        return s.at.doing === 'busy'
          ? no(s, `${s.acting} is under way, and is answered first`)
          : go(
              s,
              { doing: 'busy' },
              {
                acting: `${m.g} ${m.app}`,
                acted: {
                  ...s.acted,
                  [`${m.g} ${m.app}`]: { ...s.acted[`${m.g} ${m.app}`], c: m.c },
                },
              },
              { run: 'yes', g: m.g, app: m.app, failed: 'unacted' },
            );
      // A yes's answer, to the yes that set `acting` and kept it in `acted`:
      // `acting` read once, cast, where it was read at each use.
      case 'acted': {
        const acting = /** @type {string} */ (s.acting);
        return go(
          s,
          { doing: 'done' },
          { acted: { ...s.acted, [acting]: { ...s.acted[acting], state: 'done' } } },
          { run: 'again', failed: 'told' },
        );
      }
      case 'unacted': {
        const acting = /** @type {string} */ (s.acting);
        return go(
          s,
          { doing: 'refused' },
          {
            acted: {
              ...s.acted,
              [acting]: {
                ...s.acted[acting],
                state: 'refused',
                said: `${/** @type {Acted} */ (s.acted[acting]).said ?? ''} ${m.why}`,
              },
            },
          },
        );
      }
      case 'stop':
        return go(s, {}, {}, { run: 'stop', lend: m.lend, app: m.app, failed: 'unstopped' });
      case 'stopped':
        return go(s, {}, {}, { run: 'again', failed: 'told' });
      case 'unstopped':
        return go(s, {}, { said: { ...s.said, [`${m.lend} ${m.app}`]: m.why } });
      case 'told':
        return go(s, {});
      default:
        return assertNever(s, m, 'what acts as you');
    }
  },
};
const NOTES = {
  opens: 'Each Stop holds from the app’s next request.',
  play: 'Any program on this computer reaches these as you while Play is on.',
};

/** @param {HOME.PlaceProps} props */
function Acting({ s: h, home }) {
  const [s, send] = useMachine(() =>
    machine(
      'ACTS',
      {
        ...ACTS,
        io: {
          ...ACTS.io,
          again: /** @returns {Message} */ () => (
            home({ type: 'again' }),
            { type: 'told', why: 'the groups read again' }
          ),
        },
      },
      { at: { reaching: 'reading', doing: 'idle' }, acted: {}, said: {} },
    ),
  );
  // HOME draws the section once the groups are read (home.js `Home`).
  const groups = /** @type {Group[]} */ (h.groups).filter((g) => !g.left && g.removed === false),
    told = /** @param {Group} g */ (g) => h.told?.[g.id],
    rows = /** @param {Group} g */ (g) => told(g)?.people ?? [];
  const lent = groups.flatMap((g) =>
    (told(g)?.lends?.lent ?? []).map((l) => ({
      ...l,
      called: called(rows(g), rows(g).find((r) => r.key === l.key)?.person_key),
    })),
  );
  const asking = lent.filter((l) => l.asks).map(label),
    ready = groups.every(told);
  useEffect(() => void (ready && send({ type: 'read', asking })), [ready, h.round, asking.join()]);
  useEffect(
    () =>
      void (
        ready &&
        s.at.reaching !== 'reading' &&
        !h.actsRead &&
        home({ type: 'told', flow: 'acts', read: true, why: 'what acts as you is read' })
      ),
    [ready, s.at.reaching],
  );
  // Drawn once read; a group new in a round (one made or joined) is said
  // read until it is told, the rest kept, where the section dropped out
  // and came back after each write (home_group.rs:219 on v2).
  if (s.at.reaching === 'reading') return null;
  // Each group `shown` is told; the casts of `told(g)` below say so, and
  // that a part drawn for its lends or its people has them, as it
  // filtered by them.
  const shown = groups.filter(told);
  /**
   * Changes' messages, a yes given its group and its change.
   * @param {Group} g
   */
  const yes = (g) => /** @param {Omit<Yes, 'g' | 'c'> | Message} m */ (m) =>
    m.type === 'yes'
      ? send({
          ...m,
          g: g.id,
          c: /** @type {{ lends: Lends }} */ (told(g)).lends.changed?.find(
            (c) => c.group === g.id && c.app === m.app,
          ),
        })
      : send(m);
  /** @param {Group} g */
  const doing = (g) =>
    Object.fromEntries(
      Object.entries(s.acted)
        .filter(([k]) => k.startsWith(`${g.id} `))
        .map(([k, a]) => [
          k.slice(g.id.length + 1),
          s.acting === k && s.at.doing === 'busy' ? { state: 'busy' } : a,
        ]),
    );
  // A change kept by its yes, once done, which the record no longer
  // lists: `a.c` and `L.changed` are read where the filter has found
  // them, as tsc does not narrow them inside `some`'s callback.
  /**
   * @param {Group} g
   * @param {Lends} [L]
   */
  const changed = (g, L = /** @type {{ lends: Lends }} */ (told(g)).lends) =>
    L.changed && [
      ...L.changed,
      ...Object.values(s.acted)
        .filter(
          (a) =>
            a.c?.group === g.id &&
            a.state === 'done' &&
            !(
              /** @type {Change[]} */ (L.changed).some(
                (c) => c.app === /** @type {Change} */ (a.c).app,
              )
            ),
        )
        .map((a) => a.c),
    ];
  const parts = {
    reads: shown
      .filter((g) => /** @type {Told} */ (told(g)).lends)
      .flatMap((g) => [
        ...once(/** @type {{ lends: Lends }} */ (told(g)).lends.lent)
          .filter((o) => o.group)
          .map(
            (l) =>
              html`<p>${carrying(l.kind, /** @type {{ lends: Lends }} */ (told(g)).lends.shelf)}</p>`,
          ),
        ...Changes({
          g,
          changed: changed(g),
          rows: rows(g),
          me: h.key,
          doing: doing(g),
          open: s.folds?.changes,
          send: yes(g),
        }),
      ]),
    opens: [
      // The yeses are read, or why not, once ACTS is past 'reading'.
      ...(/** @type {Yeses} */ (s.yeses).why !== undefined
        ? [
            html`<p>This computer cannot say which apps open your friends’ things as you: ${/** @type {Yeses} */ (s.yeses).why}</p>`,
          ]
        : Object.entries(/** @type {Record<string, Choice>} */ (s.yeses)).map(
            ([app, c]) =>
              html`<p>${reading(
                /** @type {(Lend & { reads: string[] }) | undefined} */ (
                  lent.find((o) => label(o) === app && o.reads?.length)
                ),
                c,
              )}${c.said === 'yes' && html`<${Reach} app=${app} words="Stop" said="stop" />`}</p>`,
          )),
      // Read past `reading`, the choices are set: the read's, or none.
      ...Uses({
        lent,
        labelOf: label,
        choices: /** @type {Record<string, Asked>} */ (s.choices),
        tag: 'p',
        said: s.said,
        send,
      }),
    ],
    knows: [html`<p>What you open knows your name.</p>`],
    wifi: wifi.part(),
    play: play.part(),
    decides: shown
      .filter((g) => /** @type {Told} */ (told(g)).people)
      .map(
        (g) =>
          html`<p>${`${g.name ?? 'Your group'}: ${deciders(rows(g), /** @type {string} */ (h.key), 'computer')}.`}</p>`,
      ),
  };
  return html`<${Acts} parts=${parts} notes=${NOTES} open=${s.folds?.acts} send=${send} />${!ready && html`<p class="said">Reading your groups…</p>`}`;
}
/** @type {{ acts: (props: HOME.PlaceProps) => Drawn, told: (props: PlaceProps) => Drawn }} */
export const view = {
  acts: ({ s, home }) =>
    /** @type {Group[]} */ (s.groups).length > 0 && html`<${Acting} s=${s} home=${home} />`,
  told: Told,
};
