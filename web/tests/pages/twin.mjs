// The home page's twin (docs/CHECKS.md harness 5), with no browser: the
// page's own edge, crates/svrngs-node/src/home/home.js, run in Node as the
// browser runs it, its features imported by home-reading.js FILES, HOME
// started and `Home` drawn by Preact into a document of plain objects, its
// routes answered from recorded answers in the shapes this computer's node
// writes. A reader, never a decider: nothing here says what the page draws
// or which route it asks; a law reads what was drawn (`text`, `all`), what
// was asked (`asked`), and what the page tried to do outside itself
// (`opened`, `copied`, `went`), as data.
//
// The shapes each answer is recorded in, and their writers:
// - GET /v1/groups: `{ groups: Group[], key }`, surface.rs `Held` (group.js `Group`);
// - GET /v1/rail/people?namespace=: `{ people: PeopleRow[] }`, rail.rs `standing_json`;
// - GET /v1/lends?namespace=: group.js `Lends`, surface.rs `lends`, its
//   `unanswered`/`unheard`/`refused` by node.rs `Unlisted`;
// - GET /lent: `{ lent, signing, readers }`, home.rs `lent` (lent-read.js);
//   GET /reaching, home.rs `reached`; GET /update, home.rs `update_check`;
// - GET and POST /v1/room: `InRoom[]`, room/route.rs, its `said` room.rs `Served::said`;
// - a refusal: its status and `{ error }`, lend.rs `LendRefusal::say`; a
//   lapsed session's 401, session.rs `refused`.
// A route with no recorded answer is answered 404 with its name, and kept
// in `unanswered`, so a law sees what it did not record.

import { register } from 'node:module';

register('./layout.mjs', import.meta.url);

const HOME = new URL('../../../crates/svrngs-node/src/home/', import.meta.url);

// A document of plain objects, as much of one as Preact 11's diff and the
// page's edge touch: elements and words, their attributes, a `value` and
// `checked` set as properties, and listeners a law fires (`press`, `type`).
class Words {
  constructor(data) {
    Object.assign(this, { nodeType: 3, data, parentNode: null });
  }
  get nextSibling() {
    return sibling(this, 1);
  }
  get textContent() {
    return this.data;
  }
  remove() {
    this.parentNode?.removeChild(this);
  }
}
class Element {
  constructor(name, document) {
    Object.assign(this, {
      nodeType: 1,
      localName: name,
      nodeName: name.toUpperCase(),
      namespaceURI: 'http://www.w3.org/1999/xhtml',
      ownerDocument: document,
      parentNode: null,
      childNodes: [],
      attributes: {},
      listeners: {},
      style: { cssText: '', setProperty() {} },
    });
  }
  get firstChild() {
    return this.childNodes[0] ?? null;
  }
  get nextSibling() {
    return sibling(this, 1);
  }
  get textContent() {
    return this.childNodes.map((child) => child.textContent).join('');
  }
  set textContent(words) {
    while (this.firstChild) this.removeChild(this.firstChild);
    if (words) this.appendChild(new Words(words));
  }
  get id() {
    return this.attributes.id ?? '';
  }
  get className() {
    return this.attributes.class ?? '';
  }
  set className(name) {
    this.attributes.class = name;
  }
  get classList() {
    const names = () => this.className.split(' ').filter(Boolean);
    return {
      contains: (name) => names().includes(name),
      toggle: (name, on) => {
        const kept = names().filter((n) => n !== name);
        this.className = (on ? [...kept, name] : kept).join(' ');
      },
    };
  }
  get value() {
    return this.typed ?? this.attributes.value ?? '';
  }
  set value(words) {
    this.typed = String(words);
  }
  get checked() {
    return Boolean(this.ticked);
  }
  set checked(on) {
    this.ticked = Boolean(on);
  }
  // An HTML element's attributes are named in lower case (Preact sets `colSpan`).
  setAttribute(name, value) {
    this.attributes[name.toLowerCase()] = String(value);
  }
  removeAttribute(name) {
    delete this.attributes[name.toLowerCase()];
  }
  getAttribute(name) {
    return this.attributes[name] ?? null;
  }
  addEventListener(type, listener) {
    this.listeners[type.toLowerCase()] = listener;
  }
  removeEventListener(type) {
    delete this.listeners[type.toLowerCase()];
  }
  appendChild(child) {
    return this.insertBefore(child, null);
  }
  insertBefore(child, before) {
    child.parentNode?.removeChild(child);
    const at = before ? this.childNodes.indexOf(before) : -1;
    this.childNodes.splice(at < 0 ? this.childNodes.length : at, 0, child);
    child.parentNode = this;
    return child;
  }
  removeChild(child) {
    const at = this.childNodes.indexOf(child);
    if (at < 0) throw new Error(`removeChild: not a child of <${this.localName}>`);
    this.childNodes.splice(at, 1);
    child.parentNode = null;
    return child;
  }
  remove() {
    this.parentNode?.removeChild(this);
  }
}
const sibling = (node, step) => {
  const kin = node.parentNode?.childNodes ?? [];
  return kin[kin.indexOf(node) + step] ?? null;
};

