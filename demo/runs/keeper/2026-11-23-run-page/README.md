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
