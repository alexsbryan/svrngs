// Creator-to-friend C2: the person can identify the app boundary and audience
// before opening. Red against the old generic record wording, before the edit.
import assert from 'node:assert/strict';
import { test } from 'node:test';
import { agreed } from '../../crates/svrngs-node/src/home/question.js?v=__VERSION__';

const rota = { key: 'jo', person: 'Jo', group: 'garden-id', name: 'rota', kind: 'Garden-rota' };

test('an unfamiliar carried app names its own entries and the actual group at consent', () => {
  const words = agreed(rota, [], { name: 'Garden' });
  assert.match(words, /read and write only (those|its own) (shared )?entries/);
  assert.match(words, /as you in Garden\./);
  assert.doesNotMatch(words, /part of the group’s record/);
  assert.doesNotMatch(agreed(rota, [], { name: 'House' }), /Garden/);
});

test('a reader declaration never describes permission to write', () => {
  const words = agreed({ ...rota, reads: ['calendar'] }, [], { name: 'Garden' });
  assert.match(words, /never write/);
  assert.match(words, /as you in Garden\./);
  assert.doesNotMatch(words, /read and write/);
});
