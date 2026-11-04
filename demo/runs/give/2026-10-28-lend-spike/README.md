# A program run on the Mac, kept in a container by hand — 2026-10-28

A spike for svrn-docs `rings/current/GIVE_A_LEND.md`: can a program a
person started on their computer, with no container, become a card that
Give's own check passes? This was done by hand, with plain `podman`
commands and the derivation done in my head rather than by code, on the
one program this Mac had on a port that wasn't a lend by GIVE.md §10's
rule: a node program on 127.0.0.1:7333. The original was never stopped,
signalled or changed. It answered 200 before and after.

## How it ran

- podman 6.0.1, its machine `applehv`, netavark; macOS 27.0; svrngs at
  `dc719d39`, `target/debug/give` built from it (`cargo build -p give`).
- Every container, pod and volume was named `svrngs-spike-*` or, by `give
  check`, `svrngs-check-canon-term*`; none was left (`podman ps --all`,
  `podman volume ls`). The three images pulled stay in Podman's machine:
  `node:24.22.1` (1.16 GB), `node:24.22.1-slim` (255 MB),
  `alpine/socat` (10.5 MB).
- The process's environment was not read.

## What was done, and what each step found

**1. From the port to the process.** `lsof -nP -iTCP:7333 -sTCP:LISTEN`
gave pid 84105. `ps`, plus `lsof -d cwd` and `lsof -d txt`, gave:
- the command, `node server.mjs`;
- the folder it runs in, `/private/tmp/canon-term` (13 entries);
- the executable, `~/.nvm/versions/node/v24.22.1/bin/node`;
- its parent, launchd;
- the only file it holds open for writing, `server.log`;
- that it listens on `127.0.0.1` alone.

The program is a web terminal: each visitor gets `bash` through `python3
-c 'import pty…'` (`server.mjs:19`), with its homes under `ROOT`, which
defaults to `/tmp/canon-term` (`server.mjs:7`).

**2. The runtime's image.** `node:24.22.1-slim`, pinned to
`sha256:932a44ad…`, arm64. It has bash and no python3.

**3. The copy.** The folder was copied into a volume with `tar | podman
volume import`. Two findings:

- **AppleDouble files.** macOS's `tar` added a `._*` file beside every
  file carrying extended attributes. `COPYFILE_DISABLE=1 tar --no-xattrs
  --no-mac-metadata` copies the folder as it is.
- **Owners.** Files arrived owned by uid 502, the Mac's. The program runs
  as the container's root, which rootless Podman maps to the person, so
  it could write them.

**4. Started as it ran.** The folder's volume was mounted at the path it
ran at, with the same working directory and command, and port 7333
published to `127.0.0.1:17333`. The program started and printed its
`terminal on http://127.0.0.1:7333/`. Through the published port: `curl`
exit 52, an empty reply. **A program that listens on loopback alone, in a
container, can't be reached through a published port** (on this Mac's
machine, with netavark; untested with pasta on Linux).

**5. The forwarder.** In a pod, `alpine/socat@sha256:603174b7…` was run
beside the program, listening on 7334 and forwarding to
`127.0.0.1:7333`. Containers in one pod share loopback, so through the
pod's published 7334 it answered 200 `text/html`, as the original does.

**6. The same answer.** Hitting `GET /` on both: the original's page was
1,775 bytes and the container's 1,749. They differed by one attribute,
`aria-label="Command line"`. The running process started at 08:52:58,
and `server.mjs` was changed at 08:53:24 (`ps -o lstart`, `stat`). So
**the original runs an older version of its code than its folder
holds**, and the container runs the folder's. Comparing the two answers
caught it. The page should tell the person.

**7. The shell route, on slim.** Opening `/events` and posting a command
exited the program: `spawn python3 ENOENT`. This is the start-test
failure GIVE_A_LEND.md predicts. The log names the missing program.

**8. The shell route, on the full image.** `node:24.22.1`, pinned to
`sha256:a217980f…`, has python3 and bash. The posted `echo
hello-from-$(uname -s) as $(id -u) home=$HOME` came back as
`hello-from-Linux as 0 home=/tmp/canon-term/homes/spike`. Two findings:

- The rule "on a missing program, try the runtime's full image" fixed it
  with no agent.
- **The home was not in the volume.** `ROOT`'s default is `/tmp/…`, and
  on a Mac `/tmp` is `/private/tmp`. The volume was mounted only at the
  real path, so the home went to the container's throwaway layer
  (`ls /tmp/canon-term/homes` showed it; the volume's didn't).

**9. Both paths.** The same volume was mounted at `/private/tmp/canon-term`
and at `/tmp/canon-term`. A file written through the shell at
`/tmp/canon-term/homes/spike2/note.txt` was in the volume, and still
there after `podman restart`.

**10. Give's own check.** The card those steps describe is
`compose.yaml` here. `Card::parse` read it with no change to Give: two
services, `command`, `working_dir`, one volume mounted twice, and the door
on the forwarder's port.

- **First run:** `failed healthy: app stopped, exit 1`, with node's
  `Cannot find module '/private/tmp/canon-term/server.mjs'`. `give check`
  starts a card from empty first, and **for a program given this way the
  code is in its state, so it can only start from a copy.**
- **A run that failed at its start** (a pod of its name left from the
  first) removed the volume that had been loaded before it ran, by its
  name (`check.rs`, "remove what the check made"). So the run after it
  started from empty again.
- **The last run,** with the check's volume loaded with the copy beforehand,
  passed every step: read, asks, machine, pull by digest, start, healthy,
  snapshot (paused 907 ms, reflink, owner `0:0:755`), stop, restore ("the
  state is the snapshot's"), start again, healthy again, remove.

## What this settles, and what it doesn't

Settled, on this Mac, for one script on an interpreter: observation, the
runtime's image by digest, the copy, the forwarder, the same-path mount
and its `/tmp` alias, the start test catching a missing program, and
Give's snapshot and restore on the result. All of it is deterministic. The
one failure that needed judgment, python3, has a general rule.

Owed to the design, from this run:

1. `give check` needs a mode that starts from a copy (`--from`). A given
   app never starts from empty.
2. The same-path mount also covers the path's aliases: `/tmp`, `/var` and
   `/etc` are `/private/…` on a Mac.
3. The copy turns off macOS's metadata.
4. The forwarder is added when the program listens on loopback alone,
   checked on Linux's pasta before it is made unconditional.
5. The answer comparison reports a running process whose code is older
   than its folder, rather than failing on it.
6. `give check` removes only what it made: by this run it removes a
   volume of its name made before it (finding in step 10).

Not tried: the handover (signalling the original, never done to a real
program here), reading the environment, a project with dependencies and a
lockfile, a Linux computer, and a second keeper.

A program like this one gives a shell to whoever reaches it. Run directly
on the Mac, that is a shell as the operator. In the container, it is a
shell as root of a rootless container whose only mount is its own volume.
A keeper who takes it over runs that shell too, so their yes says what it
is: "node server.mjs from Alex's folder, not a published app".
