// SPDX-License-Identifier: AGPL-3.0-or-later
// House rules' page, carried in a group (canon-house (c); svrn-docs
// rings/current/CANON_IN_SVRNGS.md §3): how your house decides, its rules,
// what's proposed with Agree and Object, and a way to propose one. It reads
// the group at its own origin, `svrngs/acts`, and writes there; svrngs writes
// each entry as you. Nothing here decides: house.js hands what it read to
// canon, the vendored build, and this page says canon's answers in the
// house's words, never canon's own (scope, ratification, commitment, act).
// An entry another person's computer wrote says so: "dee, through cy’s
// computer" (gv-via; Money's words, apps/expenses/index.js).
import { bind } from './vendor/call.mjs';
import * as build from './vendor/canon.js';
import { CHOICES, agree, decide, fold, object, payload, propose } from './house.js';

const ACTS = 'svrngs/acts';
const call = bind(build);
// Each decision the page and house.js make, at debug in the console, never
// on the page (AGENTS.md 1; #210), as web/app.js traces the phone's page.
const trace = (what, fields = {}) => console.debug('House rules:', what, JSON.stringify(fields));
const app = document.getElementById('app');
const el = (tag, text, ...kids) => {
  const e = document.createElement(tag);
  e.append(text ?? '', ...kids);
  return e;
};
const button = (text, act) => Object.assign(el('button', text), { onclick: act });
const day = (t) => new Date(t * 1000).toLocaleDateString('en-GB', { day: 'numeric', month: 'short' });
const cap = (s) => `${s[0].toUpperCase()}${s.slice(1)}`;
const list = (names) => (names.length < 2 ? names.join('') : `${names.slice(0, -1).join(', ')} and ${names.at(-1)}`);

// How a way of deciding reads, from the rule canon says is in force.
function how(rule) {
  if (rule.rule === 'consent') return `Anyone proposes a rule, and it holds after ${rule.days === 1 ? 'a day' : `${rule.days} days`} unless someone objects.`;
  if (rule.rule === 'majority') return 'A rule holds once more than half of the people in your house when it was proposed agree, whoever proposed it among them.';
  if (rule.rule === 'unanimous') return 'A rule holds once everyone in your house when it was proposed agrees, and one objection stops it.';
  if (rule.rule === 'share') return `A rule holds once ${rule.more_than ? 'more than' : 'at least'} ${rule.numerator} in ${rule.denominator} of the people in your house when it was proposed agree, whoever proposed it among them.`;
  if (rule.rule === 'threshold') return `A rule holds once ${rule.approve} of you agree, whoever proposed it among them, and ${rule.block === 1 ? 'one objection stops it' : `${rule.block} objections stop it`}.`;
  if (rule.rule === 'standing') return 'Anyone in your house makes a rule at once.';
  return 'Your house decides in a way this page can’t say yet.';
}

// The ways to decide, each with what it means, to tick.
function ways(name) {
  return Object.keys(CHOICES).map((choice) => {
    const more = choice === 'consent' ? '' : ` ${how(decide(choice).rule)}`;
    return el('p', '', el('label', '', Object.assign(el('input'), { type: 'radio', name, value: choice }), ` ${CHOICES[choice].words}.${more}`));
  });
}

