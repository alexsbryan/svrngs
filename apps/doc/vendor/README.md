# Vendored: Yjs and the editor, as one ESM bundle

`doc-bundle.js` is everything the doc's page and its server import from
npm, built by `../bundle/build.sh` from `../bundle/package.json` and its
lockfile with esbuild 0.25.12, minified: Yjs 13.6.32 with y-protocols' sync
and awareness and y-indexeddb 9.0.12, and Tiptap 3.31.3's core with only
the extensions the page uses (document, paragraph, text, bold, italic,
heading, lists, hard break, collaboration and its carets, y-tiptap 3.0.9).
One copy of Yjs for both, so the editor's binding, the browser's copy and
the sync share one `Y`. Nothing is fetched at run time.

It replaces upstream's `ring-doc-bundle.js` (1,170,287 bytes, unminified,
with Tiptap's whole starter kit), at 449,565 bytes: a phone reaches the
page through n0's relay, and its lend's worker asks for no compression.

`adapter.test.mjs` checks it against the sha256 below (`apps/pinned.mjs`),
and fails on one byte changed. `../bundle/build.sh` builds it again and
prints its sha256, to compare.

| file | sha256 |
|---|---|
| `doc-bundle.js` | `f941d928ee9cabd3e2ecdb407a5480ea8d9c1078e6d539cc0653ddae58a64934` |
