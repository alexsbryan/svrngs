// SPDX-License-Identifier: AGPL-3.0-or-later
// A link made in a group's section (page-join (e); home.js@19b63b2b:133-140), as
// `svrngs link` makes it (start-home): each button says what it makes, to
// invite someone, to add a phone or another computer, or a guest's, whose
// length is asked first, 48 hours chosen to start and nothing longer
// offered (guests-link; GUESTS §5). Its machine, LINK, is the place's own
// (ui/view.js `useMachine`), one per group, and a second press while one
// is made is refused. The link is then said in p.link, in its words, with
// Copy (ui/copy.js) beside it, its last answer alone, where home.js@19b63b2b:137 appends
// each (page-join (e)'s fix); a refusal said there in its place.
import {
  Act,
  assertNever,
  Field,
  go,
  html,
  machine,
  no,
  useMachine,
} from '../ui/kit.js?v=__VERSION__';
import { COPY, drawn } from '../ui/copy.js?v=__VERSION__';
import { post } from './group.js?v=__VERSION__';
/** @import { Group, PlaceProps } from './group.js?v=__VERSION__' */
/** @import { Machine } from '../ui/machine.js?v=__VERSION__' */
/** @import { Drawn } from '../ui/view.js?v=__VERSION__' */
/** @import { CopyProps } from '../ui/copy.js?v=__VERSION__' */

/**
 * A press of one of the buttons: whether the link adds a device of yours,
 * and, for a guest's, how long the visit is, as said.
 * @typedef {{ type: 'make', device: boolean, guest?: true, hours?: string }} Make
 */

/**
 * The link made, as /link answers it (home.rs `link`): the client's link,
 * and the page's when this computer has one; and the press it answers.
 * @typedef {{ type: 'made', a: { client: string, https: string | null }, asked: Make, why: string }} Made
 */

/**
 * A group's links' state: where its link is, the guest's visit in seconds
 * as the Field holds it, whether a guest's link is offered, the last link
 * made, and why the last was refused.
 * @typedef {object} State
 * @property {{ link: 'none' | 'making' | 'made' | 'refused' }} at
 * @property {Record<string, number>} [asked]
 * @property {string} visit
 * @property {boolean} [guest]
 * @property {Made} [shown]
 * @property {string} [why]
 */

/**
 * The messages a link takes: Have a guest pressed; the visit's length
 * chosen; a button pressed; its link made, or refused, with why.
 * @typedef {{ type: 'guest' }
 *   | { type: 'typed', field: 'visit', value: string }
 *   | Make
 *   | Made
 *   | { type: 'refused', why: string }} Message
 */

/**
 * The effect its step answers: a link made for the press `asked`, a
 * guest's for `visit` seconds.
 * @typedef {{ run: 'make', asked: Make, visit: string | undefined, failed: 'refused' }} Effect
 */

/**
 * @param {string} group
 * @returns {Machine<State, Message, Effect>}
 */
const LINK = (group) => ({
  states: { link: ['none', 'making', 'made', 'refused'] },
  io: {
    make: ({ asked, visit }) =>
      post('/link', { group, device: asked.device, ...(visit && { visit: Number(visit) }) }).then(
        (a) => ({ type: 'made', a, asked, why: a.https ?? a.client }),
      ),
  },
  step: (s, m) => {
    switch (m.type) {
      case 'guest':
        return go(s, {}, { guest: true });
      case 'typed':
        return go(s, {}, { visit: m.value });
      case 'make':
        return s.at.link === 'making'
          ? no(s, 'a link is being made, and is answered first')
          : go(
              s,
              { link: 'making' },
              {},
              { run: 'make', asked: m, visit: m.guest && s.visit, failed: 'refused' },
            );
      case 'made':
        return go(s, { link: 'made' }, { shown: m });
      case 'refused':
        return go(s, { link: 'refused' }, { why: m.why });
      default:
        return assertNever(s, m, 'a link');
    }
  },
});
// A Copy of the link that says only its last answer, where ui/copy.js's
// says each in turn.
/** @param {CopyProps} props */
const Copy = (props) =>
  ((s, send) => drawn({ ...s, said: s.said.slice(-1) }, send, props))(
    ...useMachine(() => machine('COPY', COPY, { at: { copy: 'idle' }, said: [] })),
  );
const WORDS = {
    copy: 'Copy the link',
    copied: ' Copied.',
    unwritten: ' Select it above to copy it.',
  },
  LAST =
    ' It works for a day, while this computer is on. A new link stops this one letting anyone new in; everyone it let in stays. ';
// The link made, in the words of whom it is for (home.js@19b63b2b:134, :139), kept while
// the next is made, as v1 keeps it until the answer (home.js@19b63b2b:135).
/**
 * @param {Made} shown
 * @param {string} name
 * @param {string} [at]
 * @returns {Drawn[]}
 */
const said = ({ a, asked }, name, at = a.https ?? a.client) => [
  ...(asked.guest
    ? [
        'Send this link to your guest, in any chat: ',
        html`<code>${at}</code>`,
        ` They open it in a browser and give their name. For ${asked.hours} from then they can open what you lend, and everyone in ${name} sees they are your guest. It lets one guest in, once, within a day, while this computer is on. `,
      ]
    : [
        asked.device
          ? 'Open this link on your other computer or phone; it joins as you, so send it to nobody else: '
          : 'Send this link to the person you are inviting: ',
        html`<code>${at}</code>`,
        a.https
          ? ' It opens in a browser, which says how to join from there.'
          : ' They join with svrngs join and this link.',
        LAST,
      ]),
  html`<${Copy} key=${at} text=${at} words=${WORDS} beside=${true} />`,
];

/**
 * A group's links, made for it.
 * @typedef {{ g: Group }} LinksProps
 */

/** @param {LinksProps} props */
function Links({ g }) {
  const [s, send] = useMachine(() =>
      machine('LINK', LINK(g.id), { at: { link: 'none' }, visit: '172800' }),
    ),
    name = g.name ?? 'this group';
  /**
   * @param {string} words
   * @param {Omit<Make, 'type'>} on
   */
  const act = (words, on) =>
    html`<${Act} words=${words} doing=${{ state: s.at.link === 'making' ? 'busy' : 'idle' }} on=${{ type: 'make', ...on }} send=${send} />`;
  // `s.visit` is the Field's string, as `Number` reads it, which is what `/`
  // made of it before tsc asked for a number.
  return html`<p>${act(`Invite someone to ${name}`, { device: false })}${act('Add your phone or another computer', { device: true })}${s.guest ? html`<${Field} name="visit" words="For how long, from when they open the link: " value=${s.visit} options=${[48, 24, 6].map((h) => [h * 3600, `${h} hours`])} send=${send} />${act('Make a link for your guest', { device: false, guest: true, hours: `${Number(s.visit) / 3600} hours` })}` : html`<${Act} words="Have a guest" on=${{ type: 'guest' }} send=${send} />`}</p><p class="link">${s.at.link === 'refused' ? s.why : s.shown && said(s.shown, name)}</p>`;
}
// Drawn once the group's people are read, as v1 draws it (home.js@19b63b2b:121-139).
/** @type {{ links: (props: PlaceProps) => Drawn }} */
export const view = { links: ({ g, s }) => s.people && html`<${Links} g=${g} />` };
