# The tutorial-app pass — 2026-10-30

This pass asked one question for svrn-docs `rings/current/GIVE_A_LEND.md`
§11. The 7333 spike's model is that the program's folder is the app
(`../2026-10-28-lend-spike/`). Does that hold for what a lightly
technical person installs from a tutorial?

Each app was taken through the spike's steps by hand. On the operator's
Mac (`mac/README.md`): n8n and Actual through `npx`, a Flask app in a
venv, and Uptime Kuma in Podman, once with a named volume and once with a
folder from the disk. On the Fedora box (`fedora/README.md`): Linux's own
container networking with a program on loopback, reading a program from
its port without root, Open WebUI through `uvx`, and n8n again. Every
original was left running and untouched. Every container, pod and volume
was removed on both machines (`podman ps --all`, `podman volume ls`).

## The answer

**Copying the folder started none of the four tutorial installs.** Their
code was in npx's or uv's cache, or in a venv whose Python is the Mac's.
Their data was in `~/.n8n`, inside the package, or in the folder they ran
in. Two other routes started every one of them, each with its writes
still there after a restart:

- **The same versions, reinstalled inside the runtime's official image**
  at the paths they ran at. This started n8n, Actual, Flask and Open
  WebUI. Pins came from npx's own `package-lock.json`, or from a venv's
  `dist-info` and `uv pip freeze`; "no lockfile" was never true. It is
  heavy: n8n's dependencies are 2.7 GB, and Open WebUI's are 7.1 GB, took
  13m45s, and are x86_64 only.
- **The app's own published image**, with the person's data copied into
  its state. This started Uptime Kuma (both variants) and Open WebUI
  (`ghcr.io/open-webui/open-webui:v0.12.5`, about 1 MB of state, a
  multi-arch digest). n8n and Actual publish images too, and weren't tried
  this way.

The folder model still holds for what the spike had: a script in its own
folder.

## Causes, counted across both machines

| Cause | Count | Where |
|---|---|---|
| The folder held no runnable program | 4 of 4 tutorial installs | mac n8n, Actual, Flask; fedora Open WebUI |
| The command `ps` shows can't be run as it stands (a renamed process, the framework's `Python.app` stub) | 2 of 4 | mac n8n, Flask |
| Data outside the folder | 2 of 4 | mac n8n (`~/.n8n`); fedora Open WebUI (inside the package) |
| Data not held open, caught only by copying a whole folder | 3 | mac n8n's key, Actual; fedora Open WebUI's `.webui_secret_key` |
| A program on loopback alone can't be reached through a published port | both machines | spike; fedora Q1, on pasta, the default and bridge alike |
| The slim image can't build a native dependency | 1 of 2 npm installs | mac Actual (`better-sqlite3`, no python3) |
| The forwarder's port is one the app also listens on | 1 | mac n8n (5679) |
| A reinstall brings back a file the first start removed | 1 | fedora Open WebUI (`data/readme.txt`, found by `RECORD`) |
| A failed start writes into the copy it started from | 1 | fedora Open WebUI |
| `podman image inspect` gives a one-platform digest | Linux | fedora |
| `give check` starting from empty | every app | as the spike found |

## Give's bugs, found on the way

- A check that fails at `healthy` leaves its pod and volumes behind, in
  3 of 3 runs. `pod stop --time 0` leaves the forwarder running, so `pod
  rm` exits 125, and `remove()` returns before the volumes
  (`apps/give/src/runtime.rs:610-629`). The run's own failure is returned
  first (`check.rs:96-99`), so this error is never shown. Its "Its last
  lines:" is printed empty.
- `remove()` removes by name, not by what the check made (the spike).

## A safety note

On the Mac, `pgrep -lf` on n8n's renamed child printed past its command
line into its environment. The agent cut the output there and recorded
none of it. So observation must read the command line by a call that
returns the arguments alone, never through `ps` or `pgrep` output.
