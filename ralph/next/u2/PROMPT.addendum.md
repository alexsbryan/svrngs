<!-- section: u2 after=intro -->
This queue is U2 (`docs/DESIGN.md` §8): join by link. Sam runs `svrngs new
house` and sends one link; each friend who opens it with `svrngs join` is
admitted once, by Sam's node, and their computer syncs the group. It is the
second of three units before Demo A (`demo/a.md`), and the U2 row's bar is the
last row's check.

The plan for every row is DESIGN §8 "U2's inventory", §6's rules and §10's
2026-10-04 decisions. Upstream is commonwealth-ai at `e1a43bcd2`, not its
HEAD. Read a file there with `git -C ~/dev/commonwealth-ai show
e1a43bcd2:<path>`; the paths in the rows are upstream paths unless they start
with `crates/`, `scripts/`, `docs/` or `demo/`.

A fork lands in two commits at least: first the files as upstream had them,
with only the edits the row names, so that a diff against upstream shows
nothing else; then any change of meaning, each named. New code is held to
`docs/principles/RUST_ARCH.md` from its first line. Anything a person reads —
the command's output, its refusals — uses only DESIGN §1's words: the group's
name, people's names, *link*, *computer*; never ring, member, key, sync,
journal, node or relay. Traces at debug may use any word.

Record-format rows change what is signed. No journal outside tests exists yet,
so there is no legacy form to read; a fixture that pins the old form is
regenerated in the commit that changes it, and the body names each one.
