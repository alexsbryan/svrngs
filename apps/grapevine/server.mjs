// Grapevine's own small server and page (GRAPEVINE_DEMO §5, §6, §13.2): the
// asks you can see in each of your groups, what you could carry and where,
// your conversations, and the forms to ask and to list what you have. Run
// on the person's own computer and opened there, never lent (§13.2).
//
//   node apps/grapevine/server.mjs <config.json>
//
// Listens on 127.0.0.1 and prints `{"port": N, "at": "http://127.0.0.1:N/"}`.
// No dependency but the vendored tweetnacl. The config:
//
//   {"record": "http://127.0.0.1:<port>"}   the groups' records, through this
//                                           computer's loopback API: one person,
//                                           this computer's, "me" read per group
//                                           from what svrngs answers (§13.3); or
//   {"file": "<path>", "acting": [names]}   a file of its own (store.mjs): a
//                                           simulation, one process acting for
//                                           each name, the page taking ?as=<name>
//   "data": "<dir>"                         where each person's state file is kept
//   "listen": <port>                        0, or absent, for any
//
// **The fold is the one decider** (AGENTS.md 8). Every page and every write
// reads its four answers through relay.mjs's person, and decides nothing.
// A kind writes no HTML: both forms are drawn from its fields, and what its
// `show` says is drawn as text (page.test.mjs).
//
// **The page's words** are the house's (svrn-docs VOICE.md): "carried by Jo",
// "one ring away", "sealed: only you two can read this", and on every
// conversation the labelled shortcut (§6), that whoever is in a group a
// message goes through sees that it went by, its size, which ask it answers
// and when. File mode says it is a simulation on every page.
//
// **Who is asking** (§13.1-§13.3): the person on this computer and no one
// else. It answers only at its own loopback address, to its own page, and
// takes a write only as a JSON body (`guard`); in record mode "me" is read
// per group from what svrngs answers (relay.mjs `heldBy`).
//
// Routes (§5; no /standing, §12.1), each write a JSON body:
//   GET  /            your groups' asks, your conversations, the forms
//   POST /ask         kind, fields, group, hops, until (a date)
//   POST /carry       ask, group: one tap, one carry
//   POST /reply       ask, group, fields: an offer
//   POST /yes         ask, them, name;   POST /no   ask, them
//   POST /withdraw    ask: your own
//   POST /list        kind, group, fields: a listing, never carried

import { readFileSync } from 'node:fs';
import http from 'node:http';
import { join } from 'node:path';

import { MAX_NAME, MAX_TEXT, typeOf } from './grapevine.js';
import { KINDS } from './kinds.mjs';
import { Refused, debug as trace, heldBy, heldIn, loop, person, stateFile } from './relay.mjs';
import { file, record } from './store.mjs';

const config = JSON.parse(readFileSync(process.argv[2], 'utf8'));
if (typeof config.data !== 'string') throw new Error('the config names no `data`, the folder each person\'s state file is kept in');
const store = config.file ? file(config.file) : record(config.record);
const SIM = store.kind === 'file';
if (SIM && !(Array.isArray(config.acting) && config.acting.length && config.acting.every((n) => typeof n === 'string' && n))) {
  throw new Error('a file of its own is a simulation, and its config names who it acts for: "acting": ["Sam", "Jo"]');
}
const people = SIM
  ? config.acting.map((name) => person({ id: name, store, held: heldIn(store, name), kinds: KINDS, state: stateFile(join(config.data, `${name}.state.json`)) }))
  : [person({ id: 'me', store, held: heldBy(config.record, store), kinds: KINDS, state: stateFile(join(config.data, 'state.json')) })];

