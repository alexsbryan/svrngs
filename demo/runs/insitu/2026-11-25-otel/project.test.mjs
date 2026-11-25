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
