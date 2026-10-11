# syp-4-production's reruns — 2026-10-10

sy-1's M2, S2 and M1 (`demo/runs/sy/2026-10-09/README.md` §2-§4), rerun on
the branch `folders` as syp-4's parts land (`ralph/next/syp/STATE.md`), by
`run.py` here: sy-1's `run.py`, its setup and its functions, with the
changes its docstring names. Three svrngs data dirs on this host, through
iroh's public relays, in one group `Shared`. Each part's data is in
`run-<part>/`: `events.jsonl` (each line timed, the host's load, cores, swap
and free disk at the start and end), `summary.json`, and each node's log.
No plant runs here: each part's plant is a test's red run (the queue's
fence 3), cited by its commit. One host, one run of each: not a rate
(AGENTS.md 7).

## M2, the store (syp-4 b)

`python3 run.py m2`, at `2254580`, load 11.59 on 12 cores at its start,
swap 5,998.56 of 7,168 MB in use at its start and at its end (not grown),
116 GB free (`run-m2/events.jsonl`, the two `load` lines).

Ana wrote 100 MiB, then rewrote 1 MiB inside it. Both versions reached Ben
and Cy (`first_s` 9.14, `edit_s` 8.37), and 0.89 s after the edit was
everywhere (`dropped_s`) each of the three stores held the edit's version,
104,857,600 bytes, and none of the first, 0 (`stores`, the data file of
each hash in `<data dir>/blobs/data/`). **Passed.** With the drop out,
both versions stayed: the test's red run, `a_store_keeps_what_the_layout_places_and_no_more`
(the commit before this run's).

Each member still received the whole edit: 100 gets of 1 MiB at Ben
(`grep -c "fetched, verified" run-m2/ben.log`: 200, the first version's
and the edit's) — a version as its blocks waits (DESIGN §9).

## S2, Ben's node stopped and started before Cy's (syp-4 d)

`python3 run.py s2`, at `1911547`, load 8.01 on 12 cores at its start,
swap 5,782.50 MB at its start and end (not grown), 109 GB free.

Cy stopped; Ana wrote `s2.bin` and Ben fetched it (`run-s2/ben.log:15`,
"fetched, verified", 00:59:53.375); Ana was killed (`kill -9`) and Ben
stopped by SIGTERM 11 ms after his fetch, then started (`events.jsonl`,
`down` ben at …393.386, `up` at …393.641). Cy then started and held
`s2.bin` 4.51 s after her command, from Ben (`bar` S2-restarted, `from`
["ben"]): Ana, killed, passed over (`cy.log:23`, "a holder gave
nothing"), and Ben's provider sent the whole file, `done=true
payload=65536` (`ben.log:30`). **Passed.** Ben's step had written its kept
state before the stop: his restarted step sent no change for `s2.bin`
(no "a change sent" in `ben.log`), and the kept state is written only
after the store's `sync_db`.

The loss itself is reproduced by a test, `tests/kept.rs`, red three times
of three without the fix and green three of three with it (its commit,
7f0c479).
