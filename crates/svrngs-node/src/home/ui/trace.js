// SPDX-License-Identifier: AGPL-3.0-or-later
// v2's trace sink, a copy of v1's (web/app.js:55-64) until the switchover
// deletes that one (page-scaffold (d)): each line ui/machine.js traces, and
// the page's own, go to the console at debug, never onto the page, and are
// kept on `window.__lines` for a test to read. `window.__ui` says which
// page drew them, so v2 is never silently v1.
const lines = [];
window.__lines = lines;
window.__ui = 'v2';
window.__trace = (l) => {
  lines.push(l);
  console.debug(l);
};
export const trace = window.__trace;
