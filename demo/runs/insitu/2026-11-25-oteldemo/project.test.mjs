// The projection's laws, each with the record that would break it.
import assert from 'node:assert/strict';
import test from 'node:test';
import { contract, project } from './project.mjs';

const KEY = 'site-key';
const s = (k, v) => ({ key: k, value: typeof v === 'number' ? { intValue: String(v) } : { stringValue: v } });
const span = (over = {}) => JSON.stringify({
  resourceSpans: [{ resource: { attributes: [s('service.name', 'notes-to-forms'), s('service.version', '0.3.0'), s('host.name', 'clinic-ws-Jordan')] },
    scopeSpans: [{ spans: [{ traceId: 'aa'.repeat(16), spanId: 'bb'.repeat(8), name: 'note', status: { code: 2 }, attributes: [s('note.file', '2026-11-03 Jordan Kowalski - session.txt'), s('input.ref', 'd186993bfa60'), s('outcome', 'error'), s('http.response.status_code', 422)],
      events: [{ name: 'exception', attributes: [s('exception.type', 'ValueError'), s('exception.message', "time data 'Nov 3, 2026 - Jordan Kowalski' does not match"), s('exception.stacktrace', 'Traceback (most recent call last):\n  File "/srv/notes-to-forms/app/notes.py", line 22, in parse\n    form["date"] = datetime.strptime(form["date"], "%Y-%m-%d")\nValueError: time data \'Nov 3, 2026 - Jordan Kowalski\'')] }], ...over }] }] }],
});
const C = contract({ attributes: { 'input.ref': 'hash', outcome: ['submitted', 'rejected', 'error', 'held', 'unjudged'] }, spans: ['note', 'parse'] });
const all = (x) => JSON.stringify(x);

test('a name never passes: not in an attribute, a resource, a message or a stack trace', () => {
  const out = all(project(span(), C, KEY));
  for (const w of ['Jordan', 'Kowalski', 'Nov 3', 'strptime']) assert.ok(!out.includes(w), `${w} leaked: ${out}`);
});

test('what the contract types passes, and what the site knows', () => {
  const [p] = project(span(), C, KEY);
  assert.equal(p.attrs['input.ref'], 'd186993bfa60');
  assert.equal(p.attrs.outcome, 'error');
  assert.equal(p.attrs['http.response.status_code'], 422);
  assert.equal(p.events[0].type, 'ValueError');
  assert.deepEqual(p.events[0].at, ['notes.py:22 parse']);
  assert.equal(p.events[0].message, 'dropped');
  assert.equal(p.name, 'note');
});

test('a word outside the contract\'s set is shaped, so a set cannot be widened by the app', () => {
  const [p] = project(span({ attributes: [s('outcome', 'Jordan Kowalski')] }), C, KEY);
  assert.equal(p.attrs.outcome.shape, true);
  assert.deepEqual(p.attrs.outcome.classes, ['upper', 'lower', 'space']);
});

test('a hash that is not hex is shaped', () => {
  const [p] = project(span({ attributes: [s('input.ref', 'Jordan')] }), C, KEY);
  assert.equal(p.attrs['input.ref'].shape, true);
});

test('equal values shape alike, so they can be matched; the key decides the hash', () => {
  const a = project(span(), C, KEY)[0].attrs['note.file'], b = project(span(), C, KEY)[0].attrs['note.file'];
  assert.equal(a.h, b.h);
  assert.notEqual(project(span(), C, 'another site')[0].attrs['note.file'].h, a.h);
});

test('a log body is shaped, its severity kept', () => {
  const line = JSON.stringify({ resourceLogs: [{ resource: { attributes: [] }, scopeLogs: [{ logRecords: [{ severityText: 'WARNING', body: { stringValue: 'claim rejected for TEST-1003: {"echo":{"client_id":"TEST-1003"}}' } }] }] }] });
  const [l] = project(line, C, KEY);
  assert.equal(l.severity, 'WARNING');
  assert.ok(!all(l).includes('TEST-1003'));
});

test('a contract that types a key as free text is refused', () => {
  assert.throws(() => contract({ attributes: { 'note.file': 'string' } }), /a type is int, hash or a list of words/);
  assert.throws(() => contract({ attributes: { x: Array.from({ length: 65 }, (_, i) => `w${i}`) } }), /at most 64/);
});

// --- The semantic-conventions profile -------------------------------------

const any = (name, attributes, extra = {}) => JSON.stringify({ resourceSpans: [{ resource: { attributes: [s('service.name', 'cart')] }, scopeSpans: [{ spans: [{ traceId: 'cc'.repeat(16), spanId: 'dd'.repeat(8), name, kind: 2, attributes, ...extra }] }] }] });

