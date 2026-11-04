# Programs run directly on a Linux computer — 2026-10-30

A by-hand pass for svrn-docs `rings/current/GIVE_A_LEND.md` §6 ("A program
run directly on the computer"), on Linux, after the Mac spike
(`../../2026-10-28-lend-spike/`). Three questions: does a program on
loopback need the forwarder under Linux's pasta; how Give reads a process
from its port on Linux without root or `lsof`; and which way a big
tutorial install, Open WebUI by `uvx`, becomes a card. n8n by `npx` was
observed briefly beside it.

## How it ran

- The Fedora box (`ssh alexbryan@fedora`): Linux 7.2.8-200.fc44 x86_64,
  podman 5.8.7 rootless, `networkBackend: netavark`, `rootlessNetworkCmd:
  pasta` (pasta 0^20260728), SELinux `Enforcing`; uv 0.11.7, node v20.20.2.
- Everything was bound to 127.0.0.1 and named `svrngs-pass-*`. At the end
  `podman ps --all`, `podman pod ps` and `podman volume ls` showed only
  what was there before (four containers and three `svrngs-gate-*`
  volumes, never touched), no process of this pass was left (`ps`, `ss`),
  and `~/.n8n`, which n8n made in this run, was removed.
- **`give check` was not run.** The box's checkout is at `3bc6e27b` and has
  no `apps/give` (`ls apps/give`: no such file); `cargo build -p give`
  answered "did not match any packages" and built nothing. So the cards
  here are written in the spike's shape and are unchecked by Give; each
  names the hand-run that stands for it.
- No process's environment was read. Account passwords and tokens used in
  the write tests were never printed.
- Left on the box: images `ghcr.io/open-webui/open-webui:v0.12.5` (4.64 GB),
  `python:3.11.15-slim` (129 MB), `python:3.11-slim` (130 MB),
  `python:3.12-slim` (124 MB), `alpine/socat` (10.1 MB); caches: the uvx
  env `~/.cache/uv/archive-v0/NGaSzeqy18D22xuYLd6WR` (6.9 GB; `~/.cache/uv`
  is 15 GB in all), uv's CPython 3.11.15 (102 MB), npx's n8n
  `~/.npm/_npx/a8a7eec953f1f314` (2.2 GB), the MiniLM model in the
  operator's `~/.cache/huggingface`, and `~/svrngs-pass` (248 KB, the
  scripts and outputs).

## Q1. Loopback on Linux: the forwarder is needed here too

`q1-loopback/q1.sh` ran `python3 -m http.server --bind <addr> 8000` in
`python:3.12-slim`, published with `-p 127.0.0.1:<port>:8000`, and curled
it from the host (`q1-loopback/notes.txt`):

- Bound to `127.0.0.1`: `curl: (56) Recv failure: Connection reset by
  peer`, under the default network (which `podman inspect` names
  `pasta`), under `--network pasta`, and under `--network bridge`.
