// SPDX-License-Identifier: AGPL-3.0-or-later
// The home page's edge: each request it makes of this computer's routes,
// and how a route's refusal is said. A feature's `io` calls these; no step
// and no view does. A route's answer is the JSON its server writes (`any`),
// typed where it is read, by the reader that knows its route.

// What a route refused for the page's lapsed session (home.rs:198,
// session.rs `refused`) is said as: how to open the page again.
export const ENDED =
  'this page’s session has ended. Open svrngs from your Applications, or on Linux your applications menu, and it opens here again';

// A response's JSON, or `{}` where it has none, so a refusal with no body
// is said by its status.
/**
 * @param {Response} response
 * @returns {Promise<{ error?: string } & Record<string, any>>}
 */
const read = (response) => response.json().catch(() => ({}));

// A refusal in the route's own words, else by its status.
/**
 * @param {Response} response
 * @param {{ error?: string }} answer
 */
const refusal = (response, answer) => new Error(answer.error || `answered ${response.status}`);

// `body` posted as JSON to `path`: the response, and its JSON as `read`
// answers it, whatever its status, for a reader that reads a refusal's
// body itself.
/**
 * @param {string} path
 * @param {unknown} body
 */
export async function sent(path, body) {
  const response = await fetch(path, {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify(body),
  });
  return { response, answer: await read(response) };
}

// `body` posted as JSON to `path`: its answer, or its words when it refuses.
/**
 * @param {string} path
 * @param {unknown} body
 * @returns {Promise<any>}
 */
export async function post(path, body) {
  const { response, answer } = await sent(path, body);
  if (response.ok) return answer;
  throw refusal(response, answer);
}

// An `io` effect that has HOME read the groups again and says so: asked by
// the sections whose own act changed what the groups' read shows.
/**
 * @param {(message: { type: 'again' }) => void} home
 * @returns {() => { type: 'told', why: string }}
 */
export const groupsAgain = (home) => () => {
  home({ type: 'again' });
  return { type: 'told', why: 'the groups read again' };
};

// A route's answer, or its words when it refuses, never read as empty;
// refused for the page's lapsed session, said as `ENDED`.
/**
 * @param {string} path
 * @returns {Promise<any>}
 */
export async function get(path) {
  const response = await fetch(path);
  if (response.ok) return response.json();
  if (response.status === 401) throw new Error(ENDED);
  throw refusal(response, await read(response));
}
