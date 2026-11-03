// The pages' pure core (page-scaffold (c); crates/svrngs-node/src/home/ui/machine.js;
// svrn-docs rings/current/PAGE_STATE.md §3), on a toy machine of two regions
// with no DOM: a step and its trace, a refusal, an effect that fails each way
// it can, a late answer, a state outside its set, and the effects' order.

import assert from 'node:assert/strict';
import { test } from 'node:test';

import {
  ask,
  go,
  late,
  machine,
  no,
} from '../../crates/svrngs-node/src/home/ui/machine.js?v=__VERSION__';

// The toy: `list` reads a list, `send` posts one thing at a time.
const states = { list: ['reading', 'read', 'unread'], send: ['idle', 'sending', 'sent'] };
const first = { at: { list: 'reading', send: 'idle' }, asked: { list: 1 }, rows: [] };
function step(s, m) {
  switch (m.type) {
    case 'read':
      return late(s, 'list', m.n)
        ? no(s, late(s, 'list', m.n))
        : go(s, { list: 'read' }, { rows: m.rows });
    case 'unread':
      return go(s, { list: 'unread' }, { said: m.why });
    case 'again': {
      const [asked, n] = ask(s, 'list');
      return go(s, { list: 'reading' }, { asked }, { run: 'read', failed: 'unread', n });
    }
    case 'send':
      if (s.at.send === 'sending') return no(s, 'sending already');
      return go(
        s,
        { send: 'sending' },
        {},
        { run: 'post', failed: 'unsent' },
        { run: 'note', failed: 'unnoted' },
      );
    case 'stray':
      return go(s, { send: 'lost' });
    default:
      return no(s, `no message ${m.type}`);
  }
}

// A machine over `io`, its trace kept, and a count of who was told.
function toy(io = {}, start = first, effects = []) {
  const lines = [];
  globalThis.__trace = (l) => lines.push(l);
  const m = machine('TOY', { states, step, io }, start, effects);
  let told = 0;
  m.subscribe(() => told++);
  return { m, lines, told: () => told };
}
const settled = () => new Promise((r) => setTimeout(r, 0));

test('a step answers the next state, tells each subscriber once, and is traced', () => {
  const { m, lines, told } = toy();
  m.dispatch({ type: 'read', n: 1, rows: ['a'] });
  assert.deepEqual([m.get().at, m.get().rows, told()], [{ list: 'read', send: 'idle' }, ['a'], 1]);
  assert.deepEqual(lines, ['TOY: read: reading/idle -> read/idle']);
  m.dispatch({ type: 'unread', why: 'the computer did not answer' });
  assert.equal(lines[1], 'TOY: unread (the computer did not answer): read/idle -> unread/idle');
});

test('a refusal keeps the same state, tells nobody, runs nothing, and is traced with why', async () => {
  let ran = 0;
  const { m, lines, told } = toy({
    post: async () => (ran++, { type: 'sent' }),
    note: async () => (ran++, { type: 'noted' }),
  });
  m.dispatch({ type: 'send' });
  const before = m.get();
  const t = told();
  m.dispatch({ type: 'send' });
  assert.equal(m.get(), before);
  assert.equal(told(), t);
  assert.equal(lines.at(-1), 'TOY: send refused in reading/sending: sending already');
  await settled();
  assert.equal(ran, 2, 'only the first send ran its two effects');
});

test('an effect rejected, thrown, or with no io answers its failed message, with why', async () => {
  const io = {
    read: async () => {
      throw new Error('the computer did not answer');
    },
  };
  for (const [name, read, why] of [
    ['rejected', () => Promise.reject(new Error('refused: 403')), 'refused: 403'],
    [
      'thrown',
      () => {
        throw new Error('not a function here');
      },
      'not a function here',
    ],
    ['no io', undefined, 'nothing here runs read'],
    ['rejected', io.read, 'the computer did not answer'],
  ]) {
    const { m, lines } = toy(read ? { read } : {}, first, [
      { run: 'read', failed: 'unread', n: 1 },
    ]);
    await settled();
    assert.deepEqual([m.get().at.list, m.get().said], ['unread', why], name);
    assert.equal(lines.at(-1), `TOY: unread (${why}): reading/idle -> unread/idle`, name);
  }
});

test('an effect that answers nothing answers its failed message, never a quiet idle', async () => {
  const { m } = toy({ read: async () => undefined }, first, [
    { run: 'read', failed: 'unread', n: 1 },
  ]);
  await settled();
  assert.deepEqual([m.get().at.list, m.get().said], ['unread', 'read answered nothing']);
});

test('a late answer is refused by name, and the newer one taken', async () => {
  const { m, lines } = toy();
  m.dispatch({ type: 'again' });
  assert.equal(m.get().asked.list, 2);
  m.dispatch({ type: 'read', n: 1, rows: ['old'] });
  assert.equal(
    lines.at(-1),
    'TOY: read refused in reading/idle: list 1 answered after list 2 was asked',
  );
  assert.deepEqual(m.get().rows, []);
  m.dispatch({ type: 'read', n: 2, rows: ['new'] });
  assert.deepEqual(m.get().rows, ['new']);
  assert.equal(late(m.get(), 'list', 2), '');
});

test('a step that answers a state outside its region is refused', () => {
  const { m, lines, told } = toy();
  const before = m.get();
  m.dispatch({ type: 'stray' });
  assert.equal(m.get(), before);
  assert.equal(told(), 0);
  assert.equal(
    lines.at(-1),
    'TOY: stray refused in reading/idle: send lost is not one of idle, sending, sent',
  );
});

test('effects run in the order the step gives them, after the state is taken', async () => {
  const order = [];
  const { m, lines } = toy({
    post: async () => (order.push(`post in ${m.get().at.send}`), { type: 'nothing' }),
    note: async () => (order.push('note'), { type: 'nothing' }),
  });
  m.dispatch({ type: 'send' });
  assert.equal(lines[0], 'TOY: send: reading/idle -> reading/sending, running post, note');
  assert.deepEqual(order, [], 'nothing runs inside the step');
  await settled();
  assert.deepEqual(order, ['post in sending', 'note']);
});
