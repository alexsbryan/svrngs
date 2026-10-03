// The bridge's core against the fake adapter and an in-memory host.
import assert from 'node:assert/strict';
import { test } from 'node:test';

import { startBridge } from '../bridge.mjs';
import { fakeAdapter } from './fake.mjs';

const AMA = { speaker: 'discord:1', name: 'Ama' };
const BEN = { speaker: 'discord:2', name: 'Ben' };

/** The guest book's answers, with the host's rule reduced to in or out. */
function memoryHost({ entered = false, down = false } = {}) {
  const book = new Map();
  return {
    book,
    group: async () => ({ name: 'the house', people: 0, entered }),
    commands: async () => [],
    due: async () => [],
    async guest({ speaker, event }) {
      if (down) throw new Error('POST /guest: 503 the journal has stopped');
      const was = book.get(speaker);
      if (was === 'out' && event === 'seen') return { removed: true };
      const now = event === 'left' || event === 'banned' ? 'out' : 'in';
      book.set(speaker, now);
      return { new: was !== now, arrived: now === 'in' && was !== 'in' };
    },
  };
}

test('a new group hears the greeting once, and each arrival a welcome', async () => {
  const chat = fakeAdapter();
  const host = memoryHost();
  await startBridge({ adapter: chat, host, owner: 'Sam' });
  await chat.input(AMA, { interaction: 'm1', text: 'pizza with @Ben', mentions: [BEN] });
  await chat.input(AMA, { interaction: 'm2', text: 'again' });
  const posts = chat.posts.map((p) => p.text);
  assert.match(posts[0], /^Hi, I'm the bot for the house\..*on Sam's computer, not on Discord/);
  assert.deepEqual(posts.slice(1), [
    "Welcome, Ben. I'm the bot for the house: I read this channel, and what I keep is stored on Sam's computer, not on Discord.",
    "Welcome, Ama. I'm the bot for the house: I read this channel, and what I keep is stored on Sam's computer, not on Discord.",
  ]);
  assert.deepEqual([...host.book], [['discord:2', 'in'], ['discord:1', 'in']]);
});

test('a group someone has entered is not greeted again', async () => {
  const chat = fakeAdapter();
  await startBridge({ adapter: chat, host: memoryHost({ entered: true }), owner: 'Sam' });
  assert.deepEqual(chat.posts, []);
  assert.deepEqual(chat.commands, []);
});

test('a write that failed is said to the person, never dropped', async () => {
  const chat = fakeAdapter();
  await startBridge({ adapter: chat, host: memoryHost({ entered: true, down: true }), owner: 'Sam' });
  await chat.input(AMA, { kind: 'button', interaction: 'i1' });
  assert.deepEqual(chat.replies, [
    { to: 'i1', text: 'I couldn\'t write that down just now. Please try again in a moment.', private: true },
  ]);
});

test('a ban removes, and a removed person is told so privately', async () => {
  const chat = fakeAdapter();
  const host = memoryHost({ entered: true });
  await startBridge({ adapter: chat, host, owner: 'Sam' });
  await chat.join(AMA);
  await chat.ban(AMA);
  await chat.input(AMA, { kind: 'command', interaction: 'i2' });
  assert.equal(host.book.get('discord:1'), 'out');
  assert.deepEqual(chat.replies.map((r) => [r.to, r.private]), [['i2', true]]);
  assert.match(chat.replies[0].text, /^You're no longer part of the house/);
});

test('a command and a tap go to the host, and its answer is the reply', async () => {
  const chat = fakeAdapter();
  const runs = [];
  const host = {
    ...memoryHost({ entered: true }),
    commands: async () => [{ name: 'paid', description: 'You paid', options: [] }],
    async run(invocation) {
      runs.push(invocation);
      return { text: 'Sam paid $60.00 for pizza.', buttons: [{ id: 'expenses:confirm:a1', label: 'Confirm' }], private: false };
    },
  };
  await startBridge({ adapter: chat, host, owner: 'Sam' });
  assert.deepEqual(chat.commands, [{ name: 'paid', description: 'You paid', options: [] }]);
  await chat.input(AMA, { kind: 'command', interaction: 'i1', command: 'paid', options: { amount: '60', what: 'pizza' } });
  await chat.input(BEN, { kind: 'button', interaction: 'i2', button: 'expenses:confirm:a1' });
  await chat.input(BEN, { interaction: 'm1', text: 'thanks' });
  assert.deepEqual(runs, [
    {
      input: { kind: 'command', command: 'paid', options: { amount: '60', what: 'pizza' } },
      on_behalf_of: { speaker: 'discord:1', interaction: 'i1' },
    },
    { input: { kind: 'button', button: 'expenses:confirm:a1' }, on_behalf_of: { speaker: 'discord:2', interaction: 'i2' } },
  ]);
  assert.deepEqual(
    chat.replies.map((r) => [r.to, r.text, r.private]),
    [
      ['i1', 'Sam paid $60.00 for pizza.', false],
      ['i2', 'Sam paid $60.00 for pizza.', false],
    ],
  );
  assert.deepEqual(chat.replies[0].buttons, [{ id: 'expenses:confirm:a1', label: 'Confirm' }]);
});

test('what is due is posted naming its person, and a post that fails is said', async () => {
  const chat = fakeAdapter();
  const reminders = [
    { person: 'discord:1', text: 'a reminder: you owe $15.00.' },
    { person: 'discord:2', text: 'a reminder: you owe $5.00.' },
  ];
  const post = chat.post;
  chat.post = async (m) => {
    if (m.mention?.[0] === 'discord:2') throw new Error('Discord is down');
    return post(m);
  };
  const logged = [];
  const write = process.stderr.write;
  process.stderr.write = (line) => logged.push(JSON.parse(line).event) && true;
  try {
    await startBridge({ adapter: chat, host: { ...memoryHost({ entered: true }), due: async () => reminders }, owner: 'Sam' });
  } finally {
    process.stderr.write = write;
  }
  assert.deepEqual(chat.posts, [{ text: 'a reminder: you owe $15.00.', mention: ['discord:1'] }]);
  assert.deepEqual(
    logged.filter((e) => e.startsWith('bridge:due')),
    ['bridge:due_posted', 'bridge:due_post_failed'],
  );
});
