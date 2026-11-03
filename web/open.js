// SPDX-License-Identifier: AGPL-3.0-or-later
// A lend's origin's start (u5-2; DESIGN §3, "Lent apps", calls 1 and 6;
// §7). The group's page opens `open.html#<what it hands>` here: the pass
// this origin's holder holds for this lend and the lends it reads, the
// holder's bytes when this origin keeps none, the lends, the relay and the
// page to ask for another pass — never the person's key, which no lend's
// origin holds. Kept in this origin's store
// (store.js `lendKeep`), handed to the lend's worker in memory (u4-10:
// Safari's worker on an iPhone read nothing its page had kept), and the
// lend opened, which the worker serves.
//
// The pass is for this origin's lend alone, whose host's first name is its
// label. Anything else is refused, so a lent page cannot open another
// lend's origin with a pass of its own.
//
// Opened with no fragment, by the worker that holds no pass — the browser
// stopped it, and it could not read this origin's store — it reads the pass
// here and hands it over again (`for`, then `back`); or, its pass lapsed
// (`again`), it opens the group's page to sign another.
//
// Safari on an iPhone loses any CryptoKey sent to the worker, `messageerror`
// in its place, while Firefox on the same phone and WebKit on a Mac take it
// (2026-10-30, demo/runs/ios/2026-10-30-observe/keyprobe/phone.log). Where
// the worker does not say it holds the key, the holder's bytes the group's
// page handed go to it in memory alone, never this origin's store, which
// the lent app's scripts read; a pass with no bytes here, the group's page
// signs another, to a new holder (`anew`).
import * as store from './store.js?v=__VERSION__';
import { signerFor, holderMatches, openMessage } from './authority.js?v=__VERSION__';

const say = (words) => (document.getElementById('said').textContent = words);

const decode = (fragment) =>
  JSON.parse(
    new TextDecoder().decode(
      Uint8Array.from(atob(fragment.replace(/-/g, '+').replace(/_/g, '/')), (c) => c.charCodeAt(0)),
    ),
  );

// An Ed25519 key's 32 bytes, `hex`, as PKCS #8 (RFC 8410): the one form
// WebCrypto imports a private Ed25519 key from in both engines.
const pkcs8 = (hex) =>
  Uint8Array.from(`302e020100300506032b657004220420${hex}`.match(/../g), (h) => parseInt(h, 16));

// The worker `reg` runs, once it runs.
function running(reg) {
  const worker = reg.installing ?? reg.waiting ?? reg.active;
  return new Promise((done) => {
    if (worker.state === 'activated') return done(worker);
    worker.addEventListener('statechange', () => worker.state === 'activated' && done(worker));
  });
}

// Whether this browser's group page issued `handed`: the group's signer,
// framed at the address derived from this origin's own host, adopts the
// open's approver on its first ask, or finds the one it adopted
// (sign.js; svrn-docs rings/current/HOLDER_APPROVAL.md §3.D).
async function checked(handed) {
  const url = signerFor(location.href, new URL('./', import.meta.url).pathname);
  const frame = document.createElement('iframe');
  frame.hidden = true;
  frame.src = url;
  const channel = new MessageChannel();
  try {
    return await new Promise((yes, no) => {
      const timeout = setTimeout(
        () => no(new Error('the group page did not verify this open')),
        15000,
      );
      channel.port1.onmessage = ({ data }) => {
        clearTimeout(timeout);
        yes(data.opened === true);
      };
      const opening = { ...JSON.parse(openMessage(handed)), proof: handed.proof };
      frame.onload = () =>
        frame.contentWindow.postMessage({ opening }, new URL(url).origin, [channel.port2]);
      document.body.append(frame);
    });
  } finally {
    frame.remove();
  }
}

async function start() {
  const asked = new URLSearchParams(location.search);
  let handed, seed;
  if (location.hash.length > 1) {
    handed = decode(location.hash.slice(1));
    history.replaceState(null, '', `${location.pathname}${location.search}`);
    if (store.labelOf(location.hostname) !== handed.label)
      throw new Error(`a pass for ${handed.label} opened at ${location.origin}`);
    if (!(await checked(handed))) return say('Open this app again from your group’s page.');
    // Lines the last pass's holder signed and no computer took yet go on
    // with this one (lend.js `writing`), each under the pass it was signed with.
    const was = await store.lendKept(`pass ${handed.label}`).catch(() => null);
    if (was?.unsent?.length) handed.unsent = was.unsent;
    // The holder, a key no script here can copy (capability-signed (H);
    // DESIGN §3): imported once from the bytes the group's page hands, kept
    // as itself, the bytes kept only when a lend of the open is at a door
    // taking no holder's link, which the worker dials as. A pass to the
    // holder kept goes on with its key and its chain; one to a holder not
    // kept here asks the group's page for a new one.
    if (handed.seed)
      handed.key = await crypto.subtle.importKey('pkcs8', pkcs8(handed.seed), 'Ed25519', false, [
        'sign',
      ]);
    else if (was?.key && was.holder === handed.holder)
      [handed.key, handed.line] = [was.key, was.line];
    else return location.replace(`${handed.page}#open=${handed.label}&anew`);
    seed = handed.seed;
    if (!handed.raw) delete handed.seed;
  } else {
    handed = asked.get('for') && (await store.lendKept(`pass ${asked.get('for')}`));
    if (!handed) return say('Open this from your group’s page.');
    if (!(await checked(handed))) return say('Open this app again from your group’s page.');
    if (asked.has('again')) return location.replace(`${handed.page}#open=${handed.label}`);
  }
  if (!(await holderMatches(handed.key, handed.holder)))
    throw new Error('the key is not the holder the group page named');
  await store.lendKeep(`pass ${handed.label}`, handed);
  console.debug(`OPEN: ${handed.label}'s open is this browser's, its holder proved and kept`);
  if (!('serviceWorker' in navigator)) return say('This browser cannot open it.');
  const script = new URL('./lend.js?v=__VERSION__', import.meta.url).pathname;
  const worker = await running(
    await navigator.serviceWorker.register(script, { scope: '/', type: 'module' }),
  );
  console.debug('OPEN: the lend’s worker runs; handing it the pass');
  const held =
    (await store.handOver(worker, { handed })) ||
    (seed && (await store.handOver(worker, { handed: { ...handed, key: pkcs8(seed) } })));
  if (!held && !seed) {
    console.debug(
      'OPEN: the worker did not take the holder’s key, and no bytes of it are here: the group’s page signs a pass to a new holder',
    );
    return location.replace(
      `${handed.page}#open=${handed.label}&anew&at=${encodeURIComponent(asked.get('back') ?? '/')}`,
    );
  }
  if (!held) throw new Error('the lend’s worker did not say it holds the pass');
  // A path of this origin's alone: the group's page hands one a lend asked for.
  const back = new URL(asked.get('back') ?? '/', location.href);
  location.replace(back.origin === location.origin ? back.href : '/');
}

start().catch((e) => {
  console.error(`OPEN: ${e}`);
  say('This could not be opened. Open it again from your group’s page.');
});
