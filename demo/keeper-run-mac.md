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
