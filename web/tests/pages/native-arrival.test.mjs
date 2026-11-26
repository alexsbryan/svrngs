// SPDX-License-Identifier: AGPL-3.0-or-later
import assert from 'node:assert/strict';
import { register } from 'node:module';
import { spawnSync } from 'node:child_process';
import { test } from 'node:test';
import { createResolver } from '../../../resolver/handler.mjs';

register('./layout.mjs', import.meta.url);
const { compose } = await import('../../page/page.js?v=__VERSION__');
const flows = Object.fromEntries(
  await Promise.all(
    ['enter', 'join', 'computer'].map(async (name) => [
      name,
      await import(`../../page/${name}.js?v=__VERSION__`),
    ]),
  ),
);
const parts = compose(flows);
const { ComputerLine } = await import('../../page/computer-line.js?v=__VERSION__');
// A drawing's words as a person reads them: what is not hidden, in order.
const said = (v) => {
  if (v == null || typeof v === 'boolean') return '';
  if (Array.isArray(v)) return v.map(said).join('');
  if (typeof v !== 'object') return String(v);
  return v.props.hidden ? '' : said(v.props.children);
};
const page = 'https://groups.example/';
const token = 'ordinary_invitation';
const address = 'https://join.example/acorn-basil-robin-0042';
const secret = { token, page, until: Math.floor(Date.now() / 1000) + 3600 };
const resolve = (result, status = 200) =>
  createResolver({
    token: 'fixture-only',
    fetch: async () => Response.json({ result: JSON.stringify(result) }, { status }),
  })(new Request(address, { headers: { accept: 'text/html' } }), '192.0.2.1');

test('a typed short address redirects only to the canonical page in native mode', async () => {
  const answer = await resolve({
    state: 'open',
    phrase: address.split('/').at(-1),
    secret: JSON.stringify(secret),
  });
  assert.equal(answer.status, 303);
  assert.equal(answer.headers.get('location'), `${page}#${token}&native`);
  assert.equal(answer.headers.get('cache-control'), 'no-store');
  assert.equal(answer.headers.get('referrer-policy'), 'no-referrer');
});

test('a resolver limit renders temporary unavailability and manual retry, never ended', async () => {
  const answer = await resolve({}, 429);
  assert.equal(answer.status, 503);
  assert.match(answer.headers.get('content-type'), /text\/html/);
  const text = await answer.text();
  assert.match(text, /Joining is temporarily unavailable/);
  assert.match(text, /minute/);
  assert.doesNotMatch(text, /invitation has ended|http-equiv="refresh"/);
});

test('native arrival asks the name once and offers a literal shell command without browser admission', () => {
  const initial = parts.first({ fine: true });
  const looked = parts.step(initial, {
    type: 'looked',
    fragment: `${token}&native`,
    kept: [],
    linked: true,
  });
  assert.equal(looked.effects.length, 1);
  assert.equal(looked.effects[0].m.type, 'join');
  let state = parts.step(looked.state, looked.effects[0].m).state;
  state = parts.step(state, {
    type: 'asked',
    told: 'issuer-answer',
    group: { id: 'abcd', name: 'House', maker: 'Founder', inviter: 'Kim' },
  }).state;
  assert.match(parts.heading(state)[0], /Kim.*House/);
  // What comes next, how long it takes and what it leaves, in a person's words.
  const form = said(flows.join.view.join({ s: state, send: () => {} }));
  assert.ok(
    form.includes(
      'Next you paste one line into Terminal. In about a minute, a page opens with House’s shared folder on this computer.',
    ),
    form,
  );
  const name = `Sam O'Brien; $(touch NEVER) \`uname\` "friend"`;
  state = parts.step(state, { type: 'typed', field: 'name', value: name }).state;
  const named = parts.step(state, { type: 'named' });
  assert.equal(named.effects.length, 1);
  assert.equal(named.effects[0].m.type, 'offer');
  const offer = parts.step(named.state, named.effects[0].m);
  assert.equal(offer.effects[0].run, 'where');
  state = parts.step(offer.state, { type: 'offered', ...offer.effects[0], page }).state;
  const line = state.line[1];
  assert.match(line, /^curl -fsSL https:\/\/group\.svrnme\.sh\/install\.sh/);
  const told = said(ComputerLine({ s: state }));
  assert.ok(told.startsWith('Paste this line into Terminal and press Return.'), told);
  assert.ok(told.includes('in about a minute a page opens with the group’s shared folder'), told);
  assert.doesNotMatch(line, /join.example/);
  const args = spawnSync(
    '/bin/sh',
    ['-c', `sh() { printf '%s\\n' "$@"; }; curl() { :; }; ${line}`],
    { encoding: 'utf8' },
  );
  assert.equal(args.status, 0, args.stderr);
  assert.deepEqual(args.stdout.trimEnd().split('\n'), [
    '-s',
    '--',
    `${page}#${token}`,
    '--at',
    '/#group-abcd',
    '--as',
    name,
  ]);
  assert.equal(state.at.join, 'installing');
  assert.equal(parts.step(state, { type: 'named' }).effects.length, 0);
});