const esc = (s) => String(s).replace(/[&<>"']/g, (c) => `&#${c.charCodeAt(0)};`);
const q = (s) => `“${esc(s)}”`;
const label = (name) => esc(name.charAt(0).toUpperCase() + name.slice(1).replaceAll('_', ' '));
const when = (s) => new Date(s * 1000).toLocaleString('en-GB', { weekday: 'short', day: 'numeric', month: 'short', hour: '2-digit', minute: '2-digit' });
const day = (s) => new Date(s * 1000).toLocaleDateString('en-GB', { weekday: 'short', day: 'numeric', month: 'short' });
const AWAY = ['Only this group', 'One ring away', 'Two rings away', 'Three rings away'];
/** Whose computer wrote it, when someone else's: ", through Cy's computer" (gv-via; the phone's words, web/app.js). */
const through = (via) => (via ? `, through ${esc(via)}’s computer` : '');
const names = (list) => (list.length > 1 ? `${list.slice(0, -1).join(', ')} and ${list.at(-1)}` : list[0] ?? '');
/** The labelled shortcut (GRAPEVINE_DEMO §6), said on every conversation. */
const SHORTCUT = 'Everyone in a group a message goes through can see that a sealed message went by, how big it was, which ask it answers and when. Never what it says.';

const STYLE = `body{font:16px/1.4 system-ui,sans-serif;max-width:40em;margin:1em auto;padding:0 1em;color:#1a1a1a}
h1{font-size:1.4em;margin:.2em 0}h2{font-size:1.1em;margin:1em 0 .3em}h3{font-size:1em;margin:.6em 0 .2em}
section{border-top:2px solid #1a1a1a;margin-top:1em}ul{margin:.2em 0;padding-left:1.2em}li{margin:.3em 0}
form{margin:.3em 0}label{display:inline-block;margin:.15em .6em .15em 0}input,select,button{font:inherit;padding:.2em .4em}
.quiet,.shortcut{color:#555;font-size:.92em}.unread,.said{color:#a01818}.notice{color:#145c2a;font-weight:600}
.sim{background:#fff3b0;border:2px solid #1a1a1a;padding:.4em .6em}details{margin:.2em 0}`;

/** The page's one script: each form sent as JSON, the page read again once it's written, a refusal said in place. */
const SCRIPT = `for (const f of document.querySelectorAll('form[data-to]')) f.addEventListener('submit', async (e) => {
  e.preventDefault();
  const body = { fields: {} };
  for (const el of f.elements) {
    if (!el.name) continue;
    const v = el.type === 'number' ? (el.value === '' ? null : Number(el.value)) : el.value;
    if (el.name.startsWith('f.')) body.fields[el.name.slice(2)] = v; else body[el.name] = el.name === 'hops' ? Number(v) : v;
  }
  const said = f.querySelector('.said');
  said.textContent = '';
  try {
    const res = await fetch(f.dataset.to + location.search, { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify(body) });
    const out = await res.json();
    if (res.ok) location.reload(); else said.textContent = out.said;
  } catch { said.textContent = "This page didn't answer. Is it still running?"; }
});`;

function page(who, body) {
  const sim = SIM
    ? `<p class="sim">This is a simulation: ${esc(names(config.acting))} are all on this one computer, in one file, so you can try it alone. ${
      who ? `You're ${esc(who)}. Be ${config.acting.filter((n) => n !== who).map((n) => `<a href="/?as=${encodeURIComponent(n)}">${esc(n)}</a>`).join(' or ')}.` : ''}</p>`
    : '';
  return `<!doctype html><html lang="en"><head><meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1"><title>Grapevine${who && SIM ? ` — ${esc(who)}` : ''}</title>
<style>${STYLE}</style></head><body>${sim}<h1>Grapevine</h1>${body}<script>${SCRIPT}</script></body></html>`;
}

/** One field of a kind, as an input of its type: the page's, never the kind's HTML. */
function input(name, spec) {
  const t = typeOf(spec);
  const at = `name="f.${esc(name)}" required`;
  if (t === 'date') return `<label>${label(name)} <input type="date" ${at}></label>`;
  if (t === 'number') return `<label>${label(name)} <input type="number" step="1" ${at}></label>`;
  if (t === 'choice') return `<label>${label(name)} <select ${at}>${spec.choice.map((c) => `<option>${esc(c)}</option>`).join('')}</select></label>`;
  return `<label>${label(name)} <input type="text" maxlength="${MAX_TEXT}" ${at}></label>`;
}
const fieldsOf = (spec) => Object.entries(spec).map(([n, s]) => input(n, s)).join(' ');
const hidden = (vals) => Object.entries(vals).map(([k, v]) => `<input type="hidden" name="${esc(k)}" value="${esc(v)}">`).join('');
/** A form sent as JSON by the page's script; without it, posted as a form, which the server refuses, saying so. */
const form = (to, inner, button) => `<form method="post" action="${to}" data-to="${to}">${inner} <button>${esc(button)}</button> <span class="said" role="status"></span></form>`;
const groupPick = (mine) => `<label>In <select name="group">${mine.map((g) => `<option value="${esc(g.id)}">${esc(g.name)}</option>`).join('')}</select></label>`;
const said = (fields) => Object.entries(fields ?? {}).map(([k, v]) => `${label(k).toLowerCase()} ${esc(v)}`).join('; ');

/** How an ask reads in group `g`, from its copies there: who asked, or who carried it in. */
function lineOf(a, g) {
  const carried = a.in.filter((c) => c.carried && c.group === g.id);
  const by = (c) => (c.by === g.me ? 'you' : esc(c.by) + through(c.via));
  if (a.asked && !carried.length) return `You asked: ${esc(a.show)}`;
  if (carried.length && carried.every((c) => c.by === g.me)) return `You carried: ${esc(a.show)}`;
  if (carried.length) return `A neighbour of ${esc(names(carried.filter((c) => c.by !== g.me).map((c) => c.by)))}’s needs: ${esc(a.show)} — carried by ${carried.map(by).join(' and ')}`;
  const own = a.in.find((c) => c.group === g.id);
  return `${esc(own.by)}${through(own.via)} asks: ${esc(a.show)}`;
}

/** An ask in its first group among the person's, with what they may do; in the others, how it reads there. */
function askItem(v, a, mine, g) {
  const here = a.in.filter((c) => mine.some((x) => x.id === c.group));
  const first = here[0];
  const nameOf = (id) => esc(v.groups.find((x) => x.id === id)?.name ?? id);
  if (first.group !== g.id) return `<li class="ask also" data-ask="${esc(a.id)}"><p class="line">${lineOf(a, g)}</p><p class="quiet">What you can do with it is under ${nameOf(first.group)}.</p></li>`;
  let out = `<li class="ask" data-ask="${esc(a.id)}"><p class="line">${lineOf(a, g)}</p><p class="quiet">Until ${day(a.expires)}${here.length > 1 ? `. Also in ${here.slice(1).map((c) => nameOf(c.group)).join(', ')}` : ''}.</p>`;
  if (a.notice) out += '<p class="notice">You might have this.</p>';
  const kind = KINDS[a.kind];
  if (a.asked) {
    out += form('/withdraw', hidden({ ask: a.id }), 'Take it back');
  } else if (!v.threads.some((t) => t.role === 'replier' && t.ask === a.id)) {
    out += form('/reply', hidden({ ask: a.id, group: first.group }) + fieldsOf(kind.reply.fields), 'Offer');
  }
  for (const c of v.carriable.can.filter((x) => x.ask === a.id)) {
    out += form('/carry', hidden({ ask: a.id, group: c.group }), `Carry to ${c.name}`);
    out += `<p class="quiet">Everyone in ${esc(c.name)} will see what was asked, and that you carried it; never who asked.</p>`;
  }
  const not = v.carriable.not.filter((x) => x.ask === a.id);
  if (not.length) out += `<details class="not"><summary>Where it can’t go</summary><ul>${not.map((n) => `<li>${esc(v.groups.find((g) => g.id === n.group)?.name ?? n.group)}: ${esc(n.words)}</li>`).join('')}</ul></details>`;
  return `${out}</li>`;
}

/** One conversation: its messages, what this end may say, and the shortcut said. */
function threadItem(v, t) {
  const about = t.show ? q(t.show) : 'an ask that has ended or was taken back';
  if (t.role === 'carrier') {
    const went = t.passed.length
      ? `<ul>${t.passed.map((p) => `<li>A sealed message went by toward the person who ${p.dir === 'up' ? 'asked' : 'offered'}: ${p.size} bytes, ${when(p.at)}.</li>`).join('')}</ul>`
      : '<p class="quiet">Nothing has gone by yet.</p>';
    return `<li class="thread carrier"><p>You carried ${about} from ${esc(t.from)} to ${esc(t.to)}. You pass on what’s said, and can’t read it.</p>${went}<p class="shortcut">${SHORTCUT}</p></li>`;
  }
  const them = t.names ? esc(t.names.them) : t.role === 'asker' ? 'Someone who saw it' : 'The person who asked';
  const lines = t.messages.map((m) => {
    const who = m.mine ? 'You' : them;
    const what = m.say === 'offer' ? `offered: ${said(m.fields)}${m.text ? `. “${esc(m.text)}”` : ''}` : m.say === 'yes' ? 'said yes' : m.say === 'no' ? 'said no thanks' : `said: “${esc(m.text)}”`;
    return `<li>${who} ${what}, ${when(m.at)}.</li>`;
  });
  let out = `<li class="thread ${t.role}"><p>${t.role === 'asker' ? `About your ask ${about}` : `About ${about}, which you offered to help with`}.</p><p class="quiet">Sealed: only you two can read this.</p><ul>${lines.join('')}</ul>`;
  if (t.names) out += `<p class="names">You both said yes: they’re ${esc(t.names.them)}, and they see you as ${esc(t.names.me)}.</p>`;
  if (t.role === 'replier' && t.send) {
    const copy = v.asks.find((a) => a.id === t.ask)?.in.find((c) => c.group === t.send.group && c.handle === t.send.handle);
    if (copy?.carried) out += `<p class="quiet">It goes through ${esc(copy.by)}’s computer, which passes it on while it’s on.</p>`;
  }
  if (t.send && t.live) {
    const ids = hidden({ ask: t.ask, them: t.them });
    if (!t.messages.some((m) => m.mine && m.say === 'yes')) {
      out += form('/yes', `${ids}<label>Your name, which they’ll see once you’ve both said yes <input type="text" name="name" maxlength="${MAX_NAME}" required></label>`, 'Yes');
      out += form('/no', ids, 'No thanks');
    }
  }
  return `${out}<p class="shortcut">${SHORTCUT}</p></li>`;
}

async function home(p) {
  const v = await p.view();
  const mine = v.groups.filter((g) => g.me != null && g.unread == null);
  let body = '';
  for (const g of v.groups.filter((x) => x.unread != null)) body += `<p class="unread">${esc(g.name)} can’t be read on this computer right now.</p>`;
  if (!mine.length) return page(p.id === 'me' ? null : p.id, `${body}<p>You’re not in any group on this computer.</p>`);
  for (const g of mine) {
    const here = v.asks.filter((a) => a.in.some((c) => c.group === g.id));
    body += `<section data-group="${esc(g.id)}"><h2>${esc(g.name)}</h2>`;
    if (g.complete === false) body += `<p class="unread">Some of ${esc(g.name)}’s asks may not have reached this computer yet.</p>`;
    body += here.length ? `<ul class="asks">${here.map((a) => askItem(v, a, mine, g)).join('')}</ul>` : '<p class="quiet">Nothing asked here yet.</p>';
    body += '</section>';
  }
  body += `<section><h2>Your conversations</h2>${v.threads.length ? `<ul class="threads">${v.threads.map((t) => threadItem(v, t)).join('')}</ul>` : '<p class="quiet">None yet.</p>'}</section>`;
  const week = new Date(Date.now() + 7 * 86_400_000).toISOString().slice(0, 10);
  body += '<section><h2>Ask your groups</h2><p class="quiet">Your name stays with you until you and whoever offers have both said yes.</p>';
  for (const k of Object.values(KINDS)) {
    const hops = AWAY.map((w, n) => `<option value="${n}"${n === (k.hops ?? 1) ? ' selected' : ''}>${w}</option>`).join('');
    body += `<div class="kind" data-kind="${esc(k.kind)}"><h3>${esc(k.kind)}</h3>${form('/ask', `${hidden({ kind: k.kind })}${fieldsOf(k.ask.fields)} ${groupPick(mine)} <label>How far <select name="hops">${hops}</select></label> <label>Until <input type="date" name="until" value="${week}" required></label>`, 'Ask')}</div>`;
  }
  body += '</section>';
  const listed = Object.values(KINDS).filter((k) => k.listing);
  if (listed.length) {
    body += '<section><h2>What you have</h2><p class="quiet">It stays in the group you choose and is never passed on. When someone asks for one, you’re told you might have it.</p>';
    for (const k of listed) body += `<div class="kind" data-kind="${esc(k.kind)}"><h3>${esc(k.kind)}</h3>${form('/list', `${hidden({ kind: k.kind })}${fieldsOf(k.listing.fields)} ${groupPick(mine)}`, 'List it')}</div>`;
    body += '</section>';
  }
  const odd = v.gaps.filter((x) => x.code !== 'unread');
  if (odd.length) body += `<details class="gaps"><summary>${odd.length} thing${odd.length > 1 ? 's' : ''} here couldn’t be read</summary><ul>${odd.map((x) => `<li>${esc(x.message)}</li>`).join('')}</ul></details>`;
  trace('a page', { person: p.id, groups: mine.length, asks: v.asks.length, threads: v.threads.length, gaps: odd.length });
  return page(p.id === 'me' ? null : p.id, body);
}

/** The end of the day `until` names, in unix seconds, or refused. */
function endOf(until) {
  const t = typeof until === 'string' && /^\d{4}-\d{2}-\d{2}$/.test(until) ? new Date(`${until}T23:59:59`).getTime() : NaN;
  if (!Number.isFinite(t)) throw new Refused('Say until when, as a date.');
  return Math.floor(t / 1000);
}

const WRITES = {
  ask: (p, b) => p.ask({ kind: b.kind, fields: b.fields, group: b.group, hops: b.hops, expires: endOf(b.until) }).then(() => 'Asked.'),
  carry: (p, b) => p.carry({ ask: b.ask, group: b.group }).then(() => 'Carried.'),
  reply: (p, b) => p.reply({ ask: b.ask, group: b.group, fields: b.fields }).then(() => 'Offered.'),
  yes: (p, b) => p.yes({ ask: b.ask, them: b.them, name: b.name }).then(() => 'Said yes.'),
  no: (p, b) => p.no({ ask: b.ask, them: b.them }).then(() => 'Said no thanks.'),
  withdraw: (p, b) => p.withdraw({ ask: b.ask }).then(() => 'Taken back.'),
  list: (p, b) => p.list({ kind: b.kind, group: b.group, fields: b.fields }).then(() => 'Listed.'),
};

/** Who the page is for: in file mode the ?as person, or null when it names no one here; in record mode this computer's alone. */
function whom(url) {
  const as = url.searchParams.get('as');
  if (!SIM) {
    if (as !== null) throw new Refused('This page is only ever yours: it takes no ?as.', 400);
    return people[0];
  }
  const p = people.find((x) => x.id === as) ?? null;
  if (!p) trace('no one to act for', { as: as === null ? null : 'unknown' });
  return p;
}

/** File mode's first page: who to be, or that ?as named no one here. */
const choose = (as) =>
  page(null, `<p>${as === null ? 'Choose who you are' : `${esc(as)} isn’t one of the people here. Choose who you are`}: ${config.acting.map((n) => `<a href="/?as=${encodeURIComponent(n)}">${esc(n)}</a>`).join(', ')}.</p>`);

/**
 * Who may ask (§13.1): the person's own browser, at this server's own
 * loopback address. A page on another site cannot send a JSON body without
 * a preflight, which this server never answers, and a rebinding page comes
 * under a name this server does not answer to. So a request is refused,
 * and said so, whose Host is not this server's own address, whose Origin
 * is another's, or that writes with anything but a JSON body. A request
 * with no Origin is a program on this computer, which is the person's
 * (DESIGN §8, "Localhost is the trust boundary").
 */
function guard(req) {
  const own = `127.0.0.1:${server.address().port}`;
  if (req.headers.host !== own) {
    trace('refused: another address', { host: String(req.headers.host).slice(0, 80) });
    throw new Refused(`This page answers only at its own address on this computer, http://${own}/.`, 421);
  }
  if (req.headers.origin !== undefined && req.headers.origin !== `http://${own}`) {
    trace('refused: another site', { origin: String(req.headers.origin).slice(0, 80) });
    throw new Refused('Another site tried to use this page, so nothing was done.', 403);
  }
  if (req.method === 'POST' && !/^application\/json\s*(;|$)/i.test(req.headers['content-type'] ?? '')) {
    trace('refused: not sent as this page sends it', { type: String(req.headers['content-type']).slice(0, 80) });
    throw new Refused('This page takes only what its own page sends, so nothing was done.', 415);
  }
}

async function json(req) {
  let text = '';
  for await (const chunk of req) {
    text += chunk;
    if (text.length > 128 * 1024) throw new Refused('That is more than this page takes.', 413);
  }
  try {
    const out = JSON.parse(text);
    if (out && typeof out === 'object' && !Array.isArray(out)) return out;
  } catch {}
  throw new Refused('That couldn’t be read.', 400);
}

const server = http.createServer(async (req, res) => {
  const headers = { 'cache-control': 'no-store' };
  const url = new URL(req.url, 'http://grapevine');
  const to = url.pathname.slice(1);
  try {
    guard(req);
    if (req.method === 'GET' && url.pathname === '/') {
      const p = whom(url);
      return res.writeHead(p ? 200 : url.searchParams.has('as') ? 404 : 200, { ...headers, 'content-type': 'text/html; charset=utf-8' }).end(p ? await home(p) : choose(url.searchParams.get('as')));
    }
    if (req.method === 'POST' && Object.prototype.hasOwnProperty.call(WRITES, to)) {
      const p = whom(url);
      if (!p) throw new Refused('Choose who you are first.', 400);
      const said = await WRITES[to](p, await json(req));
      trace('answered', { person: p.id, route: to, status: 200 });
      return res.writeHead(200, { ...headers, 'content-type': 'application/json' }).end(JSON.stringify({ said }));
    }
    throw new Refused('There is nothing here.', 404);
  } catch (e) {
    const status = e instanceof Refused ? e.status : 502;
    const words = e instanceof Refused ? e.message : "This page can't be made on this computer right now.";
    trace('answered', { route: to, status, refused: e instanceof Refused, error: e instanceof Refused ? null : e.message });
    if (req.method === 'POST') return res.writeHead(status, { ...headers, 'content-type': 'application/json' }).end(JSON.stringify({ said: words }));
    res.writeHead(status, { ...headers, 'content-type': 'text/html; charset=utf-8' }).end(page(null, `<p class="unread">${esc(words)}</p>`));
  }
});

loop(people);
server.listen(config.listen ?? 0, '127.0.0.1', () => {
  const { port } = server.address();
  console.log(JSON.stringify({ port, at: `http://127.0.0.1:${port}/`, store: store.kind }));
});
