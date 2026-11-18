// SPDX-License-Identifier: AGPL-3.0-or-later
// Magnets' page: the board read from the group's database at this app's own
// origin as a subscription (`GET svrngs/sql?q=`), and moved by one statement
// a tap (`POST svrngs/sql`). What it draws and says is board.js's; the
// route's shape is the one tests/standin.mjs pins.
import {
  answeredWith,
  BACK,
  BOARD,
  NONE,
  order,
  PUT,
  refused,
  RETRY_MS,
  SAID,
  shut,
  UNREAD,
} from './board.js';

const $ = (id) => document.getElementById(id);
/** Every branch, at debug. */
const trace = (what, detail = '') => console.debug(`Magnets: ${what}`, detail);

/** What the page knows: the pack, the rows last read, whether it may show them, and what it says. */
const seen = { pack: null, rows: [], state: 'opening', words: '', note: '' };

/** A fault as the page says it: the network's is unreachable; any other, the board shut with `words`. */
function failed(e, what, words = UNREAD) {
  trace(what, String(e));
  Object.assign(seen, e instanceof TypeError ? { state: 'unreachable' } : { state: 'shut', words });
}

/** A refusal as the route says it, `{refused, words}`, or 503's unreachable; our own words only where it gave none. */
async function why(res) {
  const body = await res.json().catch(() => null);
  trace('svrngs/sql refused', { status: res.status, body });
  const unreachable = res.status === 503 || body?.unreachable === true;
  const words =
    typeof body?.words === 'string' ? body.words : unreachable ? NONE : answeredWith(res.status);
  return { state: unreachable ? 'unreachable' : 'shut', words };
}

/** Each `data:` of an event stream, parsed, given to `each`, until the stream ends. */
async function events(body, each) {
  const reader = body.pipeThrough(new TextDecoderStream()).getReader();
  let held = '';
  for (let read = await reader.read(); !read.done; read = await reader.read()) {
    held += read.value.replace(/\r\n?/g, '\n');
    for (let end = held.indexOf('\n\n'); end >= 0; end = held.indexOf('\n\n')) {
      const data = held
        .slice(0, end)
        .split('\n')
        .filter((line) => line.startsWith('data:'))
        .map((line) => line.slice(5).trimStart())
        .join('\n');
      held = held.slice(end + 2);
      if (data) each(JSON.parse(data));
    }
  }
}

/** One answer of the board's subscription: still syncing, or the rows to draw. */
function answered(answer) {
  if (!Array.isArray(answer.rows)) throw new SyntaxError('an answer with no rows');
  seen.state = answer.syncing === true ? 'syncing' : 'ready';
  if (seen.state === 'ready') seen.rows = answer.rows;
  trace(seen.state === 'ready' ? 'the board answered' : 'the board is still syncing', answer.rows);
  draw();
}

/** The board as a subscription; past its end or a fault, said, then read again after RETRY_MS. */
async function listen() {
  try {
    const res = await fetch(`svrngs/sql?q=${encodeURIComponent(BOARD)}`, {
      headers: { accept: 'text/event-stream' },
    });
    if (res.ok) {
      await events(res.body, answered);
      trace('the board stopped answering');
      seen.state = 'unreachable';
    } else {
      Object.assign(seen, await why(res));
    }
  } catch (e) {
    failed(e, 'the board could not be read');
  }
  draw();
  setTimeout(listen, RETRY_MS);
}

/** A magnet tapped: into the sentence's end from the tray, or back to the tray from the sentence. */
async function move(magnet) {
  const back = seen.rows.some((r) => r.id === magnet.id);
  trace(back ? 'a magnet goes back to the tray' : 'a magnet goes on the sentence', magnet.id);
  let words = null;
  try {
    const res = await fetch('svrngs/sql', {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ sql: back ? BACK : PUT, params: [magnet.id] }),
    });
    if (!res.ok) words = (await why(res)).words;
  } catch (e) {
    trace('the move got no answer', String(e));
    words = NONE;
  }
  trace(words ? 'the move was refused' : 'the move was made', words ?? magnet.id);
  seen.note = words ? refused(magnet.word, words) : '';
  draw();
}

function button(magnet) {
  const b = document.createElement('button');
  b.type = 'button';
  b.textContent = magnet.word;
  b.dataset.id = magnet.id;
  b.addEventListener('click', () => move(magnet));
  return b;
}

/** The board button with focus: its magnet, its list and its place there; null when none has it. */
function focused() {
  const b = document.activeElement;
  if (!(b instanceof HTMLButtonElement) || !b.dataset.id) return null;
  const list = b.closest('#tray') ? 'tray' : 'placed';
  return { id: b.dataset.id, list, at: [...$(list).querySelectorAll('button')].indexOf(b) };
}

/** Focus where a keyboard left it: on its magnet if it stayed in its list, else on what took its place there, else on the magnet where it went. */
function refocus({ id, list, at }) {
  const here = [...$(list).querySelectorAll('button')];
  const to =
    here.find((b) => b.dataset.id === id) ??
    here[Math.min(at, here.length - 1)] ??
    $('board').querySelector(`button[data-id="${CSS.escape(id)}"]`);
  to?.focus();
}

/** One list redrawn only when the magnets it shows changed, so a screen reader hears the sentence once a change. */
function fill(list, magnets, make) {
  const key = magnets.map((m) => m.id).join(' ');
  if ($(list).dataset.key === key) return;
  $(list).dataset.key = key;
  $(list).replaceChildren(...make());
}

/** A tray place: the magnet's button, or, while it is in the sentence, an empty outline its size that nothing reads. */
function place(magnet, gone) {
  const li = document.createElement('li');
  if (gone) {
    li.className = 'gap';
    li.setAttribute('aria-hidden', 'true');
    li.append(Object.assign(document.createElement('span'), { textContent: magnet.word }));
  } else li.append(button(magnet));
  return li;
}

function draw() {
  const { pack, state } = seen;
  const ready = state === 'ready' && pack !== null;
  $('said').textContent = ready ? seen.note : state === 'shut' ? shut(seen.words) : SAID[state];
  $('said').dataset.state = ready ? (seen.note ? 'refused' : '') : state;
  $('board').hidden = !ready;
  if (!ready) return;
  const { sentence, tray, unknown } = order(pack, seen.rows);
  if (unknown.length) trace('rows the pack doesn’t name, left out', unknown);
  const keep = focused();
  const gone = new Set(sentence.map((m) => m.id));
  fill('placed', sentence, () => sentence.map(button));
  fill('tray', tray, () => pack.magnets.map((m) => place(m, gone.has(m.id))));
  $('blank').hidden = sentence.length > 0;
  $('out').classList.toggle('unseen', sentence.length === 0);
  if (keep && !document.activeElement?.dataset?.id) refocus(keep);
}

async function start() {
  try {
    const res = await fetch('pack.json');
    if (!res.ok) throw new Error(`pack.json answered ${res.status}`);
    seen.pack = await res.json();
  } catch (e) {
    failed(e, 'the word pack could not be read', 'Its words couldn’t be read.');
    return draw();
  }
  trace('the word pack read', seen.pack.magnets.length);
  $('prompt').textContent = seen.pack.prompt;
  listen();
}

start();
