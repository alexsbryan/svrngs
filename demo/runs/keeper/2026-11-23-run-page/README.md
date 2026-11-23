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

## Prefill, 2026-11-23

`cargo test --locked -p svrngs-keeper --test promote -- --nocapture` checks
the real keeper CLI. Both fixture processes can answer `/props`; only the
one identified as llama-server should suggest a command. The supplied
model path has spaces and an apostrophe and must parse as one argument.
On the old card-only answer the new test failed:

```text
assertion `left == right` failed
  left: Null
 right: "coder"
test facts_suggests_fields_only_for_llama_server ... FAILED
```

With the fields in place:

```text
llama-server: "prefill"
other-listener: "unavailable"
test facts_suggests_fields_only_for_llama_server ... ok
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

`cargo test --locked -p svrngs-node --test home prefill_is_local_and_read_only -- --nocapture`
drives the home origin with a session proof. Before the closed query was
parsed, the extra command reached the list handler; red, then green:

```text
assertion `left == right` failed: closed prefill query: {"error":"svrngs couldn’t verify this. Check it from your terminal, then look again.","kind":"closed"}
  left: 403
 right: 400
test prefill_is_local_and_read_only ... FAILED

foreign: 403; extra command: 400; local unread: null; no lend
test prefill_is_local_and_read_only ... ok
```

`bash demo/runs/keeper/2026-11-23-run-page/gate.sh --prefill` uses WebKit,
the real local routes and the keeper. With `--plant-submit`, the form is
submitted programmatically before the confirming press; with `--plant-edit`,
the fixture lets a person edit the field while the reply is pending but
intercepts the input event before it can invalidate the suggestion. Both
plants failed on their own assertion:

```text
AssertionError [ERR_ASSERTION]: Run submitted without a confirming press
1 !== 0
test run_prefill_waits_for_confirmation_in_webkit ... FAILED

AssertionError [ERR_ASSERTION]: late answer replaced Sam’s edit
+ `llama-server --model "/tmp/.tmpyqe2rS/Sam's model.gguf" --host 127.0.0.1 --port 20100`
- 'Sam’s own command'
test run_prefill_waits_for_confirmation_in_webkit ... FAILED
```

Without the plants, the late answer leaves the edit intact. A second
late answer cannot fill the form opened from another row. The model's
suggestion shows its actual port and folder, and the edited command runs
only after a press. The original listener remains alive and no lend exists.
The gate stops its test service and discards its scratch home and manager.

```text
WebKit prefill: {"command":"llama-server --model \"/tmp/.tmpKO1RM3/Sam's model.gguf\" --host 127.0.0.1 --port 20100","confirmedPosts":1,"folder":"/tmp/.tmpKO1RM3","lateEdit":"preserved","otherRow":"blank command","port":20100}
test run_prefill_waits_for_confirmation_in_webkit ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 50 filtered out; finished in 5.75s
```