- Bound to `0.0.0.0`: 200 under all three.
- In a pod with `alpine/socat` (the spike's digest) forwarding 8001 to
  `127.0.0.1:8000`: 200 under the pod's default (pasta), `--network
  pasta` and `--network bridge`. The same pod publishing 8000 straight to
  the loopback program: reset.

The reason is visible in the container's `/proc/net/tcp` during a
working curl: the connection arrives at `192.168.1.12:8000`, the host's own
address that pasta copies into the namespace, not at loopback. So **the
Mac spike's finding holds on Linux with pasta and with netavark's
bridge, and the forwarder can be unconditional for a loopback-only
program**, as GIVE_A_LEND §6 step 4 says.

pasta has a switch that makes the forwarder unnecessary:
`--network pasta:--host-lo-to-ns-lo` ("Translate host-loopback forwards
to namespace loopback", `pasta --help`) answered 200 for the
loopback-only program. A card can't ask for it, since `card.rs:296-302`
refuses every `network_mode`, and it wouldn't help a Mac; the forwarder
stays the one way.

## Q2. Observation on Linux: `ss` and `/proc`, as the person

`q2-observe/observe.sh <port>` goes from a port to everything the spike's
`lsof` gave, as uid 1000 with no root. On a test program (a python
server on 127.0.0.1:18100 appending to `app.log`, `q2-observe/notes.txt`)
and on Open WebUI and n8n:

- **The pid:** `ss -ltnpH 'sport = :<port>'` shows `users:(("python3",
  pid=…,fd=4))` for the person's own sockets. For another user's socket it
  shows no `users:` at all (sshd on :22, same file), so a lend whose
  process isn't the person's is visible as such. The same answer comes
  without `ss`: the socket's inode in `/proc/net/tcp{,6}` (state `0A`,
  which also gives the bound address and the owner's uid), matched against
  `readlink /proc/*/fd/*` = `socket:[inode]`.
- **The command:** `/proc/<pid>/cmdline`. **The folder:** `readlink
  /proc/<pid>/cwd`. **The executable:** `readlink /proc/<pid>/exe`, which
  resolves the symlink to the versioned binary (`/usr/bin/python3.14`;
  uv's `cpython-3.11.15-…/python3.11`), a better runtime key than
  `--version`.
- **The parent and how it was started:** `PPid` in `/proc/<pid>/status`,
  walked up; and `/proc/<pid>/cgroup`, which names a systemd unit when
  there is one (here every process was in `session-*.scope`, so no
  service manager would restart it). For both tutorials the listener is
  a child of a launcher, not the command the person typed: `uv tool uvx
  …` for Open WebUI, `npm exec n8n` for n8n (which also has a second child,
  its task runner). A handover's SIGTERM goes to the launcher's process
  group, as this pass's own stop did (`kill -TERM -- -<pgid>`).
- **Files open for writing:** each `/proc/<pid>/fd/*` that links to a
  regular file, with `flags:` in `/proc/<pid>/fdinfo/<fd>` having
  `O_ACCMODE` 1 or 2. It gave `app.log`; n8n's `~/.n8n/database.sqlite`
  and its `-wal`/`-shm`; Open WebUI's `webui.db` and `chroma.sqlite3`.
- **What it misses:** a file written once at start and closed. Open WebUI
  writes `.webui_secret_key` into the folder it was started in and never
  holds it open, and its embedding model is only mmapped, from the
  operator's shared `~/.cache/huggingface`. So the open-for-writing list
  finds the data's directory, not every piece of state; the folder it ran
  in has to be copied as well, as the spike did.
- `/proc/<pid>/environ` was never read. `/proc/1/fd` of a root process is
  `Permission denied`, as expected.

## Q3. Open WebUI by `uvx`: where it is, and which way makes it a card

**The install.** The README now shows `pip install open-webui` on Python
3.12; the brief's tutorial line was run as given, from an empty folder,
with no `DATA_DIR`: `uvx --python 3.11 open-webui@latest serve --host
127.0.0.1 --port 18080`. It resolved open-webui 0.12.5, installed 261
packages (6.9 GB, CUDA torch included), fetched an embedding model from
huggingface.co at first start, and listened about 13 minutes after the
command (`q3-open-webui/notes.txt`, "The install").

**Where the code is:** an ephemeral env in uv's cache,
`~/.cache/uv/archive-v0/NGaSzeqy18D22xuYLd6WR`, on uv's own CPython
3.11.15 under `~/.local/share/uv/python` (`pyvenv.cfg`). **Where the data
is: inside the package, in that cache**, at
`…/site-packages/open_webui/data` (`webui.db`, `vector_db`, `uploads`).
Confirmed from the open files and from the code: `serve` sets
`FROM_INIT_PY`, and `env.py:230-250` then makes `OPEN_WEBUI_DIR / 'data'`
the data dir. For the person this means their accounts and chats live in a
directory that `uv cache clean`, or a later `@latest` that resolves
differently, can drop. The secret that signs sessions is a third place,
`.webui_secret_key` in the folder it was started in
(`open_webui/__init__.py:14`).

**Model A, copy the folder it ran in onto the runtime's image: no.** The
folder holds only `.webui_secret_key`. On `python:3.11.15-slim` with the
folder at its path, the observed command fails at once: `crun: executable
file …/bin/python not found`, exit 127 (`model-a.txt`). For a tool run by
`uvx` or `npx`, the code is not in the folder it ran in. (n8n is the same:
it ran in an empty folder, its code is in `~/.npm/_npx/<hash>`, its data in
`~/.n8n`; `q4-n8n/notes.txt`.)

**Model B, reinstall the same pins into a deps volume, data dir as state:
yes, after one fix.** There is no lockfile, but the env is its own lock:
`uv pip freeze` of it gives 261 exact pins with no URL or git pins. In
`python:3.11.15-slim` (the patch version `exe` named), uv installed them
with `--no-deps` into a volume mounted at the env's own path, in 13m45s,
7.1 GB (`q3b-install.sh`). The data dir and the folder it ran in were
copied into volumes at their paths (`q3b-run.sh`), with a socat door, as Q1
requires for `--host 127.0.0.1`.

The first start **failed**: `PermissionError … open_webui/data/readme.txt`
at `env.py:242`, in `shutil.copy2`'s copy of the `security.selinux`
xattr. The cause is not the copy of the data but the reinstall: the wheel
ships `site-packages/data/readme.txt` (its `RECORD`, line 2), and the
original's first start moved that folder into `open_webui/data`, zipped it
and deleted it. A fresh install brings it back, so the container's first
start ran that one-time move again, onto the copied state, and SELinux
refused the label copy. It had already overwritten `readme.txt` in the
copy before failing. `envdiff.py`, which compares an env with its own
`RECORD` files, finds exactly this: "removed since install:
`data/readme.txt`", and, not in any RECORD, the data files and
`open_webui_data.zip`. Removing `site-packages/data` from the deps volume
and taking a fresh copy of the state, it started in 116 s (fetching 931 MB
of models into a cache volume) and passed the start test below.

**Route C, Open WebUI's own image, data dir as state: yes, simpler.**
`ghcr.io/open-webui/open-webui:v0.12.5`, pinned, with the observed data dir
copied to `/app/backend/data` (`q3c-run.sh`). Two things the image does
differently: it keeps its key file in `/app/backend`, its working
directory and not a volume (`start.sh:31`), so the card sets
`WEBUI_SECRET_KEY_FILE` into the state and the original's key is copied
there; and it listens on `0.0.0.0:8080` (`start.sh:34`), so no forwarder.
The data written by 0.12.5 on Python 3.11 opened under the image's Python
3.12 without a migration error.

**The start test.** `GET /` on the original, B and C: each 200,
`text/html; charset=utf-8`, 11,775 bytes, the bodies identical. The
headers differ only in `last-modified`/`etag`, which come from the file's
mtime (the original's install at 17:02:44 GMT, the reinstall at 17:20:53,
the image's build on 29 Oct), and in `x-process-time`. `/api/version`
said 0.12.5 on all three. In each container one write was made: the first
account (it became `admin`) and one chat. Both were in the state volume's
`webui.db`, read from a copy of its files, and after `podman restart` the
same account signed in and listed the chat (`write-b.txt`, `write-c.txt`).
The original still had no user afterwards (`onboarding: True`).

**What B and C cost.** B's card (`compose-b-reinstall.yaml`) has to carry
the 7.2 GB env as state, since a keeper has no install step, and its CUDA
wheels are x86_64: it can never run on an arm64 Mac keeper. C's card
(`compose-c-own-image.yaml`) carries about 1 MB of state plus the models
the app fetches into it (932 MB here), and pulls a 4.64 GB image by a
multi-arch digest (amd64 and arm64) that every keeper can share.

## What this settles, and what it doesn't

Settled, on this box:

1. The forwarder is needed for a loopback-only program on Linux, under
   pasta and under bridge (Q1). The `--host-lo-to-ns-lo` alternative is a
   network option a card can't carry.
2. Observation on Linux needs nothing but `ss` (or `/proc/net/tcp`) and
   `/proc/<pid>/{cmdline,cwd,exe,status,cgroup,fd,fdinfo}`, as the person.
3. "Copy the folder it ran in" is the rule for a script in its folder (the
   spike), and fails for a tool installed by `uvx`/`npx`/`pip`, whose code
   is elsewhere.
4. For Python, a tool's env is its own lock (`uv pip freeze`) and its own
   diff (`RECORD`): files a RECORD lists that are gone are the program's
   own removals, which a reinstall must replay; files no RECORD lists are
   the program's writes into its install, which are state.
5. Where the app publishes its own image at the version observed, that
   image with the observed data as state is the simpler and more portable
   card. That makes a small table, Give's data like `runtimes.txt`: a
   package name and version → the publisher's image, its data path and the
   settings that move its state into it (here `open-webui` →
   `ghcr.io/open-webui/open-webui:v<version>`, `/app/backend/data`,
   `WEBUI_SECRET_KEY_FILE`). Reinstalling is the fallback for what the
   table lacks.

Found on the way, owed to the design:

1. `podman image inspect`'s `.Digest` on Linux is the platform manifest,
   not the index (`skopeo inspect --raw`: `image.manifest` for 78b39ef1…,
   `image.index` with amd64 and arm64 for 90744cff…). A card must name the
   index digest, or a keeper of the other architecture pulls nothing it
   can run.
2. Files arrive in a volume from `tar | podman volume import` owned by
   uid 1000 inside the container, not root (`model-a.txt`); the app ran as
   root and wrote them anyway. On the Mac the spike saw uid 502 the same
   way.
3. A start test that fails can have written into the copy (B's first start
   rewrote `readme.txt`), so each attempt starts from a fresh copy, as the
   spike's `give check` finding already suggests for the check's volumes.
4. A first start can take minutes and fetch a gigabyte (B, C), so the
   health wait for a given app is longer than for a card written by hand.
5. Caches outside the data dir (`~/.cache/huggingface`, shared with the
   operator's other models) are not state: the copy fetches its own.

Not tried: `give check` on these cards (no `apps/give` on the box), a
handover of a real program, a lend served by these, n8n's card (its
`package-lock.json` in `~/.npm/_npx/<hash>` would be its pins, and `~/.n8n`,
with an `encryptionKey` in `config`, its state), an arm64 keeper, and
whether the copied sqlite with its WAL, taken while the original ran, is
always consistent (the original was idle here).
