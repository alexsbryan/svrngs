// A request at a lend's origin on a phone, from the address the worker hands
// it to the answer the page reads (web/lend.js `whereOf`, `lent`, `proxied`):
// a read of another lend is made inert (`inert`), and a write whose answer
// is lost is said once and sent no more, the next ask dialling afresh
// (`doorAnswer`, `away`). lend.js is a service worker whose imports are a
// browser's, so its functions are cut from its text (as lost-answer.test.mjs
// and inert.test.mjs do), the dial a stand-in that answers as told; inert.test.mjs
// and lost-answer.test.mjs hold each function's own rule, this the path
// through them. Moved from browser/main.rs
// `a_phone_page_is_handed_a_lend_s_script_refused_at_its_own_origin` and
// browser/keep_going.rs
// `an_action_whose_answer_is_lost_is_not_sent_again_and_can_be_checked`.
// Planted, each watched failing: `proxied` without its `where.read` test,
// and the script reached the page whole; `whereOf` reading no lend under
// `READS`, and the read was served as the page's own; `away` saying 503,
// and the lost post was not said `away` at 502.
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { test } from 'node:test';
import { runInNewContext } from 'node:vm';
import { silent, unreached } from '../../crates/svrngs-node/src/home/question.js?v=__VERSION__';

const LEND = readFileSync(new URL('../lend.js', import.meta.url), 'utf8');
const defined = (name) => {
  const at = LEND.search(new RegExp(`^(async )?function ${name}\\(`, 'm'));
  assert.notEqual(at, -1, `lend.js declares ${name}`);
  return LEND.slice(at, LEND.indexOf('\n}\n', at) + 2);
};
const constant = (name) => {
  const found = new RegExp(`^const ${name} = [^]*?;\\n`, 'm').exec(LEND)?.[0];
  assert.ok(found, `lend.js declares ${name}`);
  return found;
};
const SOURCE = [
  ...['PLAIN', 'QUIET', 'UNSURE', 'READS', 'APPEND', 'REFUSED', 'DATA', 'INERT'].map(constant),
  ...['say', 'closed', 'unheld', 'posts'].map(constant),
  ...['whereOf', 'toldOf', 'lent', 'proxied', 'bodyFor', 'doorAnswer', 'sent'].map(defined),
  ...['lenderOf', 'dialAt', 'letGo', 'away', 'inert', 'unpacked'].map(defined),
  '({ whereOf, toldOf, lent })',
].join('\n');

const ORIGIN = 'https://page-label.page.example';
const takes = { label: 'page-label', key: 'ada', person: 'ada', name: 'takes', kind: 'k' };
const shelf = { label: 'shelf-label', key: 'cy', person: 'cy', name: 'shelf', kind: 'k' };

// The worker at `ORIGIN`, holding a pass to its own lend and cy's shelf,
// each dial answering as `answer(lender, method, path)` says: a Response,
// or thrown.
function worker(answer) {
  const sends = [];
  const dialled = [];
  const said = [];
  const dialFor = async (lender) => {
    dialled.push(lender);
    const dial = {
      fetch: async (method, path, _headers, body) => {
        sends.push({ lender, method, path, body: body ? await new Response(body).text() : null });
        return answer(lender, method, path);
      },
    };
    scope.dials.set(lender, dial);
    return dial;
  };
  const scope = {
    SCOPE: new URL(`${ORIGIN}/`),
    OWN: takes.label,
    ASK: '/svrngs-ask',
    SLICE: 1 << 20,
    handedNow: () => ({ lends: [takes, shelf] }),
    baseOf: (label) => (label === takes.label ? `${ORIGIN}/` : `https://${label}.page.example/`),
    holding: async () => ({ lends: [takes, shelf], unanswered: [] }),
    jarOf: async () => [],
    headersFor: async () => [],
    ranged: () => null,
    keepCookies: async () => {},
    refusedAppend: async () => null,
    redirected: () => null,
    served: (_asked, res) => res,
    opening: () => assert.fail('no navigation here'),
    unheldAt: () => assert.fail('the pass is held'),
    actsPosted: () => assert.fail('no acts posted here'),
    awayPage: async () => undefined,
    dials: new Map(),
    dialFor: (lender) => scope.dials.get(lender) ?? dialFor(lender),
    failed: new Set(),
    trace: (line) => said.push(line),
    silent,
    unreached,
    Response,
    Headers,
    Blob,
    ReadableStream,
    DecompressionStream,
    URL,
    RegExp,
  };
  const run = runInNewContext(SOURCE, scope);
  // What the page at `url` is answered, by the path the fetch listener takes.
  const fetched = async (url, init) => {
    const at = new URL(url);
    const where = run.whereOf(at);
    assert.ok(where, `${url} is a lend's`);
    return run.lent(new Request(at, init), at, where, !where.read);
  };
  return { fetched, toldOf: run.toldOf, sends, dialled, said };
}

test("a lend's script read at a page's own origin is refused closed: no cookie, nothing to run", async () => {
  const script = () =>
    new Response('export const ran = 1;', {
      headers: { 'content-type': 'text/javascript', 'set-cookie': 'lend=1' },
    });
  const { fetched, toldOf, sends } = worker(script);
  const { address } = toldOf(shelf);
  assert.equal(address, `${ORIGIN}/svrngs-reads/shelf-label/`, 'the address the page is told');
  const res = await fetched(`${address}code.js`);
  assert.equal(res.status, 403);
  assert.equal(res.headers.get('x-svrngs-refused'), 'closed');
  assert.equal(res.headers.get('set-cookie'), null);
  assert.equal(res.headers.get('content-type'), 'text/plain; charset=utf-8');
  assert.equal(
    await res.text(),
    'This is not open to this page.',
    'svrngs’s words, never the code',
  );
  assert.deepEqual(sends, [
    { lender: 'cy', method: 'GET', path: '/lend/shelf-label/code.js', body: null },
  ]);

  const data = worker(() =>
    Response.json({ on: 'the shelf' }, { headers: { 'set-cookie': 'lend=1' } }),
  );
  const read = await data.fetched(`${address}list`);
  assert.equal(read.status, 200, 'data is read');
  assert.deepEqual(await read.json(), { on: 'the shelf' });
  assert.equal(read.headers.get('set-cookie'), null);
});

test('a write whose answer is lost is said once, 502 away, and the check after dials afresh', async () => {
  let taken = 0;
  let cut = true;
  const { fetched, sends, dialled } = worker((_, method) => {
    if (method === 'POST') {
      taken += 1;
      if (cut) throw new Error('connection lost');
    }
    return new Response(`${taken} taken`, { headers: { 'content-type': 'text/plain' } });
  });
  const posted = await fetched(`${ORIGIN}/took`, { method: 'POST', body: 'an entry' });
  assert.equal(posted.status, 502);
  assert.equal(posted.headers.get('x-svrngs-refused'), 'away');
  assert.equal(
    await posted.text(),
    'The computer lending this did not answer, so it may be off. What you sent may have gone through, so check before you send it again.',
  );
  cut = false;
  const checked = await fetched(`${ORIGIN}/took`);
  assert.equal(await checked.text(), '1 taken');
  assert.deepEqual(
    sends.map((s) => [s.method, s.path, s.body]),
    [
      ['POST', '/lend/page-label/took', 'an entry'],
      ['GET', '/lend/page-label/took', null],
    ],
    'the post went once, and the check sent none of it again',
  );
  assert.deepEqual(dialled, ['ada', 'ada'], 'the lost dial let go, the check dialled afresh');
});
