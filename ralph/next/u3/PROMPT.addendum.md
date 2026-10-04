<!-- section: u3 after=intro -->
This queue is U3 (`docs/DESIGN.md` §8): lend a service. Sam runs `svrngs share
8096`; Alex, Bo and Cy, each a member, reach Sam's Jellyfin through the door on
Sam's node, which sets who is asking, strips what a client typed, injects
Sam's secret and never lets it out, streams and seeks, tells each request its
share of Sam's declared upload, and refuses a person past the lend's limit
with a reason. An app on a member's computer reaches the record over
loopback. It is the last unit before Demo A (`demo/a.md`), and the U3 row's
bar is the bar of the rows that carry it.

The plan for every row is DESIGN §8 "U3's inventory" and its four facts, §3
"What an app gets", and §10. Upstream is commonwealth-ai at `e1a43bcd2`, not
its HEAD. Read a file there with `git -C ~/dev/commonwealth-ai show
e1a43bcd2:<path>`; the paths in the rows are upstream paths unless they start
with `crates/`, `apps/`, `scripts/`, `docs/` or `demo/`.

A fork lands in two commits at least: first the files as upstream had them,
with only the edits the row names, so that a diff against upstream shows
nothing else; then any change of meaning, each named. New code is held to
`docs/principles/RUST_ARCH.md` from its first line. Anything a person reads —
the command's output, its refusals, a refusal the door sends — uses only
DESIGN §1's words. The infra never learns an app's nouns: `scripts/vocab_check.py`
refuses Films' and Expenses' nouns in infra code, so the door says *lend*,
*service* and *kind*, never *film* or *library*, and Jellyfin appears only in
tests, fixtures and the probe of a well-known port.
