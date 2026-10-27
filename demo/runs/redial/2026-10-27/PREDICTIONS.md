# give-redial (a): predictions, written before the first run

The instrument: `cargo test -p svrngs --test scenes -- redial --nocapture`
(`crates/svrngs/tests/scenes/redial.rs`), the grapevine scene's step 12
without a browser. Sam's write reaches Jo once before the restart, so Sam
keeps a connection to Jo. Jo's svrngs is killed outright (`people.rs`
`again`: `Child::kill`, SIGKILL) and started again on a new UDP port. Sam
writes at once, and the test prints how long until Jo's copy holds the
write, up to 70 s. With `SCENES_TRACED=transport=debug,iroh::_events=debug`,
Sam's log holds every dial and every path iroh selects.

## The cause, predicted from iroh 1.0.2's source

- iroh keeps one state per remote key. While it has a selected path, it
  sends every new connection's packets to that path alone
  (`iroh-1.0.2/src/socket/remote_map/remote_state.rs:799-809`). The same
  was found for a SIGTERM at syp-6 b (c50d7f2a).
- Closing a connection selects no new path: `handle_connection_close`
  (`:473-490`) clears `selected_path` only when no connection to the key is
  left. A new path is selected only when a path is established or
  abandoned (`:588-627`).
- Restarted, Jo dials Sam through the relay at once. Sam's state for Jo then
  holds a connection whose path is the relay, beside the kept one, whose
  path is old Jo's dead direct address. That address stays selected.
- Sam's request on the kept connection fails when it has been silent for
  `DIAL_WITHIN`, 4 s (`dial.rs` `held`). `fetch` then dials afresh
  (`iroh.rs` `fetch`). Its handshake goes to the selected path, the dead
  one, so Jo never sees it, and it is `Asleep(4s)` at about 8 s. Nothing
  tries again until Sam's next round, at about 60 s.
- A run passes only when a path to new Jo is established and selected
  before the fresh dial's 4 s run out. This happens when holepunching on Jo's
  connection into Sam succeeds, or when the dead path is abandoned on it.

## What each run will show

- Failing runs: about 60 s (the next round), and over 18 s every time. In
  Sam's log, `path::selected` names old Jo's address until after the fresh
  dial gave up.
- Passing runs: under 10 s (4 s on the kept connection, then a fresh dial
  that is answered), and a `path::selected` to new Jo's address before the
  fresh dial.
- This Mac: mostly passing, since the scene's step 12 passed in every gate
  here but one (give-docs-2). I predict 1 to 3 failures in 10 runs.
- The Fedora box, in the gate's image on podman's own network (pasta): most
  or all runs fail, as the scene did 5 of 5 (d5c34ccc). The prediction is
  that holepunching there does not reach new Jo within 8 s.

## The smallest change, predicted

Before Sam's fresh dial, the dead path must stop being selected. The traces
will decide whether closing the connection kept to Jo's old self is enough.
That is enough only if iroh selects again when it is the last connection
with the dead path.
