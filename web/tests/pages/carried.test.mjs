// Carried apps' own pages, with no browser: each app's index.js run in
// Node over its index.html's body, read as served, in a document of plain
// objects, its requests at its own origin answered as a computer's door
// answers a carried app's page: `svrngs/acts` as carried.rs `acts` writes
// it (`you`, `you_key`, `people`, `spans` as svrngs-core membership.rs
// `Span`, `ops` as admit.rs `AdmittedOp`, its kind's prefix taken off), and
// `svrngs-reads` as surface.rs `reads` tells a carried label (each lend at
// `<origin>svrngs-lend/<label>/`, opened by `<origin>svrngs-to?…`), the
// lends' own answers as the stand-ins of browser/sends.rs gave them. A law
// reads what the page drew and what it asked. Each holds a claim the
// browser step held in Chromium and WebKit, named beside it; that a
// carried page loads its own files under the carried policy stays there
// (browser/carried.rs `a_font_in_the_folder_loads`,
// `a_carried_page_reaches_no_other_host`).
//
// Plants, each watched failing in an app's own module, then put back:
// named in the commit that brought these laws.

import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { test } from 'node:test';

const APPS = new URL('../../../apps/', import.meta.url);

// A document of plain objects, as much of one as the apps' pages touch:
// elements and words, attributes, `dataset`, `hidden`, properties set by
// `Object.assign`, and listeners a law fires (`press`, `change`).
class Words {
  constructor(data) {
    Object.assign(this, { nodeType: 3, data, parentNode: null });
  }
  get textContent() {
    return this.data;
  }
}
class Element {
  constructor(name) {
    Object.assign(this, {
      nodeType: 1,
      localName: name,
      tagName: name.toUpperCase(),
      parentNode: null,
      childNodes: [],
      attributes: {},
      dataset: {},
      listeners: {},
    });
  }
  get id() {
    return this.attributes.id ?? '';
  }
  set id(id) {
    this.attributes.id = String(id);
  }
  get className() {
    return this.attributes.class ?? '';
  }
  set className(name) {
    this.attributes.class = name;
  }
  get hidden() {
    return 'hidden' in this.attributes;
  }
  set hidden(on) {
    if (on) this.attributes.hidden = '';
    else delete this.attributes.hidden;
  }
  get textContent() {
    return this.childNodes.map((child) => child.textContent).join('');
  }
  set textContent(words) {
    this.replaceChildren(...(words ? [words] : []));
  }
  setAttribute(name, value) {
    this.attributes[name] = String(value);
  }
  getAttribute(name) {
    return this.attributes[name] ?? null;
  }
  addEventListener(type, listener) {
    (this.listeners[type] ??= []).push(listener);
  }
  append(...kids) {
    for (const kid of kids) {
      const node = typeof kid === 'string' ? new Words(kid) : kid;
      node.parentNode?.childNodes.splice(node.parentNode.childNodes.indexOf(node), 1);
      node.parentNode = this;
      this.childNodes.push(node);
    }
  }
  replaceChildren(...kids) {
    for (const child of this.childNodes) child.parentNode = null;
    this.childNodes = [];
    this.append(...kids);
  }
  // Each element a selector of compounds names, each a descendant of the
  // one before it.
  querySelectorAll(selector) {
    let found = [this];
    for (const part of selector.trim().split(/\s+/))
      found = [...new Set(found.flatMap((at) => under(at).filter(matches(part))))];
    return found;
  }
  querySelector(selector) {
    return this.querySelectorAll(selector)[0] ?? null;
  }
}

// Every element under `root`, in the order a reader meets them.
const under = (root) =>
  root.childNodes.flatMap((child) => (child.nodeType === 1 ? [child, ...under(child)] : []));