test('a span instrumented by the conventions says what it did, with no contract', () => {
  const [p] = project(any('oteldemo.CartService/GetCart', [s('rpc.system', 'grpc'), s('rpc.service', 'oteldemo.CartService'), s('rpc.method', 'GetCart'), s('rpc.grpc.status_code', 2), s('server.address', 'valkey-cart'), s('db.system', 'redis')]), contract(), KEY);
  assert.equal(p.name, 'oteldemo.CartService/GetCart');
  assert.equal(p.span_kind, 'server');
  assert.deepEqual([p.attrs['rpc.service'], p.attrs['rpc.method'], p.attrs['rpc.grpc.status_code'], p.attrs['server.address'], p.attrs['db.system']], ['oteldemo.CartService', 'GetCart', 2, 'valkey-cart', 'redis']);
});

test('a name or route that holds data is shaped whole', () => {
  for (const name of ['GET /patients/jordan.kowalski@clinic.org', 'GET /api/products/OLJCESPC7Z', 'GET /orders/48151623', 'GET /search?q=kowalski']) {
    const [p] = project(any(name, [s('http.route', name.split(' ')[1])]), contract(), KEY);
    assert.equal(p.name.shape, true, name);
    assert.equal(p.attrs['http.route'].shape, true, name);
  }
  assert.equal(project(any('GET /api/products/{id}', [s('http.route', '/api/products/{id}')]), contract(), KEY)[0].name, 'GET /api/products/{id}');
});

test('keys that carry data by nature always shape, whatever their value', () => {
  const [p] = project(any('HTTP GET', [s('url.full', 'http://x/api/cart?userId=1'), s('db.query.text', "SELECT * FROM users WHERE email='a@b.c'"), s('client.address', '10.0.0.7'), s('user.email', 'jordan@clinic.org'), s('app.payment.card_number', '4111111111111111')]), contract(), KEY);
  for (const k of ['url.full', 'db.query.text', 'client.address', 'user.email', 'app.payment.card_number']) assert.equal(p.attrs[k].shape, true, k);
});

test('a metric keeps its name, unit and numbers; its attributes are projected', () => {
  const line = JSON.stringify({ resourceMetrics: [{ resource: { attributes: [s('service.name', 'email')] }, scopeMetrics: [{ metrics: [{ name: 'process.runtime.memory.rss', unit: 'By', gauge: { dataPoints: [{ asInt: '734003200', timeUnixNano: '1795600000000000000', attributes: [s('user.email', 'jordan@clinic.org'), s('db.system', 'postgresql')] }] } }] }] }] });
  const [m] = project(line, contract(), KEY);
  assert.equal(m.name, 'process.runtime.memory.rss');
  assert.equal(m.unit, 'By');
  assert.equal(m.points[0].value, 734003200);
  assert.equal(m.points[0].attrs['user.email'].shape, true);
  assert.equal(m.points[0].attrs['db.system'], 'postgresql');
});

test('stack traces keep code locations in each runtime\'s form, never the message', () => {
  const st = 'java.lang.IllegalStateException: card 4111111111111111 for jordan@clinic.org\n\tat oteldemo.AdService.getAds(AdService.java:212)\n' +
    "Error: no user jordan@clinic.org\n    at chargeCard (/app/charge.js:37:11)\n    at async Server.handler (/app/index.js:12:5)\n" +
    "System.Exception: Cart failure for jordan\n   at cart.cartstore.ValkeyCartStore.GetCartAsync(String userId) in /app/src/cartstore/ValkeyCartStore.cs:line 241";
  const [p] = project(any('x', [], { events: [{ name: 'exception', attributes: [s('exception.type', 'java.lang.IllegalStateException'), s('exception.message', 'card 4111111111111111'), s('exception.stacktrace', st)] }] }), contract(), KEY);
  assert.deepEqual(p.events[0].at, ['AdService.java:212 oteldemo.AdService.getAds', 'ValkeyCartStore.cs:241 cart.cartstore.ValkeyCartStore.GetCartAsync', 'charge.js:37 chargeCard', 'index.js:12 Server.handler']);
  assert.ok(!JSON.stringify(p).includes('4111') && !JSON.stringify(p).includes('jordan'));
});

test('the control arm\'s RAW key keeps every value as written, and nothing else does', async () => {
  const { RAW } = await import('./project.mjs');
  const [p] = project(span(), C, RAW);
  assert.equal(p.attrs['note.file'], '2026-11-03 Jordan Kowalski - session.txt');
  assert.match(p.events[0].message, /Jordan Kowalski/);
  assert.ok(!JSON.stringify(project(span(), C, 'RAW')).includes('Jordan'));
});