// Every element under `root`, in the order a reader meets them.
const under = (root) =>
  root.childNodes.flatMap((child) => (child.nodeType === 1 ? [child, ...under(child)] : []));

// A selector's one compound, `tag#id.class[attr]`, held to an element.
const compound = (part) => {
  const tag = part.match(/^[a-z][a-z0-9]*/)?.[0];
  const ids = [...part.matchAll(/#([\w-]+)/g)].map(([, id]) => id);
  const classes = [...part.matchAll(/\.([\w-]+)/g)].map(([, name]) => name);
  const attrs = [...part.matchAll(/\[([\w-]+)(?:="?([^\]"]*)"?)?\]/g)];
  return (el) =>
    (!tag || el.localName === tag) &&
    ids.every((id) => el.id === id) &&
    classes.every((name) => el.classList.contains(name)) &&
    attrs.every(([, name, value]) =>
      value === undefined ? name in el.attributes : el.attributes[name] === value,
    );
};

// Each element under `root` a selector of compounds names, each compound
// a descendant of the one before it, or a child after `>`.
const select = (root, selector) => {
  const parts = selector
    .trim()
    .split(/\s*(>)\s*|\s+/)
    .filter(Boolean);
  let found = [root];
  for (let i = 0; i < parts.length; i++) {
    const child = parts[i] === '>';
    const holds = compound(parts[child ? ++i : i]);
    found = [
      ...new Set(
        found.flatMap((at) =>
          (child ? at.childNodes : under(at)).filter((el) => el.nodeType === 1 && holds(el)),
        ),
      ),
    ];
  }
  return found;
};

// What a person reads of an element: its words, each run of space one.
export const words = (el) => el.textContent.replace(/\s+/g, ' ').trim();

let opened = 0;

/**
 * The home page opened at `address`, answered from `answers`: each route,
 * by `METHOD path` (its query too), a recorded body, `{ status, body }`, or
 * a function of the request's body answering either, or a promise of one,
 * which a law may hold unanswered. Answers once its first reads settle.
 */
export async function open(address, answers) {
  const url = new URL(address);
  const document = {
    title: '',
    visibilityState: 'visible',
    listeners: {},
    createElement: (name) => new Element(name, document),
    createElementNS: (_space, name) => new Element(name, document),
    createTextNode: (data) => new Words(data),
    getElementById: (id) => select(document.body, `#${id}`)[0] ?? null,
    addEventListener: (type, listener) => (document.listeners[type] = listener),
  };
  document.body = new Element('body', document);
  document.body.className = 'layout-b';
  // home/index.html's #home, the one place the edge draws into.
  document.body.appendChild(document.createElement('main')).setAttribute('id', 'home');

  const page = { asked: [], unanswered: [], opened: [], copied: [], went: [], pending: 0 };
  const answer = async (method, path, body) => {
    const key = `${method} ${path}`;
    page.asked.push(body === undefined ? [method, path] : [method, path, body]);
    let given = answers[key];
    if (given === undefined) {
      page.unanswered.push(key);
      given = { status: 404, body: { error: `no recorded answer for ${key}` } };
    }
    if (typeof given === 'function') given = await given(body);
    const { status, body: out } =
      given && typeof given === 'object' && 'status' in given
        ? given
        : { status: 200, body: given };
    return {
      ok: status >= 200 && status < 300,
      status,
      json: async () => (out === undefined ? Promise.reject(new Error('no body')) : out),
    };
  };
  Object.assign(globalThis, {
    window: globalThis,
    document,
    location: {
      ...Object.fromEntries(
        ['href', 'origin', 'pathname', 'search', 'hash'].map((k) => [k, url[k]]),
      ),
      assign: (to) => page.went.push(['assign', String(to)]),
      reload: () => page.went.push(['reload']),
    },
    history: { replaceState: (_s, _t, to) => page.went.push(['replace', String(to)]) },
    navigator: {
      platform: 'MacIntel',
      clipboard: { writeText: async (text) => void page.copied.push(text) },
    },
    open: (to, target) => void page.opened.push([String(to), target]),
    requestAnimationFrame: (run) => setTimeout(run, 0),
    cancelAnimationFrame: (id) => clearTimeout(id),
    fetch: async (path, init = {}) => {
      page.pending++;
      try {
        const body = init.body === undefined ? undefined : JSON.parse(init.body);
        return await answer(init.method ?? 'GET', String(path), body);
      } finally {
        page.pending--;
      }
    },
  });
  // ui/trace.js keeps each line on `window.__lines` and says it at debug,
  // which a law reads, not its output.
  console.debug = () => {};
  // home.js and every module it imports anew for each page (layout.mjs).
  await import(new URL(`home.js?v=__VERSION__&twin=${++opened}`, HOME).href);
  const root = document.getElementById('home');
  Object.assign(page, {
    document,
    root,
    lines: () => [...(globalThis.__lines ?? [])],
    all: (selector) => select(document.body, selector),
    text: (selector = '#home') => page.all(selector).map(words),
    // The page come to rest: no request unanswered, and nothing drawn or
    // traced anew for ten turns of the loop, each of Preact's effects run
    // two turns after the frame it was drawn in (vendor/hooks.js).
    settle: async () => {
      let [last, still] = ['', 0];
      for (let turn = 0; turn < 2000 && still < 10; turn++) {
        await new Promise((resolve) => setTimeout(resolve, 0));
        const now = `${page.pending} ${globalThis.__lines?.length} ${words(document.body)}`;
        [last, still] = [now, now === last && page.pending === 0 ? still + 1 : 0];
      }
      return page;
    },
    // A button or link pressed, by its selector and the words it holds.
    press: (selector, holding = '') => {
      const el = page.all(selector).find((at) => words(at).includes(holding));
      if (!el) throw new Error(`nothing to press: ${selector} holding ${holding}`);
      // As a browser does: a link not taken by the page is followed, and a
      // button in a form sends it, which the form's address takes if the
      // page does not.
      const form = ancestor(el, 'form');
      if (fire(el, 'click')) {
        if (el.localName === 'a')
          page.went.push(['link', el.attributes.href, el.attributes.target]);
        else if (form && el.localName === 'button' && el.attributes.type !== 'button')
          if (fire(form, 'submit')) page.went.push(['submit', form.attributes.action]);
      }
      return page.settle();
    },
    // Words typed into a field, as a person types them.
    type: (selector, value) => {
      const [el] = page.all(selector);
      if (!el) throw new Error(`nothing to type in: ${selector}`);
      el.value = value;
      fire(el, 'input');
      fire(el, 'change');
      return page.settle();
    },
    // The `nth` box a selector names ticked, or unticked, as a person does.
    tick: (selector, nth = 0) => {
      const el = page.all(selector)[nth];
      el.checked = !el.checked;
      fire(el, 'change');
      fire(el, 'click');
      return page.settle();
    },
    // Hidden, then shown again, as a person comes back to the tab.
    shown: () => {
      for (const state of ['hidden', 'visible']) {
        document.visibilityState = state;
        document.listeners.visibilitychange?.({ type: 'visibilitychange' });
      }
      return page.settle();
    },
  });
  // Its features' modules load as a browser loads them, off the loop: the
  // page has started once it has drawn into #home.
  for (let waited = 0; !root.childNodes.length; waited += 5) {
    if (waited > 10_000) throw new Error('the home page drew nothing in 10 s');
    await new Promise((resolve) => setTimeout(resolve, 5));
  }
  return page.settle();
}

const ancestor = (el, name) =>
  !el ? null : el.localName === name ? el : ancestor(el.parentNode, name);

// An event as Preact's handler is handed it, carried up to each element
// that listens for it, as a browser carries it: whether nothing prevented
// what the browser does next.
function fire(el, type) {
  const event = {
    type,
    target: el,
    defaultPrevented: false,
    preventDefault() {
      event.defaultPrevented = true;
    },
    stopPropagation() {},
  };
  for (let at = el; at; at = at.parentNode) {
    const listener = at.listeners?.[type];
    if (listener) listener.call(at, Object.assign(event, { currentTarget: at }));
  }
  return !event.defaultPrevented;
}
