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

Pass `--tty` to run this scene with the node on an unanswered terminal and
linger off; see the [stdio proof](../2026-11-23-run-stdio/README.md).

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

## Run confirmation and named audience, 2026-11-23

Run `bash demo/runs/keeper/2026-11-23-run-page/gate.sh` with the new
assertions, first leaving the old Run sentence in `plan/words.rs` as the
plant. The test reads the confirmation immediately after Run, checks it
against the rendered heading, and still checks the separate lifecycle row.
The Share review and the saved row must both name `house`, whose real seed
Sam and Alex both keep. The stored audience must use that group's id.

**Red — the old Run response:**

```text
assertion `left == right` failed: Run confirmation under #run-form
  left: "test-page-coder is running here. Share it separately with `svrngs share`."
 right: "test-page-coder is running here. To share it, find it under Already running here."
test explicit_run_then_share_in_webkit ... FAILED
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 49 filtered out; finished in 8.35s
```

**Green — the same command with the new Run response** (review excerpt):

```text
WebKit Run confirmation: test-page-coder is running here. To share it, find it under Already running here.
house can open it again after you sign in.
WebKit Share: ["test-page-coder: everyone in the groups you chose at once, 0 using it now. Lent to house (yours, since 23 Nov)."]
WebKit Alex opened: {"title":"Run page witness","url":"http://umfrj1nxbj23n9mv5p0w3rv7rnghrcq9d0mbfh9wrvfadqasq7ra5fbvu8pvr9.localhost:55077/"}
test explicit_run_then_share_in_webkit ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 49 filtered out; finished in 20.02s
```
