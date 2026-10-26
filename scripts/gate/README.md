# The gate on a Linux computer

`scripts/check.sh` is the definition of done (AGENTS.md). The Containerfile
here is what it needs on a Linux computer that has none of it: Rust, Node,
Playwright's Chromium and WebKit, the wasm tools and Jellyfin, every one
pinned, in a rootless podman image. Its command runs `npm ci --prefix web`,
then `scripts/check.sh`, in the checkout at `/work`; its output and exit are
check.sh's.

From a svrngs checkout:

    podman build -t svrngs-gate -f scripts/gate/Containerfile scripts/gate
    podman run --rm --init --userns=keep-id:uid=1000,gid=1000 \
      --security-opt label=disable --shm-size=2g --pids-limit=-1 \
      --network=host -v "$PWD:/work" \
      -v svrngs-gate-cargo-registry:/home/gate/.cargo/registry \
      -v svrngs-gate-cargo-git:/home/gate/.cargo/git \
      -v svrngs-gate-npm:/home/gate/.npm svrngs-gate

The build takes about 10 minutes, once. Each flag is for a reason:

- `keep-id` maps you onto the image's user, `gate`, uid 1000, so the
  checkout's `target/` and `web/node_modules/` stay yours and no test runs
  as root: one that makes a file unreadable and expects it unread would
  pass as root for the wrong reason. `label=disable` lets the container
  read the checkout without relabelling it for SELinux.
- Chromium wants more shared memory than podman's 64 MB, and a test run
  more threads than its 2,048 pids.
- The host's network, as the gate has it on a Mac. In podman's own
  (pasta), the grapevine scene's restarted member was not reached again
  within its 18 s in 5 runs of 5, and was in 3 of 3 on the host's: Sam's
  fresh dial to the restarted Jo, through the scene's relay, got no
  answer in 4 s, though Jo had already dialled Sam through it. So the
  ephemeral ports are the host's too, 32768-60999 on Linux, which hold
  the room's 40000-48999 (crates/svrngs-node/src/room.rs `port_of`;
  docs/PLAN.md, nextest's row).
- The three volumes are the caches every run shares; `podman volume rm`
  them to start clean. `target/` stays the checkout's own: cargo judges a
  build fresh by mtime, so a target shared by two checkouts could keep one
  older than its sources. Give each run a checkout of its own, or move one
  on with `git fetch` and `git checkout`.

## Why a container

Playwright's WebKit is built for Ubuntu. On Fedora it does not start
("Host system is missing dependencies to run browsers"), and the libraries
it asks for need `sudo` and Ubuntu's names. The image is Ubuntu 24.04, its
apt archive fixed to one day's snapshot, so the gate needs nothing on the
host but podman, and installs nothing there.
