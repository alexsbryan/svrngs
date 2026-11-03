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
import init, { actor, approve_line } from './wasm/svrngs_web.js?v=__VERSION__';
import { handoffCookie, hostCookie, openMessage, openSeal } from './authority.js?v=__VERSION__';
import * as store from './store.js?v=__VERSION__';
/** @import { Handoff } from './authority.js?v=__VERSION__' */
let loaded;
/** @param {string} text */
const decode = (text) => JSON.parse(atob(text.replace(/-/g, '+').replace(/_/g, '/')));
/** @param {ArrayBuffer} bytes */
const hex = (bytes) =>
  [...new Uint8Array(bytes)].map((byte) => byte.toString(16).padStart(2, '0')).join('');
/** @param {Handoff} handoff */
const digest = async (handoff) =>
  hex(await crypto.subtle.digest('SHA-256', new TextEncoder().encode(openMessage(handoff))));
/** @param {Uint8Array} seed */
const pkcs8 = (seed) =>
  Uint8Array.from([
    0x30,
    0x2e,
    0x02,
    0x01,
    0x00,
    0x30,
    0x05,
    0x06,
    0x03,
    0x2b,
    0x65,
    0x70,
    0x04,
    0x22,
    0x04,
    0x20,
    ...seed,
  ]);

// One transaction on this origin's store `name`, its one object store kept
// by `keyPath`: `act(store)`'s request's answer once it commits.
/**
 * @template T
 * @param {string} name
 * @param {string} keyPath
 * @param {IDBTransactionMode} mode
 * @param {(store: IDBObjectStore) => IDBRequest<T>} act
 * @returns {Promise<T>}
 */
function stored(name, keyPath, mode, act) {
  return new Promise((yes, no) => {
    const request = indexedDB.open(name, 1);
    request.onupgradeneeded = () => request.result.createObjectStore('kept', { keyPath });
    request.onerror = () => no(request.error);
    request.onsuccess = () => {
      const tx = request.result.transaction('kept', mode);
      const asked = act(tx.objectStore('kept'));
      tx.oncomplete = () => {
        request.result.close();
        yes(asked?.result);
      };
      tx.onabort = tx.onerror = () => {
        request.result.close();
        no(tx.error ?? new Error('this browser’s storage refused'));
      };
    };
  });
}
/**
 * @template T
 * @param {IDBTransactionMode} mode
 * @param {(store: IDBObjectStore) => IDBRequest<T>} act
 */
const approvers = (mode, act) => stored('svrngs-approvers', 'label', mode, act);

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
    const all = objects.getAll();
    all.onsuccess = () =>
      all.result
        .filter((record) => record.until <= now)
        .forEach((record) => objects.delete(record.label));
    return all;
  });
  const name = handoffCookie(label);
  const seed = await openSeal(handoff, hostCookie(document.cookie, name));
  if (seed) {
    const key = await crypto.subtle.importKey('pkcs8', pkcs8(seed), 'Ed25519', false, ['sign']);
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
  const kept = await approvers('readonly', (objects) => objects.get(label));
  if (!kept) throw new Error(`no approver is kept for ${label}: its cookie is gone or never came`);
  if (kept.digest !== (await digest(handoff)))
    throw new Error(`${label} presented an open other than the one adopted`);
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
  await stored('svrngs-approvals', 'id', 'readwrite', (entries) => {
    const read = entries.getAll();
    read.onsuccess = () => {
      const live = read.result.filter((pending) => pending.until > now);
      for (const pending of read.result) if (pending.until <= now) entries.delete(pending.id);
      const existing = live.find((pending) => pending.id === id);
      if (existing) chosen = existing.pass;
      else if (live.length >= 4096) entries.transaction.abort();
      else entries.put({ id, pass, until: JSON.parse(line).ts_unix + 301 });
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
  const app = new URL(origin),
    suffix = `.${location.host}`;
  const label = app.host.slice(0, -suffix.length);
  if (
    app.protocol !== location.protocol ||
    !app.host.endsWith(suffix) ||
    !/^[a-z0-9]{62}$/.test(label)
  )
    throw new Error(`${origin} is not an app under ${location.host}`);
  return label;
}

addEventListener('message', (asked) => {
  const port = asked.ports[0];
  if (asked.source !== parent || !port) return;
  (async () => {
    const label = framer(asked.origin);
    const now = Math.floor(Date.now() / 1000);
    await (loaded ??= init({ module_or_path: store.runtime() }));
    if (asked.data?.opening) {
      const handoff = asked.data.opening;
      if (handoff.label !== label)
        throw new Error(`an open for ${handoff.label} asked from ${label}`);
      await approver(label, handoff, now);
      return port.postMessage({ opened: true });
    }
    const { group, reach, line, authority } = asked.data?.approve ?? asked.data ?? {};
    if (
      typeof reach !== 'string' ||
      reach.length > 128 * 1024 ||
      typeof line !== 'string' ||
      line.length > 70 * 1024
    )
      throw new Error('an entry is too large');
    if (authority?.label !== label || authority.pass !== reach || authority.group !== group)
      throw new Error('this open grants no authority here');
    const kept = await approver(label, authority, now);
    for (const grant of [authority.pass, authority.writes].filter(Boolean)) {
      let made;
      try {
        made = approve_line(kept.pub, group, reach, grant, line, now);
      } catch {
        continue; /* Only the grants this open issued may authorize an entry. */
      }
      const approval = hex(await crypto.subtle.sign('Ed25519', kept.key, made[1]));
      const pass = btoa(JSON.stringify({ ...JSON.parse(made[0]), approval }))
        .replace(/\+/g, '-')
        .replace(/\//g, '_')
        .replace(/=+$/, '');
      return port.postMessage({ pass: await stableApproval(group, pass, line, now) });
    }
    throw new Error('this open grants no entry of that kind');
  })().catch((error) => {
    console.debug(`SIGN: refused: ${error.message}`);
    port.postMessage({
      opened: false,
      error: 'Open this app again from your group’s page to write in it.',
    });
  });
});
