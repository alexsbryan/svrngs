// What a phone's open signs (web/page/hand.js): the pass names the lends
// its person's yes named (capability connections §6 bar 7, the computer's
// door/choices.rs `Choice`), and a lent server named after a folder of the
// group writes nothing (§6 bar 8, the computer's origin.rs `judged`).
// hand.js imports what the build gives it, so the functions are cut from
// its text (as lend-frames.test.mjs does). Planted, watched failing: the
// pass naming every lend of a kind whatever the yes names, Cy's new library
// in the pass (the first test); `writesOf` without its `lent` test, a
// writes pass signed for a server called Money; `writesBlocked` without its
// `lend.group` test, a lent server's entries blocked (the last).
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { test } from 'node:test';
import { runInNewContext } from 'node:vm';

const HAND = readFileSync(new URL('../page/hand.js', import.meta.url), 'utf8').replaceAll(
  /^export /gm,
  '',
);
const defined = (name) => {
  const at = HAND.search(new RegExp(`^(async )?function ${name}\\(`, 'm'));
  assert.notEqual(at, -1, `hand.js declares ${name}`);
  return HAND.slice(at, HAND.indexOf('\n}\n', at) + 2);
};
const arrow = (name) => {
  const found = new RegExp(`^const ${name} = [\\s\\S]*?;\\n`, 'm').exec(HAND)?.[0];
  assert.ok(found, `hand.js declares ${name}`);
  return found;
};
const parts = [
  arrow('readingOf'),
  arrow('writesOf'),
  arrow('writesBlocked'),
  defined('withYes'),
  defined('passLends'),
  defined('taken'),
].join('\n');
const SOURCE = `${parts}\n({ readingOf, writesOf, writesBlocked, withYes, passLends })`;
const kept = [];
const store = {
  opened: async (group) => {
    kept.push(group);
    return group;
  },
};
const { readingOf, writesOf, writesBlocked, withYes, passLends } = runInNewContext(SOURCE, {
  store,
  trace: () => {},
  Date,
  Math,
  Set,
  Object,
});
const core = { relay_taken: () => true, carried_label: () => 'writes-label' };

const page = {
  label: 'page',
  key: 'ada',
  kind: 'app',
  reads: ['films'],
  name: 'page',
  person: 'ada',
};
const sam = { label: 'sam', key: 'sam', kind: 'films', name: 'films', person: 'sam' };
const cy = { label: 'cy', key: 'cy', kind: 'films', name: 'films', person: 'cy' };
// The functions run in their own realm, so what they answer is read into this one.
const labels = (lends) => Array.from(lends, (one) => one.label);

test("a pass names what the person's yes named, nothing new since", () => {
  const said = { said: 'yes', at: 1, lends: ['sam'] };
  const hand = { lent: [page, sam, cy], label: 'page' };
  const lends = passLends(core, hand, page, { reads: { page: said } });
  assert.deepEqual(labels(lends), ['page', 'sam'], "Cy's new library is not in the pass");
});

test('an open is its yes: it names every lend it read, by label', async () => {
  const group = { id: 'g', reads: {} };
  const opened = await withYes({ group, lent: [page, sam, cy], label: 'page' }, page);
  assert.deepEqual(Array.from(opened.reads.page.lends), ['sam', 'cy']);
  assert.equal(opened.reads.page.said, 'yes');
});

test('a re-signed pass keeps the yes it had', async () => {
  const group = { id: 'g', reads: { page: { said: 'yes', at: 1, lends: ['sam'] } } };
  const opened = await withYes({ group, lent: [page, sam, cy], label: 'page', at: '/x' }, page);
  assert.deepEqual(Array.from(opened.reads.page.lends), ['sam']);
  const lends = passLends(core, { lent: [page, sam, cy], label: 'page' }, page, opened);
  assert.deepEqual(labels(lends), ['page', 'sam']);
});

test('a yes from before lends were named covers none, as a computer’s', () => {
  const group = { reads: { page: { said: 'yes', at: 1 } } };
  const lends = passLends(core, { lent: [page, sam], label: 'page' }, page, group);
  assert.deepEqual(labels(lends), ['page']);
});

test('a stop names none', () => {
  const opened = { reads: { page: { said: 'stop', at: 1, lends: ['sam'] } } };
  assert.deepEqual(labels(passLends(core, { lent: [page, sam], label: 'page' }, page, opened)), [
    'page',
  ]);
});

test('its line says what is new since its last yes', () => {
  const said = { said: 'yes', at: 1, lends: ['sam'] };
  const reading = readingOf(page, [page, sam, cy], said);
  assert.deepEqual(
    Array.from(reading, (one) => [one.label, one.new]),
    [
      ['sam', false],
      ['cy', true],
    ],
  );
});

test('a lent server named after a folder of the group is signed no writes', () => {
  const server = { label: 'money', key: 'bo', kind: 'Money' };
  const signed = () => 'signed';
  const group = { id: 'g' };
  const folder = { label: 'folder', key: 'ana', kind: 'Money', group: 'g' };
  assert.equal(writesOf(core, { group, lent: [server, folder] }, server, signed), null);
  assert.equal(writesOf(core, { group, lent: [server] }, server, signed), 'signed');
  assert.equal(writesOf(core, { group, lent: [folder] }, folder, signed), null, 'a folder its own');
});

test('a carried app once let read lends has no entry approved, through its Stop', () => {
  const folder = { label: 'folder', key: 'ana', kind: 'Reader', group: 'g', reads: ['films'] };
  const yes = { reads: { folder: { said: 'yes', at: 1, lends: ['sam'] } } };
  const stop = { reads: { folder: { said: 'stop', at: 2, lends: ['sam'] } } };
  assert.equal(writesBlocked(folder, yes), true, 'after the yes');
  assert.equal(writesBlocked(folder, stop), true, 'after its Stop');
  assert.equal(writesBlocked(folder, { reads: {} }), false, 'never let read');
  assert.equal(writesBlocked({ ...page, label: 'folder' }, yes), false, 'a lent app, §6');
});
