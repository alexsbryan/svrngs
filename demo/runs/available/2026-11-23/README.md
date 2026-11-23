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
