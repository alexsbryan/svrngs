// A new version's yes and a bring, the page's halves, with no browser: the
// home page's yes run whole by twin.mjs (its POST /runs, home.rs, and the
// line said after), and the phone's edge cut from its text and run with
// what leaves the page stood in for: web/page/dial.js `through` and
// `exchange`, which post a group's act to the computer the page reaches,
// and web/page/acts.js `running` and `carry`, which sign the yes as the
// chain's next line and carry it. What a computer then serves each person,
// and what /shelf brings, is the node's: crates/svrngs-node/tests/carried.rs
// a_yes_runs_the_new_version_for_its_person_alone_from_a_computer_or_a_phone
// and a_phone_brings_from_the_shelf_through_the_computer_it_reaches.

import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { register } from 'node:module';
import { test } from 'node:test';
import { runInNewContext } from 'node:vm';
import { open } from './twin.mjs';

register('./layout.mjs', import.meta.url);
const acts = await import('../../page/acts.js?v=__VERSION__');

const AT = 'http://svrngs.localhost:4000';
const [ADA, KIM] = ['a1', 'c3'].map((k) => k.repeat(32));
const AT_CHANGE = 1_792_000_000;

// `names` of the module at `path`, each a top-level declaration cut from
// its text up to the next one, run with `given` in scope, as
// lost-answer.test.mjs cuts lend.js's; what it answers read back as data
// of this realm.
const cut = (path, names, given) => {
  const text = readFileSync(new URL(path, import.meta.url), 'utf8');
  const each = names.map((name) => {
    const start = new RegExp(`^(?:export )?(?:async function ${name}\\b|const ${name} = )`, 'm');
    const at = text.search(start);
    if (at < 0) throw new Error(`${name} not found in ${path}`);
    const rest = text.slice(at);
    const end = rest.slice(1).search(/\n(?:export |const |async function |function |\/\/|\/\*\*)/);
    return rest.slice(0, end < 0 ? undefined : end + 1).replace(/^export /, '');
  });
  return runInNewContext(`${each.join('\n')}\n({ ${names.join(', ')} })`, {
    ...given,
    Blob,
    JSON,
    Math,
    Date,
    Error,
    Promise,
    TextEncoder,
  });
};
const data = (value) => JSON.parse(JSON.stringify(value));

// browser::carried::a_new_version_runs_as_each_person_only_after_their_yes,
// ada's home page: kim changed Money's files (carried.rs `changed`, by name
// and first key, in /v1/lends); the page asks in one line, its fold names
// who and when; Open the new version posts [group, app] to /runs, and says
// it opens at its new version once the computer took it.
test('a new version is asked of the home page, and its yes posts /runs and is said', async () => {
  const answers = {
    'GET /v1/groups': {
      key: ADA,
      groups: [
        {
          id: 'house',
          name: 'house',
          person: 'ada',
          removed: false,
          removals: [],
          left: false,
          keeps: true,
          made: null,
          folders: [],
          waiting: [],
          visiting: [],
          fallback: null,
        },
      ],
    },
    'GET /v1/rail/people?namespace=house': {
      people: [
        { person: 'ada', person_key: ADA, key: ADA, device: 'computer', joined: 'started' },
        { person: 'kim', person_key: KIM, key: KIM, device: 'computer', joined: { at: 1 } },
      ],
    },
    'GET /v1/lends?namespace=house': {
      lent: [],
      unanswered: [],
      unheard: [],
      refused: [],
      older: [],
      shelf: [],
      changed: [{ group: 'house', app: 'Money', by: 'kim', by_key: KIM, at: AT_CHANGE }],
    },
    'GET /publication?group=house': { published: false },
    'GET /lent': { lent: [], signing: [], readers: {} },
    'GET /reaching': {},
    'GET /v1/room': [],
    'GET /v1/play': [],
    'GET /update': { state: 'current' },
    'POST /runs': {},
  };
  const page = await open(`${AT}/?ui=a`, answers);
  assert.deepEqual(page.unanswered, []);
  const asked = page.text('#acts')[0];
  assert.ok(asked.includes('1 app changed. See what’s new'), asked);
  assert.ok(asked.includes('Money: kim changed its files on'), asked);
  await page.press('#acts button', 'Open the new version');
  assert.deepEqual(
    page.asked.filter(([method]) => method === 'POST'),
    [['POST', '/runs', ['house', 'Money']]],
  );
  const said = page.text('#acts')[0];
  assert.ok(said.includes('Money opens at its new version for you now.'), said);
});

