# Four tutorial installs, given by hand — 2026-10-30, on the Mac

The lend spike (`../../2026-10-28-lend-spike/`) gave one program, `node
server.mjs`, by copying the folder it ran in, code and data together. Its
model, written into svrn-docs `rings/current/GIVE_A_LEND.md` §6, is: the
folder is the app; run it on the runtime's official image. This run asks
whether that holds for programs a lightly technical person starts from a
tutorial, where `npx` or `pip` puts the code somewhere else and the app
puts its data where it likes. Four were tried: n8n (`npx n8n`), Actual's
sync server (`npx @actual-app/sync-server`), a 35-line Flask app in a
`.venv`, and Uptime Kuma in Podman, run with a named volume and then with a
bind mount.

**In short:** copying the folder (model A) started none of the three
programs that ran without a container. Reinstalling the same versions
inside the image, into a volume at the path they had on the Mac (model B),
started all three. Their data was found from the files each held open for
writing. Every card passed `give check` once its volumes were loaded with
the copy. Five things went wrong along the way, each with a general rule.
In one place a person still has to judge: a write test needs each app's
own API.

Each folder here has the card (`compose.yaml`) and `notes.txt`, the raw
outputs this README cites.

## How it ran

- podman 6.0.1, machine `applehv`, netavark; macOS 27.0; svrngs at
  `abba7495`. `target/debug/give` was built on 2026-10-27, and `git log
  --since='2026-10-27 11:50' -- apps/give` is empty, so it is current.
- node v24.22.1 (nvm) for both npx apps: n8n 2.45.7 needs `>=24.0.0` and
  Actual 26.10.0 needs `>=22` (`npm view <pkg> version engines`). So node
  20 was not used. The Mac's Python is 3.13.3.
- Each program ran from `/private/tmp/svrngs-pass/<app>/work`, bound to
  127.0.0.1. Both npx programs ran with `HOME=/private/tmp/svrngs-pass/<app>/home`,
  which stands in for the person's home. That way the npx cache and
  `~/.n8n` landed where they would under a real home, without touching the
  real one.
- No process's environment was read. Container inspect was read field by
  field, never `.Config.Env`. One slip: `pgrep -lf` on n8n's task-runner
  child printed its retitled command line, and that ran on into its
  environment block (`PATH=…`). I cut the output there and did not record
  it. That is a finding of its own (below).
- At the end, `podman ps --all`, `podman pod ps` and `podman volume ls`
  were all empty. Every process I started was stopped with SIGTERM after a
  check that its cwd was under `/private/tmp/svrngs-pass`. The node on
  7333 still answers. Images left in Podman's machine: `node:24.22.1`
  (1.16 GB), `node:24.22.1-slim` (255 MB), `uptime-kuma:1` (469 MB),
  `python:3.13.3-slim` (151 MB), `alpine/socat` (10.5 MB). The scratch tree
  `/private/tmp/svrngs-pass` holds 4.5 GB, most of it n8n's npx folder and
  npm cache. It is left for anyone who wants to look again.

## What each app needed

| app | code was | data was | model A | what started it | `give check` |
|---|---|---|---|---|---|
| n8n | `~/.npm/_npx/a8a7…/` (2.9 GB) | `~/.n8n/` (6.9 MB) | no: the folder is empty, `ps` says `n8n` | B: `npm ci` from the npx folder's lockfile, full image, `HOME` set, door off 5679 | passed, preloaded |
| Actual | `~/.npm/_npx/ae37…/` (137 MB) | the folder it ran in (72 KB) | no: the code is not in the folder | B for the code, the folder for the data; full image | passed, preloaded |
| Flask | the folder, with a `.venv` (18 MB) | `notes.db` in the folder | no: `.venv/bin/python` points to the Mac's Python | B: a venv made in the image at `.venv`, pinned from its `dist-info` | passed, preloaded |
| Kuma, volume | the image | volume at `/app/data` | (container) | the inspect fields; the image's own entrypoint | passed, empty and preloaded |
| Kuma, bind | the image | `work/data` on the Mac | (container) | bind made a named volume filled from the folder | passed, preloaded |

### n8n (`n8n/`)

**Observation.** Port 5678 led to pid 15851. `ps` shows its command as
`n8n` alone, because the process renames itself. Its parent, `npm exec
n8n`, is the tutorial's `npx n8n`. Its folder is empty (`ls -la work`:
`total 0`). The code shows only through native modules in `lsof -d txt`,
such as `…/home/.npm/_npx/a8a7eec953f1f314/node_modules/sqlite3/build/Release/node_sqlite3.node`.
The data shows through the files it holds open for writing:
`~/.n8n/database.sqlite`, `-wal` and `-shm`. The rest of `~/.n8n` is not
open, including `config`, which holds the instance's encryption key. So
the whole folder of the open database was copied, not just the open files.
It also listens on 127.0.0.1:5679, its task broker.