test('calibrated on the demo\'s clean traffic: a version segment and parentheses are code', () => {
  for (const n of ['flagd.evaluation.v2.Service/ResolveBoolean', 'render route (pages) /']) assert.equal(project(any(n, []), contract(), KEY)[0].name, n, n);
  for (const n of ['GET /orders/v12345', 'GET /a/(4111111111111111)']) assert.equal(project(any(n, []), contract(), KEY)[0].name.shape, true, n);
});

test('calibrated: runtime metric names and long function names are code; a word with a number is not', () => {
  for (const n of ['nodejs.eventloop.delay.p99', 'v8js.memory.heap.space.available_size', 'prepareOrderItemsAndShippingQuoteFromCart']) assert.equal(project(any(n, []), contract(), KEY)[0].name, n, n);
  for (const n of ['GET /users/jordan99', 'GET /c/OLJCESPC7Z']) assert.equal(project(any(n, []), contract(), KEY)[0].name.shape, true, n);
});

// --- The benchmark's five findings, each held -----------------------------

test('a shape gives a length band, so a number shows growth but not its value', async () => {
  const { band } = await import('./project.mjs');
  const len = (v) => project(any('x', [s('demo.product.count', v)]), contract(), KEY)[0].attrs['demo.product.count'].len;
  assert.equal(len('109.78'), len('9999.9')); // an amount: 6 and 6 characters, one band
  assert.equal(len('12.50'), len('1234.56')); // amounts 100x apart, within one band
  assert.notEqual(len('10'), len('16181115')); // F3b's growth, 2 digits to 8, still shows
  assert.deepEqual([band(0), band(1), band(2), band(3), band(4), band(7), band(8), band(15), band(16)], ['0', '1', '2-3', '2-3', '4-7', '4-7', '8-15', '8-15', '16-31']);
});

test('an event named like a message is shaped; conventional and one-word names pass', () => {
  const ev = (name) => project(any('x', [], { events: [{ name, attributes: [] }] }), contract(), KEY)[0].events[0].name;
  assert.equal(ev('Error: Product Catalog Fail Feature Flag Enabled').shape, true);
  assert.equal(ev('declined for Jordan Kowalski').shape, true);
  for (const n of ['feature_flag.evaluation', 'fetchStart', 'responseEnd', 'message']) assert.equal(ev(n), n);
});

test('a metric attribute from its closed set passes; anything else under that key shapes', () => {
  const pt = (k, v) => project(JSON.stringify({ resourceMetrics: [{ resource: { attributes: [] }, scopeMetrics: [{ metrics: [{ name: 'process.cpu.time', sum: { dataPoints: [{ asDouble: 0.9, attributes: [s(k, v)] }] } }] }] }] }), contract(), KEY)[0].points[0].attrs[k];
  assert.equal(pt('type', 'user'), 'user');
  assert.equal(pt('type', 'involuntary'), 'involuntary');
  assert.equal(pt('system.memory.state', 'used'), 'used');
  assert.equal(pt('v8js.heap.space.name', 'old_space'), 'old_space');
  assert.equal(pt('type', 'Jordan Kowalski').shape, true);
  assert.equal(pt('state', 'jordan@clinic.org').shape, true);
});

test('a convention written another way is judged as that convention', () => {
  const [p] = project(any('x', [s('feature_flag_key', 'productCatalogFailure'), s('feature_flag_provider_name', 'flagd'), s('feature_flag_key_extra', 'Jordan'), s('rpc_service', 'oteldemo.CartService')]), contract(), KEY);
  assert.equal(p.attrs.feature_flag_key, 'productCatalogFailure');
  assert.equal(p.attrs.feature_flag_provider_name, 'flagd');
  assert.equal(p.attrs.rpc_service, 'oteldemo.CartService');
  assert.equal(p.attrs.feature_flag_key_extra.shape, true);
  assert.equal(project(any('x', [s('feature_flag_key', 'jordan@clinic.org')]), contract(), KEY)[0].attrs.feature_flag_key.shape, true);
});

test('a log with no time of its own takes the time it was observed', () => {
  const line = JSON.stringify({ resourceLogs: [{ resource: { attributes: [] }, scopeLogs: [{ logRecords: [{ timeUnixNano: '0', observedTimeUnixNano: '1795649437949000000', severityText: 'INFO', body: { stringValue: 'x' } }] }] }] });
  assert.equal(project(line, contract(), KEY)[0].t, 1795649437949);
});
