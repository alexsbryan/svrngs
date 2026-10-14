// Events' own small server and page (EVENTS.md §1-§5): post an event to one
// or more of your groups, say you're coming (with guests by name), say you
// came, and see who is coming and who came. Lent by the host's computer
// under svrngs, or run alone behind a proxy that signs people in.
//
//   node apps/events/server.mjs <config.json>
//
// Listens on 127.0.0.1 and prints `{"port": N}`. No dependency. The config:
//
//   {"record": "http://127.0.0.1:<port>"}   the group's record, through this
//                                           computer's loopback API, or
//   {"file": "<path>"}                      a file of its own (store.mjs)
//   "who": "<header>"                       the header naming the person
//                                           asking: under svrngs the door's
//                                           `x-svrngs-key`; alone, whatever
//                                           the proxy sets (`remote-user`)
//   "listen": <port>                        0, or absent, for any
//
// **Which group a request is for** is the path's, `/g/<group>/…`, and the
// person asking must stand in it (`people`, from `/v1/rail/people`): never
// read from the key, which a computer shares across its groups
// (CHARTER.md). A member of one group never reads another's answers here:
// each event page folds the path's group alone, and only its host is shown
// the other groups' answers, from the groups this computer holds and the
// host stands in.
//
// **Machines build, people speak.** Every act is written as the person
// asking, for themselves; nothing is answered, sent or invited in anyone's
// name. Changing your mind corrects your own answers; nothing writes a "no".
//
// Routes:
//   GET  /                          your groups' events, and the form to post one
//   POST /post                      what, when, where, group (one per group): one
//                                   id minted here, the event posted in each
//   GET  /g/<group>/e/<event>       the event, as its group's people see it
//   POST /g/<group>/e/<event>/coming      guests: "Maya, Jo"
//   POST /g/<group>/e/<event>/not-coming  your coming taken back
//   POST /g/<group>/e/<event>/came        guest: one you brought, or none for you

import { randomBytes } from 'node:crypto';
import { readFileSync } from 'node:fs';
import http from 'node:http';

import { CAME, COMING, EVENT, actsOf, answers, guestName, hosted, init, ownActs, reduce, validate } from './events.js';
import { file, record } from './store.mjs';

const config = JSON.parse(readFileSync(process.argv[2], 'utf8'));
const store = config.file ? file(config.file) : record(config.record);
const WHO = (config.who ?? 'x-svrngs-key').toLowerCase();
// Every decision branch, at debug (AGENTS.md 1), when EVENTS_DEBUG is set.
const trace = process.env.EVENTS_DEBUG ? (what, fields) => console.error(JSON.stringify({ at: new Date().toISOString(), what, ...fields })) : () => {};

/** A refusal said to the person, in their words. */
class Said extends Error {
  constructor(status, words) {
    super(words);
    this.status = status;
  }
}

