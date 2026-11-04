import assert from 'node:assert/strict';
import { test } from 'node:test';
import { authorized, destination } from './boundary.mjs';

test('caller authorization is a credential, never loopback or browser metadata', () => {
  const key = 'only-the-interface-holds-this';
  assert.equal(authorized({}, key), false);
  assert.equal(authorized({ authorization: 'Bearer another' }, key), false);
  assert.equal(
    authorized({ authorization: `Bearer ${key}`, origin: 'http://127.0.0.1' }, key),
    false,
  );
  assert.equal(
    authorized({ authorization: `Bearer ${key}`, 'sec-fetch-site': 'same-origin' }, key),
    false,
  );
  assert.equal(authorized({ authorization: `Bearer ${key}` }, key), true);
  assert.equal(authorized({ authorization: 'Bearer ' }, ''), false);
});

test('every accepted request target preserves the selected lend authority', () => {
  const base = 'http://a.localhost:1234/';
  for (const path of [
    'http://elsewhere/leak',
    '//elsewhere/leak',
    '/\\elsewhere/leak',
    '/\n/elsewhere',
    'relative',
    '/x#fragment',
  ]) {
    assert.throws(() => destination(base, path), undefined, path);
  }
  for (const path of [
    '/v1/models',
    '/v7/odd/path?x=1',
    '/a/../v1/chat/completions',
    '/%2felsewhere',
  ]) {
    assert.equal(destination(base, path).origin, new URL(base).origin);
  }
});
