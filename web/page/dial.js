// SPDX-License-Identifier: AGPL-3.0-or-later
// The phone's edge: the runtime, the page's own trace, and every dial the
// page makes as the person. Each flow's `io` comes here to reach a
// computer, and no step does: a step answers an effect as data, and its
// flow's `io` runs it (ui/machine.js `machine`).
//
// The group's key and its one endpoint live here, never in state. One
// endpoint serves every dial, since the relay keeps one per key, so a
// second endpoint of the key would displace the first.
import { silent, unreached, whoseComputer } from '../question.js?v=__VERSION__';
/** @import { Group, Computer } from '../store.js?v=__VERSION__' */
/** @import * as Wasm from '../wasm/svrngs_web.js?v=__VERSION__' */
/** @import { Dialled, Endpoint } from '../wasm/svrngs_web.js?v=__VERSION__' */
/** @import { Lent } from './enter.js?v=__VERSION__' */

// How long a dial waits for a computer to answer, in milliseconds.
export const PATIENCE = 60_000;
const PEOPLE_LENDS = '/people/lends';
// This page's svrngs version, as the wasm export names it, sent so a lender
// refuses an old discovery request that cannot name a scope
// (SCOPE_PARALLEL.md, "Shared API shape"); compatibility data, never
// permission.
/** @param {typeof Wasm} core */
const SCOPED = (core) =>
  /** @type {[string, string]} */ (['x-svrngs-version', String(core.current_version())]);

// The runtime by its one URL, imported only once app.js has run its init.
export const wasm = () => import('../wasm/svrngs_web.js?v=__VERSION__');

// The page's own lines, to the sink ui/trace.js sets on `window`. It is
// read from `globalThis` and not imported, so the laws can import a flow
// in node, which has no `window`.
export const trace = (/** @type {string} */ line) => globalThis.__trace?.(line);

// `message` once the time `until`, in seconds, has come.
/**
 * @template T
 * @param {number} until
 * @param {T} message
 * @returns {Promise<T>}
 */
export const when = (until, message) =>
  new Promise((resolve) => setTimeout(() => resolve(message), until * 1000 - Date.now()));

// An error in words the page says to the person, as a failed message's
// `said`, and the status it answers when a computer refused.
/**
 * @param {string} words
 * @param {number} [status]
 */
export const saying = (words, status) => Object.assign(new Error(words), { said: words, status });

// `token`, refused when this build cannot read a link's way in from it.
/**
 * @param {typeof Wasm} core
 * @param {string} token
 */
export const readable = (core, token) => {
  if (!core.relay_of(token))
    throw saying('This link was made by an older svrngs. Ask whoever sent it for a new one.');
};

/**
 * What `dialling` answers, or an error once PATIENCE has passed with no
 * answer.
 * @template T
 * @param {Promise<T>} dialling
 * @returns {Promise<T>}
 */
export const patiently = (dialling) =>
  Promise.race([
    dialling,
    /** @type {Promise<never>} */ (
      new Promise((_, reject) => setTimeout(() => reject(new Error('no answer')), PATIENCE))
    ),
  ]);

/**
 * `bytes` posted as JSON at `path` through `connection`, or `path` read
 * when there are none. The answer is read whole before the connection
 * closes, since a close cuts off a body not yet read.
 * @param {Dialled} connection
 * @param {string} path
 * @param {Uint8Array<ArrayBuffer>} [bytes]
 * @returns {Promise<[ok: boolean, status: number, text: string]>}
 */
export const exchange = (connection, path, bytes) =>
  connection
    .fetch(
      bytes ? 'POST' : 'GET',
      path,
      bytes
        ? [
            ['content-type', 'application/json'],
            ['content-length', String(bytes.length)],
          ]
        : [['accept', 'application/json']],
      bytes ? new Blob([bytes]).stream() : null,
    )
    // The wasm's `fetch` answers the page's `Response` (svrngs_web.d.ts `Dialled`).
    .then(
      async (/** @type {Response} */ answer) =>
        /** @satisfies {[boolean, number, string]} */ ([
          answer.ok,
          answer.status,
          await answer.text(),
        ]),
    )
    .finally(() => connection.close());

