<!-- section: sync after=intro -->
This queue runs the mesh-Dropbox spike (`docs/DESIGN.md` §9's folders row;
§8 "Then"; the operator, 2026-10-09, §10) in the worktree
`~/dev/svrngs-folders` on the branch `folders`, after group folders' spike
(`demo/runs/gf/2026-10-09/`). That spike proved one holder's read-only
folder streamed into one Jellyfin, every byte verified and nothing beyond
the folder leaving its holder. This one asks the general question: does a
group's folder, a real directory on each member's disk, sync both ways —
added, edited, deleted by anyone — stay available while its author sleeps,
keep both sides of a conflict, and give no byte to anyone outside the group?

Inventory, read 2026-10-09: iroh-docs 0.101.0 (2026-06-15) takes `iroh ^1`,
`iroh-blobs ^0.103` and `iroh-gossip ^0.101` — this branch's iroh 1.0.2 and
iroh-blobs 0.103 — and is n0's multi-writer sync of signed per-author entries
naming blob hashes, reconciled between peers. commonwealth-ai holds no
iroh-docs and no folder sync. Group folders' spike already serves and fetches
a folder's files by iroh-blobs behind the group (`crates/svrngs-node/src/folder.rs`).

Fences the rows keep:
1. *A spike, not production.* The smallest slice that answers the bars,
   every line counted; group folders' spike's code is reused before any is
   written again.
2. *The bars are fixed at sy-1's first commit*, with the predictions, before
   any code. A miss is a miss, with its number.
3. *One source for who may read.* Membership is the group's record's alone;
   whatever indexes the folder admits no one the record does not.
4. *Nothing a folder's app would name enters svrngs* (`scripts/vocab_check.py`):
   svrngs moves files by hash and path, and knows no document, photo or film.
5. *The budget on this branch is 15,750* (15,163 at 396b0b6), svrngs-core's
   3,000. A row that would pass either stops for the operator.

One host, one run of each: not a rate (AGENTS.md 7). A run that could not
finish is could-not-judge, never a pass (AGENTS.md 6). A member that did not
answer is never an empty folder, and a refusal is never an empty answer.
