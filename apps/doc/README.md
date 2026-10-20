# Doc

A document a room writes together: a Yjs CRDT under a Tiptap editor, each
person's caret named, and each paragraph saying who last edited it.

Ported from commonwealth-ai `cmnwlth/apps/ring-doc/` at 47b8acfd0, where it
ran as a ring app (`svrn ring show ring-doc`) over commonwealth-ai's
`window.ring`: edits as signed acts on the rail, carets on its live lane.
svrngs has no `window.ring`, so as ported the page draws its editor and
reaches nothing. Upstream's expense book, a copy of its template's
(commonwealth-ai `docs/RING_APP_LIBRARY.md:121`), is not kept: svrngs's is
`apps/expenses/`. `adapter.test.mjs` runs here as it
ran there (`scripts/check.sh`, "app laws"). `apps/` is outside the size
budget (`scripts/lean_check.py`).
