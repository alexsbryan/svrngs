// Saturday's page: the board read from the group's database at this app's
// own address as two subscriptions (`GET svrngs/sql?q=`), changed by writes
// by key (`POST svrngs/sql {write}`), each signed as the person asking, whom
// `GET svrngs/acts` names. What it reads, writes and says is board.js's.
import * as b from './board.js';

const $ = (id) => document.getElementById(id);
/** Every branch, at debug. */
const trace = (what, detail = '') => console.debug(`Saturday: ${what}`, detail);

/**
 * What the page knows: who is asking, the rows each subscription last gave
 * and its state, the plans this person voted for, what it says, and the
 * claim each plan's form is changing, if any.
 */
const seen = {
  you: null,
  names: new Map(),
  link: null,
  plans: { rows: [], state: 'opening', words: '' },
  things: { rows: [], state: 'opening', words: '' },
  mine: [],
  note: '',
  changing: new Map(),
};

/** A refusal as the route says it, `{refused, words}`, or 503's unreachable; our own words only where it gave none. */
async function why(res) {
  const body = await res.json().catch(() => null);
  trace('svrngs refused', { status: res.status, body });
  const unreachable = res.status === 503 || body?.unreachable === true;
  const words =
    typeof body?.words === 'string'
      ? body.words
      : typeof body?.error === 'string'
        ? body.error
        : unreachable
          ? b.NONE
          : b.answeredWith(res.status);
  return { state: unreachable ? 'unreachable' : 'shut', words };
}

/** Who is asking, their group's names by key, and the link a member opens the board by. */
async function whoAsks() {
  const res = await fetch('svrngs/acts');
  if (!res.ok) return why(res);
  const read = await res.json();
  seen.you = read.you_key;
  seen.link = read.link ?? null;
  for (const s of read.spans ?? []) seen.names.set(s.person_key, s.person);
  trace('who asks', { you: read.you, names: seen.names.size, link: seen.link });
  return null;
}

/** How long a page waits before asking svrngs/acts again for a name it lacks. */
const NAMES_MS = 10_000;
let named = 0;

/**
 * Names asked again where a claim's person has none yet: someone who
 * joined after the page opened. `people` holds names alone, so a row's
 * person is named from `spans`, which pairs each name with its key.
 */
async function nameEveryone() {
  const unnamed = seen.things.rows.filter((r) => r.person_key && !seen.names.has(r.person_key));
  if (!unnamed.length || Date.now() - named < NAMES_MS) return;
  named = Date.now();
  trace('names asked again', unnamed.length);
  try {
    await whoAsks();
  } catch (e) {
    trace('names could not be read', String(e));
  }
}

/** One read, `{rows}`, of `sql` with `params`; throws the route's words where it refuses. */
async function read(sql, params = []) {
  const res = await fetch('svrngs/sql', {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify({ sql, params }),
  });
  if (!res.ok) throw new Error((await why(res)).words);
  return res.json();
}

/** Each `data:` of an event stream, parsed, given to `each`, until the stream ends. */
async function events(body, each) {
  const reader = body.pipeThrough(new TextDecoderStream()).getReader();
  let held = '';
  for (let r = await reader.read(); !r.done; r = await reader.read()) {
    held += r.value.replace(/\r\n?/g, '\n');
    for (let end = held.indexOf('\n\n'); end >= 0; end = held.indexOf('\n\n')) {
      const data = held
        .slice(0, end)
        .split('\n')
        .filter((line) => line.startsWith('data:'))
        .map((line) => line.slice(5).trimStart())
        .join('\n');
      held = held.slice(end + 2);
      if (data) await each(JSON.parse(data));
    }
  }
}

/** `name`'s subscription to `sql`, kept open; past its end or a fault, said, then asked again after RETRY_MS. */
async function listen(name, sql, then = async () => {}) {
  const sub = seen[name];
  try {
    const res = await fetch(`svrngs/sql?q=${encodeURIComponent(sql)}`, {
      headers: { accept: 'text/event-stream' },
    });
    if (res.ok) {
      await events(res.body, async (answer) => {
        if (!Array.isArray(answer.rows)) throw new SyntaxError('an answer with no rows');
        sub.state = answer.syncing === true ? 'syncing' : 'ready';
        if (sub.state === 'ready') {
          sub.rows = answer.rows;
          sub.more = answer.more === true;
          if (sub.more) trace(`${name}: more rows than one read gives`);
          await then();
        }
        trace(`${name} answered`, { state: sub.state, rows: answer.rows.length });
        draw();
      });
      trace(`${name} stopped answering`);
      sub.state = 'unreachable';
    } else Object.assign(sub, await why(res));
  } catch (e) {
    trace(`${name} could not be read`, String(e));
    Object.assign(
      sub,
      e instanceof TypeError ? { state: 'unreachable' } : { state: 'shut', words: b.UNREAD },
    );
  }
  draw();
  setTimeout(() => listen(name, sql, then), b.RETRY_MS);
}

