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
