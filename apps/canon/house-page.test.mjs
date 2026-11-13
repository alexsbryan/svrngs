// House rules' page (canon-house (c)), with no browser: the page's own
// index.js run in Node as a browser runs it, drawing into a document of
// plain objects as much as index.js asks of one (createElement, append,
// replaceChildren, getElementById, a checked input), its `svrngs/acts`
// answered as svrngs's door answers it (crates/svrngs-node/src/carried.rs
// `acts`): the asker, the group, its people and their time in it, and the
// app's entries, each new one written as the asker, through the computer
// the test names. A reload is index.js evaluated anew. Four people founded
// the house. Ana brings it, answers the question, "Most of you agree", and
// proposes "Quiet hours are 10pm to 7am"; Bo agrees through Dee's computer,
// and Ana's page says so; Cy agrees, and it reads as a rule. No page says
// canon's own words. What a browser itself does with the page, its policy,
// its module and layout, in each engine, is page.test.mjs's.

import assert from 'node:assert/strict';
import { test } from 'node:test';

/** canon's words and svrngs's own, which the page never says (fence 6). */
const NOT_SAID = /\b(scope|ratif\w*|commitments?|acts?|actors?|canon|member|key|record)\b/i;
const BLOCK = new Set(['main', 'h1', 'h2', 'h3', 'p', 'ul', 'li', 'section', 'details', 'summary']);

/** An element as index.js uses one. */
class Element {
  constructor(tag) {
    Object.assign(this, { tag, children: [], id: '', className: '', value: '', checked: false });
  }
  append(...kids) {
    this.children.push(...kids);
  }
  replaceChildren(...kids) {
    this.children = [];
    this.append(...kids);
  }
  *all() {
    for (const k of this.children)
      if (typeof k !== 'string') {
        yield k;
        yield* k.all();
      }
  }
  /** `input`, or `input[name=…]:checked`: the two index.js asks. */
  querySelector(selector) {
    const [, tag, name, checked] = selector.match(/^(\w+)(?:\[name=([\w-]+)\])?(:checked)?$/);
    return (
      [...this.all()].find(
        (e) => e.tag === tag && (!name || e.name === name) && (!checked || e.checked),
      ) ?? null
    );
  }
  click() {
    return this.onclick?.();
  }
}

/** What a reader reads of `e`, a line per block; a closed details shows its summary alone. */
function text(e) {
  const out = [];
  const walk = (x) => {
    if (typeof x === 'string') return out.push(x);
    const kids = x.tag === 'details' ? x.children.filter((k) => k.tag === 'summary') : x.children;
    if (BLOCK.has(x.tag)) out.push('\n');
    kids.forEach(walk);
    if (BLOCK.has(x.tag)) out.push('\n');
  };
  walk(e);
  return out
    .join('')
    .split('\n')
    .map((l) => l.replace(/[ \t]+/g, ' ').trim())
    .filter(Boolean)
    .join('\n');
}

/** The house's record at its own origin, `svrngs/acts` as svrngs's door answers it. */
function house(spans) {
  const ops = [];
  let n = 0;
  const at = { person: null, computer: null };
  const fetch = async (url, init = {}) => {
    assert.equal(url, 'svrngs/acts');
    if ((init.method ?? 'GET') === 'GET') {
      const people = spans.filter((s) => s.left == null).map((s) => s.person);
      return Response.json({ you: at.person, group: 'house1', people, spans, ops });
    }
    const { op } = JSON.parse(init.body);
    ops.push({
      id: `op${++n}`,
      ts_unix: Math.floor(Date.now() / 1000),
      person: at.computer ?? at.person,
      on_behalf_of: at.person,
      voided: false,
      payload: op,
    });
    return Response.json({});
  };
  return { at, fetch };
}

/** How many times a page has loaded: each load's module URL is new, so index.js runs anew. */
const loaded = { n: 0 };
const settle = async () => {
  for (let i = 0; i < 50; i++) await new Promise((r) => setImmediate(r));
};

