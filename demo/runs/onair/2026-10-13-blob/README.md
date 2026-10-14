# On air, bytes by hash for an app

onair-blob's check, "the harness's 4.1 rows green". The harness
`apps/on-air/harness/run.py`, its scoped bar's letter rewritten in
`429525c` and otherwise as onair-red wrote it, ran once on this host at
`1391572` (the predictions' commit; its code is `db6ba51`'s, onair-blob
(a)), in 2 min 26 s (`events.jsonl`, `start` to `end`), exit 0, the load
8.3 at its start and 17.3 at its end:

    W=/tmp/onair-blob-w python3 apps/on-air/harness/run.py demo/runs/onair/2026-10-13-blob

The predictions (`PREDICTIONS.md`) were committed in `1391572`, before it
ran. Every bar came out as predicted. One host, one run: not a rate
(AGENTS.md 7). `data/` is what the harness writes (see
`../2026-10-13-red/README.md`); beside it, after the run,
`data/stores-after.txt` (Bo's and Cy's store directories) and
`data/ana-restart.txt` (Ana's, before and after her computer started
again).

## The bars

| bar | verdict | what the run said |
|---|---|---|
| 4.1 | **passed** | every put 200, answering the bytes' BLAKE3 (`verdicts.json`, `station_puts`); Bo held 10 of 10 pieces, from 26 to 120 ms after each line arrived, against the 4,000 ms bound set before the run (`held_ms`), none refused |
| 4.1, carried | **passed** | Ana's computer stopped, Cy's get of Y answered 200, its 4,112 bytes Y's, in 3,009 ms — Ana's computer not answering the ask for 3 s (`ANSWER_WITHIN`) — from Bo: Cy's trace says Bo's key `b172f921…` held it with 44,008 ms left, fetched, verified, 5 ms (`cy_trace`). The plant Z, held by Ana's computer alone: 502 `unanswered`, "no holder answered", Ana's "did not answer within 3s", Bo's `never_held` — no empty piece |
| 4.1, lapses | **passed** | 65.1 s after the last put, Bo and Cy each said 410 `lapsed` of piece 0 and of Y; of a hash nobody put, each said 502 `unanswered`, Ana's computer stopped and the other `never_held` — told apart (`answers`) |
| 4.1, scoped | **passed** | Dee, removed, was refused Y by her own computer, 403 "this computer does not stand in" the group, traced `blob: refused` in her log (`own_refusals`), asking no one. The provider's group check, with a dialer it admits from another group, and its plant, are `db6ba51`'s test |
| 4.2 | passed | the page on `evening` played 10 of 10; `flipped` played all but 1, `bo-signed` none, `act-sig` 2 to 9, `no-ended` 0 and 1, as in `../2026-10-13-sign/` |
| 4.3 | passed | Bo drained 11 and Cy 12, each line naming Ana's key and person key; Bo's note in Cy's drain named Bo |
| off air | passed | Bo, Cy and the page said "ended" within one piece; the plant `no-ended` said "cannot reach" |

The four 4.1 rows were red before this row: in `../2026-10-13-sign/`
every put and get 404, Bo held 0 of 10, lapsed was not told apart from
never held, and Dee's get was 404. Their plants, each red on its own bar,
are onair-blob (a)'s tests (`db6ba51`): Bo not serving (Cy told no holder
answered), `keep` ignored (both stores still holding the bytes), and the
provider's group check out (Ana serving Cy, of another group, her films
bytes).

## Carrying, and what it spent

Each listener carried for the other: of Bo's 11 gets, 6 came from Ana's
computer and 5 from Cy's (`bo.trace`, `folder: fetched, verified`).
Once, piece 3, Bo asked Cy while Cy's own fetch of it was under way —
a computer says it holds a hash from its fetch's start — and Cy's store,
not yet whole, reset the stream; Bo took it from Ana, 120 ms after its
line, the one case `PREDICTIONS.md` named.

What each provider sent (`<who>.trace`, `folder: sent`, `payload`):
Ana's computer 966,670 bytes in 16 gets (15 pieces and Y), Cy's 318,904
in 5 (and one aborted, 0), Bo's 4,112 in 1 (Y, to Cy). The show's 10
pieces are 640,731 bytes. So the station's computer sent about one and a
half times the show for two listeners, and a listener up to half of it
again. No cap on a member's upload is set, and this run shows none
needed: at 128 kbit/s two listeners' carrying is about 25 kB/s at most,
on loopback. A cap is a follow-up row only when a run shows one needed
(onair-blob (b)).

## What the stores held after

- **Ana's computer**, stopped at the carried step, 25 s after the show's
  last put, with its 60 s not out: its store kept the show's 10 pieces on
  disk, `.data` files of 61,440 to 64,366 bytes. A stopped computer runs
  no sweep. Started again, its first sweep, 1 s after the store opened,
  dropped all 10 (`ana-restart.txt`: 10 files, then 0, `kept=0`): what
  apps placed is kept in memory alone, so a restart keeps none of it.
- **Bo's and Cy's**, running until the end: each piece's bytes were
  dropped. Left behind is each piece's `.obao4`, 192 bytes, the hash
  tree of a 64 kB blob (`stores-after.txt`). iroh-blobs 0.103 writes a
  fetched blob's tree to that file while the blob is partial; when it
  completes with a tree small enough to keep inline it copies the tree
  into its database and leaves the file (`store/fs/bao_file.rs:215-224`),
  and a delete of a complete entry whose tree is inline removes its
  database row alone (`store/fs/meta.rs:594-596`). Folders' fetches leave
  the same. No byte of a piece remains; the hashes carried can be read
  from the file names. Said in DESIGN §9, with its trigger.

## What this run cannot show

The provider refusing a member of another group: Dee, removed, stands in
no group with Bo, so Bo's endpoint would close her before any provider
ran, and her own computer refused first. That refusal is `db6ba51`'s
test, with its plant.

A lapse while a computer is stopped: shown for Ana's stores, on restart,
by hand after the run, not by the harness.

The sound is the red run's: 40 s generated here by ffmpeg's `aevalsrc`,
nobody's rights, heard by nobody.
