// SPDX-License-Identifier: AGPL-3.0-or-later
// Give's page, carried in a group (give-app-keep; svrn-docs
// rings/current/GIVE.md §3 scenes 1, 3, 4 and 5, §7a): the shelf of apps
// the group can keep, who keeps each, "Start it for the group", which asks
// what the app's makers ask of whoever runs it, and "Move it to…", which
// asks the same of the person whose computer it goes to. When the computer
// that keeps one stops answering, or a copy doesn't start, "Keep it here?"
// offers the copy the fold names, with the same question; a computer's
// state from before another kept it is listed, to "Start instead". It reads the group at its own
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
// Which card's asks are shown, which card's computers to move it to,
// which copy's asks are shown to start it instead, and what the last write
// was refused with.
let asking = null;
let moving = null;
let taking = null;
let refused = null;
let cards = null;

/** A time as a person reads it: hours and minutes today, with the day before. */
function when(ts) {
  const d = new Date(ts * 1000);
  const today = d.toDateString() === new Date().toDateString();
  const hm = { hour: '2-digit', minute: '2-digit', hourCycle: 'h23' };
  return d.toLocaleString([], today ? hm : { month: 'short', day: 'numeric', ...hm });
}

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

/** `op` written as this computer's person; `failed` says what didn't happen, if svrngs refuses it. */
async function write(op, failed) {
  const r = await fetch(ACTS, { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ op }) }).catch(
    (e) => (trace(`${op.kind}, not written: svrngs not reached`, { app: op.app, error: String(e) }), null),
  );
  // A write that can't reach svrngs wrote nothing, and is said where the
  // page stands: what was open stays open to press again (give-page-down
  // (c)). The round reads nothing while an ask is open, so only this says it;
  // the next read, once svrngs answers, drops it.
  if (r === null) {
    const p = Object.assign(el('p', `${failed}: svrngs on this computer isn’t answering, so nothing reached your group. Press again in a moment.`), { id: 'unsent' });
    document.getElementById('unsent')?.remove();
    return app.querySelector('h1 + p').after(p);
  }
  const said = await r.json().catch(() => null);
  trace(`${op.kind}, written or refused`, { app: op.app, status: r.status, error: said?.error ?? null });
  refused = r.ok ? null : `${failed}: ${said?.error ?? `this computer answered ${r.status}`}`;
  asking = moving = taking = null;
  await show();
}

const start = (card, agreed) => write({ kind: 'started', app: card.kind, card: card.version, agreed }, `${card.title} didn’t start`);
const offer = (card, keeping, to) => write({ kind: 'offered', app: card.kind, over: keeping.act, to }, `${card.title} wasn’t offered`);
const answer = (card, o, yes) =>
  write(yes ? { kind: 'accepted', app: card.kind, offer: o.act, agreed: card.asks.length > 0 } : { kind: 'declined', app: card.kind, offer: o.act }, `Your answer wasn’t saved`);

/** Keep `card` here from `copy`, the keeping it follows being `keeping`'s. */
const take = (card, keeping, copy) =>
  write(
    { kind: 'taken', app: card.kind, card: copy.card, over: keeping.act, from: copy.from, sum: copy.sum, agreed: card.asks.length > 0 },
    `${card.title} wasn’t kept here`,
  );

/** The card's asks, each with its link. */
const asks = (card) => card.asks.map((ask) => Object.assign(el('p', `${ask.say} `, link('Read them', ask.link)), { className: 'ask' }));

/** A kept app's row past who keeps it: the move, as the fold says it stands for the person reading. */
function move(card, keeping, kept, read, out) {
  const o = keeping.offer;
  const mine = keeping.person_key === read.you_key;
  trace('the offer, as it stands', { app: card.kind, offer: o?.act ?? null, answer: o?.answer ?? null, mine });
  if (o?.answer === 'yes') return out.append(el('p', `Moving to ${o.person_key === read.you_key ? 'your' : `${o.person}’s`} computer…`));
  if (o?.answer === 'waiting' && o.person_key === read.you_key) {
    out.append(el('p', `${mine ? 'You want' : `${keeping.person} wants`} to move ${card.title} to your computer.`), ...asks(card));
    return out.append(button(card.asks.length ? 'Agree and keep it here' : 'Keep it here', () => answer(card, o, true)), button('Not now', () => answer(card, o, false), true));
  }
  if (!mine) return;
  if (o?.answer === 'waiting') return out.append(el('p', `Waiting for ${o.person} to say yes.`));
  if (o?.answer === 'no') out.append(el('p', `${o.person} said not now.`));
  const others = Object.entries(kept.can_keep).filter(([key]) => key !== keeping.keeper);
  if (others.length === 0) return;
  if (moving !== card.kind) return out.append(button('Move it to…', () => ((moving = card.kind), show()), true));
  out.append(el('p', `Move ${card.title} to:`));
  for (const [key, c] of others) out.append(button(c.person_key === read.you_key ? 'Your other computer' : `${c.person}’s computer`, () => offer(card, keeping, key)));
  out.append(button('Not now', () => ((moving = null), show()), true));
}

