// If the answer to an action is lost, the action is not quietly sent again
// (KEEP_GOING_AT_HOME 19). lend.js `doorAnswer` tries a read a second time on
// a dial let go of; what `bodyFor` hands it for anything else, a stream even
// when it is empty, ends it at the first try, the person told, and the
// `dials` entry let go so the next ask dials afresh. lend.js is a service
// worker whose imports are a browser's, so the three functions are cut from
// its text (as lend-frames.test.mjs does) and run on a dial that never
// answers. Planted, watched failing: `body ||` out of `doorAnswer`'s test,
// and the POST was sent twice.

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
const posts = /^const posts = .*$/m.exec(LEND)?.[0];
assert.ok(posts, 'lend.js declares posts');
const SOURCE = [posts, ...['bodyFor', 'doorAnswer', 'sent'].map(defined)].join('\n');

// Each method's asks of a dial that fails with `error`, `body` the page's.
async function asked(method, error, body) {
  const sends = [];
  const said = [];
  const dial = {
    fetch: async (how, path, headers, stream) => {
      sends.push({ how, path, body: stream !== null });
      throw error;
    },
  };
  const scope = {
    Blob,
    ReadableStream,
    SLICE: 1 << 20,
    dials: new Map([['key', dial]]),
    dialFor: async () => dial,
    trace: (line) => said.push(line),
  };
  const run = runInNewContext(
    `${SOURCE}\n(async (asked) => doorAnswer(asked, [], await bodyFor(asked, [])))`,
    scope,
  );
  const request = new Request('https://lend.example/thing', { method, body });
  const lost = await run({ request, lend: { key: 'key' }, path: '/thing' });
  return { sends, lost, said, dials: scope.dials };
}

for (const [method, body] of [
  ['POST', 'an entry'],
  ['POST', undefined],
  ['PUT', 'an entry'],
  ['PATCH', 'an entry'],
  ['DELETE', undefined],
]) {
  test(`${method} ${body ? 'with' : 'with no'} body is sent once and its loss said`, async () => {
    const { sends, lost, said, dials } = await asked(method, new Error('connection lost'), body);
    assert.equal(sends.length, 1, 'the action goes to the computer once');
    assert.equal(String(lost.lost), 'Error: connection lost', 'the loss reaches the person');
    assert.equal(dials.size, 0, 'the dial is let go, so the next ask dials afresh');
    assert.match(said.at(-1), /failed \(try 1\)/);
  });
}

test('a read is tried once more, so the POST above is held by the rule and not by chance', async () => {
  const { sends, lost } = await asked('GET', new Error('connection lost'));
  assert.equal(sends.length, 2, 'a read lost its dial, so it dials afresh once');
  assert.ok('lost' in lost);
});

test('a read given up on by the idle timeout is not tried again', async () => {
  const { sends } = await asked('GET', new Error('asleep: timed out'));
  assert.equal(sends.length, 1);
});
