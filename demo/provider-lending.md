# The library lends to two groups

Local run-of-show and acceptance contract, 2026-11-20.
See [provider lending due diligence](../docs/PROVIDER_LENDING.md) for the
architecture. This scene tests the native provider boundary in a terminal.

## Run it on one Mac

From the svrngs checkout, with the Rust dependencies already in Cargo's cache:

```sh
cargo test --offline -p svrngs-node --test door provider:: -- --nocapture
```

Run it with Wi-Fi off if you like. Nodes use temporary data directories,
disposable keys, ephemeral loopback ports and real, direct iroh connections.
The fixture uses no relay, public website, browser, Docker, or installed
svrngs daemon. The printed lines are the demo transcript.

**Today:** the command runs the baseline. The same counting localhost
service answers through a regular scoped lend (`200`, one hit), then a node
with no groups refuses a correctly signed outside pass targeting its paused
lend (`403`, no extra hit). It prints `PROVIDER_BASELINE=passed`. The native
provider scene is **never-ran** until the implementation adds it to this
module. A green baseline, or a command selecting zero tests, is not native
acceptance. The native scene must print `PROVIDER_ACCEPTANCE=passed` only
after all six steps below pass.

## Run of show

Play the librarian, Nora from **Waller Block**, and Eli from **Film Club**.
They want to know when they can use the same recording booth. Set up two
separate real group records with disjoint standing keys; one person per
group is enough for this seam. Neither group has the library key, and the
library node has no groups at all. Nora and Eli each use a distinct
service holder key. A third holder, Kim, knows the offer and has no grant.

The library runs a single localhost fixture that counts requests and
streams availability. It produces an initial reading and the operator can
release another reading on demand. The fixture doesn't authorize callers.
The runner stages the **same non-secret reference** for both readers and
uses the real provider-side provisioning path to approve the two keys.
Manually provisioning those keys is enough. Passing the reference through
a real chat or adding it to a group shelf isn't part of this seam.

| Bar | What the terminal should show | What must be observed independently |
|---|---|---|
| P1 · One service | **Waller Block: Saturday 2 pm free. Film Club: Saturday 2 pm free.** | Both requests went through the native lend door to the same approved endpoint and got bytes from it. Assert the library's namespaces are empty and the two group records name disjoint keys. Neither group gains the library key. |
| P2 · Address isn't access | **Kim: the booth isn't open to you.** | From Kim's key, a copied reference, Nora's claimed identity, or a valid ordinary member pass don't increase the origin count. A grant for a different label and an expired grant also hit nothing. These requests reach a live door; a timeout or an unknown route isn't a refusal. |
| P3 · Service only | **Booth opens; the rest stays closed.** | The upstream sees no group ID, group-page header, member identity, forged identity header, or injected owner credential on a successful read. A write or upgrade doesn't reach the fixture. Test the separate protocol boundaries using the same allowed holder: no access to member routes, carried apps, blob or raw-port ingress. |
| P4 · Revoke while open | **Nora: access ended. Eli: Saturday 3 pm free.** | Confirm both streams have received the first reading; keep the origin alive. Start timing when the provider confirms revocation. Nora's in-flight response is cut within **2 seconds** (reuse the existing door's sweep bar); her next request on a held connection and a fresh dial both fail before the origin. Release a new, distinguishable reading *after* that cut and observe it on Eli's already-open stream. |
| P5 · Reissue | **Nora opens again after the library approves her again.** | The old stream doesn't revive; a fresh open succeeds. Exercise a revoke/reissue before the recheck too, so replacing the grant does not keep the old stream alive. |
| P6 · The library closes | **Booth unavailable. Both groups still work.** | Stop only the provider. Each group still reads its own record and appends an independent entry using its normal authority. Assert it is present only in that group's record. No provider is needed to read or write either record. |

## How to judge the run

Reuse the [door tests](../crates/svrngs-node/tests/door/main.rs) for direct
QUIC, the counting origin, and the active-response instrumentation in
[`live.rs`](../crates/svrngs-node/tests/door/live.rs). For P4, retain the same
QUIC connection for the same-connection probe. The generic TCP bridge
alone is not proof of connection reuse. Time the cut against the completed
revocation using a monotonic clock. Check that the fixture still has data
to give so its own EOF can't satisfy the bar. Exercise the failed requests
against the protected route while the other positive controls remain live.
For P3, also check a provider with a separate staff group and a working
staff-only resource. Show that a staff member can reach it with existing
authority before checking that the provider holder cannot.

The gate must be seen failing with the verified-holder check dropped, with
the label check dropped, and with the in-flight grant recheck skipped. Save
those failures and the successful step transcript with the command, commit
and measured cut time under `demo/runs/provider-lending/`. This is an
automated engineering run with zero human participants. Phone, walk-up QR,
cross-network reach and real library use have their own acceptance later.
