// SPDX-License-Identifier: AGPL-3.0-or-later
// Give's page, carried in a group (give-app-keep; svrn-docs
// rings/current/GIVE.md §3 scene 1, §7a): the shelf of apps the group can
// keep, who keeps each, and "Start it for the group", which asks what the
// app's makers ask of whoever runs it. It reads the group at its own
// origin, `svrngs/acts`, and writes there; svrngs writes each act as this
// computer's person. Nothing here decides who keeps an app: the fold does,
// the vendored build of the one the keeper program runs (vendor/), and this
// page says its answer in the group's words. A person never reads
// container, image, volume, snapshot, keeper or port forwarding here.
import { bind } from './vendor/call.mjs';
import * as build from './vendor/fold.js';

const ACTS = 'svrngs/acts';
const PODMAN = 'https://podman.io/docs/installation';
const PROGRAM = 'https://github.com/alexsbryan/svrngs/tree/main/apps/give#keeping-an-app';
// Where this page remembers, on this computer, that it asked about the
// install once and was told "Not now".
const ASKED = 'give: asked to install';
const fold = bind(build);
// Each decision, at debug in the console, never on the page (AGENTS.md 1),
// as House rules traces its own (apps/canon/index.js).
const trace = (what, fields = {}) => console.debug('Give:', what, JSON.stringify(fields));
const told = new Set();
const app = document.getElementById('app');
const el = (tag, text, ...kids) => {
  const e = document.createElement(tag);
  e.append(text ?? '', ...kids);
  return e;
};
const button = (text, act, quiet) => Object.assign(el('button', text), { onclick: act, className: quiet ? 'quiet' : '' });
const link = (text, href) => Object.assign(el('a', text), { href, target: '_blank', rel: 'noopener' });
// Which card's asks are shown, and what the last write was refused with.
let asking = null;
let refused = null;
let cards = null;

/** Who keeps an app, in the group's words. */
const who = (keeping, read) => (keeping.person_key === read.you_key ? 'your computer' : `${keeping.person}’s computer`);

/** The install a computer needs to keep apps, said once. */
function install(why) {
  return el(
    'p',
    `${why} svrngs needs Podman (free, one install) and Give’s program for computers. `,
    link('Get Podman', PODMAN),
    ' · ',
    link('Get Give’s program', PROGRAM),
  );
}

async function start(card, agreed) {
  const op = { kind: 'started', app: card.kind, card: card.version, agreed };
  const r = await fetch(ACTS, { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ op }) });
  const said = await r.json().catch(() => null);
  trace('started, written or refused', { app: card.kind, status: r.status, error: said?.error ?? null });
  refused = r.ok ? null : `${card.title} didn’t start: ${said?.error ?? `this computer answered ${r.status}`}`;
  asking = null;
  await show();
}

/** One card's row: its name and line, and who keeps it, or how to start it. */
function row(card, kept, read) {
  const keeping = kept.apps[card.kind];
  const out = el('li', '', el('h2', card.title), el('p', card.about));
  if (keeping) {
    out.append(el('p', `${card.title} · kept by ${who(keeping, read)}`));
    out.lastChild.className = 'kept';
    return out;
  }
  // This person's own start of it, refused because the computer it was
  // written on can't keep apps: said, with what it needs.
  const mine = kept.refused.findLast((r) => r.app === card.kind && read.ops.some((o) => o.id === r.act && o.person_key === read.you_key));
  if (mine?.refusal === 'cannot_keep') out.append(install('This computer can’t keep apps yet. To keep them,'));
  const canKeep = Object.values(kept.can_keep).some((c) => c.person_key === read.you_key);
  if (!canKeep && !mine && localStorage.getItem(ASKED) === null) {
    out.append(
      install(`To keep ${card.title} on this computer,`),
      button('Not now', () => (localStorage.setItem(ASKED, 'yes'), show()), true),
    );
    return out;
  }
  if (asking !== card.kind) {
    out.append(button('Start it for the group', () => (card.asks.length ? ((asking = card.kind), show()) : start(card, false))));
    return out;
  }
  for (const ask of card.asks) out.append(Object.assign(el('p', `${ask.say} `, link('Read them', ask.link)), { className: 'ask' }));
  out.append(button('Agree and start', () => start(card, true)), button('Not now', () => ((asking = null), show()), true));
  return out;
}

async function show() {
  if (cards === null) {
    const c = await fetch('cards.json');
    cards = c.ok ? await c.json().catch(() => null) : null;
    trace('cards.json read', { status: c.status, cards: cards?.length ?? null });
    if (!Array.isArray(cards)) return app.replaceChildren(el('h1', 'Give'), el('p', 'The shelf can’t be read just now: bring Give to your group again.'));
  }
  const r = await fetch(ACTS);
  // A body that is not JSON is said as that, never read as nothing kept.
  const read = await r.json().catch((e) => (trace('the answer is not JSON', { status: r.status, error: String(e) }), null));
  trace('svrngs/acts answered', { status: r.status, ops: read?.ops?.length ?? null });
  if (!r.ok || read === null || !Array.isArray(read.ops)) {
    return app.replaceChildren(el('h1', 'Give'), el('p', `What your group keeps can’t be read just now: ${read?.error ?? `this computer answered ${r.status}`}.`));
  }
  const kept = fold(read.ops);
  for (const r of kept.refused.filter((r) => !told.has(r.act))) {
    told.add(r.act);
    trace('the fold refused an act', r);
  }
  trace('folded', { kept: Object.keys(kept.apps), can_keep: Object.keys(kept.can_keep).length });
  app.replaceChildren(
    el('h1', 'Give'),
    el('p', 'Apps your group keeps on one of your computers at a time. Everyone in the group can use one while it’s kept.'),
    ...(refused ? [el('p', refused)] : []),
    el('ul', '', ...cards.map((card) => row(card, kept, read))),
  );
}

show();
setInterval(() => asking === null && show(), 3000);
