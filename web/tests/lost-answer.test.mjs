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
const SOURCE = [
  posts,
  ...['bodyFor', 'doorAnswer', 'sent', 'lenderOf', 'dialAt', 'letGo'].map(defined),
].join('\n');

// Each method's asks of a dial that fails with `error`, `body` the page's.
async function asked(method, error, body, lend = { key: 'key' }) {
  const [sends, said, dialled] = [[], [], []];
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
    dialFor: async (key) => (dialled.push(key), dial),
    failed: new Set(),
    trace: (line) => said.push(line),
  };
  const run = runInNewContext(
    `${SOURCE}\n(async (asked) => doorAnswer(asked, [], await bodyFor(asked, [])))`,
    scope,
  );
  const request = new Request('https://lend.example/thing', { method, body });
  const lost = await run({ request, lend, path: '/thing' });
  return { sends, lost, said, dials: scope.dials, dialled, failed: scope.failed };
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

// What a person is told, cut from lend.js as above: a write that may have
// reached the app is not one to send again unlooked, and a read still is.
const SAID = [
  ...['QUIET', 'UNSURE'].map((name) => new RegExp(`^const ${name} = [^]*?;$`, 'm').exec(LEND)?.[0]),
  defined('away'),
].join('\n');
const told = (method, mode) =>
  runInNewContext(`${SAID}\n(away)`, {
    posts: (request) => !['GET', 'HEAD'].includes(request.method),
    silent: (whom) => `${whom} did not answer, so it may be off.`,
    unreached: (_, quiet) => quiet,
    trace: () => {},
    say: (_, words) => words,
    awayPage: async () => undefined,
    Response,
  })({ method, mode }, { label: 'l', person: 'Sam', name: 'Money' }, new Error('lost'));

for (const method of ['POST', 'PUT', 'PATCH', 'DELETE']) {
  test(`a ${method} whose answer was lost says it may have gone through`, async () => {
    const words = await told(method, 'cors');
    assert.match(words, /may have gone through/);
    assert.match(words, /[Cc]heck before you send it again/);
    assert.doesNotMatch(words, /Try again later/);
  });
}

test('a form sent in a whole tab is told the same, not shown the dark window', async () => {
  assert.match(await told('POST', 'navigate'), /may have gone through/);
});

for (const method of ['GET', 'HEAD']) {
  test(`a ${method} keeps today's words`, async () => {
    assert.equal(
      await told(method, 'cors'),
      'The computer lending this did not answer, so it may be off. Try again later.',
    );
  });
}

// A carried app is one origin whichever computer lends it (svrngs-core
// `Label::carried`): a read lost at one computer is tried at the next, and
// a write lost is sent no more, but the next goes to the next computer.
// Planted, watched failing: `letGo` not marking the computer `failed`, and
// the read was tried at `ada` twice.
test('a carried app is tried at the next computer lending it', async () => {
  const lend = { key: 'ada', name: 'Money', lenders: [{ key: 'cy' }] };
  const read = await asked('GET', new Error('connection lost'), undefined, lend);
  assert.deepEqual(read.dialled, ['ada', 'cy'], 'the read went on to cy');
  const wrote = await asked('POST', new Error('connection lost'), 'an entry', lend);
  assert.deepEqual(wrote.dialled, ['ada'], 'the write is sent once');
  assert.deepEqual([...wrote.failed], ['ada'], 'and the next goes to cy');
});
