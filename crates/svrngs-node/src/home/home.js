// SPDX-License-Identifier: AGPL-3.0-or-later
// The home page (crates/svrngs-node/src/home.rs): the person's groups, the
// people in each and what each lends, from the routes that answer this
// computer's apps (/v1/groups, /v1/lends, /v1/rail/people), and who was
// removed or asked to be, from /v1/groups (v1-keeper-office (b)); the ask at
// /ask; what apps use, each with Stop; and what this computer lends, each
// with how many it lets in at once, to set, clear or stop (bash-limit). Its
// only writes are the person's yes and stop, to its own origin's /choices,
// and their change to what this computer lends, to its /lent. Its words are
// DESIGN §1's: groups, people, what they lend, computer; its question and
// what apps use and its removals are question.js's, the group's page's too.
import { named, question, removal, unasked, uses as used } from './question.js';

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

const post = (path, body) => fetch(path, {
  method: 'POST',
  headers: { 'content-type': 'application/json' },
  body: JSON.stringify(body),
}).then(async (r) => r.ok || Promise.reject(new Error((await r.json().catch(() => ({}))).error || `answered ${r.status}`)));
const said = (lend, app, what) => post('/choices', { lend, app, said: what });

// The lend's label, the first name of its origin here.
const label = (l) => new URL(l.origin).hostname.split('.')[0];

// Each group with its people and what they lend, or why it cannot be read.
// `removed` is null when who is in the group cannot be read here: never
// read as not removed (ralph/REVIEW_FINDINGS.md #123).
async function groups() {
  const { groups } = await get('/v1/groups');
  return Promise.all(groups.map(async (g) => {
    if (g.removed !== false) return g;
    try {
      const ns = encodeURIComponent(g.id);
      const [lends, people] = await Promise.all([get(`/v1/lends?namespace=${ns}`), get(`/v1/rail/people?namespace=${ns}`)]);
      return { ...g, ...lends, people: people.people };
    } catch (e) {
      return { ...g, unread: e.message };
    }
  }));
}

// Each person once, of the rows /v1/rail/people gives a key each; two of
// one name told apart by when each joined (bash-same-name), in the form
// `svrngs remove --joined` takes (home.rs `WHEN`): this computer's time as
// the `sv-SE` locale writes it, then its offset as ±hhmm, so a fall-back
// night's repeated hour reads back to one instant (#126).
function people(rows) {
  const each = [...new Map(rows.map((r) => [r.person_key, r])).values()];
  const at = (d, m = -d.getTimezoneOffset()) => `${d.toLocaleString('sv-SE')} ${m < 0 ? '-' : '+'}${String(Math.trunc(Math.abs(m) / 60) * 100 + (Math.abs(m) % 60)).padStart(4, '0')}`;
  const when = (j) => (j === 'started' ? 'started the group' : `joined ${at(new Date(j.at * 1000))}`);
  return each.map((p) => (each.filter((q) => q.person === p.person).length > 1 ? `${p.person} (${when(p.joined)})` : p.person)).join(', ');
}

function list(all) {
  for (const g of all) {
    const s = el('section', '', el('h2', g.name ?? 'A group whose name this computer cannot read'));
    if (g.removed === null) s.append(el('p', `This computer could not read who is in ${g.name ?? 'this group'}.`));
    else if (g.removed) s.append(el('p', `You are no longer in ${g.name ?? 'this group'}. What you already have stays on this computer; nothing new reaches it.`));
    else if (g.unread) s.append(el('p', `What this group lends cannot be read on this computer: ${g.unread}`));
    else {
      s.append(el('p', `People: ${people(g.people)}`));
      // Where the group's folder is on this disk (v1-onboarding (d)).
      const its = g.name ? `${g.name}’s folder` : 'The group’s folder';
      if (g.folders === null) s.append(el('p', `Where ${its} is cannot be read on this computer.`));
      for (const dir of g.folders ?? []) s.append(Object.assign(el('p', `${its} is on this computer at ${dir}`), { className: 'folder' }));
      if (g.removals.length) s.append(Object.assign(el('ul', '', ...g.removals.map((r) => el('li', removal(r)))), { className: 'removals' }));
      const ul = el('ul');
      // Each with its address for a program, copied by a button that says
      // so, then said (start-home): the clipboard may refuse a page.
      for (const l of g.lent) {
        const li = ul.appendChild(el('li', '', Object.assign(el('a', `Open ${named(l)}`), { href: l.origin }), ' '));
        li.append(button('Copy its address for a program', () => Promise.resolve().then(() => navigator.clipboard.writeText(l.origin)).then(() => li.append(` Copied: ${l.origin}`), () => li.append(` Copy it from here: ${l.origin}`))));
      }
      // Each computer that did not say by how (bash-first-minute (c)):
      // one that answered is never said not to have.
      for (const p of g.unanswered) ul.append(el('li', `${p}'s computer did not answer, so what they lend is not listed.`));
      for (const p of g.unheard) ul.append(el('li', `${p}'s computer has not heard of this computer yet, so what they lend is not listed yet.`));
      for (const p of g.refused) ul.append(el('li', `${p}'s computer answered without saying what they lend, so it is not listed.`));
      if (!ul.children.length) ul.append(el('li', 'Nobody lends anything here yet.'));
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

// What this computer lends, each with how many it lets in at once — its
// person's choice, a number or everyone in their groups — to set, clear or
// stop here, taken by the door at its next request.
async function lending() {
  const s = Object.assign(el('section', '', el('h2', 'What this computer lends')), { id: 'lent' });
  main.append(s);
  let lent;
  try {
    ({ lent } = await get('/lent'));
  } catch (e) {
    return s.append(el('p', `What this computer lends cannot be read on it: ${e.message}`));
  }
  if (!lent.length) s.append(el('p', 'This computer lends nothing.'));
  const ul = s.appendChild(el('ul'));
  for (const l of lent) {
    const n = l.limit;
    const li = el('li', `${l.name}: ${n ? `up to ${n} ${n === 1 ? 'person' : 'people'}` : 'everyone in your groups'} at once, ${l.using} using it now. `);
    const change = (to) => post('/lent', { lend: l.name, to }).then(() => location.reload(), (e) => li.append(e.message));
    const many = Object.assign(el('input'), { type: 'number', min: 1, value: n ?? '', ariaLabel: 'How many at once' });
    li.append(many, button('Let in this many', () => (/^[1-9][0-9]*$/.test(many.value) ? change({ limit: Number(many.value) }) : li.append('Say how many: 1 or more. '))));
    if (n) li.append(button('Let everyone in', () => change({ limit: null })));
    li.append(button('Stop lending', () => change('stop')));
    ul.append(li);
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
    // With no group, how to make one or join one: the command's, refused
    // while svrngs runs (cli.rs `claim`), so stopped first (start-home).
    if (!all.length) main.append(Object.assign(el('p', 'This computer is in no group yet. Where svrngs runs in a terminal, stop it with Ctrl-C, then type svrngs new <a name for your group> to make one, or svrngs join <the link you were sent> to join one.'), { className: 'none' }));
    list(all);
    await lending();
    await uses(lent);
    // Said as the command and install.sh say it: it is kept (start-address).
    main.append(Object.assign(el('p', `Your home page, the same address each time svrngs starts: ${location.origin}/`), { className: 'address' }));
  }
} catch (e) {
  main.replaceChildren(el('p', `Your groups cannot be read on this computer: ${e.message}`));
}