**Model A.** The empty folder, run as `n8n`: `Cannot find module
'/private/tmp/svrngs-pass/n8n/work/n8n'`.

**Model B.** The npx folder holds its own `package.json` and
`package-lock.json` (2,439 resolved entries). Those two files went into a
volume mounted at the npx folder's path, and `npm ci` ran in
`node:24.22.1`: 2,209 packages, 55.7 s, 2.7 GB, n8n 2.45.7. The card runs
`<npx folder>/node_modules/.bin/n8n` with `HOME` set to the person's home
path, and mounts the data at `~/.n8n`.

**Start test.** The first try put the socat door on 5679, and n8n exited
with `Task Broker's port 5679 is already in use`: the door had taken the
port of the app's own second listener in the pod's shared loopback. With
the door on 25678, it started in 3 s, with none of the "Generated new … key"
lines its first start printed. `GET /`: 200 `text/html` 1,708 bytes from
both. The bodies differ only in the per-response CSP nonce (5 lines, all
`nonce=`). The owner made on the original logged in to the container, and a
workflow made there (`written-in-container`) was in the volume's
`workflow_entity` and still listed after `podman restart`.

**Give's check.** From empty, the run failed at `healthy`, twice. The
preloaded run passed every step. Its snapshot paused the app for
**12.3 s**, because the 2.7 GB deps volume is part of the state.

**Afterwards.** `npm ci` passed on slim too, and n8n started there. The
Python task runner that slim lacks fails on the Mac and on the full image
as well ("its virtual environment is missing"). So the rule "slim first"
would have picked slim for n8n.

### Actual (`actual/`)

**Observation.** Here `ps` shows the whole command, `node
…/_npx/ae37a5acd0f97261/node_modules/.bin/actual-server`, with `npm exec
@actual-app/sync-server` as its parent. The one file open for writing is
`work/server-files/account.sqlite`, in the folder it runs in. The package's
config default points somewhere else: the package's own folder, inside the
npx cache (`load-config-*.js:39`). Its bin overrides that: the data goes in
`./data` if it exists, else `./` (`actual-server.js:47-58`). I read the
config first and guessed wrong. The open file was right.

**Model B.** On `node:24.22.1-slim`, `npm ci` failed, twice:
`better-sqlite3` ran `node-gyp rebuild` and found no python3. The package
ships a `binding.gyp` and no install script, so npm runs node-gyp
implicitly. It even ships `prebuilds/linux-arm64.node`, but the rebuild
runs anyway. On `node:24.22.1`: 209 packages, 7.7 s, 138 MB.

**Start test.** `GET /`: 200 `text/html` 4,256 bytes from both, and
`cmp` found the bodies identical. The container answered
`bootstrapped: true` and took the original's password. A password change
made there held after `podman restart`: the old password then got 400 and
the new one 200. The original still took the old password.

**Give's check.** From empty: failed at `healthy`. Preloaded: passed, with
a 2.5 s pause.

### Flask (`flask/`)

**Observation.** `ps` shows
`/Library/Frameworks/Python.framework/…/Python.app/Contents/MacOS/Python
app.py`. That is the framework's stub, not `.venv/bin/python`, so `ps`
does not show the venv at all. The only sign of it is a native module
loaded from it (`lsof -d txt`:
`.venv/lib/python3.13/site-packages/markupsafe/_speedups.cpython-313-darwin.so`),
and `.venv/pyvenv.cfg` in the folder. A venv of pure-Python packages would
show nothing in `txt`. The file open for writing is `work/notes.db`.