export const AWAY = `${silent('The computer that sent this link')} Try again later.`;

/**
 * What a link's refusal says, by the issuer's status.
 * @param {number} status
 */
export const linkRefused = (status) =>
  status === 403
    ? 'This link lets nobody in. Ask for a new one.'
    : 'The computer that sent this link cannot let anyone in just now. Try again later.';

/**
 * `body` posted at `path` to the issuer of `token`, dialled as `as` within
 * PATIENCE: its answer's text, or the refusal `refused` words for its
 * status. A join's and a visit's one way to the computer that made a link.
 * @param {typeof Wasm} core
 * @param {Uint8Array} as
 * @param {string} token
 * @param {{ path: string, body: string, refused?: (status: number, text: string) => string }} request
 * @returns {Promise<string>}
 */
export async function posted(core, as, token, { path, body, refused = linkRefused }) {
  const dialled = await patiently(core.connect_to_join(as, token)).catch((error) => {
    trace(`PAGE: the dial failed: ${error}`);
    throw saying(unreached(error, AWAY));
  });
  const [ok, status, text] = await exchange(dialled, path, new TextEncoder().encode(body));
  if (ok) return text;
  trace(`PAGE: ${path} answered ${status}: ${text}`);
  throw saying(refused(status, text), status);
}

// Each typed apart, as JSDoc types only the first of several declared at once.
// The group's key, set by enter.js's `open`, which comes before every read.
/** @type {Uint8Array<ArrayBuffer>} */
let key;
/** @type {Promise<Endpoint> | null} */
let bound;
// The group's relays `bound` was bound with.
let boundAt = '';

/**
 * The group's key, unsealed as it opens. The endpoint of the key before it
 * is let go, so the next dial binds one of this key.
 * @param {Uint8Array<ArrayBuffer>} unsealed
 */
export const hold = (unsealed) => {
  key = unsealed;
  bound = null;
};

/**
 * This page's key as the record spells it: public, as the record's rows
 * name it, never the key itself.
 * @param {typeof Wasm} core
 */
export const actor = (core) => core.actor(key);

/**
 * The one endpoint, bound as the person on the first dial with the group's
 * relays as last read, and bound again when a read changes them: a relay
 * keeps one endpoint per key, so the old one is closed. One refused (no
 * relay answered in time, web/src/lib.rs `bind`) is let go, so the next
 * dial binds again rather than the refusal for good (review F3).
 * @param {typeof Wasm} core
 * @param {Group} group
 */
const endpoint = (core, group) => {
  const ways = String(group.meets ?? []);
  if (bound && ways === boundAt) return bound;
  if (bound) {
    trace(`PAGE: the group's relays changed to ${ways}, so the endpoint binds again with them`);
    void bound.then((old) => old.close()).catch(() => {});
  }
  boundAt = ways;
  const binding = (bound = core.endpoint(key, group.token, group.meets));
  binding.catch((error) => {
    if (bound !== binding) return;
    trace(`PAGE: the endpoint was not bound, so the next dial binds one: ${error}`);
    bound = null;
  });
  return binding;
};

/**
 * `group` dialled through the first to answer of its link's computer and
 * each computer its last read named, all at once.
 * @param {typeof Wasm} core
 * @param {Group} group
 * @param {string[]} tried the computers asked already, by key
 * @returns {Promise<[Dialled, Computer | undefined]>} and the computer that answered, none when the link's did
 */