async function show() {
  const r = await fetch(ACTS);
  // A body that is not JSON is said as that, never read as an empty answer (#209).
  const read = await r.json().catch((e) => (trace('the answer is not JSON', { status: r.status, error: String(e) }), null));
  trace('svrngs/acts answered', { status: r.status, ops: read?.ops?.length ?? null });
  if (!r.ok) return app.replaceChildren(el('h1', 'House rules'), el('p', `Your house’s rules can’t be read just now: ${read?.error ?? `it answered ${r.status}`}.`));
  if (read === null || typeof read !== 'object') return app.replaceChildren(el('h1', 'House rules'), el('p', 'Your house’s rules can’t be read just now: this computer answered in a way House rules can’t read.'));
  let view;
  try {
    view = fold(call, read, Math.floor(Date.now() / 1000), trace);
  } catch (e) {
    trace('not read', { error: e.message });
    return app.replaceChildren(el('h1', 'House rules'), el('p', `This version of House rules can’t read your house’s rules: ${e.message}.`));
  }
  if (view.unsettled) return app.replaceChildren(el('h1', 'House rules'), el('p', 'This computer can’t yet say who is in your house, so it can’t say what your house agreed. Open this page again in a minute.'));
  const { you } = read;
  const said = Object.assign(el('p'), { id: 'said' });
  const who = (by, via) => {
    const name = by === you ? 'you' : by;
    return via ? `${name}, through ${via === you ? 'your' : `${via}’s`} computer` : name;
  };
  // An entry checked by house.js, then sent; the page drawn again once it
  // is saved, or why not said.
  const send = (act) => {
    let body;
    try {
      body = JSON.stringify({ op: payload(act) });
    } catch (e) {
      trace('not sent', { op: act?.op, error: e.message });
      return said.replaceChildren(`That isn’t saved: ${e.message}.`);
    }
    trace('sent', { op: act.op });
    const sent = fetch(ACTS, { method: 'POST', headers: { 'content-type': 'application/json' }, body });
    return sent.then(async (w) => (w.ok ? show() : Promise.reject(new Error((await w.json().catch(() => ({}))).error ?? `it answered ${w.status}`)))).catch((e) => (trace('not saved', { op: act.op, error: e.message }), said.replaceChildren(`That isn’t saved: ${e.message}.`)));
  };
  const ticked = (name) => app.querySelector(`input[name=${name}]:checked`)?.value;
  const people = el('section', '', el('h2', 'Who has a say'), el('p', `Everyone in your house has a say: ${list(view.deciding.map((p) => (p === you ? 'you' : p)))}.`));
  for (const s of view.spans.filter((s) => s.joined != null || s.left != null)) {
    const name = s.person === you ? 'You' : s.person;
    people.append(el('p', s.left == null ? `${name} joined on ${day(s.joined)}.` : `${name} had a say${s.joined == null ? '' : ` from ${day(s.joined)}`} until ${day(s.left)}, when they left.`));
  }
  const notes = el('section');
  if (view.dropped.length) notes.append(el('p', `${view.dropped.length === 1 ? 'One entry isn’t' : `${view.dropped.length} entries aren’t`} counted:`), el('ul', '', ...view.dropped.map((d) => el('li', `${cap(who(d.by ?? 'someone', d.via))}: ${d.why}.`))));
  if (view.ungoverned.length) notes.append(el('p', `${view.ungoverned.length === 1 ? 'One entry' : `${view.ungoverned.length} entries`} took no effect: whoever wrote it had no say over it then.`));
  if (view.bootstrap.length || view.unattended.length) notes.append(el('p', 'Some entries took effect before anyone in your house had a say.'));

  if (!view.rule) {
    trace('asking how the house decides');
    return app.replaceChildren(
      el('h1', 'House rules'),
      el('h2', 'How does your house decide its rules?'),
      el('p', 'Choose how a proposal becomes your house’s rule. Everyone in your house sees what you chose, and your house can change it later the way it decides anything.'),
      ...ways('way'),
      el('p', '', button('Make this how your house decides', () => (ticked('way') ? send(decide(ticked('way'))) : said.replaceChildren('Choose one first.')))),
      said,
      people,
      notes,
    );
  }

  // What people said of something proposed, and the buttons for it.
  const voices = (p) => {
    const agreed = p.said.filter((s) => s.agrees).map((s) => who(s.by, s.via));
    const against = p.said.filter((s) => !s.agrees).map((s) => el('p', `${cap(who(s.by, s.via))} ${s.by === you ? 'object' : 'objects'}: “${s.why}”`));
    const out = [el('p', `${cap(who(p.by, p.via))} proposed it on ${day(p.at)}.${agreed.length ? ` Agreed: ${agreed.join('; ')}.` : ''}`), ...against];
    if (p.by === you || p.said.some((s) => s.by === you)) return out;
    const why = el('p');
    const asking = () => why.replaceChildren(el('label', 'Why do you object? ', Object.assign(el('input'), { className: 'why' })), ' ', button('Send your objection', () => {
      try {
        return send(object(p.id, why.querySelector('input').value));
      } catch (e) {
        trace('not sent', { op: 'position', error: e.message });
        return said.replaceChildren(`${cap(e.message)} first.`);
      }
    }));
    return [...out, el('p', '', button('Agree', () => send(agree(p.id))), ' ', button('Object', asking)), why];
  };
  const rules = view.rules.filter((c) => c.status.status === 'active');
  const proposed = [
    ...view.rules.filter((c) => c.status.status === 'proposed').map((c) => el('li', '', el('p', '', el('strong', c.text)), ...voices(c))),
    ...view.changes.filter((c) => c.verdict.verdict === 'proposed').map((c) => el('li', '', el('p', '', el('strong', `Change how your house decides: ${how(c.rule)}`)), ...voices(c))),
  ];
  // One person's objection stops it, or, under a share of the house, too
  // many objected or left for it to be agreed; canon says which by `by`.
  const stopped = (p, s) => {
    if (s.by) return `${cap(who(s.by.replace(/^human:/, ''), null))} objected: “${s.why}”`;
    const against = p.said.filter((x) => !x.agrees).map((x) => `${cap(who(x.by, x.via))}: “${x.why}”`);
    return `Too many of you objected or left for it to be agreed.${against.length ? ` ${against.join(' ')}` : ''}`;
  };
  const refused = [
    ...view.rules.filter((c) => c.status.status === 'refused').map((c) => [c.text, c, c.status]),
    ...view.changes.filter((c) => c.verdict.verdict === 'refused').map((c) => [`Change how your house decides: ${how(c.rule)}`, c, c.verdict]),
  ].map(([text, p, s]) => el('li', `${text} ${stopped(p, s)}`));
  app.replaceChildren(
    el('h1', 'House rules'),
    el('h2', 'How your house decides'),
    el('p', how(view.rule)),
    el('h2', 'Your house’s rules'),
    rules.length ? Object.assign(el('ul', '', ...rules.map((c) => el('li', `${c.text} (${who(c.by, c.via)} proposed it on ${day(c.at)})`))), { className: 'rules' }) : el('p', 'Your house has no rules yet.'),
    el('h2', 'What’s proposed'),
    proposed.length ? Object.assign(el('ul', '', ...proposed), { className: 'proposed' }) : el('p', 'Nothing is waiting on anyone.'),
    ...(refused.length ? [el('h2', 'Not adopted'), el('ul', '', ...refused)] : []),
    el('h2', 'Propose a rule'),
    el('p', '', el('label', 'Your rule: ', Object.assign(el('input'), { id: 'rule' })), ' ', button('Propose this rule to everyone in your house', () => {
      const text = document.getElementById('rule').value.trim();
      return text ? send(propose(text)) : said.replaceChildren('Write the rule first.');
    })),
    said,
    el('details', '', el('summary', 'Change how your house decides'), el('p', 'Your house agrees to a change the way it agrees to a rule now.'), ...ways('change'), el('p', '', button('Propose this change to everyone in your house', () => (ticked('change') ? send(decide(ticked('change'))) : said.replaceChildren('Choose one first.'))))),
    people,
    notes,
  );
}

show();
