# Run a native model on macOS

On beefymac, use the scratch build of `svrngs` and its sibling keeper,
scratch HOME and SVRNGS_DIR, and one `svrngs-keeper-test-*` launch agent.
Use Qwen2.5-0.5B-Instruct Q4_K_M from the approved demo model.

1. Run with a listener already on the port, each of IPv4 wildcard, IPv4
   loopback and IPv6 loopback. Expect a busy refusal, no setup written, and
   the original listener unchanged.
2. Run llama-server with Metal through `svrngs run`. Expect a verified
   port, a suggestion from `facts`, a running list row, and HTTP health.
3. Crash just that agent's process with SIGKILL. Expect a new launchd
   PID and HTTP health again. Stop through `svrngs-keeper`; the job and port
   should stay down past the restart interval, with no login plist.
4. Start and restart again, then remove. Check the agent and its files
   are gone and the card and model still exist before cleaning scratch.

Sign-in and reboot are HUMAN-release-hosts checks.

## Run and lifecycle controls

Run `runs/keeper/2026-11-23-mac-proof/proof.py` with the scratch keeper on
beefymac. The HTTP fixture needs no model or group. Run the controls first
with the respective guard absent, then with the shipping binary:

1. Wait for a real launch agent that never binds the requested port. Only
   then let an unrelated process listen there and answer HTTP 200. Record
   both PIDs and the Run result: ready and running, but unverified.
2. Bootstrap a test plist at the keeper's expected path without using
   Run. With no saved card, Stop, Start, Restart and Remove must all refuse
   with the registration and PID unchanged.
3. Verify a genuine Run on its own listener. Refuse a symlink in place of
   its plist, then restore the regular file and exercise Stop, Start and
   Remove. Confirm the card remains but the agent and plist do not.
