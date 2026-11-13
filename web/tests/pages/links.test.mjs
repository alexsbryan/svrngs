// A link a lent app writes into itself, `<page>#open=<label>&at=<path>`
// (events-links (c)), opened on a computer whose browser keeps no group:
// the group's page says it opens in svrngs there and shows the link to
// copy (web/page/computer.js `opens`, computer-line.js). Composed and
// stepped as enter.test.mjs steps the page. Moved from browser/main.rs
// `a_lent_app_writes_a_link_into_itself_that_opens_on_a_phone`.
import assert from 'node:assert/strict';
import { register } from 'node:module';
import { test } from 'node:test';

register('./layout.mjs', import.meta.url);
const { compose } = await import('../../page/page.js?v=__VERSION__');
const FLOWS = ['enter', 'computer', 'join', 'ask', 'index', 'door', 'shelf', 'visit', 'among'];
const parts = compose(
  Object.fromEntries(
    await Promise.all(
      FLOWS.map(async (f) => [f, await import(`../../page/${f}.js?v=__VERSION__`)]),
    ),
  ),
);

test('a lend’s link on a computer keeping no group is said to open in svrngs, and shown', () => {
  const link = 'https://group.example:8443/#open=app-label&at=/at/a/path?n=1';
  const fragment = link.split('#')[1];
  const looked = parts.step(parts.first({ fine: true }), {
    type: 'looked',
    fragment,
    kept: [],
    linked: true,
  });
  const sent = looked.effects.map((f) => f.m);
  assert.deepEqual(
    sent.map((m) => m?.type),
    ['opens'],
  );
  const opens = parts.step(looked.state, sent[0]);
  assert.deepEqual(
    opens.effects.map((f) => [f.run, f.opens]),
    [['where', true]],
  );
  // What computer.js's io answers `where` with: the page's own address, and
  // the link as the browser holds it.
  const offered = parts.step(opens.state, {
    type: 'offered',
    page: 'https://group.example:8443/',
    href: link,
  });
  assert.equal(offered.refused, undefined, offered.refused);
  assert.deepEqual(parts.heading(offered.state), [
    'This link opens in svrngs on your computer.',
    'good',
  ]);
  assert.deepEqual(offered.state.line, ['opens', link], 'the link shown in #line, to copy');
});
