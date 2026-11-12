// Independent classification witnesses: moving a suite cannot make it disappear
// from the complete gate or quietly introduce Redis/WebKit into the fast tier.
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { readdirSync } from 'node:fs';
import { test } from 'node:test';
import { catalog, select } from './run.mjs';

test('unit, integration and app ownership are disjoint and exhaustive', () => {
  const files = [
    'web/tests/a.test.mjs',
    'web/tests/pages/b.test.mjs',
    'web/tests/style.integration.test.mjs',
    'apps/garden/fold.test.mjs',
    'resolver/adapter.test.mjs',
    'web/tests/helper.mjs',
  ];
  assert.deepEqual(select('unit', files), [
    'resolver/adapter.test.mjs',
    'web/tests/a.test.mjs',
    'web/tests/pages/b.test.mjs',
  ]);
  assert.deepEqual(select('integration', files), ['web/tests/style.integration.test.mjs']);
  assert.deepEqual(select('apps', files), ['apps/garden/fold.test.mjs']);
});

test('the real catalog keeps Redis/WebKit outside unit and loses no web tests', () => {
  const unit = catalog('unit');
  const integration = catalog('integration');
  for (const file of [
    'web/tests/resolver.integration.test.mjs',
    'web/tests/page-server.integration.test.mjs',
  ]) {
    assert.ok(integration.includes(file));
    assert.ok(!unit.includes(file));
  }
  assert.ok(unit.includes('web/tests/question.test.mjs'));
  assert.ok(unit.includes('web/tests/resolver-contracts.test.mjs'));
  assert.ok(unit.includes('resolver/adapter.test.mjs'));
  const all = ['web/tests/', 'resolver/']
    .flatMap((dir) =>
      readdirSync(new URL(`../../${dir}`, import.meta.url), { recursive: true })
        .filter((file) => file.endsWith('.test.mjs'))
        .map((file) => dir + file),
    )
    .sort();
  assert.equal(new Set([...unit, ...integration]).size, all.length);
  assert.deepEqual([...unit, ...integration].sort(), all);
});

test('unknown or empty selections are never-ran, not passing tests', () => {
  assert.throws(() => select('typo', ['web/tests/a.test.mjs']), /never-ran: unknown/);
  for (const tier of ['unit', 'integration', 'apps'])
    assert.throws(() => select(tier, []), /never-ran: no/);
  const child = spawnSync(
    process.execPath,
    [new URL('./run.mjs', import.meta.url).pathname, 'typo'],
    { encoding: 'utf8' },
  );
  assert.equal(child.status, 3);
  assert.match(child.stderr, /never-ran: unknown/);
});
