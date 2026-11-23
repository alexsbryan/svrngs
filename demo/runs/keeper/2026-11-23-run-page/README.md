# Explicit Run, then Share, in WebKit

From the keeper worktree, `bash demo/runs/keeper/2026-11-23-run-page/gate.sh --plant`
truncates the closing quote of the command typed into the real page. Repeat
without `--plant` for the scene. The driver checks a busy port, then starts
an HTTP fixture, stops and starts it again, and checks that no share exists
yet. It reviews and confirms Share separately and opens the fixture as the
second computer. This witnesses the browser and keeper seam; the real model
and no-login boots are the separate `2026-11-23-run-linux` proof.

The gate gets a private D-Bus and systemd user manager, a scratch home and
scratch groups. `ss` is installed inside the disposable container. Both
test units and their data are discarded with it. Neither the host's user
manager nor a host group takes part.

## Recorded run, 2026-11-23

The loop ran both commands at `d4765ba5` in 3m17s. Source:
`ralph/next/keeper/ctl/log/sessions/avail-run-page-2.await.log`.

**Red — `bash demo/runs/keeper/2026-11-23-run-page/gate.sh --plant`:**
The unterminated quote prevented Run on the free port, so the page never
showed “Run by svrngs” and the browser test failed (log lines 519–561):

```text
the page did not finish: TimeoutError: page.waitForFunction: Timeout 30000ms exceeded.
error: Failed to load resource: the server responded with a status of 403 (Forbidden)
test explicit_run_then_share_in_webkit ... FAILED
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 49 filtered out; finished in 37.74s
```

**Green — `bash demo/runs/keeper/2026-11-23-run-page/gate.sh`:**
The page said “Port 20000 is in use by browser, the copy you started: stop
it, then press Run.” The busy listener remained bound. On port 20001 the
fixture went from “Run by svrngs” to “Stopped” and back to “Run by svrngs”.
The test found no lend before Share, then found the saved job grant after
review and confirmation. The second node opened the fixture (log lines
617–623):

```text
WebKit Alex opened: {"title":"Run page witness","url":"http://umfrj1nxbj23n9mv5p0w3rv7rnghrcq9d0mbfh9wrvfadqasq7ra5fbvu8pvr9.localhost:49175/"}
test explicit_run_then_share_in_webkit ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 49 filtered out; finished in 17.01s
```
