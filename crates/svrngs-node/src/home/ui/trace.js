// SPDX-License-Identifier: AGPL-3.0-or-later
// @ts-nocheck: not typed yet, so scripts/types.mjs skips it (the js-types row types it).
// The pages' trace sink, both pages', v1's moved here (web/app.js@19b63b2b:
// 55-64; page-scaffold (d)): each line ui/machine.js traces, and the page's
// own, go to the console at debug, never onto the page, and are kept on
// `window.__lines` for a test to read.
const lines = [];
window.__lines = lines;
window.__trace = (l) => {
  lines.push(l);
  console.debug(l);
};
export const trace = window.__trace;
