# Withdrawal ownership experiment

Run after the survey commit `0f2a2e02`, before any production scope policy.
`measure.py prototype` copies the retained `prototype.rs` and
`prototype.toml` into a temporary directory outside the workspace, executes
each variant in the foreground with offline cargo, and removes the scratch.
The manifest uses only dependencies already in the node's manifest; it is
not a workspace member or a product dependency addition.

## Result, including the planted failures

One experiment, not a deployment measurement or a rate. The guard uses the
existing one-second interval and fake current policy (`watch<bool>`). Each
plant disables that guard for exactly one transport. The unchanged 1,250 ms
assertion must then fail; the driver checks the actual exit is 101. Actual
output is retained in the four `prototype-{body,splice,raw,udp}.txt` files.

```text
$ python3 demo/runs/economize/2026-11-04-lend-scope/measure.py prototype
plant=body
body: ... Err(Elapsed(())), elapsed 1.250783s
upstream must close inside 1250 ms even when body is unpolled
exit 101

plant=splice
body: ... Ok(Ok(())), elapsed 980.462333ms
splice: peer EOF: Err(Elapsed(())), elapsed 1.252156125s
upgraded copy must close inside 1250 ms
exit 101

plant=raw
body: ... Ok(Ok(())), elapsed 979.776084ms
splice: peer EOF: Ok(Ok(0)), elapsed 980.330166ms
raw: owned child socket EOF: Err(Elapsed(())), elapsed 1.252192333s
raw children must close inside 1250 ms
exit 101

plant=udp
body: ... Ok(Ok(())), elapsed 980.531042ms
splice: peer EOF: Ok(Ok(0)), elapsed 980.96475ms
raw: owned child socket EOF: Ok(Ok(0)), elapsed 979.318958ms
udp: owned flow dropped: Err(Elapsed(())), elapsed 1.252577625s
UDP child must be dropped inside 1250 ms
exit 101

plant=none
body: ... Ok(Ok(())), elapsed 980.53025ms
splice: peer EOF: Ok(Ok(0)), elapsed 980.297792ms
splice: unrelated TCP channel still carries bytes
raw: owned child socket EOF: Ok(Ok(0)), elapsed 980.150375ms
udp: owned flow dropped: Ok(Ok(())), elapsed 982.260875ms
udp: closed flow receives no post-withdrawal packet
passed: independent body/upgrade/raw cancellation primitives
exit 0
```

The actual logs contain the full commands, source lines, exit codes and
outputs; ellipses above abbreviate only the repeated observation label.

## What was exercised

- **HTTP body, actual Hyper HTTP/1 over loopback TCP:** upstream sends an
  endless response. A task pumps frames through a capacity-one channel. The
  downstream body is intentionally never polled until after withdrawal; a
  separate readiness channel proves the pump filled and blocked on send.
  Withdrawal drops the upstream body and its `AbortOnDropHandle` connection
  driver. The observation is the **server's transport driver ending**, not
  an echo of policy. Subsequent body polling returns refusal before queued
  data. This closes the source even under complete downstream backpressure.
- **Upgrade splice primitive:** actual TCP streams, the same Tokio
  `copy_bidirectional` used after WebSocket upgrade, carry bytes before the
  change. Guard cancellation drops only those streams; peer sees EOF while
  an unrelated live TCP pair still carries new bytes. This does **not**
  exercise a WebSocket handshake, `OnUpgrade` or iroh multiplexing.
- **Raw TCP children:** `JoinSet` owns the copier's task and both sockets.
  Cancelling the parent drops its task set and the origin peer sees EOF.
- **UDP flow:** an owned task sends a real packet to a loopback UDP socket.
  Withdrawal drops the task/socket owner, its drop observation arrives, and
  the packet-input receiver closes; a subsequent packet is refused and none
  arrives at the echo socket. UDP has no peer EOF, so this observes ownership
  destruction plus no new packet instead of inventing a transport close.

## Minimum production design resolved

1. Reuse one current audience/witness resolver; store no second membership
   snapshot in capacity counts. Check original device (or live guest),
   lender standing, selected IDs and the originally injected person key.
   Re-resolve over the union of eligible groups, so withdrawal of one group
   leaves an independently selected witness for the **same identity** valid;
   ambiguity or loss of all witnesses cancels only this use. Owner uses are
   explicit. The declaration and registers, not the fake watch value, are
   authoritative in production.
2. Use an independently scheduled interval guard around the **whole**
   upstream request and its response pump; include connect/header/upload
   waits and blocked queue sends. A body wrapper timer alone cannot revoke
   while downstream is unpolled. Give the pump ownership of the upstream
   driver and capacity hold so cancellation releases them without waiting
   for downstream drop. Hold an abort-on-drop task handle in the returned
   body. Distinguish normal EOF, source error, consumer abandonment and scope
   withdrawal; the scratch is a cancellation experiment, not a finished
   reusable HTTP body implementation.
3. Guard both upgrade acquisition and upgraded copy; transfer ownership
   across the 101 handoff. Do not close the shared HTTP/member iroh
   connection to withdraw one use. This is the same drop/`select!` primitive,
   not a registry/service of another component's lifecycle.
4. For raw, keep the existing **one-lend** connection sweep, but reload its
   declaration and witness. Own per-flow/per-stream tasks in a task set or
   give each its own guard; do not leave detached UDP flows/TCP copiers
   depending on eventual channel drain. Closing a dedicated raw connection
   is legitimate; closing a shared member/sync connection is not.
5. Keep existing app-choice yeses on audience edits: they record consent to
   an app, not a group grant, and can never bypass the live scope check.
   Audience-only edits preserve credentials/issuer/client and unrelated
   configuration. Removing/re-adding a group does not invent a second
   consent reset policy.

**Tested primitive bound:** 1,250 ms ceiling, observed worst green 982.261 ms
after withdrawing just after an initial sweep. The one-second sweep plus
scheduler/I/O overhead is not a hard real-time guarantee. The implementation
row should target and prove **within two seconds** on real node processes,
including pending headers, active/backpressured bodies, WS, TCP/UDP,
continued selected-group access and unrelated lends/sync. This scratch
does not establish that production bound: disk reads, membership folds,
upgrade races and iroh stream reset remain to be integrated and tested.

## Estimate after prototype

No additional cancellation dependency, active-use registry or core change
is needed by the prototype. Retain **+400–700 non-JS / +135–235 JS**, Give
separately **+10–25**, with the HTTP/upgrade slice **+80–150**. The prototype
contains experiment servers, observations and four test subjects, so its raw
length is not a measured production addition. The production implementation
must stay within those guardrails, with necessary actual increments only;
no caps change and no budget savings are booked in this row.
