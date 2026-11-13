// leaving.mjs: an error is judged by whose page it came from and the step it
// came in, never by its words.

import assert from 'node:assert/strict';
import { test } from 'node:test';
import { leaving } from './leaving.mjs';

const LEAVE = 'Leaving Jellyfin for HolyArrs';
const leaves = new Map([[LEAVE, ['bo']]]);
const error = (who, at, message) => ({ who, at, message });

test('an error from a page its step leaves is not judged, whatever it says', () => {
  const words = [
    'Unhandled Promise Rejection: TypeError: Load failed',
    'Unhandled Promise Rejection: ChunkLoadError: Loading chunk 7 failed.',
    'anything a later engine says',
  ];
  const errors = words.map((m) => error('bo', LEAVE, m));
  assert.deepEqual(leaving(errors, leaves), { left: errors, judged: [] });
});

test('every other error is judged: another person’s page, or a step that leaves nothing', () => {
  const ana = error('ana', LEAVE, 'Unhandled Promise Rejection: TypeError: Load failed');
  const staying = error('bo', 'Bo asks', 'Unhandled Promise Rejection: TypeError: Load failed');
  const left = error('bo', LEAVE, 'TypeError: Load failed');
  assert.deepEqual(leaving([ana, left, staying], leaves), { left: [left], judged: [ana, staying] });
  assert.deepEqual(leaving([left], new Map()), { left: [], judged: [left] }, 'no step leaves');
});
