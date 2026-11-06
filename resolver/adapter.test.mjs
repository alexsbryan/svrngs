// SPDX-License-Identifier: AGPL-3.0-or-later
// The Vercel hosting adapter's one decision: which IP handler.mjs sees, and
// that a missing token or missing/absent ingress IP refuses service. It drives
// the real handler with a stubbed provider fetch and reads the IP-scoped quota
// key handler.mjs derives (handler.mjs:154), so it judges the adapter, not a
// reimplementation of it.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { test } from 'node:test';

const PREFIX = 'svrngs:{onboarding}';
const hash = (text) => createHash('sha256').update(text).digest('hex');
const quotaKey = (ip) => `${PREFIX}:quota:ip:${hash(ip)}`;
// The EVAL command's fifth key (index 7) is handler.mjs's `quota:${subject}`
// (handler.mjs:229-240); for a lookup the subject is the ingress IP.
const SUBJECT = 7;

// The provider fetch handler.mjs captures at construction. It records the
// command and answers a store refusal, so the adapter's transport is exercised
// end to end without a network.
const seen = [];
globalThis.fetch = async (_url, options) => {
  seen.push(JSON.parse(options.body));
  return Response.json({ result: JSON.stringify({ state: 'ended', phrase: '', secret: false }) });
};

// A fresh adapter instance per case: it reads the environment and builds one
// handler at import, so the token case needs its own module instance. Each
// request's captured command is appended to the shared `seen`.
async function adapter(token) {
  if (token === undefined) delete process.env.SVRNGS_RESOLVER_TOKEN;
  else process.env.SVRNGS_RESOLVER_TOKEN = token;
  const module = await import(`./api/join.mjs?case=${encodeURIComponent(String(token))}`);
  return module.default.fetch;
}

const lookup = (headers) =>
  new Request('https://join.svrnme.sh/acid-acorn-acre-acts-afar', { headers });

async function refused(response) {
  assert.equal(response.status, 503);
  assert.equal(response.headers.get('retry-after'), '60');
  assert.match(await response.text(), /joining is temporarily unavailable/);
}

test('the overwritten x-forwarded-for is the IP handler.mjs sees', async () => {
  const fetch = await adapter('fixture-token');
  const before = seen.length;
  const response = await fetch(lookup({ 'x-forwarded-for': '203.0.113.7' }));
  assert.equal(response.status, 200);
  assert.equal(seen.length, before + 1);
  assert.equal(seen.at(-1)[SUBJECT], quotaKey('203.0.113.7'));
});

test('a client-supplied x-real-ip cannot displace the platform value', async () => {
  const fetch = await adapter('fixture-token');
  await fetch(lookup({ 'x-forwarded-for': '203.0.113.7', 'x-real-ip': '198.51.100.9' }));
  assert.equal(seen.at(-1)[SUBJECT], quotaKey('203.0.113.7'));
});

test('the left-most value of a forwarded chain is the original client', async () => {
  const fetch = await adapter('fixture-token');
  await fetch(lookup({ 'x-forwarded-for': '203.0.113.7, 10.0.0.1' }));
  assert.equal(seen.at(-1)[SUBJECT], quotaKey('203.0.113.7'));
});

test('an absent ingress IP refuses service', async () => {
  const fetch = await adapter('fixture-token');
  await refused(await fetch(lookup({})));
});

test('a malformed ingress IP refuses service', async () => {
  const fetch = await adapter('fixture-token');
  await refused(await fetch(lookup({ 'x-forwarded-for': 'not-an-ip' })));
});

test('a missing server-side token refuses service', async () => {
  const fetch = await adapter(undefined);
  await refused(await fetch(lookup({ 'x-forwarded-for': '203.0.113.7' })));
});
