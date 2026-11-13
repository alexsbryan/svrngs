// What keeps a phone from going on at home (web/store.js `unready`,
// KEEP_GOING_AT_HOME 29), cut from its text and run over a stood-in seal,
// storage and permission, with no browser: the readiness lines
// browser::keep_going::a_phone_reopens_the_group_and_its_app_with_the_public_site_gone
// read in Chromium, which keeps what a real Cache and restart do.

import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { test } from 'node:test';
import { runInNewContext } from 'node:vm';

const STORE = readFileSync(new URL('../store.js', import.meta.url), 'utf8');
const cut = (pattern, name) => {
  const found = pattern.exec(STORE)?.[0];
  assert.ok(found, `store.js declares ${name}`);
  return found;
};
const SOURCE = [
  cut(/^const LOCAL =[^]*?;\n/m, 'LOCAL'),
  cut(/^export async function unready\([^]*?\n\}\n/m, 'unready').replace(/^export /, ''),
  '({ unready })',
].join('\n');

/**
 * `unready` as a page whose seal holds `saved`, each mark the build its
 * signer saved at (store.js `keepSaved`), and whose browser keeps its
 * storage or not, `local` the home-network permission's state, or none
 * when the browser names no such permission.
 */
const unreadyIn = ({ saved = {}, keeps = true, local }) => {
  const { unready } = runInNewContext(SOURCE, {
    inSeal: async (key) => saved[key],
    navigator: {
      storage: { persisted: async () => keeps },
      permissions: {
        query: async () => {
          if (local === undefined) throw new TypeError('no such permission');
          return { state: local };
        },
      },
    },
  });
  // Its answer read into this realm.
  return async (lent, named) => [...(await unready(lent, named))];
};

const films = { label: 'films0lbl', name: 'films' };
const named = (lend) => lend.name;
const LOCAL =
  'This browser asks before it reaches devices on your home network. Allow it, so this phone can find your group when the internet is out.';

test('an app not saved, storage not kept and the home network not let in are each said, until they are', async () => {
  assert.deepEqual(await unreadyIn({ local: 'granted' })([films], named), [
    'films is not saved here: open it once while online.',
  ]);
  assert.deepEqual(
    await unreadyIn({ saved: { 'saved films0lbl': '__VERSION__' }, local: 'granted' })(
      [films],
      named,
    ),
    [],
    'an app its signer marked saved, at this build',
  );
  assert.deepEqual(
    await unreadyIn({ saved: { 'saved films0lbl': 'an older build' }, local: 'granted' })(
      [films],
      named,
    ),
    ['films is not saved here: open it once while online.'],
    'a new build asks it again',
  );
  for (const state of ['denied', 'prompt'])
    assert.deepEqual(await unreadyIn({ local: state })([], named), [LOCAL], state);
  assert.deepEqual(await unreadyIn({})([], named), [], 'a browser that names no such permission');
  assert.deepEqual(await unreadyIn({ keeps: false, local: 'granted' })([], named), [
    'The browser has not agreed to keep what this page saves.',
  ]);
});
