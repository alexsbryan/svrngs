# Packaged availability on a Fedora VM

`avail-proof`, KEEP_IT_RUNNING_RELEASE §5 row 9. Predictions before the run:

| Case | Prediction |
|---|---|
| Busy port | Run refuses the live listener. Its process, response and installed setup stay the same; Run writes no new job or share. |
| Failed save | A directory planted at the declaration's staging path makes Share fail, says the app is running but not shared, and leaves its process and data alone. Clearing the obstruction and retrying publishes without a second Run. |
| Concurrent withdrawal | Withdraw while a terminal Share is waiting for its review confirmation. The declaration goes away, and the stale confirmation cannot restore it. |
| Old link and data | Alex reads a fresh marker written by Sam before shutdown. On a different boot, with no Sam login or action, the identical address returns the marker and the share's approval is refreshed under the same grant. A wrong marker must fail the same comparison. |
| Stop sharing | An open member answer carrying bytes ends within two seconds of whole-share withdrawal. A continuing local response shows the app stayed up. With the withdrawal omitted, the same cut assertion must time out. |

The proof uses two scratch accounts and a local relay in the disposable
guest. A stateful native HTTP fixture gives the marker and an unfinished
answer independently of model output. The VM does not judge a real laptop's
GPU, disk unlock or suspend, the Mac reboot, or the people sessions.

## Door test: whole-share withdrawal

At `crates/svrngs-node/tests/door/live.rs`, the member receives bytes
before the timer starts. Both names use the same app. The second name
must still answer after the first stream is cut.

Planted: omit `lend::withdraw` while keeping the open stream and timing
assertion. `TOKIO_WORKER_THREADS=4 cargo test --locked -p svrngs-node --test
door stopping_the_share_cuts_a_members_live_answer_without_stopping_the_app
-- --nocapture` exited 101:

```text
after Stop sharing, the shelf's answer ended after None
the shelf's answer still carries on after 2 s
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 9 filtered out; finished in 2.05s
```

Restored the withdrawal. `TOKIO_WORKER_THREADS=4 cargo test --locked -p
svrngs-node --test door live:: -- --nocapture` exited 0:

```text
after Stop sharing, the shelf's answer ended: Some(1.388481ms) (1.498668ms including withdrawal)
test live::stopping_the_share_cuts_a_members_live_answer_without_stopping_the_app ... ok
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 8 filtered out; finished in 4.13s
```

The finer-grained W1 check also passed (`TOKIO_WORKER_THREADS=4 cargo
test --locked -p svrngs-node --lib w1_a_concurrent_withdrawal_wins`):

```text
test lend::tests::w1_a_concurrent_withdrawal_wins ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 238 filtered out; finished in 0.00s
```

## Packaged replay

From the checkout root, with the keeper spike's prepared Fedora 45 Cloud
guest under `/tmp/keeper-demo`:

```sh
toolbox run -c sovereign-vulkan bash -c 'cd ~/dev/svrngs-keeper && scripts/build-release.sh --only linux-x86_64 --out /tmp/opencode/avail-proof-site'
W=/tmp/opencode/avail-proof-vm3 python3 -B demo/runs/available/2026-11-23/run.py /tmp/opencode/avail-proof-site
```

The build and final replay both exited 0. The release builder printed:

```text
built: /tmp/opencode/avail-proof-site/get
74e84c77c28884b66d9524f76768a3100b3943e6668679af2cc3091ce2a60ab3  svrngs-linux-x86_64.tar.gz
af242283bd8e80b88bb5a6e2bbee025e201b48e900e89fb67a4c162928f2190f  give-linux-x86_64.tar.gz
```

Both scratch accounts installed and ran `svrngs 1.13.3 (540cb60e)` from
the checksum-verified local site. Both checked for the installed keeper
beside it. The installer opener was a stub. Alex enabled Play once and
saved the full `/marker` address before the reboot. His requests use the
installed node through that address. `run.py` adds `RUST_LOG=info,door=debug`
to Sam's installed door via a guest-only drop-in; the installed commands
and startup targets are unchanged. Two earlier attempts found harness
issues: Alex has no SSH key in the prepared image, so the cases never ran
on the first attempt; on the second, the reboot's renewal was
**could-not-judge** without the door's debug trace. The first three cases
already passed on that second attempt.

**Busy port — passed.** Run first opened the fixture on a free port.
A second Run under a different name hit that listener; the proposed job
was not loaded or written, there was no share, and the original process
and marker were unchanged:

```text
svrngs: Port 8190 is in use by python3, the copy you started: stop it, then press Run.
```

```json
{"case":"busy port","verdict":"passed","exit":1,"pid":"1867","load_state":"not-found","marker":"d37fe84e7385476fa613991c75699d95"}
```

**Failed save — passed.** Plant a directory at the partial declaration
path. The failed Share wrote no declaration and left the app answering
with the same PID and marker. Removing the directory let Share publish
the unit grant without a second Run:

```text
svrngs: Running here; not shared. The share couldn’t be saved: /home/sam/.svrngs/lends/.test-available.json.partial cannot be written: Is a directory (os error 21).
Shared with house. house can open it again after you sign in.
```

```json
{"case":"failed save","verdict":"passed","exit":1,"retry":0,"pid":"1867"}
```

**Concurrent withdrawal — passed.** The no-withdrawal control confirmed
successfully (exit 0). At the same terminal prompt in the next attempt,
`unshare` removed the standing declaration before the worker sent `y`.
The stale review did not restore it or advance its tombstone:

```text
svrngs: Something changed since you reviewed this.
```

```json
{"case":"concurrent withdrawal","verdict":"passed","exit":1,"before":3,"after":4}
```

**Old link and data — passed.** Before and after the reboot, the same
address returned the same bytes and HTTP 200. Sam's only session after
boot was the lingering user manager; no command was typed as Sam between
the two reads. The approval changed and the unit grant did not. The
successful renewal and wrong-marker control were recorded:

```text
why="written, unless it changed meanwhile" kept=Some(Ok(true)) renewed=Ok(())
```

```json
{"event":"planted wrong marker","expected":"wrong-d37fe84e7385476fa613991c75699d95\n\n200","received":"d37fe84e7385476fa613991c75699d95\n\n200","accepted":false}
{"case":"old link and data","verdict":"passed","url":"http://xhz2gx1p9lu2g2tcb0m3k5q2rvawzq9gxfklzlbak5k47ng2tdwt5hx9rzhkf1.localhost:43375/marker","boot_before":"77b3f695-b76a-4566-b086-e99264d383ce","boot_after":"3856806d-014d-4994-a656-caf03ca9e7ff","before":"d37fe84e7385476fa613991c75699d95\n\n200","after":"d37fe84e7385476fa613991c75699d95\n\n200","renewed":true}
```

**Stop sharing — passed.** `cut.py` waits for the first marker line before
starting a monotonic clock in the guest. Omitting `unshare` leaves the
same stream running past the deadline. With `unshare`, Alex's answer is
cut in 1.011 s including the command, a fresh request gets 404, and Sam's
local answer still returns the marker:

```text
the member's answer still carries on after 2 s
member stream ended after 1.011 s (exit 18): curl: (18) end of response with 65340 bytes missing
```

```json
{"case":"stop sharing","verdict":"passed","local":"d37fe84e7385476fa613991c75699d95","fresh_request_status":"404"}
```

The driver removed `svrngs-keeper-test-available.service`, powered off the
guest and discarded its scratch disk. The physical GPU, passphrase disk,
idle suspend, Mac reboot and real people sessions are **never-ran** in
this unit and belong to the release-hosts evidence.
