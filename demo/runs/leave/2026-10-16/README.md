# groups-leave: scripts/check.sh at the close

At 81e1b9ef, `scripts/check.sh` exited 0, every gate passed, from "passed
principles pinned" to "passed app laws", in 10 min 24 s (`check.out`).

Two closes before it failed `tests`, each fixed in its own commit:
`scale.rs` `the_account_matches_a_hand_count`, a second read of the kept
fold (802519a1), and `surface.rs`'s pins of /v1/groups, which the row's
`left` and `keeps` changed (81e1b9ef).

One host, one run: not a rate (AGENTS.md 7).
