// SPDX-License-Identifier: AGPL-3.0-or-later
// The group's signer (svrn-docs rings/current/HOLDER_APPROVAL.md §3.D),
// framed by a lent app's pages at the group's origin. It runs no app code
// and never loads the person's key. It holds one approver per app, the key
// the group's page minted for that app's open: adopted on the app's first
// ask from the open's seal and the `__Host-` cookie the page left, kept in
// this origin's own store by the app's label, the cookie then deleted. With
// it, it says whether this browser's group page issued an open, and
// approves an app's exact entry while the open's pass is live, by its own
// clock. Every refusal tells the person to open the app again from the
// group's page.
import init, {
  actor,
  approve_line,
  set_checked,
  set_sealed,
} from './wasm/svrngs_web.js?v=__VERSION__';
import {
  base64url,
  ed25519,
  handoffCookie,
  hostCookie,
  openMessage,
  openSeal,
} from './authority.js?v=__VERSION__';
import * as store from './store.js?v=__VERSION__';
/** @import { Handoff } from './authority.js?v=__VERSION__' */

// An open's approver as `approver` keeps it, by its label: the key it
// signs with, its public half, and the digest of the open it was adopted for.
/** @typedef {{ label: string, pub: string, digest: string, key: CryptoKey, until: number }} Approver */

// The runtime, started on the first ask this frame answers and kept for the rest.
/** @type {Promise<unknown> | undefined} */
let loaded;

// A pass's base64url JSON, each character as `atob` answers it. open.js
// `decode` reads its fragment as UTF-8 instead; the two are kept apart, so
// what this reads of a pass stays as it was.
/** @param {string} text */
const decode = (text) => JSON.parse(atob(text.replace(/-/g, '+').replace(/_/g, '/')));

// What an approver remembers of the open it was adopted for.
/** @param {Handoff} handoff */
async function digest(handoff) {
  const message = new TextEncoder().encode(openMessage(handoff));
  return store.hex(new Uint8Array(await crypto.subtle.digest('SHA-256', message)));
}

// This origin's approvers, by an app's label.
/**
 * @template T
 * @param {IDBTransactionMode} mode
 * @param {(store: IDBObjectStore) => IDBRequest<T>} act
 */
const approvers = (mode, act) => store.stored('svrngs-approvers', 'label', mode, act);

// The approver for `label`'s open `handoff`. A cookie for the label that opens the
// open's seal is adopted first: kept, then deleted, so a second frame finds
// the cookie or the record, never neither; the newest open adopted is the
// one approved (last open wins). A cookie that does not open this seal is
// another open's, left for it. Then the record kept for `label`, which must
// be this open's. Records whose pass lapsed go.
/**
 * @param {string} label
 * @param {Handoff} handoff
 * @param {number} now in seconds
 */
async function approver(label, handoff, now) {
  await approvers('readwrite', (objects) => {
    const all = /** @type {IDBRequest<Approver[]>} */ (objects.getAll());
    all.onsuccess = () => {
      for (const record of all.result) {
        if (record.until <= now) objects.delete(record.label);
      }
    };
    return all;
  });
  const name = handoffCookie(label);
  const seed = await openSeal(handoff, hostCookie(document.cookie, name));
  if (seed) {
    const jwk = ed25519(store.hex(seed), actor(seed));
    const key = await crypto.subtle.importKey('jwk', jwk, 'Ed25519', false, ['sign']);
    const record = {
      label,
      pub: actor(seed),
      digest: await digest(handoff),
      key,
      until: decode(handoff.pass).until,
    };
    seed.fill(0);
    await approvers('readwrite', (objects) => objects.put(record));
    document.cookie = `${name}=; Max-Age=0; Path=/; Secure; SameSite=Strict`;
    console.debug(`SIGN: adopted the approver of ${label}'s newest open and deleted its cookie`);
  }
  const kept = await approvers(
    'readonly',
    (objects) => /** @type {IDBRequest<Approver | undefined>} */ (objects.get(label)),
  );
  if (!kept) throw new Error(`no approver is kept for ${label}: its cookie is gone or never came`);
  if (kept.digest !== (await digest(handoff))) {
    throw new Error(`${label} presented an open other than the one adopted`);
  }
  return kept;
}

// A refreshed delegation must not give the same entry another authorization/id.
// Keep approvals for the whole ±300 s authoring window, atomically across frames.
/**
 * @param {string} group
 * @param {string} pass
 * @param {string} line
 * @param {number} now in seconds
 */
