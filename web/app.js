// ported from commonwealth-ai sovereign/apps/ring-runtime/web/app.js@e1a43bcd2
// The page's logic. External (not inline) so a strict CSP can run here
// — see index.html's `script-src 'self' 'wasm-unsafe-eval'`: the wasm module
// needs `wasm-unsafe-eval`, and nothing else needs an allowance.
//
// The version query on the wasm import is stamped at build time (the content
// hash of the module), so a service worker or CDN cache cannot serve a stale
// runtime after a rebuild.
//
// Joining (u4-5; DESIGN §3, "Joining"): a group's link, `<origin>#<token>`,
// opens this page. A link this phone joined by, or the page with no link,
// is in at once: the group kept last opened is listed. A new link is asked
// of its issuer — which group it lets into — and a group this phone is in
// already, by another link, is in too. Otherwise the page asks the
// person's name once, makes the person's key, redeems the link at its
// issuer as a browser, and keeps the key sealed and the group's name
// (store.js). Every word a person reads here is DESIGN §1's.
import init, { ask, believe, connect, connect_to, connect_to_join, redemption } from './wasm/svrngs_web.js?v=__VERSION__';
import * as store from './store.js?v=__VERSION__';

const heading = document.getElementById('status');
const said = document.getElementById('response');
const list = document.getElementById('lent');
const form = document.getElementById('join');
// The dial's layers, and the page's own steps, go to the console at debug,
// never onto the page; the lines are kept on `window.__lines` for a test to
// read. Upstream printed them in a "Layers" panel (index.html:42).
const lines = [];
window.__lines = lines;
window.__trace = (l) => {
  lines.push(l);
  console.debug(l);
};
const trace = window.__trace;

// How long a computer has to answer a dial before the page says it did not.
const PATIENCE = 60_000;

// The people's lends, asked of the computer the page dials (u4-5;
// crates/svrngs-node/src/node.rs, PEOPLE_LENDS_ROUTE).
const PEOPLE_LENDS = '/people/lends';

// Why the page stopped, in a person's words.
class Said extends Error {}

function show(words, good) {
  heading.textContent = words;
  heading.className = good ? 'good' : 'bad';
}

async function reach(dialling, away) {
  const timeout = new Promise((_, no) => setTimeout(() => no(new Error('no answer')), PATIENCE));
  try {
    return await Promise.race([dialling, timeout]);
  } catch (e) {
    trace(`PAGE: the dial failed: ${e}`);
    throw new Said(away);
  }
}

// POST `body`, JSON, on `dialled`: the answer's text, or the computer's
// refusal in its words.
async function post(dialled, path, body) {
  const bytes = new TextEncoder().encode(body);
  const res = await dialled.fetch(
    'POST',
    path,
    [['content-type', 'application/json'], ['content-length', String(bytes.length)]],
    new Blob([bytes]).stream(),
  );
  const text = await res.text();
  if (res.ok) return text;
  trace(`PAGE: ${path} answered ${res.status}: ${text}`);
  if (res.status === 403) throw new Said('This link lets nobody in. Ask for a new one.');
  throw new Said('The computer that sent this link cannot let anyone in just now. Try again later.');
}

// The person's name, asked once, in the group's name.
function askName(group, why) {
  trace(`PAGE: asking the person's name to join ${group}`);
  document.getElementById('group').textContent = group;
  document.getElementById('why').textContent = why ?? '';
  heading.textContent = `Join ${group}`;
  heading.className = '';
  form.hidden = false;
  const input = document.getElementById('name');
  input.focus();
  return new Promise((done) => {
    form.onsubmit = (e) => {
      e.preventDefault();
      form.hidden = true;
      done(input.value);
    };
  });
}

