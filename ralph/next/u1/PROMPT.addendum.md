<!-- section: u1 after=intro -->
This queue is U1 (`docs/DESIGN.md` §8): a group is not a mesh. Two nodes sync a
group's journal over iroh, and who may connect and who is synced come from the
group's own register, where upstream asked the mesh. It is the first of three
units before Demo A (`demo/a.md`), and the U1 row's bar is the last row's check.

The plan for every row is DESIGN §8 "U1's inventory": what each upstream piece
becomes. Upstream is commonwealth-ai at `e1a43bcd2`, not its HEAD, which has
moved on. Read a file there with
`git -C ~/dev/commonwealth-ai show e1a43bcd2:<path>`; the paths in the rows are
upstream paths unless they start with `crates/`, `scripts/` or `docs/`.

A fork lands in two commits at least: first the files as upstream had them,
with only the edits the row names, so that a diff against upstream shows
nothing else; then any change of meaning, each named. Upstream's comments
come over with the code and keep its words (mesh, ring, roster) until a unit
renames them (DESIGN §5 "Words"); a comment that would now be false is
corrected in the same commit, and the commit says which.