// One compound, `tag#id.class[attr="value"]`.
const matches = (selector) => {
  const tag = selector.match(/^[a-z][a-z0-9]*/)?.[0];
  const id = selector.match(/#([\w-]+)/)?.[1];
  const classes = [...selector.matchAll(/\.([\w-]+)/g)].map(([, name]) => name);
  const attrs = [...selector.matchAll(/\[([\w-]+)(?:="([^"]*)")?\]/g)];
  const value = (el, name) =>
    name.startsWith('data-')
      ? el.dataset[name.slice(5).replace(/-(\w)/g, (_, c) => c.toUpperCase())]
      : el.attributes[name];
  return (el) =>
    (!tag || el.localName === tag) &&
    (!id || el.id === id) &&
    classes.every((name) => el.className.split(' ').includes(name)) &&
    attrs.every(([, name, want]) =>
      want === undefined ? value(el, name) !== undefined : value(el, name) === want,
    );
};

const VOID = new Set(['input', 'meta', 'link', 'br', 'img']);

// `index.html`'s body as its elements, as a browser builds them.
function body(html) {
  const root = new Element('body');
  const inner = /<body>([^]*)<\/body>/.exec(html)?.[1];
  assert.ok(inner !== undefined, 'the page has a body');
  let at = root;
  for (const [, close, name, attrs, text] of inner.matchAll(
    /<(\/?)([a-z][\w-]*)([^>]*)>|([^<]+)/g,
  )) {
    if (text !== undefined) {
      if (text.trim()) at.append(text);
    } else if (close) at = at.parentNode;
    else {
      const el = new Element(name);
      for (const [, key, quoted, bare] of attrs.matchAll(
        /([\w-]+)(?:=(?:"([^"]*)"|([^\s"]+)))?/g,
      )) {
        const value = quoted ?? bare ?? '';
        if (key.startsWith('data-')) el.dataset[key.slice(5)] = value;
        else el.attributes[key] = value;
      }
      at.append(el);
      if (!VOID.has(name)) at = el;
    }
  }
  return root;
}

// What a person reads of an element: its words, each run of space one.
const words = (el) => el.textContent.replace(/\s+/g, ' ').trim();

let loads = 0;

/**
 * The carried app in `apps/<dir>/` opened at `origin`: its index.html's
 * body, its index.js run as its page runs it, each request answered by
 * `answer(method, url, body)`, a Response; `kept`, its localStorage, kept
 * across opens as a browser keeps it. Answers once it has come to rest.
 */
async function open(dir, origin, answer, kept = new Map()) {
  const page = { asked: [], pending: 0 };
  const document = { body: body(readFileSync(new URL(`${dir}/index.html`, APPS), 'utf8')) };
  document.getElementById = (id) => document.body.querySelector(`#${id}`);
  document.createElement = (name) => new Element(name);
  const window = { addEventListener: () => {} };
  const interval = globalThis.setInterval;
  Object.assign(globalThis, {
    document,
    window,
    localStorage: {
      getItem: (key) => kept.get(key) ?? null,
      setItem: (key, value) => kept.set(key, String(value)),
      removeItem: (key) => kept.delete(key),
    },
    fetch: async (path, init = {}) => {
      const url = new URL(path, origin);
      const method = init.method ?? 'GET';
      const sent = init.body === undefined ? undefined : JSON.parse(init.body);
      page.asked.push(sent === undefined ? [method, url.href] : [method, url.href, sent]);
      page.pending++;
      try {
        return await answer(method, url, sent);
      } finally {
        page.pending--;
      }
    },
    // Money looks again every 5 s (apps/expenses/index.js, its last line),
    // which a law does by opening the page again, so none is kept running.
    setInterval: () => 0,
  });
  try {
    await import(new URL(`${dir}/index.js?page=${++loads}`, APPS).href);
  } finally {
    globalThis.setInterval = interval;
  }
  Object.assign(page, {
    document,
    all: (selector) => document.body.querySelectorAll(selector),
    text: (selector) => page.all(selector).map(words),
    said: () => words(document.body),
    // The page come to rest: no request unanswered, and nothing drawn anew
    // for ten turns of the loop.
    settle: async () => {
      let [last, still] = ['', 0];
      for (let turn = 0; turn < 2000 && still < 10; turn++) {
        await new Promise((resolve) => setTimeout(resolve, 0));
        const now = `${page.pending} ${page.said()}`;
        [last, still] = [now, now === last && page.pending === 0 ? still + 1 : 0];
      }
      return page;
    },
    // A field filled, as Playwright's `fill` fills it.
    fill: (selector, value) => {
      const [el] = page.all(selector);
      assert.ok(el, `nothing to fill: ${selector}`);
      el.value = value;
    },
    // A button or link pressed, by its selector and the words it holds.
    press: (selector, holding = '') => {
      const el = page.all(selector).find((at) => words(at).includes(holding));
      assert.ok(el, `nothing to press: ${selector} holding ${holding}`);
      const event = { type: 'click', target: el, preventDefault() {} };
      el.onclick?.(event);
      for (const listener of el.listeners.click ?? []) listener(event);
      return page.settle();
    },
    // A file chosen in an `<input type=file>`.
    choose: (selector, text) => {
      const [el] = page.all(selector);
      el.files = [{ text: async () => text }];
      for (const listener of el.listeners.change ?? []) listener({ type: 'change', target: el });
      return page.settle();
    },
  });
  return page.settle();
}