async function reach(core, group, tried) {
  readable(core, group.token);
  const issuer = core.issuer_of(group.token);
  const link = group.computers?.find((one) => one.key === issuer);
  const linked = !tried.includes(issuer);
  const others = (group.computers ?? []).filter((one) => one !== link && !tried.includes(one.key));
  trace(
    `PAGE: dialling ${linked ? "the link's computer and " : ''}${others.length} other computer(s) at once`,
  );
  const tries = [
    ...(linked
      ? [
          endpoint(core, group)
            .then((dialer) => dialer.dial_link())
            .then((reached) => /** @satisfies {[Dialled]} */ ([reached])),
        ]
      : []),
    ...others.map((other) =>
      endpoint(core, group)
        .then((dialer) => dialer.dial(other.key, other.relay))
        .then((reached) => /** @satisfies {[Dialled, Computer]} */ ([reached, other])),
    ),
  ];
  const asked = [...(linked ? [link] : []), ...others];
  const silence = asked.map((one) => silent(whoseComputer(one))).join(' ');
  const away = `${silence} Try again later.`;
  const [dialled, computer] = await patiently(Promise.any(tries)).catch((error) => {
    trace(`PAGE: the dial failed: ${error}`);
    const said = unreached(error, away);
    // Only a silence is asked again (enter.js `failing`), never a refusal.
    return Promise.reject(Object.assign(saying(said), { silence: said === away && silence }));
  });
  trace(
    `PAGE: reached ${group.name} through ${computer ? `${computer.person}'s` : "the link's"} computer`,
  );
  // A computer that answered after the first is closed, so it holds
  // nothing open at the relay.
  for (const attempt of tries)
    attempt.then(
      ([answered]) => {
        if (answered === dialled) return;
        trace('PAGE: a computer answered late, so closed');
        void answered.close();
      },
      () => {},
    );
  return [dialled, computer];
}

/**
 * The group's lends, asked of the next computer to answer when an answer
 * fails, and the connection it came through, still open.
 * @param {typeof Wasm} core
 * @param {Group} group
 * @param {string[]} tried
 * @returns {Promise<[Dialled, Computer | undefined, Response]>}
 */
// The discovery read names this page's svrngs version with
// `x-svrngs-version`, so a lender refuses an old request that cannot name
// a scope (SCOPE_PARALLEL.md, "Shared API shape"); compatibility data,
// never permission.
export const lends = (core, group, tried) =>
  reach(core, group, tried).then(([dialled, via]) =>
    dialled.fetch('GET', PEOPLE_LENDS, [['accept', 'application/json'], SCOPED(core)], null).then(
      (res) => [dialled, via, res],
      (error) => {
        trace(
          `PAGE: ${PEOPLE_LENDS} failed through ${whoseComputer(via)}, so the next to answer is asked: ${error}`,
        );
        void dialled.close();
        return lends(core, group, [...tried, via?.key ?? core.issuer_of(group.token)]).catch(() =>
          Promise.reject(error),
        );
      },
    ),
  );

/**
 * One lender's computer, `lend`, dialled as the person through the one
 * endpoint, within PATIENCE: what apps use, and the ask's yes, posted to
 * its door's /choices (uses.js).
 * @param {Group} group
 * @param {Lent} lend
 */
export const dialled = (group, lend) =>
  wasm().then((core) =>
    patiently(
      Promise.resolve(endpoint(core, group)).then((dialer) =>
        dialer.dial(lend.key, lend.relay ?? null),
      ),
    ),
  );

/**
 * A write through the computer the page reaches: `body` posted with the
 * group's id, as JSON, and its answer read, or the computer's refusal in
 * its own words.
 * @param {Group} group
 * @param {string} path
 * @param {unknown} body
 */
export const through = (group, path, body) =>
  wasm().then(async (core) => {
    const [connection] = await reach(core, group, []);
    const bytes = new TextEncoder().encode(JSON.stringify([group.id, body]));
    const [ok, status, text] = await exchange(connection, path, bytes);
    if (ok) return JSON.parse(text);
    trace(`PAGE: ${path} answered ${status}: ${text}`);
    return Promise.reject(new Error(JSON.parse(text).error));
  });
