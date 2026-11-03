// SPDX-License-Identifier: AGPL-3.0-or-later
// What a page's flow draws and steps with, at one URL (page-switchover (a)):
// the core, the parts and question.js's words, so each flow imports them in
// one line, where it named each module (185 of v2's 1,076 lines were
// imports; demo/runs/page/2026-11-01-parity/README.md, "The count beside
// the bar"). Each name is exported by one module only, but `drawn`, which
// copy.js and shelf.js both name and a flow imports from its own module: a
// name two modules export is no export here, and importing it fails as the
// page loads, not quietly. ui/'s own files import their modules by name.
//
// A type is a name too: tsc refuses two modules' names meeting here,
// either kind, so each part's types are named for it (copy.js
// `CopyState`), and question.js's people row is `PeopleRow`, where
// parts.js's `Row` draws a table's. `drawn` is declared below.
export * from './view.js?v=__VERSION__';
export * from './machine.js?v=__VERSION__';
export * from './parts.js?v=__VERSION__';
export * from './copy.js?v=__VERSION__';
export * from './door.js?v=__VERSION__';
export * from './visitors.js?v=__VERSION__';
export * from './shelf.js?v=__VERSION__';
export * from './acts.js?v=__VERSION__';
export * from './changes.js?v=__VERSION__';
export * from './uses.js?v=__VERSION__';
export * from '../question.js?v=__VERSION__';

// No `drawn` here: copy.js's and shelf.js's meet. Declared a type of
// nothing, which ends tsc's refusal of the two and makes it refuse an
// import of `drawn` from here as a value, as the browser refuses it.
/** @typedef {never} drawn */