/** The plans this person voted for, read again whenever the totals change. */
async function readMine() {
  try {
    seen.mine = (await read(b.MINE, [seen.you])).rows;
  } catch (e) {
    trace('your votes could not be read', String(e));
  }
}

/** `ops` written as one list; true once saved, else the route's words said, prefixed by `what`. */
async function save(what, ops) {
  trace(`saving: ${what}`, ops);
  let words = null;
  try {
    const res = await fetch('svrngs/sql', {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ write: ops }),
    });
    if (res.status === 413) {
      trace('the write is too large to take at once', await res.text());
      words = b.TOO_LONG;
    } else if (!res.ok) words = (await why(res)).words;
  } catch (e) {
    trace('the write got no answer', String(e));
    words = b.NONE;
  }
  trace(words ? 'the write was refused' : 'the write was saved', words ?? what);
  seen.note = words ? b.unsaved(what, words) : '';
  draw();
  return words === null;
}

/** The whole board's state: the worse of its two subscriptions'. */
function state() {
  for (const s of ['unreachable', 'shut', 'syncing', 'opening'])
    for (const sub of [seen.plans, seen.things]) if (sub.state === s) return { state: s, ...sub };
  return { state: 'ready' };
}

/** Each plan's element, kept across draws so a form being typed in is never redrawn. */
const shown = new Map();

/** A list redrawn only when what it shows changed, focus kept on the button its person had, if it is still there. */
function fill(list, items, key, make) {
  const now = JSON.stringify(items.map(key));
  if (list.dataset.key === now) return;
  const had = list.contains(document.activeElement) ? document.activeElement.dataset.focus : null;
  list.dataset.key = now;
  list.replaceChildren(...items.map(make));
  if (had)
    (
      list.querySelector(`[data-focus="${CSS.escape(had)}"]`) ??
      list.closest('article').querySelector('.title')
    ).focus();
}

function button(text, focus, onclick) {
  const el = Object.assign(document.createElement('button'), { type: 'button', textContent: text });
  el.dataset.focus = focus;
  el.addEventListener('click', onclick);
  return el;
}

/** A claim's own buttons, Change and Take back, which only its claimer is shown. */
const own = (plan, c) => [
  ' ',
  button(`Change “${c.what}”`, `change ${c.id}`, () => changing(plan, c)),
  ' ',
  button(`Take back “${c.what}”`, `withdraw ${c.id}`, () =>
    save(`Taking back “${c.what}”`, b.withdraw(c)),
  ),
];

/** A claim for something not on the list: who brings what, and the claimer's own buttons. */
function claimLine(plan, c) {
  const li = document.createElement('li');
  li.append(`${c.yours ? 'You' : c.who}: ${c.what}`, ...(c.yours ? own(plan, c) : []));
  return li;
}

/** A thing to bring: who brings it, I'll bring it unless you already do, your own claim's buttons, and Take it off. */
function thingLine(plan, t) {
  const li = document.createElement('li');
  li.append(t.words);
  const who = b.bringing(t.claims);
  if (who) li.append(` (${who})`);
  const mine = t.claims.filter((c) => c.yours);
  if (!mine.length)
    li.append(
      ' ',
      button(`I’ll bring ${t.words}`, `bring ${t.thing}`, () =>
        save(`“${t.words}”`, b.claim(plan.id, t.words, seen.you)),
      ),
    );
  li.append(
    ...mine.flatMap((c) => own(plan, c)),
    ' ',
    button(`Take ${t.words} off the list`, `unlist ${t.thing}`, () =>
      save(`Taking ${t.words} off`, b.unlist(plan.id, t.thing)),
    ),
  );
  return li;
}

/** A plan's claim form, set to change claim `c`, its words in the box. */
function changing(plan, c) {
  seen.changing.set(plan.id, c);
  const el = shown.get(plan.id);
  el.querySelector('[name=what]').value = c.what;
  el.querySelector('.say').textContent = 'Save the change';
  el.querySelector('.cancel').hidden = false;
  el.querySelector('[name=what]').focus();
  trace('changing a claim', c.id);
}

/** A plan's claim form back to saying a new claim. */
function claimAnew(id) {
  seen.changing.delete(id);
  const el = shown.get(id);
  el.querySelector('.claim').reset();
  el.querySelector('.say').textContent = 'Say you’ll bring it';
  el.querySelector('.cancel').hidden = true;
}

