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
