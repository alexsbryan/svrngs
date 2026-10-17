<!-- section: folders after=intro -->
This queue runs group folders' second spike (`docs/DESIGN.md` §9's row;
§8 "Then"; the operator, 2026-10-09, §10), in the worktree
`~/dev/svrngs-folders` on the branch `folders`, beside scoped use made
production, whose loop runs in the main tree at `~/dev/svrngs`. Never write
there. This branch merges into main only by the operator's word, after the
verdict.

The first run (svrn-docs `rings/runs/strm-2026-10-05/`, `457f120`) put
Jellyfin 12.1.0 over two `.strm` stubs pointing at a range server. Every
browser played the MP4 straight from the stub's URL, and WebKit remuxed the
MKV through Jellyfin's own HLS, reading the whole file unpaced. It failed
one bar of four: only 93.0% and 85.8% of the MP4's bytes went to the
browser, because of Jellyfin's per-play `bytes=0-` probe of a 47 MB file.
This spike puts the design DESIGN §9 names in the range server's place:
the folder's file list in the group's record, and its bytes
content-addressed and fetched by verified range from whoever holds them
(iroh-blobs 0.103 over svrngs's iroh 1.0.2), served to Jellyfin by the
meeting point's own node.

Fences the rows keep:
1. *A spike, not production.* The smallest slice that answers the bars,
   with every line it writes counted. Its code lives on this branch.
2. *The bars are fixed at gf-0's commit.* A miss is a miss, with its
   number. Predictions are committed before any data, as mnp-2's were.
3. *A Jellyfin of the spike's own*, from `/Applications/Jellyfin.app` with
   its own data, configuration and cache directories and its own port, never
   the operator's. No container is needed; if one ever is, it is podman.
4. *Only what is shared leaves a holder.* A file beside the shared folder,
   or one a holder stopped sharing, is never served, shown by a scan of every
   byte the meeting point received.
5. *Apps' nouns stay in `apps/`* (`scripts/vocab_check.py`): svrngs moves a
   folder's files by hash and never learns a film. The `.strm` writer and the
   run's Jellyfin driving live in `apps/` and `demo/runs/gf/`.
   `scripts/check.sh` stays free of Jellyfin.
6. *The budget is 15,200* and svrngs-core's 3,000. At b482e08 that leaves
   671 lines and 365, and the spike's estimate is about 430, apps/ apart. A
   row that would pass either budget stops for the operator.

One host, one run of each: not a rate (AGENTS.md 7). A run that could not
finish is could-not-judge, never a pass (AGENTS.md 6). A holder that did not
answer is never an empty folder, and a refusal is never an empty answer.
