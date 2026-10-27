# give-redial (b): which remedy lands

**The question:** should the restarted computer be reached through a
patched iroh (B, the cause's own three-line fix, published as a fork), or
through svrngs closing every connection to that key before it dials again
(A, about 15 lines of svrngs, which needs a raise)?

The row says to make "the smallest change the traces name". The traces name
a rule inside iroh 1.0.2, and the row does not say whether svrngs patches a
dependency. Neither remedy fits the cap's one line of room in svrngs's own
code. A is a raise, which is the operator's. B adds no counted line, but it
needs an iroh fork published somewhere, which is outward-facing.

## What was found

`README.md` beside this file has the full account and its evidence:

- **The rate at 15cacd50.** This Mac: 9 of 10 runs reached Jo within 18 s,
  and 1 waited 59.97 s. Fedora under pasta: 0 of 3, each about 60 s.
- **The cause, the same on both hosts.**
  - Sam's iroh keeps old Jo's dead direct address selected.
  - While the kept connection holds that address, its stale RTT keeps the
    selector from switching to the path Jo's own dial establishes.
  - When the kept connection times out, iroh selects nothing new
    (`iroh-1.0.2/src/socket/remote_map/remote_state.rs:473-490`).
  - The fresh dial's handshake then goes only to the dead address (`:799-809`).
  - A pass on this Mac is Jo's second holepunching round, which happens to
    land within the dial's 4 s.
- **iroh 1.3.0 behaves the same.** It is the newest iroh in this computer's
  cargo registry: `remote_state.rs:475-492` and `:805-811` are unchanged
  there.

## The two remedies, each tried alone on 15cacd50 and reverted

| | this Mac | Fedora, pasta |
|---|---|---|
| A: svrngs closes every connection to the key | 3 of 3, 4.08-6.74 s | 3 of 3, 4.04-4.05 s |
| B: iroh selects again when a connection closes | 3 of 3, 4.07-6.73 s | 3 of 3, 4.03-4.12 s |

### B: iroh selects again when a connection closes (recommended)

This is the cause's own fix. Closing a connection that held the selected
path now selects among the paths still open, as an abandoned path already
does (`remote_state.rs:598-627`). The diff, against iroh 1.0.2, as tried
(the comment to be written for upstream):

```diff
--- a/src/socket/remote_map/remote_state.rs
+++ b/src/socket/remote_map/remote_state.rs
@@ -486,6 +486,9 @@
         if self.connections.is_empty() {
             trace!("last connection closed - clearing selected_path");
             self.state.selected_path = None;
+        } else {
+            // The closed connection may have held the selected path.
+            self.select_path();
         }
     }
```

How it lands in svrngs, once the operator publishes the fork. Two lines go in
`Cargo.toml`, and `lean_check` counts neither:

```toml
[patch.crates-io]
iroh = { git = "<the operator's fork of n0-computer/iroh>", rev = "<v1.0.2 + this commit>" }
```

- **Cost:** a fork to keep until n0 takes the fix upstream, an issue or PR
  for n0 (the operator's to file), and the fork re-made at each iroh upgrade.
  Vendoring iroh in this repository instead would put its whole crate under
  `lean_check`'s count.
- **Nothing a person does breaks.** No connection is closed that iroh would
  have kept.

### A: svrngs closes every connection to the key before it dials again

iroh drops a key's selected path only when no connection to that key is
left (`remote_state.rs:486-488`). So when a request on the kept connection
fails, `fetch` would close every connection to that computer before its one
fresh dial, including the ones it dialled into this computer. The diff as
tried, crude, with a static the clean version replaces:

```diff
@@ impl IrohTransport { fn fetch
         tracing::debug!(target: "transport", ..., "iroh: a request failed on the connection kept to it, ...");
+        for c in EXPERIMENT.lock().unwrap().iter().filter(|c| c.remote_id() == peer.key) { c.close(0u32.into(), b"experiment"); }
+        for c in self.conns.borrow().iter().filter(|(k, _)| k.0 == *peer.key.as_bytes()).map(|(_, c)| c) { c.close(0u32.into(), b"experiment"); }
         whole(self.send(peer, again.map(Into::into))).await
@@ IrohAcceptor::spawn
                     let alpn = conn.alpn().to_vec();
+                    EXPERIMENT.lock().unwrap().push(conn.clone());
```

The clean version:

- **What it adds:**
  - the transport holds the connections its acceptor admitted, keyed by
    computer, in a `watch` like `conns`;
  - `IrohAcceptor::spawn` takes the transport (in `node.rs`, and in two
    tests in `iroh/tests.rs`);
  - the acceptor removes a connection when it closes;
  - `fetch` closes them all, with a trace saying why.
- **Size:** about 15 lines in `.`, none in svrngs-core. The raise, to
  land after an economizing pass (AGENTS.md "Done"):

  ```diff
  -BUDGET = {"crates/svrngs-core": 3242, ".": 19431}
  +BUDGET = {"crates/svrngs-core": 3242, ".": <where it lands + 1>}
  ```

- **Cost:** what the restarted computer just dialled into this one closes
  once. Say Jo opened a lend on Sam's computer in the 4-8 s before Sam's
  request failed: that stream is cut once, and Jo's page must open it again
  (fence 2).
- **It leans on iroh's rule, `:486-488`.** That rule is iroh's own and can
  change in any upgrade.

## Where things are

- The instrument is committed at 15cacd50 and the run at f4514ae7. No
  product code has changed: `python3 scripts/lean_check.py` prints
  `. 19430 / 19431 lines`.
- The Fedora box has a checkout at `~/dev/svrngs-redial`, at 15cacd50, with
  a warm `target/`. Two scripts in its `target/` run the instrument in the
  gate image: `redial-pod.sh <cmd>` on pasta, and `redial-runs.sh <label>
  <first> <last>`. The iroh copy tried there is in
  `target/redial/vendor/iroh`.
- Once a remedy is chosen, the row's (b) and its check are still to do:
  - red first: the instrument at HEAD, and a plant that restores the cause;
  - the grapevine scene green in ten runs in a row here;
  - the Fedora image green on pasta;
  - `scripts/check.sh` once, at the close.