test('a house answers the question, proposes a rule, and two others agree to it', async (t) => {
  const founders = ['ana', 'bo', 'cy', 'dee'].map((person) => ({
    person,
    joined: null,
    left: null,
  }));
  const record = house(founders);
  const traced = [];
  const was = { fetch: globalThis.fetch, document: globalThis.document, debug: console.debug };
  globalThis.fetch = record.fetch;
  console.debug = (...words) => traced.push(words.join(' '));
  t.after(() => {
    globalThis.fetch = was.fetch;
    globalThis.document = was.document;
    console.debug = was.debug;
  });
  let app;
  const said = [];
  const main = () => text(app);
  const as = async (person, computer = person) => {
    Object.assign(record.at, { person, computer });
    // A page loaded: index.html's main, then its module.
    app = new Element('main');
    app.id = 'app';
    const reading = new Element('p');
    reading.id = 'said';
    reading.append('Reading your house’s rules…');
    const h1 = new Element('h1');
    h1.append('House rules');
    app.append(h1, reading);
    globalThis.document = {
      createElement: (tag) => new Element(tag),
      getElementById: (id) =>
        app.id === id ? app : ([...app.all()].find((e) => e.id === id) ?? null),
    };
    loaded.n += 1;
    await import(`./index.js?load=${loaded.n}`);
    for (let i = 0; i < 100 && main().includes('Reading your house'); i++) await settle();
    assert.doesNotMatch(main(), /Reading your house/, `${person}'s page never read the house`);
    said.push(main());
  };
  const press = async (name) => {
    const before = main();
    const button = [...app.all()].find((e) => e.tag === 'button' && text(e) === name);
    assert.ok(button, `no ${name} on the page`);
    await button.click();
    for (let i = 0; i < 100 && main() === before; i++) await settle();
    assert.notEqual(main(), before, `${name} changed nothing`);
    said.push(main());
  };
  const list = (className) => {
    const ul = [...app.all()].find((e) => e.tag === 'ul' && e.className === className);
    assert.ok(ul, `no ul.${className}`);
    return ul;
  };

  await as('ana');
  assert.match(main(), /How does your house decide its rules\?/);
  assert.match(
    main(),
    /Most of you agree\. A rule holds once more than half of the people in your house when it was proposed agree, whoever proposed it among them\./,
  );
  [...app.all()].find((e) => e.tag === 'input' && e.name === 'way' && e.value === 'most').checked =
    true;
  await press('Make this how your house decides');
  assert.match(
    main(),
    /How your house decides\n+A rule holds once more than half of the people in your house when it was proposed agree, whoever proposed it among them\./,
  );
  document.getElementById('rule').value = 'Quiet hours are 10pm to 7am';
  await press('Propose this rule to everyone in your house');
  assert.match(text(list('proposed')), /Quiet hours are 10pm to 7am\n+You proposed it on/);

  await as('bo', 'dee');
  await press('Agree');
  await as('ana');
  assert.match(text(list('proposed')), /Agreed: bo, through dee’s computer\./);
  assert.equal(
    [...list('proposed').all()].filter((e) => e.tag === 'button').length,
    0,
    'Ana is not asked about her own proposal',
  );

  await as('cy');
  await press('Agree');
  assert.match(text(list('rules')), /^Quiet hours are 10pm to 7am \(ana proposed it on /);
  assert.match(main(), /Nothing is waiting on anyone\./);
  assert.match(main(), /Everyone in your house has a say: ana, bo, you and dee\./);
  for (const words of said) assert.doesNotMatch(words, NOT_SAID);
  // Each decision traced at debug, house.js's included (#210).
  for (const what of [
    'svrngs/acts answered',
    'asking how the house decides',
    'sent',
    'canon asked',
    'how the house decides',
  ]) {
    assert.ok(
      traced.some((l) => l.startsWith(`House rules: ${what} `)),
      `traced: ${what}`,
    );
  }
});