// Ada's computer, in house alone, which she started: the app's route,
// `svrngs/acts`, as carried.rs `acts` answers it, each write recorded as
// her own computer's act and read back with its kind's prefix taken off.
const ADA = 'a1'.repeat(32);
function acts() {
  const ops = [];
  return async (method, url, sent) => {
    assert.equal(url.pathname, '/svrngs/acts', `a carried page asks its own route: ${url}`);
    if (method === 'POST') {
      const seq = ops.length;
      const id = `op-${seq}`;
      const payload = sent.op ?? undefined;
      ops.push({
        id,
        actor: ADA,
        person: 'ada',
        person_key: ADA,
        seq,
        ts_unix: 1_792_000_000 + seq,
        voided: false,
        payload,
      });
      return Response.json({ id, counted: true });
    }
    const spans = [{ person: 'ada', joined: null, left: null, person_key: ADA }];
    return Response.json({
      you: 'ada',
      you_key: ADA,
      group: 'house',
      page: null,
      people: ['ada'],
      spans,
      ops,
    });
  };
}

// browser::carried::money_and_events_open_and_write_at_their_origin, Money:
// carried from apps/expenses, ada adds the gas bill, 42, and it is listed as
// hers. That a carried app's file is served at its origin and its act
// written and read back as its person's is tests/carried.rs
// an_app_one_computer_carried_opens_and_writes_on_another_with_it_stopped.
test('Money, carried, adds what ada paid and lists it as hers', async () => {
  const page = await open('expenses', 'http://money0lbl.localhost:4000/', acts());
  assert.ok(page.said().includes('Nobody has added anything yet.'), page.said());
  page.fill('#what', 'the gas bill');
  page.fill('#amount', '42');
  await page.press('#add');
  assert.ok(
    page.said().includes('You paid $42.00 for the gas bill, split between ada.'),
    page.said(),
  );
  assert.deepEqual(
    page.asked.filter(([method]) => method === 'POST'),
    [
      [
        'POST',
        'http://money0lbl.localhost:4000/svrngs/acts',
        { op: { kind: 'expense', amount_cents: 4200, description: 'the gas bill' } },
      ],
    ],
  );
});

// browser::carried::money_and_events_open_and_write_at_their_origin, Events:
// carried from apps/events, ada posts a party, Saturday, at Elm Street,
// hosts it, and says she's coming.
test('Events, carried, posts ada’s party as hers and says she’s coming', async () => {
  const page = await open('events', 'http://events0lbl.localhost:4000/', acts());
  page.fill('#what', 'a party');
  page.fill('#when', 'Saturday');
  page.fill('#where', 'Elm Street');
  await page.press('#post');
  assert.ok(page.said().includes('Saturday, at Elm Street. You host it.'), page.said());
  assert.ok(page.said().includes('Nobody has said they’re coming yet.'), page.said());
  await page.press('button', 'Say you’re coming');
  assert.ok(page.said().includes('Coming: you.'), page.said());
  const posted = page.asked.filter(([method]) => method === 'POST').map(([, , sent]) => sent.op);
  assert.deepEqual(
    posted.map(({ kind, what, when, where }) => ({ kind, what, when, where })),
    [
      { kind: 'event', what: 'a party', when: 'Saturday', where: 'Elm Street' },
      { kind: 'coming', what: undefined, when: undefined, where: undefined },
    ],
  );
  assert.equal(posted[1].event, posted[0].event, 'coming to the party posted');
});

// Taste's origin on ada's computer, and Sam's two lends as surface.rs
// `reads` tells a carried page them: each at an address under its own
// origin, and `to`, svrngs's opener of the lender's own page.
const TASTE = 'http://taste0lbl.localhost:4000/';
const SAM = 'b2'.repeat(32);
const lent = (kind, name) => ({
  kind,
  name,
  lender: 'Sam',
  address: `${TASTE}svrngs-lend/${name}0lbl/`,
  to: `${TASTE}svrngs-to?lender=${SAM}&lend=${name}`,
});
// Three films, as Jellyfin's `/Items` lists a library's (browser/sends.rs
// `FILMS`): two science fiction, one not.
const FILMS = {
  Items: [
    {
      Id: '1',
      Name: 'Alien',
      ProductionYear: 1979,
      Genres: ['Science Fiction', 'Horror'],
      Overview: 'A crew answers a distress call.',
    },
    {
      Id: '2',
      Name: 'Aliens',
      ProductionYear: 1986,
      Genres: ['Science Fiction', 'Action'],
      Overview: 'Ripley goes back.',
    },
    {
      Id: '3',
      Name: 'Amélie',
      ProductionYear: 2001,
      Genres: ['Comedy', 'Romance'],
      Overview: 'A waitress helps the people around her.',
    },
  ],
};
// Taste's door: what it reads and sends to, Sam's Jellyfin, and a
// llama-server's `/v1/embeddings` giving each text its fixed numbers in
// order, science fiction one way and anything else the other
// (browser/sends.rs `embeddings`). What each lend was asked, by its name.
function tasting() {
  const asked = { jellyfin: [], 'llama-server': [] };
  const answer = async (method, url, sent) => {
    if (url.pathname === '/svrngs-reads')
      return Response.json({
        lends: [lent('jellyfin', 'jellyfin'), lent('llama-server', 'llama-server')],
        unanswered: [],
      });
    const [, name, path] = /^\/svrngs-lend\/(.+?)0lbl(\/.*)$/.exec(url.pathname) ?? [];
    assert.ok(name in asked, `Taste asks only what it was told: ${url}`);
    asked[name].push([method, `${path}${url.search}`, sent]);
    if (name === 'jellyfin') return Response.json(FILMS);
    const data = sent.input.map((text, index) => ({
      index,
      embedding: text.includes('Science Fiction') ? [1, 0] : [0, 1],
    }));
    return Response.json({ data });
  };
  return { answer, asked };
}