**Model A.** The folder, `.venv` included: `.venv/bin/python` is a symlink
to `/Library/Frameworks/…/python3.13`, so `sh: .venv/bin/python: not found`
(exit 127). The image's own python found no flask: `ModuleNotFoundError: No
module named 'flask'`.

**Model B.** `requirements.txt` says `flask`, with no version. The venv's
`site-packages/*.dist-info` gave the exact versions (seven pins, pip left
out). A venv made with the image's python, in a volume mounted over
`.venv`, installed them in 3.8 s (19 MB). `pip check` was clean.

**Start test.** `GET /`: 200, 164 bytes, identical bodies. A note posted
in the container was in the volume's `notes.db` after `podman restart`.

**Give's check.** From empty, it failed at `start`, exit 127: the
executable `.venv/bin/python` was not found. Preloaded: passed, with a
1.9 s pause.

### Uptime Kuma (`kuma/`, `kuma-bind/`)

**Observation.** On a Mac, port 3001 is held by `gvproxy`, Podman's
machine forwarder, which serves every published port. Its cwd is
`~/dev/commonwealth-ai`, wherever the machine was started. From there the
way to the app is Podman's table (`podman ps --format '{{.Ports}}'`), and
from the container the card follows from the inspect fields. The
entrypoint, command, working directory and user are the image's own, so
the card leaves them out. The restart policy `always` is dropped. The
container's `ImageDigest` is the tag's multi-arch index (`70233f4a…`), not
the arm64 manifest that was pulled (`7752fbff…`), so the card pins the
index. The image has no healthcheck. Its manifest is OCI
(`application/vnd.oci.image.manifest.v1+json`) and carries none, though
`/app/extra/healthcheck` exists. So "healthy" means "running", and Give's
snapshot can land while Kuma is still starting.

**Bind variant.** The README's command with `-v ./data:/app/data` failed
under Podman, `Error: lstat data: no such file or directory`: Podman does
not create a missing bind source the way Docker does. After `mkdir data`
it ran. The literal card, with the bind kept, is refused when the card is
read: `a bind mount reaches this computer's own files`. Made into a named
volume and filled with a `tar` of the Mac folder, it works.

**Start tests.** Both variants: `/dashboard` 200, 2,493 bytes, identical
bodies. The admin made on the original logged in to the copy. A monitor
added there was still listed after `podman restart`, and the original had
none.

**Give's check.** Named volume, from empty and preloaded: passed, with a
0.8 s pause. Bind variant, preloaded: passed.

## What failed, and why

Across the three programs that ran without a container:

- **The folder did not hold the program: 3 of 3.** For n8n and Actual the
  code was in the npx cache. Flask's code was in the folder, but its
  `.venv` is the Mac's (a symlink to the framework Python, plus darwin
  `.so` files).
- **The command `ps` shows could not be run as it stood: 2 of 3.** n8n
  renames itself to `n8n`. Flask shows the framework stub, not the venv's
  python. Only Actual's command was usable as `ps` gave it.
- **The data was outside the folder: 1 of 3** (n8n, `~/.n8n`). Actual's
  data was in its folder, in spite of its config's default. Flask's was in
  its folder.
- **Some of the data was not held open: 2 of 3.** n8n's `config` (its
  encryption key) and Actual's `.migrate` and `user-files/`. Copying the
  folder of each open file, whole, caught both.
- **The slim image could not install the deps: 1 of 2 npm installs**
  (Actual, node-gyp needs python3). n8n's install and run worked on slim.
- **The door took a port the app uses: 1 of 3** (n8n's 5679).
- **No pinned versions in the project: 1 of 3** (Flask's
  `requirements.txt`). The installed `dist-info` pinned them.
- **`give check` from empty failed: 3 of 3.** The spike found the same.
- **Settings in the environment were not carried: 3 of 3.**
  `N8N_LISTEN_ADDRESS`, `N8N_DIAGNOSTICS_ENABLED`, `ACTUAL_HOSTNAME` and
  `HOME` were never read. None stopped a start. Two changed behaviour: in
  the container both servers listen on every interface rather than
  loopback (harmless in a pod), and n8n's telemetry is back on.

For the container, the derivation from the inspect fields was mechanical,
and both variants passed. The two snags were Podman's missing bind source,
a tutorial step rather than Give's, and the image having no healthcheck.

Give itself:

- **A `give check` that fails at `healthy` leaves its pod and volumes: 3
  of 3 runs** (n8n twice, Actual once). With `RUST_LOG=debug`,
  `pod stop --time 0` exits 0 but leaves the door container running. Then
  `pod rm` exits 125 ("cannot remove container … as it is running").
  `remove()` returns there, before the volumes (`runtime.rs:610-629`), and
  `check.rs:96-99` returns the run's failure first, so that error is never
  printed. The one run that failed at `start` (Flask) cleaned up. Also,
  the `healthy` failure prints "Its last lines:" followed by nothing; the
  lines appear only in the debug trace.
- **netavark's "nft did not return successfully"** came up twice on my own
  `podman pod rm -f` / `podman rm -f`, as `runtime.rs:613-616` records. A
  second try, or the container being gone anyway, settled both.

## The rules each start needed

All of them are general; none names an app.

1. **The program's code is wherever its loaded code lives, not its
   folder.** On a Mac, `lsof -d txt` lists the native modules loaded, and
   a path under `node_modules` or `site-packages` names the install: the
   npx folder, or the venv. When no native module is loaded (a pure
   JavaScript package, a pure-Python venv), this shows nothing. In that
   case the parent's command (`npm exec <pkg>`) and the folder's
   `pyvenv.cfg` are what is left.
2. **Reinstall, don't copy, the install.** For npm: `npm ci` in the image
   from that install's own `package-lock.json`, which npx writes every
   time, so an npx app always has a lockfile. For a venv: a new venv at the
   same path, pinned from its `dist-info`. The design's rule, "no lockfile
   means no", would wrongly refuse Flask: the venv's installed metadata is
   a lockfile.
3. **The data is the folder of each file held open for writing, copied
   whole, plus the folder the program runs in.** This found every app's
   data, including files that weren't open.
4. **The command is the install's entry point, not what `ps` shows,** when
   the process has renamed itself (n8n) or `ps` shows an interpreter stub
   (Flask). For npx that is `<npx folder>/node_modules/.bin/<bin>`; for a
   venv, `<venv>/bin/python` and the script.
5. **`HOME` in the container is the person's home path,** so whatever the
   program reads under `~` is at the same path.
6. **The door's port is outside every port the process and its children
   listen on.**
7. **Slim first, the full image when the install or the start fails for a
   missing program.** This is the spike's rule, reached here at install
   time as well as at the start.
8. **For a container: from the port to the container through Podman's
   ports table, never through the process** (on a Mac the process is
   `gvproxy`). Each bind mount becomes a named volume filled from its
   folder. The restart policy is dropped, and the image is pinned to the
   digest the container names.

## What this changes in the design, and what contradicts the spike

- **"The folder… Code and data go together"** (GIVE_A_LEND §6, step 3)
  held for the spike's `node server.mjs` and for none of these three.
  Model B, together with the data found from open files (rules 1–5), is
  what the evidence favors for npx and pip installs. Copying the folder is
  only the data half, and only when the data is there.
- **"Observe … gives its command"** (§6, step 1): true for the spike and
  for Actual, false for n8n and Flask.
- **"Without a lockfile … the answer is no"** (§6, step 3): npx always
  leaves a lockfile, and a venv's `dist-info` is one.
- **Deps are not state.** Putting n8n's 2.7 GB deps volume in `state`
  made the snapshot pause 12.3 s, against 0.8–2.5 s for the others. Every
  copy to the group would carry it. They can be rebuilt from the lockfile
  on the keeper, as they were here.
- **The spike's finding 6, from the other side.** There, a check that
  failed at its start removed a volume made before it. Here, a check that
  failed at `healthy` left everything behind. Both say the same thing:
  `remove()` is not scoped to what the check made, and its errors go
  unreported.
- **Settings.** Every program here had a setting in its environment that
  Give cannot see. None of them stopped a start. The design's "settings by
  name" ask only comes in after a failed start, so a setting that changes
  behaviour without failing (n8n's telemetry) goes unnoticed.
- **`ps` on a Mac can print environment.** A process that renames itself
  with a long title can show its environment in `ps`/`pgrep -f` (n8n's
  task-runner did). Observing by the command alone isn't safe from the
  "never read the environment" rule. `observe.rs` should take the
  executable from `txt` and argv from `KERN_PROCARGS2` only up to `argc`.

## Where a person's knowledge was still needed

- **The write test needed each app's own API:** n8n's REST login and
  `/rest/workflows`, Actual's `/account/*`, Kuma's socket.io events, the
  Flask form. Give's start test compares `GET /`. That would not catch
  data written somewhere that wasn't copied: every `GET /` here matched
  before any write. Only the writes showed the state is where the card
  says.
- **Kuma's real healthcheck** (`extra/healthcheck`) is in the image but
  nowhere in its metadata. Only someone who knows the image would add it.
- **Making the programs listen on loopback** (`N8N_LISTEN_ADDRESS`,
  `ACTUAL_HOSTNAME`) needed their docs. Their tutorials listen on every
  interface. That was for this run's rule, not for Give.

Nothing else needed the apps' docs. I read Actual's code to explain where
its data went, but the open file had already shown it.

## Not tried

A Linux computer (`ss`/`/proc`, pasta), a pure-Python venv with no native
module, a global `npm install -g` (the Actual README's first suggestion),
`uvx`/`pipx`, the handover, a program kept by a service manager, a second
keeper, and the copy taken with the app stopped. Every copy of a live
SQLite file here was taken with the app running, and none came back
damaged. Kuma's was paused.
