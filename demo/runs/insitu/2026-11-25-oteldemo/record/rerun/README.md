# The rerun on the fixed projection (b8d33b6c) — projected arm only

Only the two results that depended on what the fixes took away were
rerun, one agent each, with the same prompt and protocol. The raw arm
sees nothing different, so it wasn't rerun.

| Fault | Before the fixes | After | What the agent used in place of what was removed |
|---|---|---|---|
| F2 one product fails | 4/4 (read the free-text event name) | **4/4** | the flag's key and value, and the per-product split by hash. It listed the event name (now shaped) as unseen and didn't need it |
| F3b cache grows | 4/4 (read exact digit counts) | **4/4** | length bands: 2–3 → 4–7 → 8–15 digits, then a reset at each restart |

Leak check: P passed in both windows (`F2-leakcheck.txt`,
`F3b-leakcheck.txt`), and the plant on R failed in both. One run each:
the fixes cost no diagnosis here, which isn't a rate.

Also fixed for the rerun, in lab.sh:
- the views now start `--fresh`, since read from the start they took
  longer than the share's wait over a 200 MB file;
- the lab waits for both views' ports before sharing.
