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
const page = 'https://groups.example/';
const token = 'ordinary_invitation';
const address = 'https://join.example/acid-acorn-acre-acts-afar';
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
  assert.match(text, /joining is temporarily unavailable/);
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
  const name = `Sam O'Brien; $(touch NEVER) \`uname\` "friend"`;
  state = parts.step(state, { type: 'typed', field: 'name', value: name }).state;
  const named = parts.step(state, { type: 'named' });
  assert.equal(named.effects.length, 1);
  assert.equal(named.effects[0].m.type, 'offer');
  const offer = parts.step(named.state, named.effects[0].m);
  assert.equal(offer.effects[0].run, 'where');
  state = parts.step(offer.state, { type: 'offered', ...offer.effects[0], page }).state;
  const line = state.line[1];
  assert.match(line, /^curl -fsSL https:\/\/groups.example\/install.sh/);
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
