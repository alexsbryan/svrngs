# give-redial (a): a computer restarted, and the write that waits for it

The instrument is `crates/svrngs/tests/scenes/redial.rs` at 15cacd50: the
grapevine scene's step 12 without a browser. The predictions were written
before the first run (`PREDICTIONS.md`). Every run here had
`SCENES_TRACED=transport=debug,iroh::_events=debug`. The one-line results
are in `runs-mac.txt` and `runs-fedora.txt`. The logs kept are Sam's, ANSI
stripped (the Fedora one cut to its transport and path events), and Jo's for
the first run on this Mac.

## The rate at 15cacd50

| host | runs | reached within 18 s | times |
|---|---|---|---|
| this Mac (12 cores, load 1.6 to 15.9) | 10 | 9 | 5.05 to 5.28 s; the failure 59.97 s |
| Fedora box, gate image, podman's own network (pasta) | 3 | 0 | 59.95 to 60.02 s |

On Fedora under pasta, the grapevine scene itself failed 5 of 5 at
a6d7d179 (d5c34ccc's body). A failure lands at Sam's next round, 60 s on
(`DEFAULT_RING_SYNC_INTERVAL`).

## The cause, from the traces

The predicted cause holds, with one correction: the dead path is kept twice,
not once. Run 1 on this Mac passed, and its log `mac-head-1-sam.log`
shows the cause best:

1. Sam keeps a Member connection to Jo. Its path is old Jo's direct address,
   `192.168.1.11:61426`, and iroh has it selected (`:82`).
2. Jo is killed and started again at a new port, `:58325`. At 59.377, Jo
   dials Sam through the relay, and Jo's holepunching establishes a direct
   path to Sam at once (`mac-head-1-jo.log`, 59.378). iroh's selector on
   Sam does not switch to it. The dead path is still on the kept connection,
   with its last measured RTT, and is of the same tier, so a new path must be
   `RTT_SWITCHING_MIN` better to win (`iroh-1.0.2/src/socket/
   biased_rtt_path_selector.rs:136-190`). No `path::selected` is logged
   between `:82` and `:178`.
3. Sam's write wakes its round at 59.43. The request rides the kept
   connection (`iroh.rs` `kept`), and that connection times out after 4 s of
   silence, at 03.435 (`:167`; `dial.rs` `held`).
4. When it closes, iroh selects no new path. `handle_connection_close`
   clears the selection only when no connection to the key is left
   (`remote_state.rs:473-490`), and Jo's connection into Sam is open. So the
   dead address stays selected, though no connection holds it any more.
5. `fetch` dials afresh at once (`:170`). Every packet of a new connection
   goes to the selected path alone (`remote_state.rs:799-809`), so its
   handshake goes to the dead `:61426`, and Jo never sees it.
6. Jo's second holepunching round, at 04.380 (`mac-head-1-jo.log`),
   establishes another path on Jo's connection. That event selects again
   (`remote_state.rs:588-596`), now without the dead path, and `:58325`
   wins (`:178`). The fresh dial is answered at 04.440, 1 s into its 4 s.

A pass on this Mac is that race won. Every passing run here took 5.05 to
5.28 s, which is 4 s on the kept connection plus Jo's second round about 5 s
after it started.

The failure, `mac-head-3-sam.log`: the request on the kept connection failed
at 19.926 (`:171`). The fresh dial went out at 19.927 with dead `:59817`
still selected (`:174-175`), and was `Asleep(4s)` at 23.930 (`:186`). iroh
selected again only at 29.206, when a path on Jo's connection was abandoned
(`:200`). Nothing dialled again until the next round.

On Fedora under pasta (`fedora-pasta-head-1-sam.log`) the cause is the
same. The kept connection timed out at 12.711 (`:51`). The fresh dial went
out with dead `192.168.1.12:53435` selected since 08.376 (`:18`, `:54-56`),
and was `Asleep(4s)` at 16.713 (`:57`). The selection changed at 23.617, when a path was abandoned
(`:61`). There, each computer has one local candidate (`qnt::init
local_candidates={192.168.1.12:40346}`), and Jo ran no second holepunching
round within the 8 s, so the race was lost every time.

What answers the row's three questions:

- **Which connection the request rides:** the one kept to Jo's old self.
  It fails only after 4 s of silence.
- **Why the fresh dial is not answered:** iroh sends its handshake to the
  dead address alone. That address is still selected because closing a
  connection selects no new path.
- **What decides when the next try comes:** for the dial, any later path
  event on Jo's connection into Sam. For the write, the next round, at 60 s.

## Both remedies, tried and not committed

Each was applied to 15cacd50 alone, then reverted. The diffs are in
`NEEDS_HUMAN.md`.

| remedy | this Mac | Fedora, pasta |
|---|---|---|
| svrngs closes every connection to the key when the kept one fails, then dials | 3 of 3: 4.08, 6.74, 4.09 s | 3 of 3: 4.04, 4.04, 4.05 s |
| iroh selects again when a connection closes (`handle_connection_close`, 3 lines) | 3 of 3: 4.07, 6.73, 4.08 s | 3 of 3: 4.12, 4.07, 4.03 s |

The 6.7 s runs are ones where the kept connection took longer to fail. Under
either remedy, the dial after it was answered at once.

## (b): A landed

The director chose A (`DECISION.md`). `fetch` now closes every connection
to the computer, dialled or accepted, before its fresh dial (`iroh.rs`
`close_every`). The lines are in `runs-mac.txt` and `runs-fedora.txt`,
labelled `b-`.

| | this Mac | Fedora, pasta |
|---|---|---|
| the instrument at HEAD | 1 of 1 passed, 5.12 s (the race won) | 0 of 1, 59.97 s |
| the instrument with A | 3 of 3, 4.03-4.06 s | 3 of 3, 4.05-6.68 s |
| the plant (A's call skipped) | not run: it is HEAD, 9 of 10 above | 0 of 1, 60.01 s |
| the grapevine scene, plant | not run, as above | failed: "not in Jo's copy 18s after Jo restarted" |
| the grapevine scene, A | 10 of 10 in a row, 39.6-40.8 s each | 1 of 1, 39.11 s |

## The close, at d21578a6

`target/redial-close.sh` ran this Mac's gate and the Fedora box's gate
image on podman's own network, the same commit on each
(give-redial-3.await.log).

- **This Mac:** `scripts/check.sh` exited 0. The grapevine scene and the
  instrument (`scene redial (alone)`) both passed.
- **Fedora, the gate image, pasta:** the grapevine scene and the instrument
  passed. The whole gate exited 1, on two failures that are not this row's:
  - `apps/give/tests/check.rs`: 7 tests panic at `:22`, "podman does not
    answer on this computer". The image has no podman (`scripts/gate/
    Containerfile`), so these fail in every run of the image, on any
    network. The give scenes there are never-ran for the same reason.
  - `crates/svrngs-node/tests/browser`, two WebKit tests:
    `a_lend_worker_that_reads_no_store_dials_with_the_pass_its_start_page_hands_it`
    (`main.rs:1265`, the film does not play again after the hand-over
    page) and `a_lent_film_plays_and_seeks_under_the_worker_which_reaches_each_lender_itself`
    (`main.rs:951`, the seeks to 40 s and 52 s do not play again within
    5 s). These are not caused by the network or by A. Run alone in the
    image (`cargo test -p svrngs-node --test browser --
    lend_worker_that_reads_no_store a_lent_film_plays_and_seeks`), both
    failed 2 of 2 with `--network=host` and 2 of 2 on pasta at d21578a6,
    and 1 of 1 with `--network=host` at a46029a8, before A. Both pass in
    this Mac's gate.

So (a)'s cause is gone from the image on pasta. The image's other two
failures are follow-ups: podman inside the image for Give's tests, and
WebKit's replay in the Ubuntu image.