/** A new plan's element, its forms wired once. */
function planElement(id) {
  const el = $('plan').content.firstElementChild.cloneNode(true);
  const titleId = `plan-${id}`;
  el.querySelector('.title').id = titleId;
  el.querySelector('article').setAttribute('aria-labelledby', titleId);
  el.querySelector('.vote').addEventListener('click', (e) => {
    const yes = e.currentTarget.getAttribute('aria-pressed') !== 'true';
    save(yes ? 'Your vote' : 'Taking back your vote', b.vote(id, seen.you, yes));
  });
  el.querySelector('.claim').addEventListener('submit', async (e) => {
    e.preventDefault();
    const what = e.currentTarget.elements.what.value.trim();
    if (!what) return;
    const was = seen.changing.get(id);
    const ok = was
      ? await save(`The change to “${was.what}”`, b.change(was, what))
      : await save(`“${what}”`, b.claim(id, what, seen.you));
    if (ok) claimAnew(id);
  });
  el.querySelector('.cancel').addEventListener('click', () => claimAnew(id));
  el.querySelector('.paste').addEventListener('submit', async (e) => {
    e.preventDefault();
    const form = e.currentTarget;
    const n = b.lines(form.elements.list.value).length;
    if (
      n &&
      (await save(
        n === 1 ? 'The thing to bring' : `The ${n} things to bring`,
        b.paste(id, form.elements.list.value),
      ))
    )
      form.reset();
  });
  el.querySelector('.off').addEventListener('click', () =>
    save('Taking the plan off', b.takeOff(id)),
  );
  return el;
}

/** One plan drawn into its element: its words, its total, your vote, and its things and claims. */
function drawPlan(plan) {
  const el = shown.get(plan.id) ?? planElement(plan.id);
  shown.set(plan.id, el);
  el.querySelector('.title').textContent = plan.title || 'A plan with no name';
  el.querySelector('.when').textContent = [plan.at, plan.place].filter(Boolean).join(' · ');
  el.querySelector('.when').hidden = !plan.at && !plan.place;
  el.querySelector('.votes').textContent = b.total(plan.votes);
  const vote = el.querySelector('.vote');
  vote.setAttribute('aria-pressed', String(plan.yours));
  for (const named of [vote, el.querySelector('.off')])
    named.setAttribute('aria-describedby', el.querySelector('.title').id);
  el.querySelector('.nothing').hidden = plan.things.length + plan.others.length > 0;
  fill(
    el.querySelector('.things'),
    plan.things,
    (t) => [t.thing, t.words, t.claims.map((c) => [c.id, c.what, c.who, c.yours])],
    (t) => thingLine(plan, t),
  );
  fill(
    el.querySelector('.others'),
    plan.others,
    (c) => [c.id, c.what, c.who, c.yours],
    (c) => claimLine(plan, c),
  );
  return el;
}

function draw() {
  const now = state();
  const ready = now.state === 'ready' && seen.you !== null;
  const more = seen.plans.more || seen.things.more;
  $('said').textContent = ready
    ? seen.note || (more ? b.MORE : '')
    : now.state === 'shut'
      ? b.shut(now.words)
      : b.SAID[now.state];
  $('said').dataset.state = ready ? (seen.note ? 'refused' : more ? 'more' : '') : now.state;
  $('board').hidden = !ready;
  if (!ready) return;
  const plans = b.board({
    plans: seen.plans.rows,
    things: seen.things.rows,
    mine: seen.mine,
    you: seen.you,
    names: seen.names,
  });
  const els = plans.map(drawPlan);
  const ids = new Set(plans.map((p) => p.id));
  for (const [id, el] of shown)
    if (!ids.has(id)) {
      // Focus in a plan that left goes to the list's heading, not the page's top.
      if (el.contains(document.activeElement)) $('plans-heading').focus();
      el.remove();
      shown.delete(id);
    }
  // Moved only when the order changed, so a plan being typed in keeps its focus.
  const order = [...$('plans').children];
  if (els.length !== order.length || els.some((el, i) => el !== order[i]))
    $('plans').replaceChildren(...els);
  $('empty').hidden = plans.length > 0;
  $('share').hidden = !seen.link;
}

$('add').addEventListener('submit', async (e) => {
  e.preventDefault();
  const form = e.currentTarget;
  const plan = {
    title: form.elements.title.value.trim(),
    at: form.elements.at.value,
    place: form.elements.place.value.trim(),
  };
  if (!plan.title) return;
  if (await save(`“${plan.title}”`, b.addPlan(plan, seen.you))) form.reset();
});

$('copy').addEventListener('click', async () => {
  try {
    await navigator.clipboard.writeText(seen.link);
    seen.note = 'Copied. Paste it in your group’s chat.';
  } catch (e) {
    trace('the link could not be copied', String(e));
    $('link').textContent = seen.link;
    seen.note = 'Copy the link beside the button.';
  }
  draw();
});

async function start() {
  try {
    const refused = await whoAsks();
    if (refused) {
      Object.assign(seen.plans, refused);
      return draw();
    }
  } catch (e) {
    trace('who asks could not be read', String(e));
    Object.assign(seen.plans, {
      state: e instanceof TypeError ? 'unreachable' : 'shut',
      words: b.UNREAD,
    });
    return draw();
  }
  listen('plans', b.PLANS, readMine);
  listen('things', b.THINGS, nameEveryone);
}

start();
