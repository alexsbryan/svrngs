// A lend's answer at a page's own origin on a phone is made anew (web/lend.js
// `inert`, as crates/svrngs-node/src/origin.rs `inert` makes it on a
// computer; CARRIED_APP_READS §5): only a type from `DATA`, spelled by
// svrngs, and none of the lend's headers. lend.js is a service worker
// whose imports are a browser's, so the function is cut from its text (as
// lend-frames.test.mjs does). Planted, watched failing: `inert` returned
// the lend's answer whole, and the script reached the page (the first test);
// the type read without `toLowerCase`, and `Application/JSON`, which is
// data, was refused (the second).
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { test } from 'node:test';
import { runInNewContext } from 'node:vm';

const LEND = readFileSync(new URL('../lend.js', import.meta.url), 'utf8');
const defined = (name) => {
  const at = LEND.search(new RegExp(`^(async )?function ${name}\\(`, 'm'));
  assert.notEqual(at, -1, `lend.js declares ${name}`);
  return LEND.slice(at, LEND.indexOf('\n}\n', at) + 2);
};
const line = (name) => {
  const found = new RegExp(`^const ${name} = [^;]*;`, 'ms').exec(LEND)?.[0];
  assert.ok(found, `lend.js declares ${name}`);
  return found;
};
const REFUSED_NAME = 'x-svrngs-refused';
const inert = runInNewContext(
  `${line('DATA')}\n${line('INERT')}\nconst REFUSED = ${JSON.stringify(REFUSED_NAME)};\nconst APPEND = 'x-svrngs-append';\n` +
    `const trace = () => {};\nconst closed = () => new Response('', { status: 403, headers: { [REFUSED]: 'closed' } });\n` +
    `const unpacked = (res) => ({ answer: res.body });\n${defined('inert')}\ninert`,
  { Response, Headers },
);

test("a lend's script never reaches a page's origin, however its type is spelled", () => {
  for (const type of ['text/javascript', 'Text/JavaScript', 'text/html', 'image/svg+xml']) {
    const res = inert(new Response('code', { headers: { 'content-type': type } }));
    assert.equal(res.status, 403, type);
    assert.equal(res.headers.get(REFUSED_NAME), 'closed', type);
  }
});

test("only svrngs's own headers reach the page, the type in its own spelling", async () => {
  const res = inert(
    new Response('{"a":1}', {
      status: 201,
      headers: {
        'content-type': 'Application/JSON; charset=utf-8',
        'set-cookie': 'lend=1',
        'clear-site-data': '"*"',
        location: 'https://elsewhere.example/',
        'cache-control': 'public, max-age=3600',
      },
    }),
  );
  assert.equal(res.status, 201);
  assert.equal(await res.text(), '{"a":1}');
  assert.deepEqual([...res.headers].sort(), [
    ['cache-control', 'no-store'],
    ['content-security-policy', 'sandbox'],
    ['content-type', 'application/json'],
    ['x-content-type-options', 'nosniff'],
  ]);
});

test("a lend's answer naming acts is refused: a read never signs", () => {
  const res = inert(
    new Response('{}', {
      headers: { 'content-type': 'application/json', 'x-svrngs-append': '[]' },
    }),
  );
  assert.equal(res.status, 403);
});

test('an answer with no type keeps its status and has no body', async () => {
  const res = inert(new Response(null, { status: 204 }));
  assert.equal(res.status, 204);
  assert.equal(await res.text(), '');
  assert.equal(res.headers.get('cache-control'), 'no-store');
});

test("svrngs's own refusal says what it is, its words not the lend's", async () => {
  const res = inert(
    new Response('nope', {
      status: 403,
      headers: { 'content-type': 'text/plain', [REFUSED_NAME]: 'unasked' },
    }),
  );
  assert.equal(res.headers.get(REFUSED_NAME), 'unasked');
  assert.equal(await res.text(), 'nope');
});
