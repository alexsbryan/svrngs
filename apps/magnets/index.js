// SPDX-License-Identifier: AGPL-3.0-or-later
// Magnets' page: the board read from the group's database at this app's own
// origin as a subscription (`GET svrngs/sql?q=`), and moved by one write a
// tap, by key (`POST svrngs/sql {write}`); the link a member opens it by,
// from `GET svrngs/acts`, to send it to the group. What it draws, writes and
// says is board.js's; the route's shape is the one tests/standin.mjs pins.
import {
  answeredWith,
  back,
  BOARD,
  CHANGED,
  COPIED,
  FRESH_MS,
  fresh,
  KEEP_AT,
  keep as keepIt,
  next,
  NIGHT,
  NONE,
  order,
  put,
  refused,
  restore,
  RETRY_MS,
  SAID,
  sent,
  shut,
  swap,
  UNCOPIED,
  UNREAD,
} from './board.js';

const $ = (id) => document.getElementById(id);
/** Every branch, at debug. */
const trace = (what, detail = '') => console.debug(`Magnets: ${what}`, detail);

/**
 * What the page knows: the pack, the rows last read and whether any were,
 * its state, what it says and whether the sentence changed since this viewer
 * last looked, the link to send, and the words new to this viewer, with the
 * ones they saw last.
 */
const seen = {
  pack: null,
  rows: [],
  read: false,
  state: 'opening',
  words: '',
  note: '',
  changed: false,
  link: null,
  fresh: new Set(),
  last: null,
};

// The sentence this viewer last saw, kept in this browser alone, so the page
// can say what changed since: a convenience, never the board's state, so a
// browser that keeps nothing (a private window) only never says it.
const LAST = 'magnets-last-seen';
function recalled() {
  try {
    return JSON.parse(localStorage.getItem(LAST) ?? 'null');
  } catch (e) {
    trace('what was last seen could not be read', String(e));
    return null;
  }
}
function remember(board) {
  try {
    localStorage.setItem(LAST, JSON.stringify(board.sentence.map((m) => m.id)));
  } catch (e) {
    trace('what was seen could not be kept', String(e));
  }
}

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

/**
 * One answer of the board's subscription: still syncing, or the rows to
 * draw. The words new to this viewer are the ones not in the sentence they
 * last saw: on opening, the one this browser kept, said as changed; after,
 * the one drawn before, outlined a few seconds.
 */
function answered(answer) {
  if (!Array.isArray(answer.rows)) throw new SyntaxError('an answer with no rows');
  seen.state = answer.syncing === true ? 'syncing' : 'ready';
  trace(seen.state === 'ready' ? 'the board answered' : 'the board is still syncing', answer.rows);
  if (seen.state === 'ready') {
    seen.rows = answer.rows;
    seen.read = true;
    const board = order(seen.pack, seen.rows);
    const now = board.sentence.map((m) => m.id);
    const opening = seen.last === null;
    const before = opening ? recalled() : seen.last;
    const arrived = Array.isArray(before) ? fresh(before, now) : [];
    trace(opening ? 'new since this viewer last looked' : 'new while they watch', arrived);
    seen.changed ||= opening && arrived.length > 0;
    for (const id of arrived) {
      seen.fresh.add(id);
      if (!opening) setTimeout(() => seen.fresh.delete(id) && mark(), FRESH_MS);
    }
    seen.last = now;
    remember(board);
  }
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

/** `ops` as one write, `what` the word or sentence it moves: said where it was refused. */
async function write(ops, what) {
  Object.assign(seen, { note: '', changed: false });
  seen.fresh.clear();
  // What this viewer places is never new to them.
  for (const op of ops) if (op.insert === 'magnets') seen.last?.push(op.row.id);
  let words = null;
  try {
    const res = await fetch('svrngs/sql', {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ write: ops }),
    });
    if (!res.ok) words = (await why(res)).words;
  } catch (e) {
    trace('the move got no answer', String(e));
    words = NONE;
  }
  trace(words ? 'the move was refused' : 'the move was made', words ?? what);
  seen.note = words ? refused(what, words) : '';
  draw();
}

/** A magnet tapped in the tray: onto the sentence's end. */
function add(magnet) {
  const op = put(magnet.id, next(seen.rows));
  trace('a magnet goes on the sentence', op);
  write([op], magnet.word);
}

/** A word in the sentence chosen: its moves, Take out, Earlier and Later, each named with it, in the popover. */
function choose(magnet) {
  const { sentence } = order(seen.pack, seen.rows);
  const at = sentence.findIndex((m) => m.id === magnet.id);
  const [before, after] = [sentence[at - 1], sentence[at + 1]];
  trace('a word in the sentence chosen', { id: magnet.id, at });
  $('out').textContent = `Take “${magnet.word}” out`;
  $('earlier').setAttribute('aria-label', `Move “${magnet.word}” earlier`);
  $('later').setAttribute('aria-label', `Move “${magnet.word}” later`);
  $('earlier').hidden = !before;
  $('later').hidden = !after;
  const done = (why, ops) => () => {
    $('moves').hidePopover();
    trace(why, ops);
    write(ops, magnet.word);
  };
  $('out').onclick = done('a magnet goes back to the tray', [back(magnet.id)]);
  if (before) $('earlier').onclick = done('a magnet moves earlier', swap(before, sentence[at]));
  if (after) $('later').onclick = done('a magnet moves later', swap(sentence[at], after));
}