/**
 * A kept app's row past who keeps it, when its computer has stopped
 * answering or its copy didn't start: the copy the fold offers, to keep
 * here, for a person whose computer can keep. True when that is the row.
 */
function takeover(card, keeping, canKeep, out) {
  const next = keeping.next;
  const keep = () => {
    if (!next || !canKeep) return;
    out.append(...asks(card), button(card.asks.length ? 'Agree and keep it here' : 'Keep it here', () => take(card, keeping, next)));
  };
  if (keeping.failed) {
    out.append(el('p', next ? `This copy didn’t start. Try the copy from ${when(next.since)}?` : 'This copy didn’t start, and there’s no copy before it.'));
    keep();
    return true;
  }
  if (!keeping.gone) return false;
  const offer = next && canKeep ? ` You have a copy from ${when(next.since)}. Keep it here?` : '';
  out.append(el('p', `${card.title}’s computer hasn’t answered since ${when(keeping.answered)}.${offer}`));
  keep();
  return true;
}

/** Each state a computer had after another kept the app, to start instead. */
function had(card, keeping, canKeep, read, out) {
  for (const c of keeping.had) {
    const whose = c.person_key === read.you_key ? 'your' : `${c.person}’s`;
    out.append(el('p', `${card.title} as ${whose} computer had it at ${when(c.since)}`));
    if (!canKeep) continue;
    if (taking !== c.act) {
      out.append(button('Start instead', () => (card.asks.length ? ((taking = c.act), show()) : take(card, keeping, c)), true));
      continue;
    }
    out.append(...asks(card), button('Agree and start', () => take(card, keeping, c)), button('Not now', () => ((taking = null), show()), true));
  }
}

/** One card's row: its name and line, and who keeps it, or how to start it. */
function row(card, kept, read) {
  const keeping = kept.apps[card.kind];
  const out = el('li', '', el('h2', card.title), el('p', card.about));
  const canKeep = Object.values(kept.can_keep).some((c) => c.person_key === read.you_key);
  if (keeping) {
    out.append(el('p', `${card.title} · kept by ${who(keeping, read)}`));
    out.lastChild.className = 'kept';
    trace('who answers, as it stands', { app: card.kind, answered: keeping.answered, gone: keeping.gone, failed: keeping.failed?.act ?? null, next: keeping.next?.act ?? null, had: keeping.had.length });
    if (!takeover(card, keeping, canKeep, out)) move(card, keeping, kept, read, out);
    had(card, keeping, canKeep, read, out);
    return out;
  }
  // This person's own start of it, refused because the computer it was
  // written on can't keep apps: said, with what it needs.
  const mine = kept.refused.findLast((r) => r.app === card.kind && read.ops.some((o) => o.id === r.act && o.person_key === read.you_key));
  if (mine?.refusal === 'cannot_keep') out.append(install('This computer can’t keep apps yet. To keep them,'));
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
  out.append(...asks(card));
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
  // A read that can't reach svrngs, as while it restarts for an update, is
  // said, and what the page last read goes with it: never shown as current
  // (give-page-down). The next round reads again.
  const r = await fetch(ACTS).catch((e) => (trace('svrngs/acts not reached', { error: String(e) }), null));
  if (r === null) {
    return app.replaceChildren(el('h1', 'Give'), el('p', 'What your group keeps can’t be read just now: svrngs on this computer isn’t answering. This page will try again by itself.'));
  }
  // A body that is not JSON is said as that, never read as nothing kept.
  const read = await r.json().catch((e) => (trace('the answer is not JSON', { status: r.status, error: String(e) }), null));
  trace('svrngs/acts answered', { status: r.status, ops: read?.ops?.length ?? null });
  if (!r.ok || read === null || !Array.isArray(read.ops)) {
    return app.replaceChildren(el('h1', 'Give'), el('p', `What your group keeps can’t be read just now: ${read?.error ?? `this computer answered ${r.status}`}.`));
  }
  const kept = fold(read.ops, Math.floor(Date.now() / 1000));
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
setInterval(() => asking === null && moving === null && taking === null && show(), 3000);
