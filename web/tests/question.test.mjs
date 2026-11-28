// Pure contracts moved intact from browser/{acts,opens,main}.rs. Browser tests
// retain actual consent, signing, Stop and rendering; these need only Node.
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { readdirSync, readFileSync } from 'node:fs';
import { test } from 'node:test';
import * as q from '../../crates/svrngs-node/src/home/question.js?v=__VERSION__';

const ROOT = new URL('../../', import.meta.url);
const PAGES = ['crates/svrngs-node/src/home/page/', 'web/page/'];
const source = (dir) =>
  readdirSync(new URL(dir, ROOT))
    .filter((name) => name.endsWith('.js'))
    .sort()
    .map((name) => readFileSync(new URL(dir + name, ROOT), 'utf8'))
    .join('');

test('what acts as you is said in one wording', () => {
  const notes = { person: 'cy', name: 'notes', kind: 'notes', reads: ['films'] };
  const at = 1_791_100_000;
  const day = new Intl.DateTimeFormat('en-GB', { day: 'numeric', month: 'short' }).format(
    new Date(at * 1000),
  );
  const friends = ['Sam', 'Bo', 'Cy', 'Dee', 'Eli'].map((person) => ({
    person,
    kind: 'jellyfin',
    new: false,
  }));
  const row = (key, person, person_key, device) => ({ key, person, person_key, device });
  const rows = [
    row('a1', 'ada', 'a1', 'computer'),
    row('a2', 'ada', 'a1', 'browser'),
    row('n1', 'ada', 'n1', 'computer'),
    row('b1', 'bo', 'b1', 'browser'),
  ];
  const [sam, bo] = friends;
  assert.deepEqual(
    [
      q.carrying({ kind: 'Money', title: 'Money Matters' }),
      q.carrying({ kind: 'Quiz' }),
      q.carrying({ kind: 'Movie-Night', reads: ['jellyfin'] }),
      q.reading(notes, { said: 'yes', at }),
      q.reading(notes, { said: 'stop', at }),
      q.reading(notes, { said: 'yes', at, lends: ['x'] }),
      q.reading(notes, { said: 'yes', at, lends: [] }),
      q.readLine([sam, bo], 2, ['jellyfin']),
      q.readLine([sam, { ...bo, new: true }], 2, ['jellyfin']),
      q.readLine([], 0, ['jellyfin']),
      q.readLine([], 2, ['jellyfin']),
      q.readLine(friends, 5, ['jellyfin']),
      q.deciders(rows, 'a1', 'computer'),
      q.deciders(rows, 'a2', 'browser'),
      q.deciders(rows, 'b1', 'browser'),
    ],
    [
      'Money Matters reads and writes its own entries, when you open it.',
      'Quiz reads and writes its own entries, when you open it.',
      'Movie Night reads your friends’ jellyfin, when you open it.',
      `cy’s notes opens your friends’ films as you. Since ${day}. `,
      `cy’s notes stopped opening your friends’ films as you on ${day}. Opening it again says yes again.`,
      `cy’s notes opens the films you chose, as you. Since ${day}. `,
      'cy’s notes opens nothing of your friends’ as you. Open it again to choose what it can. ',
      'Reads Sam’s and Bo’s jellyfin. They can see it’s you, and what it asks for.',
      'Reads Sam’s and Bo’s jellyfin. Bo’s is new since you last opened it.',
      'Reads your friends’ jellyfin. Nobody lends one now.',
      'Reads none of your friends’ jellyfin.',
      'Reads jellyfin from 5 friends. They can see it’s you, and what it asks for.',
      'This computer, and your phone',
      'This phone, and your computer',
      'This phone',
    ],
  );
});

test('the open says what an app does in one wording', () => {
  for (const page of PAGES) assert.ok(source(page).includes('agreed('), page);
  const carried = (kind) => ({ group: 'g', kind, person: 'cy', name: 'x', reads: [] });
  const about = 'Who owes whom, and for what.';
  const lends = [
    { ...carried('Money'), title: 'Money', about },
    carried('Quiz'),
    { person: 'cy', name: 'notes', kind: 'notes', reads: ['films', 'music'] },
    { person: 'cy', name: 'shelf', kind: 'k', reads: [] },
  ];
  assert.deepEqual(
    lends.map((lend) => [q.named(lend), q.agreed(lend, { name: 'House' })]),
    [
      [
        'Money',
        '“Who owes whom, and for what.” Opening it lets it read and write only its own entries, as you in House.',
      ],
      ['Quiz', 'Opening it lets it read and write only its own entries, as you in House.'],
      [
        'cy’s notes',
        'Opening it lets it write its own part of House’s record, as you. It also opens your friends’ films and music as you, and they can see what it asks.',
      ],
      ['cy’s shelf (k)', 'Opening it lets it write its own part of House’s record, as you.'],
    ],
  );
});