const esc = (s) => String(s).replace(/[&<>"']/g, (c) => `&#${c.charCodeAt(0)};`);
const page = (title, body) => `<!doctype html><html lang="en"><head><meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1"><title>${esc(title)}</title>
<style>body{font:17px/1.45 system-ui,sans-serif;max-width:34em;margin:1.5em auto;padding:0 1em;color:#222}
h1{font-size:1.5em;margin:.2em 0}h2{font-size:1.15em;margin:1.2em 0 .3em}h3{font-size:1em;margin:.8em 0 .2em;color:#555}
ul{margin:.2em 0;padding-left:1.2em}form{margin:.5em 0}input[type=text]{font:inherit;padding:.3em;width:100%;box-sizing:border-box}
button{font:inherit;padding:.35em .9em;margin:.2em .3em .2em 0}.quiet{color:#666}.unread{color:#a33}section{border-top:1px solid #ddd;margin-top:1em}</style>
</head><body>${body}</body></html>`;

/** The person asking, as the header the config names says. */
function asker(req) {
  const id = req.headers[WHO];
  if (typeof id !== 'string' || !id) {
    trace('refused: no one named', { header: WHO });
    throw new Said(401, 'This page opens for people signed in to it, and no one is.');
  }
  return id;
}

/** `id`'s name in `group`, or null when they do not stand in it. */
async function standing(group, id) {
  let people;
  try {
    people = await store.people(group);
  } catch (e) {
    trace('a group could not be read', { group, error: e.message });
    throw new Said(502, "This group can't be read on this computer right now.");
  }
  const me = people.find((p) => p.id === id);
  trace('standing', { group, asker: id, in: Boolean(me) });
  return me ? me.name : null;
}

/** The groups there are and each one's name in the asker's: [{id, name, me}], `me` null outside it, or `unread`. */
async function myGroups(id) {
  const out = [];
  for (const g of await store.groups()) {
    try {
      out.push({ ...g, me: await standing(g.id, id) });
    } catch (e) {
      trace('a group could not be read', { group: g.id, error: e.message });
      out.push({ ...g, me: null, unread: e.message });
    }
  }
  return out;
}

/** The group's acts, folded: its state and what they come to. */
async function folded(group) {
  const log = await store.log(group);
  const state = actsOf(log).reduce(reduce, init());
  return { state, answers: answers(state), complete: log.complete };
}

/** Write `act` as `name`'s in `group`, once the fold would read it. */
async function write(group, act, name) {
  const op = act.op === 'record' ? act.payload : act.replacement;
  const gaps = op ? validate({ id: 'new', ts: 0, by: name, op }) : [];
  if (gaps.length) throw new Said(400, gaps.map((g) => g.message).join('; '));
  try {
    const id = await store.append(group, act, name);
    trace('written', { group, by: name, act: act.op, kind: op?.kind ?? null, corrects: act.corrects ?? null, id });
    return id;
  } catch (e) {
    trace('could not write', { group, by: name, error: e.message });
    throw new Said(502, "This computer didn't save that just now. Try again in a moment.");
  }
}

const named = (groups, id) => groups.find((g) => g.id === id)?.name ?? id;

async function home(id) {
  const groups = await myGroups(id);
  const mine = groups.filter((g) => g.me);
  let body = '<h1>Events</h1>';
  for (const g of groups.filter((x) => x.unread)) body += `<p class="unread">${esc(g.name ?? g.id)} can't be read on this computer right now.</p>`;
  if (!mine.length) return page('Events', `${body}<p>You're not in any group here.</p>`);
  for (const g of mine) {
    body += `<section><h2>${esc(g.name ?? g.id)}</h2>`;
    try {
      const { events } = (await folded(g.id)).answers;
      const list = Object.entries(events).sort(([, a], [, b]) => b.at - a.at);
      body += list.length
        ? `<ul>${list.map(([e, x]) => `<li><a href="/g/${encodeURIComponent(g.id)}/e/${encodeURIComponent(e)}">${esc(x.what)}</a> <span class="quiet">${esc(x.when)}</span></li>`).join('')}</ul>`
        : '<p class="quiet">Nothing posted yet.</p>';
    } catch (e) {
      trace('a group could not be read', { group: g.id, error: e.message });
      body += `<p class="unread">This group's events can't be read on this computer right now.</p>`;
    }
    body += '</section>';
  }
  body += `<section><h2>Post an event</h2><form method="post" action="/post">
<label>What <input type="text" name="what" required maxlength="200"></label>
<label>When <input type="text" name="when" maxlength="200"></label>
<label>Where <input type="text" name="where" maxlength="200"></label>
<p>To ${mine.map((g) => `<label><input type="checkbox" name="group" value="${esc(g.id)}"> ${esc(g.name ?? g.id)}</label>`).join(' ')}</p>
<button>Post</button></form></section>`;
  return page('Events', body);
}

async function post(id, form) {
  const chosen = [...new Set(form.getAll('group'))];
  if (!chosen.length) throw new Said(400, 'Choose at least one group to post it to.');
  const names = {};
  for (const group of chosen) {
    const me = await standing(group, id);
    if (!me) throw new Said(403, "You can only post to groups you're in.");
    names[group] = me;
  }
  // One id, minted on the host's computer, for every group it is posted to.
  const event = randomBytes(8).toString('hex');
  const payload = { kind: EVENT, event, what: (form.get('what') ?? '').trim(), when: (form.get('when') ?? '').trim(), where: (form.get('where') ?? '').trim() };
  for (const group of chosen) await write(group, { op: 'record', payload }, names[group]);
  trace('posted', { event, groups: chosen });
  return `/g/${encodeURIComponent(chosen[0])}/e/${event}`;
}

/** One group's answers to `e`, said: who is coming and who came. */
function said(e) {
  const people = Object.keys(e.coming).sort();
  const coming = people.length
    ? `<ul class="coming">${people.map((p) => `<li>${esc(p)}${e.coming[p].map((g) => ` + ${esc(g)}`).join('')}</li>`).join('')}</ul>`
    : '<p class="quiet">No one has said yet.</p>';
  const guests = Object.entries(e.guests_came).flatMap(([p, gs]) => gs.map((g) => `${esc(g)}, ${esc(p)}'s guest`));
  const came = [...e.came.map(esc), ...guests];
  return `<h3>Coming</h3>${coming}<h3>Came</h3>${came.length ? `<ul class="came">${came.map((c) => `<li>${c}</li>`).join('')}</ul>` : '<p class="quiet">No one yet.</p>'}`;
}

async function eventPage(id, group, event) {
  const me = await standing(group, id);
  const groups = await myGroups(id);
  if (!me) throw new Said(403, `You're not in ${named(groups, group)}, so this event isn't yours to see.`);
  const here = await folded(group);
  const e = here.answers.events[event];
  if (!e) throw new Said(404, `This event isn't in ${named(groups, group)}.`);
  let body = `<h1>${esc(e.what)}</h1><p>${[e.when, e.where].filter(Boolean).map(esc).join(' · ')}</p><p class="quiet">${esc(e.host)} is hosting.</p>`;
  if (!here.complete) body += '<p class="unread">Some answers may not have reached this computer yet.</p>';
  if (e.host === me) {
    // The host's view: each group this computer holds where they host it.
    const all = [];
    for (const g of groups.filter((x) => x.me || x.unread)) {
      if (g.unread) {
        all.push({ group: g.id, unread: g.unread });
        continue;
      }
      try {
        all.push({ group: g.id, answers: g.id === group ? here.answers : (await folded(g.id)).answers, me: g.me });
      } catch (err) {
        all.push({ group: g.id, unread: err.message });
      }
    }
    for (const g of hosted(event, all)) {
      body += `<section data-group="${esc(g.group)}"><h2>${esc(named(groups, g.group))}</h2>`;
      body += g.unread ? `<p class="unread">This group's answers can't be read on this computer right now.</p>` : said(g.event);
      body += '</section>';
    }
    trace('the host\'s view', { event, asker: me, groups: all.map((g) => g.group) });
  } else {
    body += `<section data-group="${esc(group)}"><h2>${esc(named(groups, group))}</h2>${said(e)}</section>`;
    trace('a member\'s view', { event, asker: me, group });
  }
  const at = `/g/${encodeURIComponent(group)}/e/${encodeURIComponent(event)}`;
  const guests = e.coming[me];
  body += `<section><h2>You</h2>
<form method="post" action="${at}/coming"><label>Bringing anyone? <input type="text" name="guests" placeholder="Maya, Jo" value="${esc((guests ?? []).join(', '))}"></label>
<button>${guests ? 'Save' : "I'm coming"}</button></form>`;
  if (guests) body += `<form method="post" action="${at}/not-coming"><button>Take my name off</button></form>`;
  if (!e.came.includes(me)) body += `<form method="post" action="${at}/came"><button>I came</button></form>`;
  for (const g of (guests ?? []).filter((x) => !(e.guests_came[me] ?? []).includes(x))) {
    body += `<form method="post" action="${at}/came"><input type="hidden" name="guest" value="${esc(g)}"><button>${esc(g)} came</button></form>`;
  }
  body += `</section><p><a href="/">All events</a></p>`;
  return page(e.what, body);
}

/** Bo says he's coming, or changes who he brings: his live answers corrected into one. */
async function answer(id, group, event, verb, form) {
  const me = await standing(group, id);
  if (!me) throw new Said(403, "You're not in this group.");
  const { state, answers: here } = await folded(group);
  if (!here.events[event]) throw new Said(404, "This event isn't in this group.");
  if (verb === 'coming' || verb === 'not-coming') {
    const names = verb === 'coming' ? String(form.get('guests') ?? '').split(',').map(guestName).filter(Boolean) : null;
    const payload = names && { kind: COMING, event, ...(names.length && { guests: [...new Set(names)] }) };
    const [newest, ...older] = ownActs(state, me, event, COMING);
    trace('changing their mind', { group, event, by: me, verb, live: [newest, ...older].filter(Boolean).length });
    for (const old of older) await write(group, { op: 'correct', corrects: old, replacement: null }, me);
    if (newest) await write(group, { op: 'correct', corrects: newest, replacement: payload }, me);
    else if (payload) await write(group, { op: 'record', payload }, me);
  } else {
    const guest = form.get('guest') ? guestName(form.get('guest')) : null;
    if (ownActs(state, me, event, CAME, guest).length) trace('came already said', { group, event, by: me, guest });
    else await write(group, { op: 'record', payload: { kind: CAME, event, ...(guest && { guest }) } }, me);
  }
  return `/g/${encodeURIComponent(group)}/e/${encodeURIComponent(event)}`;
}

async function body(req) {
  let text = '';
  for await (const chunk of req) {
    text += chunk;
    if (text.length > 16_384) throw new Said(413, 'That is more than this page takes.');
  }
  return new URLSearchParams(text);
}

async function route(req) {
  let path;
  try {
    path = new URL(req.url, 'http://events').pathname.split('/').map(decodeURIComponent);
  } catch {
    throw new Said(404, 'There is nothing here.');
  }
  const id = asker(req);
  if (req.method === 'GET' && req.url === '/') return { html: await home(id) };
  if (req.method === 'POST' && path[1] === 'post' && path.length === 2) return { to: await post(id, await body(req)) };
  if (path[1] === 'g' && path[3] === 'e' && path[2] && path[4]) {
    if (req.method === 'GET' && path.length === 5) return { html: await eventPage(id, path[2], path[4]) };
    if (req.method === 'POST' && path.length === 6 && ['coming', 'not-coming', 'came'].includes(path[5])) {
      return { to: await answer(id, path[2], path[4], path[5], await body(req)) };
    }
  }
  throw new Said(404, 'There is nothing here.');
}

const server = http.createServer(async (req, res) => {
  const headers = { 'cache-control': 'no-store' };
  try {
    const out = await route(req);
    if (out.to) res.writeHead(303, { ...headers, location: out.to }).end();
    else res.writeHead(200, { ...headers, 'content-type': 'text/html; charset=utf-8' }).end(out.html);
  } catch (e) {
    const status = e instanceof Said ? e.status : 502;
    const words = e instanceof Said ? e.message : "This page can't be made on this computer right now.";
    trace('answered', { path: req.url, status, words, error: e instanceof Said ? null : e.message });
    res.writeHead(status, { ...headers, 'content-type': 'text/html; charset=utf-8' }).end(page('Events', `<p class="unread">${esc(words)}</p><p><a href="/">All events</a></p>`));
  }
});
server.listen(config.listen ?? 0, '127.0.0.1', () => console.log(JSON.stringify({ port: server.address().port, store: store.kind })));