async function stableApproval(group, pass, line, now) {
  const approval = decode(pass);
  const id = `${group}/${approval.person}/${approval.app}/${approval.op}`;
  let chosen = pass;
  await store.stored('svrngs-approvals', 'id', 'readwrite', (entries) => {
    const read = /** @type {IDBRequest<{ id: string, pass: string, until: number }[]>} */ (
      entries.getAll()
    );
    read.onsuccess = () => {
      const live = read.result.filter((pending) => pending.until > now);
      for (const pending of read.result) {
        if (pending.until <= now) entries.delete(pending.id);
      }
      const existing = live.find((pending) => pending.id === id);
      if (existing) {
        chosen = existing.pass;
      } else if (live.length >= 4096) {
        entries.transaction.abort();
      } else {
        entries.put({ id, pass, until: JSON.parse(line).ts_unix + 301 });
      }
    };
    return read;
  });
  return chosen;
}

// The label of the app framing this signer: the first name of its parent's
// origin, set by the browser (`e.origin`), never by what it asks; refused
// unless it is a lend's label directly under this host.
/** @param {string} origin */
function framer(origin) {
  const app = new URL(origin);
  const suffix = `.${location.host}`;
  const label = app.host.slice(0, -suffix.length);
  const under = app.protocol === location.protocol && app.host.endsWith(suffix);
  if (!under || !/^[a-z0-9]{62}$/.test(label)) {
    throw new Error(`${origin} is not an app under ${location.host}`);
  }
  return label;
}

addEventListener('message', (asked) => {
  const port = asked.ports[0];
  if (asked.source !== parent || !port) return;
  answered(asked, port).catch((error) => {
    console.debug(`SIGN: refused: ${error.message}`);
    port.postMessage({
      opened: false,
      error: error instanceof Told ? error.message : AGAIN,
    });
  });
});

// A refusal written for the person, said as it is, because opening the app
// again would not help. Every other refusal is told `AGAIN`.
class Told extends Error {}
const AGAIN = 'Open this app again from your group’s page to write in it.';

// What the app framing this signer `asked`, answered on `port`: its open
// adopted, a write to its database signed, or an entry approved; anything
// else refused, by a throw.
/**
 * @param {MessageEvent<{ opening?: Handoff, saved?: string, set?: SetAsked } | null>} asked an open
 *   to adopt, a set to sign, else an entry, which `entryAsked` reads
 * @param {MessagePort} port
 */
async function answered(asked, port) {
  const label = framer(asked.origin);
  const now = Math.floor(Date.now() / 1000);
  await (loaded ??= init({ module_or_path: store.runtime() }));
  if (asked.data?.opening) {
    const handoff = asked.data.opening;
    if (handoff.label !== label) {
      throw new Error(`an open for ${handoff.label} asked from ${label}`);
    }
    await approver(label, handoff, now);
    if (asked.data.saved === '__VERSION__') await store.keepSaved(label);
    port.postMessage({ opened: true });
    return;
  }
  if (asked.data?.set) {
    port.postMessage(await setsSigned(asked.data.set, label, now));
    return;
  }
  port.postMessage({ pass: await entryApproved(asked.data, label, now) });
}

// A computer's prepare of the framing app's write to its own database, as
// lend.js `setWritten` hands it on: the open it is under, the page's ops,
// the prepare's text, the computer that signed it and how, and the places
// of the sets a computer has kept since.
/**
 * @typedef {{ authority: Handoff, ops: unknown, prepared: string, by: string, sig: string,
 *   confirmed: string[] }} SetAsked
 */

// The lend of `asked`'s open for `label`, the app that frames this signer,
// where it names the computer that prepared `asked`; else thrown.
/**
 * @param {SetAsked} asked
 * @param {string} label
 */
function lendOf(asked, label) {
  const lend = asked.authority.lends.find((one) => one.label === label);
  const computers = [lend?.key, ...(lend?.lenders ?? []).map((one) => one.key)];
  if (!lend?.group || !computers.includes(asked.by)) {
    throw new Error(`${asked.by} is no computer of this open`);
  }
  return lend;
}

// What this signer keeps for one app of a group, in one record, so
// whatever clears any of it clears it all: its store's id, drawn as the
// record is made; the seq it last signed at each computer, the home of
// that set; and each set it signed, by its place, with the signature of the
// prepare it was signed for, until a computer confirms it.
/**
 * @typedef {{ id: string, store: number[], counts: Record<string, number>,
 *   sets: [string, string, string][] }} Signed
 */

// This origin's records of the sets it signed (`Signed`), by an app's group
// and kind.
/**
 * @template T
 * @param {IDBTransactionMode} mode
 * @param {(store: IDBObjectStore) => IDBRequest<T>} act
 */
const signedSets = (mode, act) => store.stored('svrngs-sets', 'id', mode, act);