// A phone has no terminal, so no line is offered there (computer.js
// `lines`): the short address opens as any group link opens on a phone
// (enter.js `addressed`), and joins it in the browser.
const atShortAddress = (kept, frame = { fine: false }) => {
  const looked = parts.step(parts.first(frame), {
    type: 'looked',
    fragment: `${token}&native`,
    kept,
    linked: true,
  });
  const join = looked.effects.find((effect) => effect.m.type === 'join');
  assert.deepEqual(join.m, { type: 'join', token, kept, wants: null });
  const joining = parts.step(looked.state, join.m);
  assert.deepEqual(joining.effects, [{ run: 'ask', token, native: undefined, failed: 'unjoined' }]);
  return parts.step(joining.state, {
    type: 'asked',
    told: 'issuer-answer',
    group: { id: 'abcd', name: 'House', maker: 'Founder' },
  });
};

test('a phone at a short address joins in the browser, never waits on an installation', () => {
  const asked = atShortAddress([]);
  assert.equal(parts.heading(asked.state)[0], 'Join House');
  const state = parts.step(asked.state, { type: 'typed', field: 'name', value: 'Sam' }).state;
  const named = parts.step(state, { type: 'named' });
  assert.equal(named.state.at.join, 'redeeming');
  assert.equal(named.effects[0].run, 'admit');
  assert.equal(named.effects[0].name, 'Sam');
});

test('a phone in the group already, at its short address, holds it and joins nobody twice', () => {
  const asked = atShortAddress([{ id: 'abcd', name: 'House', token: 'another' }]);
  assert.equal(asked.state.at.join, 'joined');
  assert.equal(asked.effects[0].m.type, 'hold');
});

// A Windows computer takes no line (computer.js `lines`), so a short
// address opens on it as any link (enter.js `addressed`): it is told what
// a Windows computer is told at any link, that svrngs runs on Mac and
// Linux for now and the same address joins from its phone, and it can
// join in the browser, as a phone does.
test('a Windows computer at a short address is offered no line, and told to use its phone', () => {
  const windows = { fine: true, windows: true };
  const looked = parts.step(parts.first(windows), {
    type: 'looked',
    fragment: `${token}&native`,
    kept: [],
    linked: true,
  });
  const [offer] = looked.effects;
  assert.deepEqual(offer.m, { type: 'offer', token });
  const where = parts.step(looked.state, offer.m);
  const told = parts.step(where.state, { type: 'offered', ...where.effects[0], page });
  assert.deepEqual(told.state.line, ['windows', '']);
  assert.equal(parts.heading(atShortAddress([], windows).state)[0], 'Join House');
});
