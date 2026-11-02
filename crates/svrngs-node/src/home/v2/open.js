// SPDX-License-Identifier: AGPL-3.0-or-later
// "Open a link from your group" (page-join (f); home.js:394-400): a link
// from a group's page, `<page>#open=<label>&at=<path>`, pasted
// (events-links (c)): the lend it names opened at its path in a tab of its
// own, through `/to` (give-session), as the group's page opens it on a
// phone. Where it opens is `to`, pure, and only within the lend (`within`,
// #153); the tab is opened at the edge. Its machine, OPEN, is the place's
// own (ui/view.js `useMachine`). A link to nothing lent here is said beside
// the button once, the last in place of the one before, where v1 appended
// each (h:397; page-join (f)'s fix).
import { Act, Field, go, html, machine, no, useMachine } from '../ui/kit.js?v=__VERSION__';

// `at` (`/` when none) as an address at `origin`, or null: resolved, and
// kept only if it is still there, as open.js judges its way back — a path
// `startsWith` let through, `/\evil.example`, left for another host (#153).
export const within = (at, origin) => ((u) => (u.origin === new URL(origin).origin ? u.href : null))(new URL(at ?? '/', origin));
// Where `link` opens through `/to` from a home page on `port`, or null.
const to = (link, port, q = new URLSearchParams(link.split('#')[1] ?? ''), lend = q.get('open') ?? '') =>
  ((at) => at && `/to?${new URLSearchParams({ lend, at: ((u) => u.pathname + u.search + u.hash)(new URL(at)) })}`)(/^[a-z0-9]+$/.test(lend) && within(q.get('at'), `http://${lend}.localhost:${port}`));
const OPEN = { states: { open: ['typing', 'opened', 'refused'] }, io: { open: ({ at }) => (window.open(at, '_blank'), { type: 'opened', why: at }) },
  step: (s, m) => ({ typed: () => go(s, { open: 'typing' }, { link: m.value }), opened: () => go(s, {}),
    open: () => ((at) => (at ? go(s, { open: 'opened' }, {}, { run: 'open', at, failed: 'refused' }) : go(s, { open: 'refused' }, { said: ' That is not a link to something lent in your groups.' })))(to(s.link, s.port)),
    refused: () => go(s, { open: 'refused' }, { said: ` ${m.why}` }) })[m.type]?.() ?? no(s, `opening a link takes no ${m.type}`) };

function Open({ origin }) {
  const [s, send] = useMachine(() => machine('OPEN', OPEN, { at: { open: 'typing' }, link: '', said: '', port: new URL(origin).port }));
  return html`<section id="open"><h2>Open a link from your group</h2><p><${Field} words="The link: " value=${s.link} send=${send} /><${Act} words="Open it" doing=${{ said: s.said }} on=${{ type: 'open' }} send=${send} /></p></section>`;
}
export const view = { open: ({ s }) => s.groups.length > 0 && html`<${Open} origin=${s.origin} />` };
