// SPDX-License-Identifier: AGPL-3.0-or-later
// The hand-over page (u4-10): what the worker answers a lend's address
// with when it holds no key for the group and cannot unseal one (sw.js
// `handOverPage`). Safari's worker on an iPhone read no sealing key where
// its page had kept one (2026-10-05), and a worker the browser stopped has
// forgotten the key the page handed it. Here, in the page, which reads the
// seal, the key is unsealed, handed to the worker in memory (store.js
// `handOver`), and the lend's address opened again, which the worker now
// serves. Once in AGAIN per address: a worker that still holds no key is
// not asked over and over; the page says so instead. Every word a person
// reads here is DESIGN §1's.
import * as store from './store.js?v=__VERSION__';

const AGAIN = 15_000;
const scope = new URL('./', import.meta.url);
const tried = `svrngs hand-over ${location.pathname}`;

async function handOver() {
  const last = Number(sessionStorage.getItem(tried) ?? 0);
  if (Date.now() - last < AGAIN) throw new Error(`handed over ${Date.now() - last} ms ago, and the worker holds no key`);
  const lender = /^l\/([0-9a-f]{64})\//.exec(location.pathname.slice(scope.pathname.length))?.[1];
  const group = lender && (await store.groupFor(lender));
  if (!group) throw new Error(`no group kept reaches ${lender}`);
  const held = await store.handOver(group, await store.unseal(group));
  if (!held) throw new Error(`the worker did not say it holds the key for ${group.id}`);
  console.debug(`HAND-OVER: the worker holds the key for ${group.id}: opening ${location.pathname} again`);
  sessionStorage.setItem(tried, String(Date.now()));
  location.replace(location.href);
}

handOver().catch((e) => {
  console.error(`HAND-OVER: ${e}`);
  const back = document.createElement('a');
  back.href = scope.href;
  back.textContent = 'your group’s page';
  document.getElementById('said').replaceChildren('This phone could not open this. Open it again from ', back, '.');
});