// Join by `token`, a link no group kept was joined by: the group joined,
// kept, or one kept already, which the link lets into.
async function join(token, kept) {
  const away = 'The computer that sent this link did not answer. Try again later.';
  const key = crypto.getRandomValues(new Uint8Array(32));
  heading.textContent = 'Opening your link…';
  let issuer = await reach(connect_to_join(key, token), away);
  const asked = await post(issuer, '/ask', ask(key, token));
  await issuer.close();
  const group = believe(asked);
  const known = kept.find((g) => g.id === group.id);
  if (known) {
    trace(`PAGE: this phone is in ${group.id} already, by another link`);
    return known;
  }
  let why;
  for (;;) {
    const name = await askName(group.name, why);
    let body;
    try {
      body = redemption(key, token, asked, name);
    } catch (e) {
      why = `${e.message[0].toUpperCase()}${e.message.slice(1)}.`;
      continue;
    }
    heading.textContent = `Joining ${group.name}…`;
    issuer = await reach(connect_to_join(key, token), away);
    await post(issuer, '/redeem', body);
    await issuer.close();
    const person = JSON.parse(body).person;
    await store.keep({ id: group.id, name: group.name, person, token, key });
    trace(`PAGE: joined ${group.id} as ${person}; the key is kept sealed`);
    return (await store.groups()).find((g) => g.id === group.id);
  }
}

// What the group's people lend, as the bare command prints it: "Sam's
// <name>", and "<name> (<kind>)" when the two differ.
function what(l) {
  return `${l.person}’s ${l.name === l.kind ? l.name : `${l.name} (${l.kind})`}`;
}

// Open a lend: dial its lender's computer by its key, never through the
// computer that listed it (DESIGN §3, call 4), and ask for its front page.
// Rendering it is the runtime's (u4-6).
async function open(group, key, l) {
  said.textContent = `Opening ${what(l)}…`;
  try {
    const d = await reach(connect_to(key, group.token, l.key), `${l.person}’s computer did not answer.`);
    const res = await d.fetch('GET', `/lend/${l.name}/`, [], null);
    await res.body?.cancel();
    await d.close();
    said.textContent = res.ok ? `${what(l)} answered.` : `${what(l)} did not open.`;
  } catch (e) {
    said.textContent = e instanceof Said ? e.message : `${what(l)} did not open.`;
  }
}

// In `group`: list what its people lend.
async function enter(group) {
  await store.opened(group);
  document.title = group.name;
  const key = await store.unseal(group);
  heading.textContent = `Reaching ${group.name}…`;
  const d = await reach(connect(key, group.token), `Your group’s computer did not answer. Try again later.`);
  const res = await d.fetch('GET', PEOPLE_LENDS, [['accept', 'application/json']], null);
  if (!res.ok) {
    trace(`PAGE: ${PEOPLE_LENDS} answered ${res.status}`);
    await d.close();
    throw new Said('Your group’s computer said no.');
  }
  const { lent, unanswered } = await res.json();
  await d.close();
  show(`You’re in ${group.name}, as ${group.person}`, true);
  list.replaceChildren(
    ...lent.map((l) => {
      const item = document.createElement('li');
      const button = document.createElement('button');
      button.textContent = what(l);
      button.onclick = () => open(group, key, l);
      item.append(button);
      return item;
    }),
  );
  said.textContent = [
    lent.length ? '' : `Nobody in ${group.name} lends anything yet.`,
    ...unanswered.map((p) => `${p}’s computer did not answer, so what they lend is not listed.`),
  ]
    .filter(Boolean)
    .join('\n');
}

// Cache the app shell after first load, so a repeat visit needs no origin
// traffic at all (upstream's docs/RING_APP_LIBRARY.md: "no traffic after
// first load").
// Registered after load and best-effort: the page works without it, and a
// failed registration must never break the dial. Asked for before joining
// awaits anyone, so a person typing their name does not miss the load.
if ('serviceWorker' in navigator) {
  window.addEventListener('load', () => {
    navigator.serviceWorker.register('./sw.js').catch(() => {});
  });
}

// The link's fragment is its token, `<origin>#<token>`, which svrngs-core
// parses as the command does (DESIGN §3, call 5). The fragment is never
// sent to this page's origin, which is why the link rides there and not in
// the query.
const token = location.hash.slice(1);
try {
  await init();
  const kept = await store.groups();
  let group = token ? kept.find((g) => g.token === token) : kept[0];
  if (token && !group) group = await join(token, kept);
  if (!group) throw new Said('Open this page from your group’s link.');
  await enter(group);
} catch (e) {
  console.error(e);
  show(e instanceof Said ? e.message : 'This browser could not open the page.', false);
}
