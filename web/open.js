// SPDX-License-Identifier: AGPL-3.0-or-later
// A lend's origin's start (DESIGN §3, "Lent apps", calls 1 and 6;
// §7). The group's page opens `open.html#<what it hands>` here: the pass
// this origin's holder holds for this lend and the lends it reads, the
// holder's bytes when this origin keeps none, the lends, the relay and the
// page to ask for another pass — never the person's key, which no lend's
// origin holds. Kept in this origin's store (store.js `lendKeep`), handed
// to the lend's worker in memory (Safari's worker on an iPhone read nothing
// its page had kept, 2026-10-05), and the lend opened, which the worker
// serves.
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
import {
  fromHex,
  holderMatches,
  openMessage,
  pkcs8,
  signerFor,
} from './authority.js?v=__VERSION__';
/** @import { Handoff } from './authority.js?v=__VERSION__' */
// The open as this page holds it: the hand-off, the holder's key once
// imported or found kept, and the lines the last pass's holder signed.
/**
 * @typedef {Handoff & {
 *   key?: CryptoKey,
 *   line?: { seq: number, id: string },
 *   unsent?: unknown[],
 * }} Handed
 */

// How long, in ms, the group's signer has to answer whether it issued an open.
const CHECKING = 15_000;

// `words` said to the person, in open.html's #said.
/** @param {string} words */
function say(words) {
  const said = /** @type {HTMLElement} */ (document.getElementById('said'));
  said.textContent = words;
}

// The hand-off, base64url JSON in the fragment, read as UTF-8.
/**
 * @param {string} fragment
 * @returns {Handed}
 */
function decode(fragment) {
  const characters = atob(fragment.replace(/-/g, '+').replace(/_/g, '/'));
  const utf8 = Uint8Array.from(characters, (char) => char.charCodeAt(0));
  return JSON.parse(new TextDecoder().decode(utf8));
}

// The holder's private key, from the seed the group's page handed as hex.
/** @param {string} seed */
const holderKey = (seed) => pkcs8(fromHex(seed));

// The worker `registration` runs, once it runs.
/**
 * @param {ServiceWorkerRegistration} registration
 * @returns {Promise<ServiceWorker>}
 */
function running(registration) {
  // Just registered, it is one of the three.
  const worker = /** @type {ServiceWorker} */ (
    registration.installing ?? registration.waiting ?? registration.active
  );
  return new Promise((done, failed) => {
    if (worker.state === 'activated') {
      done(worker);
      return;
    }
    worker.addEventListener('statechange', () => {
      if (worker.state === 'activated') done(worker);
      // An install that could not save its pieces (lend.js `installed`):
      // an older worker goes on, and with none this cannot open.
      if (worker.state === 'redundant') {
        if (registration.active) done(registration.active);
        else failed(new Error('the lend’s worker did not install'));
      }
    });
  });
}

// Whether this browser's group page issued `handed`, said to the person
// when it did not.
/** @param {Handoff} handed */
async function checked(handed) {
  const yes = await issued(handed);
  if (!yes) say('Open this app again from your group’s page.');
  return yes;
}

// Whether this browser's group page issued `handed`: the group's signer,
// framed at the address derived from this origin's own host, adopts the
// open's approver on its first ask, or finds the one it adopted
// (sign.js; svrn-docs rings/current/HOLDER_APPROVAL.md §3.D).
/**
 * @param {Handoff} handed
 * @param {string} [saved] the build this origin holds whole, for the signer to mark
 * @returns {Promise<boolean>}
 */
async function issued(handed, saved) {
  const url = signerFor(location.href, new URL('./', import.meta.url).pathname);
  const frame = document.createElement('iframe');
  frame.hidden = true;
  frame.src = url;
  const channel = new MessageChannel();
  try {
    return await new Promise((yes, no) => {
      const timeout = setTimeout(
        () => no(new Error('the group page did not verify this open')),
        CHECKING,
      );
      channel.port1.onmessage = ({ data }) => {
        clearTimeout(timeout);
        yes(data.opened === true);
      };
      const opening = { ...JSON.parse(openMessage(handed)), proof: handed.proof };
      frame.onload = () => {
        // Loaded, the frame has its window.
        const signer = /** @type {Window} */ (frame.contentWindow);
        signer.postMessage({ opening, saved }, new URL(url).origin, [channel.port2]);
      };
      document.body.append(frame);
    });
  } finally {
    frame.remove();
  }
}

// The open this page starts from, and the holder's bytes when they came
// with it, to hand the worker should it not take the key.
/** @typedef {{ handed: Handed, seed?: string | null }} Begun */