// An app the group carries is named by its own page and says its own
// line, as its words, and who brought it, by first key: "you" to them, a
// namesake told apart, and the day alone once they have gone. Planted,
// each watched failing: `named` by kind alone; `brought` naming by the
// name the record kept.
test('a carried app is named in its own words, with who brought it', () => {
  const at = 1_791_100_000;
  const on = q.day(at);
  const row = (key, person, joined) => ({ key, person, person_key: key, joined });
  const rows = [
    row('a1', 'ana', 'started'),
    row('b1', 'bo', { at }),
    row('b2', 'bo', { at: at + 86_400 }),
  ];
  const fridge = { group: 'g', kind: 'fridge', title: 'Word fridge', person: 'cy', name: 'x' };
  const by = (key) =>
    q.brought({ ...fridge, brought: { by_key: key, at } }, { people: rows }, 'a1');
  assert.deepEqual(
    [
      q.named(fridge),
      q.named({ ...fridge, title: null }),
      by('a1'),
      by('b1'),
      by('gone'),
      q.brought(fridge, undefined, 'a1'),
    ],
    [
      'Word fridge',
      'fridge',
      ` Brought by you on ${on}.`,
      ` Brought by bo (joined ${q.day(at, true)}) on ${on}.`,
      ` Brought on ${on}.`,
      '',
    ],
  );
});

test('the index row is judged once for both pages', () => {
  for (const page of PAGES) {
    const text = source(page);
    for (const judged of ['using >=', 'signing you in']) assert.ok(!text.includes(judged), page);
    assert.ok(text.includes('opens(l'), page);
  }
  assert.deepEqual(
    [
      { person: 'cy', limit: 1, using: 1 },
      { person: 'cy', limit: null, using: 4 },
    ].map(q.opens),
    ['full', 'on'],
  );
});

// The two golden rows formerly repeated inside slow native workflows. Their
// independent text contract is here; those workflows compare actual DOM to q.
test('the real browser writer and reader rows keep their complete wording', () => {
  const writer = { group: 'g', kind: 'Money' };
  assert.equal(
    `Money ${q.agreed({ ...writer, about: 'Who owes whom, and for what.' }, { name: 'house' })}`,
    'Money “Who owes whom, and for what.” Opening it lets it read and write only its own entries, as you in house.',
  );
  const reader = { group: 'g', kind: 'Probe', reads: ['films'], reading: [] };
  const film = { person: 'cy', name: 'films', kind: 'films' };
  assert.equal(
    `Open ${q.named(reader)}${q.readLine([film], 1, ['films'])} ${q.named(film)} ${q.agreed(reader, {})}`,
    'Open ProbeReads cy’s films. They can see it’s you, and what it asks for. cy’s films Opening it lets it read but never write, as you in your group.',
  );
});

test('a lend alias never collides within a group, including opposite time zones', () => {
  const row = (person, key, person_key, joined) => ({
    person,
    key,
    person_key,
    joined,
    device: 'computer',
  });
  const rows = [
    row('ada', 'a1', 'a1', 'started'),
    row('ada', 'a2', 'a2', { at: 1_791_100_000 }),
    row('sam', 's1', 's1', { at: 1_791_200_000 }),
    row('sam', 's2', 's1', { at: 1_791_300_000 }),
  ];
  const lend = (key, person, name) => ({ key, person, name, kind: 'k' });
  const money = (key) => ({ key, person: 'ada', name: 'g Money', kind: 'Money', group: 'g' });
  const fresh = () => [
    lend('a1', 'ada', 'app'),
    lend('a2', 'ada', 'app'),
    lend('s1', 'sam', 'films'),
    lend('s2', 'sam', 'films'),
    money('a1'),
    money('s1'),
  ];
  const lent = fresh();
  q.apart(lent, rows);
  const said = q.once(lent).map((one) => q.alias(one, lent, rows));
  // Dates independently read with date -u -r <timestamp>, before extraction.
  const expected = [
    'app@ada (started the group)',
    'app@ada (joined 4 Oct)',
    'films@sam (computer joined 5 Oct)',
    'films@sam (computer joined 6 Oct)',
    'Money',
  ];
  assert.deepEqual(said, expected);
  const module = new URL(
    '../../crates/svrngs-node/src/home/question.js?v=__VERSION__',
    import.meta.url,
  ).href;
  const script = `const q = await import(${JSON.stringify(module)});
    const [lent, rows] = JSON.parse(process.argv[1]); q.apart(lent, rows);
    console.log(JSON.stringify({aliases: q.once(lent).map(l => q.alias(l, lent, rows)), person: q.called(rows, 'a2')}));`;
  for (const TZ of ['Pacific/Kiritimati', 'Pacific/Pago_Pago']) {
    const answer = JSON.parse(
      execFileSync(
        process.execPath,
        ['--input-type=module', '-e', script, JSON.stringify([fresh(), rows])],
        { encoding: 'utf8', env: { ...process.env, TZ } },
      ),
    );
    assert.deepEqual(answer.aliases, said, TZ);
    assert.equal(answer.person, 'ada (joined 4 Oct)', TZ);
  }
  const asked = [...expected, 'app@ada', 'films@sam', 'app@Ada (started the group)', 'nothing@ada'];
  assert.deepEqual(
    asked.map((alias) => q.aliased(alias, lent, rows).map((one) => one.key)),
    [['a1'], ['a2'], ['s1'], ['s2'], ['a1'], [], [], [], []],
  );
});