// browser::sends::a_carried_taste_sends_and_draws_what_comes_back, its
// page: Taste reads Sam's films at its own origin, sends them, described,
// in one POST to his llama-server's `/v1/embeddings`, and lists them; a
// Letterboxd file that loves Alien and rates Heat keeps two ratings; one
// rated of three guesses nothing yet; Amélie rated "Not for me", it guesses
// Aliens because ada loved Alien, to watch by svrngs's `to`; back from
// watching, it asks how Aliens was and keeps the answer. One POST each
// time it opens, its `input` the three films in Jellyfin's order; Jellyfin
// only read.
test('Taste sends its films once each time it opens and draws its guesses from what comes back', async () => {
  const { answer, asked } = tasting();
  const kept = new Map();
  const page = await open('taste', TASTE, answer, kept);
  assert.equal(page.document.body.dataset.ready, 'yes');
  assert.equal(
    page.text('#said')[0],
    'Sam’s llama-server described 3 films. Rate a few you’ve seen.',
  );
  const titles = (selector) => page.all(selector).map((li) => li.dataset.title);
  assert.deepEqual(titles('li[data-title]'), ['Alien', 'Aliens', 'Amélie']);
  const csv =
    'Date,Name,Year,Letterboxd URI,Rating\n' +
    '2026-11-01,Alien,1979,https://boxd.it/a,5\n' +
    '2026-11-02,Heat,1995,https://boxd.it/h,4\n' +
    '2026-11-03,Brazil,1985,https://boxd.it/b,\n';
  await page.choose('#file', csv);
  assert.equal(page.text('#brought')[0], 'Kept 2 ratings from Letterboxd.');
  assert.equal(page.text('#unguessed h2')[0], 'First, rate 2 films you’ve seen (1 of 2 so far)');
  await page.press('li[data-title="Amélie"] button', 'Not for me');
  assert.deepEqual(titles('#likely li'), ['Aliens']);
  assert.deepEqual(page.text('#likely .because'), ['Because you loved Alien']);
  const watch = page.all('#likely a.watch')[0];
  assert.ok(
    watch.href.includes('/svrngs-to?') && watch.href.endsWith('#/details?id=2'),
    watch.href,
  );
  assert.equal(
    page.text('#guessed')[0],
    'Guesses use Sam’s llama-server. It gets the list of films in your friends’ libraries, never what you rated. You also rated 1 film no library here has.',
  );
  await page.press('#likely a.watch');

  const again = await open('taste', TASTE, answer, kept);
  assert.equal(again.text('#after h2')[0], 'How was Aliens?');
  await again.press('#after button', 'Liked it');
  assert.deepEqual(
    again.all('#rated li').map((li) => li.dataset.title),
    ['Alien', 'Aliens', 'Amélie'],
  );
  const sends = asked['llama-server'];
  assert.deepEqual(
    sends.map(([method, path]) => [method, path]),
    [
      ['POST', '/v1/embeddings'],
      ['POST', '/v1/embeddings'],
    ],
    'one send each time it opens, to the path its sends: row names',
  );
  const input = sends[0][2].input;
  assert.equal(input.length, 3);
  for (const [i, film] of ['Alien (1979).', 'Aliens (1986).', 'Amélie (2001).'].entries())
    assert.ok(input[i].startsWith(film), `${film}: ${input[i]}`);
  assert.ok(
    asked.jellyfin.every(([method, path]) => method === 'GET' && path.startsWith('/Items?')),
    JSON.stringify(asked.jellyfin),
  );
});