// The framing app's write to its own database, signed for this phone by
// its open's approver, as an entry is approved (svrn-docs
// rings/current/APP_STATE.md §4, "Phones"; the operator's B′, 2026-11-22):
// under a live open this browser's group page issued for the app, prepared
// by a computer that open names, and checked against the page's ops by
// svrngs's own wasm (web/src/set.rs, `set_checked`), which stamps it by this
// signer's clock and puts in the open's pass, which names the approver
// every copy checks it by. This signer picks its place (svrn-docs
// app-state-diligence/invariants-2026-11-22/, change 5): at the computer
// that prepared it, its home, the seq after the last it signed there, for
// its store, so no place is signed twice while the record holds, and a
// record made anew is a site of its own. Two writes at once each sign at
// the next seq, and one is kept only where nothing moved the record since
// it was read; the other signs again at the seq after. Each set is kept
// until a computer confirms it, and every answer gives the kept ones back
// to carry. A prepare kept already, the page relaying it again, is no
// second set, and one that changes nothing, its ops checked so, no set.
/**
 * @param {SetAsked} asked
 * @param {string} label
 * @param {number} now in seconds
 * @returns {Promise<{ sets: Signed['sets'] }>}
 */
async function setsSigned(asked, label, now) {
  const signer = await approver(label, asked.authority, now);
  const lend = lendOf(asked, label);
  Object.assign(asked, { group: lend.group, app: lend.kind, pass: asked.authority.pass, ts: now });
  const id = `${lend.group}/${lend.kind}`;
  /** @param {Signed} record */
  const unconfirmed = (record) => record.sets.filter(([at]) => !asked.confirmed.includes(at));
  const drawn = () => [...crypto.getRandomValues(new Uint8Array(16))];
  for (;;) {
    /** @type {Signed | undefined} */
    const found = await signedSets('readonly', (objects) => objects.get(id));
    const was = found ?? { id, store: drawn(), counts: {}, sets: [] };
    if (was.sets.some(([, , sig]) => sig === asked.sig)) return { sets: unconfirmed(was) };
    const at = { store: was.store, seq: (was.counts[asked.by] ?? 0) + 1 };
    const [message, made, place] = set_checked(label, JSON.stringify({ ...asked, ...at }));
    if (message === undefined) return { sets: [] };
    if (message === null) throw new Told(made);
    const signature = await crypto.subtle.sign('Ed25519', signer.key, message);
    const set = set_sealed(made, store.hex(new Uint8Array(signature)));
    /** @type {Signed['sets']} */
    const sets = [...unconfirmed(was), [place, set, asked.sig]];
    const record = { ...was, counts: { ...was.counts, [asked.by]: at.seq }, sets };
    const same = (/** @type {Signed | undefined} */ read) =>
      JSON.stringify(read) === JSON.stringify(found);
    // The record as the write read it: put only where it was still `found`.
    const over = await signedSets('readwrite', (objects) => {
      const read = /** @type {IDBRequest<Signed | undefined>} */ (objects.get(id));
      read.onsuccess = () => {
        if (same(read.result)) objects.put(record);
      };
      return read;
    });
    if (same(over)) return { sets };
  }
}

// The entry asked, `data`, approved under a grant the open kept for
// `label` issued: the pass that authorizes it.
/**
 * @param {any} data
 * @param {string} label
 * @param {number} now in seconds
 */
async function entryApproved(data, label, now) {
  const { group, reach, line, authority } = entryAsked(data, label);
  // A pass naming no approver is a carried app that was let read
  // friends' lends (web/page/hand.js `writesBlocked`), so it is told that.
  if (typeof decode(reach).approver !== 'string')
    throw new Told('An app that reads your friends’ apps can’t write in the group’s record.');
  const kept = await approver(label, authority, now);
  for (const grant of [authority.pass, authority.writes].filter(Boolean)) {
    let made;
    try {
      made = approve_line(kept.pub, group, reach, grant, line, now);
    } catch {
      // Only the grants this open issued may authorize an entry.
      continue;
    }
    const signed = await crypto.subtle.sign('Ed25519', kept.key, made[1]);
    const approval = store.hex(new Uint8Array(signed));
    const pass = base64url(JSON.stringify({ ...JSON.parse(made[0]), approval }));
    return stableApproval(group, pass, line, now);
  }
  throw new Error('this open grants no entry of that kind');
}

// The entry `data` asks, refused when it is too large or its open grants
// no authority to `label`.
/**
 * @param {any} data
 * @param {string} label
 */
function entryAsked(data, label) {
  const { group, reach, line, authority } = data?.approve ?? data ?? {};
  if (tooLarge(reach, line)) throw new Error('an entry is too large');
  if (authority?.label !== label || authority.pass !== reach || authority.group !== group) {
    throw new Error('this open grants no authority here');
  }
  return { group, reach, line, authority };
}

/**
 * @param {unknown} reach
 * @param {unknown} line
 */
const tooLarge = (reach, line) =>
  typeof reach !== 'string' ||
  reach.length > 128 * 1024 ||
  typeof line !== 'string' ||
  line.length > 70 * 1024;
