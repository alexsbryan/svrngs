# Mac renewal on beefymac, 2026-11-23

The first evidence below is the prototype at `b3a2ce78`; its `run.py`
is available at that revision. The current `beefymac.sh` exercises the
first Share with both login services registered under scratch names, then
each independent plant and the restored implementation. Its plants and
the in-flight withdrawal barrier change only the Mac's scratch archive.

## Prototype evidence

`run.py` uses two scratch svrngs nodes, one group they create for the
run, and the native `svrngs-keeper-test-mac-renew` launch agent. A child
of the agent holds the listening socket and takes requests. The fixture
records every nonempty request and sends a fixed reply. Alex opens Sam's
share through Play, so the same address is exercised through the door.

Build a scratch `git archive` checkout under `~/svrngs-keeper-scratch/repo`
with `SOURCE_DATE_EPOCH=1795392000 cargo build --locked --ignore-rust-version
-p svrngs -p svrngs-keeper`. Copy `run.py` alongside `repo` and run
`/usr/bin/python3 -u ../run.py` from `repo`. Use beefymac's GUI session;
the fixture uses separate scratch HOME and SVRNGS_DIR for each process.

The scratch node isn't an installed login service. `plan.rs:151–164`
therefore reviews this run only; the driver follows it with an explicit
`svrngs approve coder`. This measures renewal of an existing grant, not
the installed Share start-chain path.

**Red on `d621e4c9`:** a real agent serves the original Share, but the
explicit approval saves no grant.

```text
Approve: launchd pid=90961 grant label=None
Cleanup: launchctl print exits 113
AssertionError: the observed launch agent must be granted
```

**Unlanded prototype:** the saved grant matched the label and SHA-256 of
the test plist. On a later PID, the same address answered, and only the
saved approval changed. Excerpts from the run:

```text
Approve: launchd pid=90649 grant label=svrngs-keeper-test-mac-renew
Restart: pid 90649 -> 90707; same address 200; approval changed=True
Changed plist: HTTP 502; bytes=0; share unchanged=True
Unrelated holder: HTTP 502; bytes=0; share unchanged=True
Already held: HTTP 502; bytes=0; share unchanged=True
Withdrawn: HTTP 404; no declaration=True
Cleanup: launchctl print exits 113
```

The prototype exceeds the row's size allowance. These runs do not close
B2: independent implementation plants for the refusal branches and the
installed Share path are still owed. Reboot and sign-in were not run.
The test agent was unloaded, the test plist and both scratch nodes were
removed with `~/svrngs-keeper-scratch`, and no containers were started.

## Projected login is not a running PID

Toolbox: `TOKIO_WORKER_THREADS=4 cargo test --locked -p svrngs-node --lib job::mac::tests`.
With the `runatload` predicate removed, the otherwise well-formed reply for
PID 37 was accepted without login registration:

```text
assertion failed: !registered(&good.replace(" | runatload", ""), path, Some(37))
test result: FAILED. 0 passed; 1 failed
```

Restored:

```text
test job::mac::tests::only_the_installed_agent_and_the_asking_daemon_match ... ok
test result: ok. 1 passed; 0 failed
```

## First Share and renewal on beefymac

**Passed.** Reproduce with `bash demo/runs/keeper/2026-11-23-mac-renew/beefymac.sh`.
The run below used the driver's stages separately: archive `613f3ef1`, build
with `controls.py plant`, run `run.py <case> <case>` for each plant, restore
the archived sources, build with `controls.py test`, and run `run.py all`.
The final withdrawal marker was added to both builds in the scratch copy.

The first attempt (`avail-run-mac-renew-4.await.log`, lines 8–9 and 39–44)
had an app witness, but no grant: the scratch door plist didn't match the
installer's `SVRNGS_DIR` spelling (`web/install.sh`, `cli.rs` `kept_over`).
With both registrations verified, the terminal's legacy restart warning
also failed the copy assertion; `613f3ef1` omits it for a saved grant.

**Red, independently planted:** each got through the new connection
and exited 5. Bytes are measured at the receiving processes; in the
withdrawal plant the declaration stayed absent even though a request got
through.

```text
changed: HTTP 200; bytes (app,foreign)=(546, 0); state unchanged=False
holder: HTTP 200; bytes (app,foreign)=(546, 0); state unchanged=False
taker: HTTP 200; bytes (app,foreign)=(0, 546); state unchanged=False
held: HTTP 200; bytes (app,foreign)=(546, 0); state unchanged=False
Withdrawal at door: carried
withdrawn: HTTP 200; bytes (app,foreign)=(546, 0); state unchanged=True
```

**Green, all five restored (exit 0):** both plists were registered from
the scratch HOME. First Share stored the label and that plist's SHA-256.
After the app restarted, Alex opened the same Play address and received
the same saved data without reapproval, and only `approved` changed in
the declaration. The fixture compares the entire declaration on each
refusal (or, for withdrawal, verifies it stayed absent). It observes
the withdrawal decision at the door because Alex's Play address may
close before returning that decision as an HTTP answer.

```text
Shared with Scratch. Scratch can open it again after you sign in.
pid = 5653
properties = runatload | inferred program
pid = 5687
properties = keepalive | runatload
Share: label=svrngs-keeper-test-mac-renew, plist hash matches=True
Restart: pid 5687 -> 5748; same address 200; approval changed=True
changed: HTTP 502; bytes (app,foreign)=(0, 0); state unchanged=True
holder: HTTP 502; bytes (app,foreign)=(0, 0); state unchanged=True
taker: HTTP 502; bytes (app,foreign)=(0, 0); state unchanged=True
held: HTTP 502; bytes (app,foreign)=(0, 0); state unchanged=True
Withdrawal at door: refused
withdrawn: HTTP 502; bytes (app,foreign)=(0, 0); state unchanged=True
Approve: instance-only
Cleanup gui/502/svrngs-keeper-test-mac-renew: launchctl print exits 113
Cleanup gui/502/svrngs-keeper-test-mac-renew-door: launchctl print exits 113
```

After the green run, `cargo check --locked --offline --ignore-rust-version
-p svrngs -p svrngs-keeper` passed against the uninstrumented Mac source.
Both labels still returned 113, and `test ! -e "$HOME/svrngs-keeper-scratch"`
passed after cleanup. The two scratch nodes, plists, and archive were
removed. The packaged reboot and sign-in proof was not run here.