async function start() {
  const asked = new URLSearchParams(location.search);
  const begun = location.hash.length > 1 ? await handedHere() : await handedKept(asked);
  if (!begun) return;
  const { handed } = begun;
  // `withHolder` gave it its key, or it was kept with one.
  if (!(await holderMatches(/** @type {CryptoKey} */ (handed.key), handed.holder)))
    throw new Error('the key is not the holder the group page named');
  await store.lendKeep(`pass ${handed.label}`, handed);
  console.debug(`OPEN: ${handed.label}'s open is this browser's, its holder proved and kept`);
  if (!('serviceWorker' in navigator)) {
    say('This browser cannot open it.');
    return;
  }
  await handedOn(begun, asked);
}

// The open the group's page handed in this page's fragment, checked as
// its, with its holder; none once this page has said why it stops, or
// gone to the group's page for a new holder.
/** @returns {Promise<Begun | null>} */
async function handedHere() {
  const handed = decode(location.hash.slice(1));
  history.replaceState(null, '', `${location.pathname}${location.search}`);
  if (store.labelOf(location.hostname) !== handed.label) {
    throw new Error(`a pass for ${handed.label} opened at ${location.origin}`);
  }
  if (!(await checked(handed))) return null;
  // Lines the last pass's holder signed and no computer took yet go on
  // with this one (lend.js `writing`), each under the pass it was signed with.
  const was = /** @type {Handed | undefined | null} */ (
    await store.lendKept(`pass ${handed.label}`).catch(() => null)
  );
  if (was?.unsent && was.unsent.length > 0) handed.unsent = was.unsent;
  if (!(await withHolder(handed, was))) {
    location.replace(`${handed.page}#open=${handed.label}&anew`);
    return null;
  }
  const seed = handed.seed;
  if (!handed.raw) delete handed.seed;
  return { handed, seed };
}

// The holder, a key no script here can copy (DESIGN §3): imported once from the bytes the group's page hands, kept
// as itself, the bytes kept only when a lend of the open is at a door
// taking no holder's link, which the worker dials as. A pass to the
// holder kept (`was`) goes on with its key and its chain; one to a holder
// not kept here asks the group's page for a new one: whether `handed`
// has its holder.
/**
 * @param {Handed} handed
 * @param {Handed | undefined | null} was
 */
async function withHolder(handed, was) {
  if (handed.seed) {
    const key = holderKey(handed.seed);
    handed.key = await crypto.subtle.importKey('pkcs8', key, 'Ed25519', false, ['sign']);
    return true;
  }
  if (was?.key && was.holder === handed.holder) {
    handed.key = was.key;
    handed.line = was.line;
    return true;
  }
  return false;
}

// The open this origin kept for the lend `for` names, asked for by its
// worker, which holds none; none once this page has said why it stops, or
// gone to the group's page for another pass (`again`).
/**
 * @param {URLSearchParams} asked
 * @returns {Promise<Begun | null>}
 */
async function handedKept(asked) {
  const name = asked.get('for');
  const handed = name
    ? /** @type {Handed | undefined} */ (await store.lendKept(`pass ${name}`))
    : undefined;
  if (!handed) {
    say('Open this from your group’s page.');
    return null;
  }
  if (!(await checked(handed))) return null;
  if (asked.has('again')) {
    location.replace(`${handed.page}#open=${handed.label}`);
    return null;
  }
  return { handed };
}

// The open handed to the lend's worker, the key, else the holder's bytes,
// and the lend opened at `back`, a path of this origin's.
/**
 * @param {Begun} begun
 * @param {URLSearchParams} asked
 */
async function handedOn({ handed, seed }, asked) {
  const script = new URL('./lend.js?v=__VERSION__', import.meta.url).pathname;
  const worker = await running(
    await navigator.serviceWorker.register(script, { scope: '/', type: 'module' }),
  );
  console.debug('OPEN: the lend’s worker runs; handing it the pass');
  // A mark for the group's readiness check, never a reason not to open.
  if (await store.whole())
    await issued(handed, '__VERSION__').catch((no) => console.debug(`OPEN: ${no}`));
  let held = await store.handOver(worker, { handed });
  // Safari's worker loses the key (above): the seed's bytes go in its place.
  if (!held && seed) {
    held = await store.handOver(worker, { handed: { ...handed, key: holderKey(seed) } });
  }
  if (!held && !seed) {
    console.debug(
      'OPEN: the worker did not take the holder’s key, and no bytes of it are here: the group’s page signs a pass to a new holder',
    );
    const at = encodeURIComponent(asked.get('back') ?? '/');
    location.replace(`${handed.page}#open=${handed.label}&anew&at=${at}`);
    return;
  }
  if (!held) throw new Error('the lend’s worker did not say it holds the pass');
  // A path of this origin's alone: the group's page hands one a lend asked for.
  const back = new URL(asked.get('back') ?? '/', location.href);
  location.replace(back.origin === location.origin ? back.href : '/');
}

start().catch((error) => {
  console.error(`OPEN: ${error}`);
  say('This could not be opened. Open it again from your group’s page.');
});
