// Grapevine's page through GRAPEVINE_DEMO §10's scene (gv-page (b)), with no
// browser: the server's handler (server.mjs `grapevine`) asked in this
// process on a copy of demo.json, a simulation acting for Sam, Jo and Kim,
// each played by `?as=`. Each form is sent as the page's own script sends it
// (server.mjs SCRIPT: its fields as JSON, `f.` ones under `fields`, `hops` a
// number), and a loop's turn is each person's pass, as relay.mjs `loop` runs
// them. Sam asks in 1400 Page for a tile saw, one ring away; Jo carries it to
// Waller block; Kim, who listed one, is told she might have it and offers;
// Sam reads the offer, which Jo's page says went by and never what it said;
// both say yes and each sees the other's name; Sam takes it back, and the
// carry is gone from Kim's page within one loop. Every page says it is a
// simulation, every conversation says the shortcut (§6), no page says a word
// a reader must look up, and what a kind's `show` says is drawn as text,
// never as HTML. That the page's script sends a form and reloads in a
// browser is page.test.mjs's.

import assert from 'node:assert/strict';
import { copyFileSync, mkdtempSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { Readable } from 'node:stream';
import { test } from 'node:test';

/** Words the page never says (the queue's fence 9). */
const NOT_SAID = /\b(keys?|members?|records?|acts?|hash(es)?|handles?|hops?|seal)\b/i;
/** What only Sam and Kim may ever read. */
const SEALED = ['Waller and Pierce', 'Sam Reyes', 'Kim Okafor'];
const OWN = '127.0.0.1:4100';
const VOID = new Set(['meta', 'input', 'br', 'img', 'link', 'hr']);

const decode = (s) =>
  s
    .replace(/&#(\d+);/g, (_, n) => String.fromCharCode(Number(n)))
    .replace(/&quot;/g, '"')
    .replace(/&lt;/g, '<')
    .replace(/&gt;/g, '>')
    .replace(/&amp;/g, '&');

/** The page's markup as a tree of {tag, attrs, children} and strings: a reader of it, as the server writes it. */
function parse(html) {
  const root = { tag: '#root', attrs: {}, children: [] };
  const open = [root];
  const body = html.replace(/<(script|style)[^>]*>[\s\S]*?<\/\1>/g, '');
  for (const [, close, tag, rest, text] of body.matchAll(
    /<(\/?)([a-zA-Z][\w-]*)([^>]*)>|([^<]+)/g,
  )) {
    const at = open.at(-1);
    if (text !== undefined) at.children.push(decode(text));
    else if (close) {
      const i = open.findLastIndex((n) => n.tag === tag.toLowerCase());
      if (i > 0) open.length = i;
    } else {
      const attrs = {};
      for (const [, k, , v] of rest.matchAll(/([\w-]+)(="([^"]*)")?/g)) attrs[k] = decode(v ?? '');
      const node = { tag: tag.toLowerCase(), attrs, children: [] };
      at.children.push(node);
      if (!VOID.has(node.tag) && !rest.trim().endsWith('/')) open.push(node);
    }
  }
  return root;
}
const nodes = (n) => n.children.filter((c) => typeof c !== 'string');
function* descendants(n) {
  for (const c of nodes(n)) {
    yield c;
    yield* descendants(c);
  }
}
/** Whether `n` is `one` of a selector: tag, .class, [attr], [attr=value] or [attr^=value]. */
function is(n, one) {
  const tag = one.match(/^[a-z]+/)?.[0];
  if (tag && n.tag !== tag) return false;
  for (const [, c] of one.matchAll(/\.([\w-]+)/g))
    if (!(n.attrs.class ?? '').split(/\s+/).includes(c)) return false;
  for (const [, k, op, v] of one.matchAll(/\[([\w-]+)(?:(\^?=)"?([^"\]]*)"?)?\]/g)) {
    const have = n.attrs[k];
    if (have === undefined) return false;
    if (op === '=' && have !== v) return false;
    if (op === '^=' && !have.startsWith(v)) return false;
  }
  return true;
}
/** Every node under `from` that `selector` (descendant steps) names. */
function all(from, selector) {
  let at = [from];
  for (const one of selector.match(/(?:[^\s[]|\[[^\]]*\])+/g))
    at = [...new Set(at.flatMap((n) => [...descendants(n)].filter((d) => is(d, one))))];
  return at;
}
const BLOCK = new Set(['p', 'li', 'h1', 'h2', 'h3', 'section', 'div', 'form', 'ul']);
/** What a reader reads of a node: its words, a line per block. */
function text(n) {
  const out = [];
  const walk = (x) => {
    if (typeof x === 'string') return out.push(x);
    if (x.tag === 'select')
      return out.push(
        ...nodes(x)
          .slice(0, 1)
          .flatMap((o) => o.children),
      );
    if (BLOCK.has(x.tag)) out.push('\n');
    x.children.forEach(walk);
    if (BLOCK.has(x.tag)) out.push('\n');
  };
  walk(n);
  return out
    .join('')
    .split('\n')
    .map((l) => l.replace(/\s+/g, ' ').trim())
    .filter(Boolean)
    .join('\n');
}

/** §10's town, demo.json, on a copy of its own: Kim's tile saw listed in Waller block. */
async function serve() {
  const dir = mkdtempSync(join(tmpdir(), 'grapevine-scene-'));
  copyFileSync(new URL('./demo.json', import.meta.url).pathname, join(dir, 'town.json'));
  const traced = [];
  const was = process.stderr.write;
  process.stderr.write = (chunk, ...rest) =>
    String(chunk).startsWith('{')
      ? traced.push(String(chunk))
      : was.call(process.stderr, chunk, ...rest);
  process.env.GRAPEVINE_DEBUG = '1';
  // relay.mjs reads GRAPEVINE_DEBUG as it loads.
  const { grapevine } = await import('./server.mjs');
  const app = grapevine(
    { file: join(dir, 'town.json'), acting: ['Sam', 'Jo', 'Kim'], data: dir },
    { address: () => OWN },
  );
  const ask = (path, { body } = {}) => {
    const req = Readable.from(body === undefined ? [] : [Buffer.from(JSON.stringify(body))]);
    req.method = body === undefined ? 'GET' : 'POST';
    req.url = path;
    req.headers = {
      host: OWN,
      ...(body !== undefined && { origin: `http://${OWN}`, 'content-type': 'application/json' }),
    };
    return new Promise((done, fail) => {
      const res = {
        writeHead(status) {
          this.status = status;
          return this;
        },
        end(out = '') {
          done({ status: this.status, out: String(out) });
          return this;
        },
      };
      app.handle(req, res).catch(fail);
    });
  };
  return {
    ask,
    /** One turn of the loop: each person's pass, in order. */
    turn: async () => {
      for (const p of app.people) await p.pass();
    },
    traced: () => traced.join(''),
    stop: () => {
      process.stderr.write = was;
    },
  };
}

test("§10's scene, by ?as=, every page a simulation", async () => {
  const server = await serve();
  try {
    const pages = [];
    let root;
    let who;
    const as = async (person) => {
      who = person;
      const { out } = await server.ask(person ? `/?as=${person}` : '/');
      root = parse(out);
      const said = text(all(root, 'body')[0]);
      pages.push([person ?? 'nobody', said, root]);
      return said;
    };
    /** `who`'s page, read again after each turn of the loop until `want` holds of it, within two. */
    const until = async (person, want, what) => {
      for (let turns = 0; ; turns += 1) {
        const said = await as(person);
        if (want(said)) return said;
        if (turns === 2)
          assert.fail(`${person}'s page never showed ${what} within two loops:\n${said}`);
        await server.turn();
      }
    };
    /** The form whose button says `name` under `scope`, sent as the page's script sends it, with `typed` ({name: value}, a select by its option's words). */
    const press = async (scope, name, typed = {}) => {
      const form = all(root, `${scope} form`).find((f) =>
        all(f, 'button').some((b) => text(b) === name),
      );
      assert.ok(form, `no ${name} under ${scope} on ${who}'s page`);
      const body = { fields: {} };
      for (const el of all(form, '[name]')) {
        const options = all(el, 'option');
        const chosen = options.find((o) => o.attrs.selected !== undefined) ?? options[0];
        let v = el.tag === 'select' ? (chosen.attrs.value ?? text(chosen)) : (el.attrs.value ?? '');
        if (el.attrs.name in typed) {
          const want = typed[el.attrs.name];
          v =
            el.tag === 'select'
              ? (options.find((o) => text(o) === want).attrs.value ?? want)
              : want;
        }
        if (el.attrs.name.startsWith('f.')) body.fields[el.attrs.name.slice(2)] = v;
        else body[el.attrs.name] = el.attrs.name === 'hops' ? Number(v) : v;
      }
      const { status, out } = await server.ask(`${form.attrs['data-to']}?as=${who}`, { body });
      assert.equal(status, 200, `${name}: ${out}`);
    };

    // 1. Sam asks in 1400 Page for a tile saw on Saturday, one ring away.
    await as('Sam');
    const tool = '.kind[data-kind=tool]';
    assert.deepEqual(
      all(root, `${tool} form[data-to="/ask"] [name^="f."]`).map((e) => [
        e.attrs.name,
        e.attrs.type,
      ]),
      [
        ['f.what', 'text'],
        ['f.when', 'text'],
      ],
      "the tool form is the kind's fields",
    );
    await press(tool, 'Ask', {
      'f.what': 'tile saw',
      'f.when': 'Saturday',
      group: '1400 Page',
      hops: 'One ring away',
    });
    assert.match(await as('Sam'), /You asked: tile saw, Saturday/);

    // 2. Jo's page shows the ask and "Carry to Waller block". Jo taps it.
    assert.match(await as('Jo'), /Sam asks: tile saw, Saturday/);
    await press('section[data-group=page] .ask', 'Carry to Waller block');

    // 3. Kim's page: carried by Jo, and the tool kind's notice. Kim offers.
    const kim = await as('Kim');
    assert.match(kim, /A neighbour of Jo’s needs: tile saw, Saturday — carried by Jo/);
    assert.match(kim, /You might have this\./);
    assert.doesNotMatch(
      text(all(root, 'section[data-group=waller]')[0]),
      /Sam/,
      "Kim's page never names who asked",
    );
    await press('section[data-group=waller] .ask', 'Offer', {
      'f.when': 'Saturday morning',
      'f.where': 'Waller and Pierce',
    });
    assert.match(await as('Kim'), /It goes through Jo’s computer/);

    // 4. Sam's page opens the offer, through Jo's loop; Jo's says a sealed message went by.
    await until(
      'Sam',
      (t) => /Someone who saw it offered: when Saturday morning; where Waller and Pierce/.test(t),
      'the offer',
    );
    const jo = await until(
      'Jo',
      (t) => /A sealed message went by toward the person who asked: \d+ bytes/.test(t),
      'that a message went by',
    );
    for (const s of SEALED) assert.ok(!jo.includes(s), `Jo's page holds ${s}`);

    // 5. Sam says yes, then Kim; each sees the other's name.
    await as('Sam');
    await press('.thread.asker', 'Yes', { name: 'Sam Reyes' });
    const before = await until('Kim', (t) => /The person who asked said yes/.test(t), "Sam's yes");
    assert.ok(!before.includes('Sam Reyes'), "Sam's name waits for Kim's yes");
    await press('.thread.replier', 'Yes', { name: 'Kim Okafor' });
    assert.match(await as('Kim'), /You both said yes: they’re Sam Reyes/);
    await until('Sam', (t) => /You both said yes: they’re Kim Okafor/.test(t), "Kim's yes");

    // 6. Sam takes it back, and the carry in Waller is gone within one loop.
    await as('Sam');
    await press('section[data-group=page] .ask', 'Take it back');
    await until('Kim', (t) => !/tile saw/.test(t.split('Your conversations')[0]), 'the carry gone');

    // A kind's `show` is drawn as text: Kim asks a question with markup in it.
    await as('Kim');
    await press('.kind[data-kind=ask]', 'Ask', {
      'f.question': 'Anyone have a <b>ladder</b>?',
      group: 'Waller block',
    });
    assert.match(await as('Jo'), /Kim asks: Anyone have a <b>ladder<\/b>\?/);
    assert.equal(all(root, '.ask b').length, 0, "the page drew the kind's own HTML");

    await as(null);
    for (const [person, said] of pages) {
      assert.match(said, /This is a simulation/, `${person}'s page says it is a simulation`);
      assert.doesNotMatch(said, NOT_SAID, `${person}'s page says a word a reader must look up`);
    }
    for (const person of ['Sam', 'Jo', 'Kim']) {
      await as(person);
      const threads = all(root, '.thread').length;
      assert.ok(threads > 0, `${person} has a conversation`);
      assert.equal(
        all(root, '.thread .shortcut').length,
        threads,
        `every one of ${person}'s conversations says the shortcut`,
      );
    }
    assert.ok(server.traced().length, 'the server traced');
    for (const s of SEALED)
      assert.ok(!server.traced().includes(s), `the server's trace holds ${s}`);
  } finally {
    server.stop();
  }
});