function button(magnet, then) {
  const b = document.createElement('button');
  b.type = 'button';
  b.textContent = magnet.word;
  b.dataset.id = magnet.id;
  b.addEventListener('click', () => then(magnet));
  return b;
}

/** A word in the sentence: its button, which opens its moves. */
function placed(magnet) {
  const b = button(magnet, choose);
  b.setAttribute('popovertarget', 'moves');
  return b;
}

/**
 * Each word in the sentence new to this viewer outlined, "new" said before
 * it, and the others not, in place: a mark that goes is a removal, which the
 * sentence's live region doesn't say again.
 */
function mark() {
  for (const b of $('placed').querySelectorAll('button')) {
    const fresh = seen.fresh.has(b.dataset.id);
    const heard = b.previousElementSibling?.matches('.heard') ? b.previousElementSibling : null;
    b.classList.toggle('fresh', fresh);
    if (fresh && !heard)
      b.before(
        Object.assign(document.createElement('span'), { className: 'heard', textContent: 'new' }),
      );
    if (!fresh && heard) heard.remove();
  }
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
  } else li.append(button(magnet, add));
  return li;
}

/** A sentence kept: its words, and Put it back, named with them, which keeps the one on the fridge then. */
function kept(old) {
  const li = document.createElement('li');
  const words = Object.assign(document.createElement('span'), {
    id: `kept-${old.id}`,
    textContent: `${old.prompt} ${old.sentence.map((m) => m.word).join(' ')}`,
  });
  const b = Object.assign(document.createElement('button'), {
    type: 'button',
    id: `put-${old.id}`,
    textContent: 'Put it back',
  });
  b.setAttribute('aria-labelledby', `${b.id} ${words.id}`);
  b.addEventListener('click', () => {
    trace('a kept sentence goes back on the fridge', old.id);
    const board = order(seen.pack, seen.rows);
    write(restore(board, old, crypto.randomUUID(), Date.now()), words.textContent);
  });
  li.append(words, ' ', b);
  return li;
}

/**
 * The page as `seen` has it. The board shows once it was read; while it
 * can't be changed (syncing, shut, or no computer answering, said in the
 * fridge's goodnight look), the last sentence read stays, its words inert.
 */
function draw() {
  const { pack, state } = seen;
  const ready = state === 'ready' && pack !== null;
  const night = seen.read && state === 'unreachable';
  const note = seen.note ? 'refused' : seen.changed ? 'changed' : '';
  const says = { refused: seen.note, changed: CHANGED, '': '' };
  $('said').textContent = ready
    ? says[note]
    : night
      ? NIGHT
      : state === 'shut'
        ? shut(seen.words)
        : SAID[state];
  $('said').dataset.state = ready ? note : state;
  document.body.classList.toggle('night', night);
  $('board').hidden = !seen.read;
  $('play').hidden = $('pile').hidden = !ready;
  if (!ready) $('moves').hidePopover();
  if (!seen.read) return;
  const board = order(pack, seen.rows);
  if (board.unknown.length) trace('rows the pack doesn’t name, left out', board.unknown);
  const held = focused();
  const gone = new Set(board.sentence.map((m) => m.id));
  $('prompt').textContent = board.prompt;
  fill('placed', board.sentence, () => board.sentence.map(placed));
  mark();
  fill('tray', board.tray, () => pack.magnets.map((m) => place(m, gone.has(m.id))));
  $('blank').hidden = board.sentence.length > 0;
  $('how').classList.toggle('unseen', board.sentence.length === 0);
  // Their room kept while unseen, as the hint's, so no word below moves as they show.
  $('send').classList.toggle('unseen', board.sentence.length === 0);
  $('keep').classList.toggle('unseen', board.sentence.length < KEEP_AT);
  $('send').onclick = () => send(sent(board, seen.link));
  $('keep').onclick = () => {
    trace('the sentence is kept, and a new one starts', board.prompt);
    write(keepIt(pack, board, crypto.randomUUID(), Date.now()), sent(board, null));
  };
  $('before').hidden = board.kept.length === 0;
  fill('kept', board.kept, () => board.kept.map(kept));
  for (const b of $('board').querySelectorAll('#placed button, #kept button')) b.disabled = !ready;
  if (held && !document.activeElement?.dataset?.id) refocus(held);
}

/**
 * The sentence sent to the group: on a phone, through its share sheet; on a
 * computer, copied, said where to paste it, or shown to copy by hand.
 */
async function send(text) {
  const phone = typeof navigator.share === 'function' && matchMedia('(pointer: coarse)').matches;
  trace(phone ? 'the sentence goes to the share sheet' : 'the sentence is copied', text);
  if (phone) {
    try {
      return await navigator.share({ text });
    } catch (e) {
      trace('the share sheet closed, or could not open', String(e));
      if (e?.name === 'AbortError') return;
    }
  }
  try {
    await navigator.clipboard.writeText(text);
    $('sent').textContent = COPIED;
  } catch (e) {
    trace('the sentence could not be copied', String(e));
    $('sent').textContent = UNCOPIED(text);
  }
}

/** The link a member opens Magnets by, to send with the sentence; none where the group keeps no page for links, or acts can't be read. */
async function linked() {
  try {
    const res = await fetch('svrngs/acts');
    const link = res.ok ? (await res.json()).link : null;
    seen.link = typeof link === 'string' ? link : null;
    trace('the link to send', { status: res.status, link: seen.link });
  } catch (e) {
    trace('the link could not be read', String(e));
  }
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
  listen();
  linked();
}

start();