// The phone's act at a computer: `through` posts [group id, body] to the
// computer `reach` reached, its answer read as JSON, a refusal its words.
const posting = (answer) => {
  const asked = [];
  const connection = {
    fetch: async (method, path, headers, stream) => {
      const body = stream ? await new Response(stream).text() : null;
      asked.push([method, path, Object.fromEntries(headers), body]);
      return new Response(JSON.stringify(answer.body), { status: answer.status });
    },
    close: () => asked.push(['closed']),
  };
  const { through } = cut('../../page/dial.js', ['exchange', 'through'], {
    wasm: async () => ({}),
    reach: async () => [connection],
    trace: () => {},
    Response,
  });
  return { through, asked };
};

// browser::carried::the_phone_brings_things_to_the_table: the shelf's
// bring (web/page/shelf.js `bring`) is `through` at /shelf; and the phone's
// yes to a new version is `through` at /runs. That the computer the phone
// reaches then brings them, or runs the new version for the phone's
// person, is tests/carried.rs's.
test('the phone posts a group’s act to the computer it reaches, its answer read and a refusal said', async () => {
  const group = { id: 'house' };
  const brought = posting({ status: 200, body: [{ app: 'Money' }, { app: 'Events' }] });
  assert.deepEqual(data(await brought.through(group, '/shelf', ['Money', 'Events'])), [
    { app: 'Money' },
    { app: 'Events' },
  ]);
  const body = JSON.stringify(['house', ['Money', 'Events']]);
  assert.deepEqual(brought.asked, [
    [
      'POST',
      '/shelf',
      { 'content-type': 'application/json', 'content-length': String(body.length) },
      body,
    ],
    ['closed'],
  ]);
  const refused = posting({ status: 403, body: { error: 'Only a member can bring.' } });
  await assert.rejects(
    refused.through(group, '/runs', 'Money'),
    /^Error: Only a member can bring\.$/,
  );
  assert.equal(refused.asked[0][3], JSON.stringify(['house', 'Money']));
});

// browser::carried::a_new_version_runs_as_each_person_only_after_their_yes,
// bo's phone: the yes pressed under a change of this group is `runs` of
// that app, one at a time; the drawing of the change and of the line said
// once done is pages/acts.test.mjs's (ui/changes.js `Changes`, both pages).
test('the phone’s yes to a new version runs it, one act at a time', () => {
  const group = { id: 'house' };
  const changed = [{ group: 'house', app: 'Money', by: 'kim', at: AT_CHANGE }];
  const state = { at: { doing: 'idle' }, group, changed, acted: {} };
  const step = acts.takes.yes(state, { app: 'Money' });
  assert.deepEqual(step.effects, [{ run: 'runs', group, app: 'Money', failed: 'unacted' }]);
  assert.deepEqual(step.state.acted['yes Money'], { c: changed[0] });
  assert.ok(acts.takes.yes(step.state, { app: 'Money' }).refused, 'a second while one is out');
});

// The phone's yes to a new version (web/page/acts.js `running`): the act
// named at the computer (/runs), signed here as the chain's next line after
// the last kept, kept unsent, then carried (/carry), none unsent after; a
// carry refused keeps the line unsent, said why.
test('the phone’s yes to a new version is signed as its next line and carried', async () => {
  const sign = { act: 'runs', app: 'Money' };
  const run = (carryAnswer) => {
    const asked = [];
    const signed = [];
    const { running } = cut('../../page/acts.js', ['carry', 'running'], {
      wasm: async () => ({
        sign_line: (key, id, now, seq, prev, act) => {
          signed.push([key, id, seq, prev, JSON.parse(act)]);
          return JSON.stringify({ seq, id: `line-${seq}`, act });
        },
      }),
      store: { unseal: async () => 'KEY', opened: async (group) => group },
      through: async (group, path, body) => {
        asked.push([path, body]);
        if (path === '/runs') return { sign };
        return carryAnswer();
      },
    });
    return { running, asked, signed };
  };
  const group = { id: 'house', line: { seq: 4, id: 'line-4' }, unsent: [] };
  const done = run(async () => ({}));
  const acted = await done.running(group, 'Money');
  assert.deepEqual(data(done.signed), [['KEY', 'house', 5, 'line-4', sign]]);
  assert.deepEqual(
    done.asked.map(([path]) => path),
    ['/runs', '/carry'],
  );
  assert.equal(done.asked[0][1], 'Money');
  assert.equal(done.asked[1][1].seq, 5);
  assert.equal(acted.type, 'acted');
  assert.deepEqual(data([acted.group.line, acted.group.unsent]), [{ seq: 5, id: 'line-5' }, []]);
  const kept = run(async () => Promise.reject(new Error('ada’s computer did not answer.')));
  const unacted = await kept.running(group, 'Money');
  assert.equal(unacted.type, 'unacted');
  assert.equal(unacted.why, 'ada’s computer did not answer.');
  assert.deepEqual(data(unacted.group.unsent.map((line) => line.seq)), [5]);
});
