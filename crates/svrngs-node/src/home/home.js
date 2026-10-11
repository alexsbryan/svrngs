// SPDX-License-Identifier: AGPL-3.0-or-later
// The home page (crates/svrngs-node/src/home.rs): the person's groups, the
// people in each and what each lends, from the routes that answer this
// computer's apps (/v1/groups, /v1/lends, /v1/rail/people); the ask at
// /ask; and what apps use, each with Stop. Its only writes are the
// person's yes and stop, to its own origin's /choices. Its words are
// DESIGN §1's: groups, people, what they lend, computer; its question and
// what apps use are question.js's, the group's page's too.
import { named, question, unasked, uses as used } from './question.js';

const main = document.getElementById('home');
const el = (tag, text, ...kids) => {
  const e = document.createElement(tag);
  e.append(text ?? '', ...kids);
  return e;
};
const button = (text, act) => Object.assign(el('button', text), { onclick: act });

// A route's answer, or its words when it refuses: never read as empty.
async function get(path) {
  const r = await fetch(path);
  const body = await r.json().catch(() => ({}));
  if (!r.ok) throw new Error(body.error || `answered ${r.status}`);
  return body;
}

const said = (lend, app, what) => fetch('/choices', {
  method: 'POST',
  headers: { 'content-type': 'application/json' },
  body: JSON.stringify({ lend, app, said: what }),
}).then(async (r) => r.ok || Promise.reject(new Error((await r.json().catch(() => ({}))).error || `answered ${r.status}`)));

// The lend's label, the first name of its origin here.
const label = (l) => new URL(l.origin).hostname.split('.')[0];

// Each group with its people and what they lend, or why it cannot be read.
async function groups() {
  const { groups } = await get('/v1/groups');
  return Promise.all(groups.map(async (g) => {
    try {
      const ns = encodeURIComponent(g.id);
      const [lends, people] = await Promise.all([get(`/v1/lends?namespace=${ns}`), get(`/v1/rail/people?namespace=${ns}`)]);
      return { ...g, ...lends, people: people.people };
    } catch (e) {
      return { ...g, unread: e.message };
    }
  }));
}

function list(all) {
  for (const g of all) {
    const s = el('section', '', el('h2', g.name ?? 'A group whose name this computer cannot read'));
    if (g.unread) s.append(el('p', `What this group lends cannot be read on this computer: ${g.unread}`));
    else {
      s.append(el('p', `People: ${g.people.map((p) => p.person).join(', ')}`));
      const ul = el('ul');
      for (const l of g.lent) ul.append(el('li', '', Object.assign(el('a', named(l)), { href: l.origin })));
      for (const p of g.unanswered) ul.append(el('li', `${p}'s computer did not answer, so what they lend is not listed.`));
      if (!g.lent.length && !g.unanswered.length) ul.append(el('li', 'Nobody lends anything here yet.'));
      s.append(ul);
    }
    main.append(s);
  }
}

// What apps use of what the people of your groups lend, asked of each
// lender now, each with Stop.
async function uses(lent) {
  const s = el('section', '', el('h2', 'What apps use'));
  main.append(s);
  for (const l of lent.filter((l) => l.asks)) {
    const at = label(l);
    try {
      const { choices } = await get(`/choices?lend=${at}`);
      const appOf = (app) => lent.find((o) => label(o) === app);
      const stop = (app, line) => said(at, app, 'stop').then(() => location.reload(), (e) => line.append(e.message));
      used(l, choices, appOf, (words) => s.appendChild(el('p', words)), stop);
    } catch (e) {
      s.append(el('p', `${unasked(l)} ${e.message}`));
    }
  }
}

// The ask: in svrngs's words around the lender's, then back to the app.
function ask(lent, q) {
  const [lend, app] = [q.get('lend'), q.get('app')].map((x) => lent.find((l) => label(l) === x));
  if (!lend || !app || !lend.asks) return main.append(el('p', 'That app, or what it asks to use, is not lent in your groups now.'));
  const back = q.get('back')?.startsWith('/') && !q.get('back').startsWith('//') ? q.get('back') : '/';
  const go = () => location.assign(new URL(back, app.origin));
  const s = el('section', '', el('h2', question(app, lend)), el('blockquote', lend.asks));
  s.append(button('Yes', () => said(label(lend), label(app), 'yes').then(go, (e) => s.append(e.message))), button('Not now', go));
  main.append(s);
}

try {
  const all = await groups();
  main.replaceChildren();
  const lent = all.flatMap((g) => g.lent ?? []);
  if (location.pathname === '/ask') ask(lent, new URLSearchParams(location.search));
  else {
    list(all);
    await uses(lent);
  }
} catch (e) {
  main.replaceChildren(el('p', `Your groups cannot be read on this computer: ${e.message}`));
}
