// SPDX-License-Identifier: AGPL-3.0-or-later
// The pages' trace sink, both pages', v1's moved here (web/app.js@19b63b2b:
// 55-64; page-scaffold (d)): each line ui/machine.js traces, and the page's
// own, go to the console at debug, never onto the page, and are kept on
// `window.__lines` for a test to read, which only this file names, so it
// is declared here and not with `__trace` (ui/globals.d.ts).
/** @type {string[]} */
const lines = [];
/** @type {Window & { __lines?: string[] }} */ (window).__lines = lines;
/** @param {string} line */
window.__trace = (line) => {
  lines.push(line);
  console.debug(line);
};
export const trace = window.__trace;
