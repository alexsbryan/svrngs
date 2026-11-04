# One app's state, from a Mac to Linux and back — 2026-10-30

A by-hand pass for Give's design: does a copy of a given app's state,
written on a Mac, start on Linux, and does a copy written there start back
on the Mac, each time by Give's own check? The app is the one the spike
gave (`../2026-10-28-lend-spike/`): the node program on the Mac's
127.0.0.1:7333, whose folder, `/private/tmp/canon-term`, is its state and
its code. The original was only read. It was pid 84105 before and after
(`lsof -nP -iTCP:7333 -sTCP:LISTEN`, `notes.txt` "Cleanup").

**Both directions passed every step of `give check`**, once the card named
its node image by the index digest. The spike's card named the arm64
manifest, which pulls on x86_64 and then can't run there.

## How it ran

- The Mac: podman 6.0.1, its machine `applehv`, arm64; svrngs at
  `ac91d77e`, `target/debug/give` up to date (`cargo build -p give`).
- The box (`ssh alexbryan@fedora`): podman 5.8.7 rootless, x86_64,
  SELinux Enforcing. Give was built there from an rsync of `apps/give`,
  `fold` included, with no change.
- A scratch binary that links the give crate, `xos-tool`, was used to make
  and load the copies and to start and stop the app by hand. It calls
  `Card::parse(..).version`, `App::snapshot`, `App::restore` and
  `App::differs`, `App::pull`, `start` and `wait_healthy`, and `App::stop`,
  so every copy here was written and read by Give's own code, in its
  format: `code.tar`, `code.root`, `card`. It lived in the session's
  scratch folder and in `~/svrngs-xos` on the box, and is not in the repo.
- Everything was named `svrngs-xos-*`, or `svrngs-check-xos-*` (`give check
  --name svrngs-check-xos`), and published on 127.0.0.1 alone. None of it
  is left on either machine (`podman ps --all`, `podman pod ps`, `podman
  volume ls`, `notes.txt` "Cleanup"). The copies stayed out of this folder,
  because they hold the program's logs and homes.

## 1. The digests

The spike's node pin, `a217980f…`, is **the arm64 manifest, not the
index**. `skopeo inspect --raw` on the box gives it as
`application/vnd.oci.image.manifest.v1+json`, and the index for
`node:24.22.1` is `d862bb5d…`, with amd64, arm64 and ppc64le. The socat pin,
`603174b7…`, is already the index (six platforms).

On the box, `podman pull` of `a217980f…` exited 0 with only a warning
("image platform (linux/arm64/v8) does not match the expected platform
(linux/amd64)"), and running it failed: `Exec format error`. By the index,
`d862bb5d…` ran as `x64 v24.22.1` with Python 3.11.2, and socat's index
ran as `x86_64`.

Through `give check`, the spike's card on the box **passed `pull` and
`start` and failed at `healthy`**: "app stopped, exit 1. Its last lines:"
followed by nothing. The cause, the exec format error, is not shown. This is
the empty-logs bug the tutorial pass found, in another case.

Which call gives the index digest:

- `podman image inspect --format '{{.Digest}}'` gives the digest the image
  was pulled by, on both systems. Pulled by tag, that was the arm64
  manifest on the Mac (`a217980f…` for node, `ed137dc2…` for socat). Pulled
  by the index on the box, it was the index. The spike's
  pin is the value this call gives on the Mac.
- `podman images --digests` showed the index on the Mac for both images.
  `RepoDigests` lists both digests, with nothing to say which is which.
- **`podman manifest inspect <ref>` decides it on both systems**: exit 0 with
  `application/vnd.oci.image.index.v1+json` and its platforms for an index,
  and exit 125 for a single manifest ("Treating single images as manifest
  lists is not implemented"). It asks the registry, so it doesn't depend on
  how the image was pulled. `skopeo inspect --raw docker://…` gives the same
  answer on the box, and the Mac has no skopeo.

The corrected card is `compose.yaml` here. Only the node pin and two
comments differ from the spike's. Its hash is `c7194a81…` on both machines.

Changing the card orphans copies. **Give's `restore` refused the Mac copy
for the spike's card**, because the copy was written under the corrected
one (`OtherCard`, wrote `c7194a81…`, runs `a8431101…`). That is the rule
working as written (`runtime.rs`, `restore`). It means a pin has to be right
before a card's first copy, or a pin fix needs a way to carry copies over.

## 2. Mac → Linux: passed

The folder was copied into a volume as the spike did it
(`COPYFILE_DISABLE=1 tar --no-xattrs --no-mac-metadata … | podman volume
import`). Give's `snapshot` wrote it out: `code.tar` (22,016 bytes, ustar),
`code.root` `0:0:755`, and `card` `c7194a81…`. The three files arrived on the
box with the same sha256.

On the box, Give's `restore` loaded the copy into `svrngs-check-xos-code`
and `differs` found nothing. `give check --name svrngs-check-xos` then passed
every step in 12.5 s: read, asks, machine, pull by digest (both "already
here … linux/amd64"), start, healthy, snapshot (reflink, paused 306 ms), stop,
restore ("the state is the snapshot's"), start again, healthy again, remove.
Nothing was left. The program's code is in its state, so `healthy` means it
ran from the copy. The spike saw it fail from empty.

## 3. One write on Linux, and a fresh copy

The app was started on the box from the Mac copy, through Give's `start`, on
127.0.0.1:17334. `GET /` answered 200 with 1,749 bytes, the same as in the
spike's container. A shell was opened with `/events?as=xos`, and `echo
from-linux > note.txt` was posted to `/cmd?as=xos`. The shell answered as
uid 0 in `/tmp/canon-term/homes/xos`. `note.txt` said `from-linux` in the
volume mounted at `/private/tmp/canon-term`, so the double mount works on
Linux, where `/tmp` is not a link. The app was stopped and Give's
`snapshot` wrote the Linux copy (24,576 bytes, `code.root` `0:0:755`, the
same `card`).

The two copies differ in four entries: `homes/` got a new mtime, and
`homes/xos/` with its `.bashrc` and `note.txt` appeared, owned `0:0`
(`notes.txt`, "Listing diff").

## 4. Linux → Mac: passed

The Linux copy arrived on the Mac with the same sha256. Give's `restore`
loaded it, and `differs` found nothing. Started on the Mac, the program's own
shell ran `cat note.txt` and printed `from-linux`. Loaded into
`svrngs-check-xos-code`, the volume held `homes/xos/note.txt` saying
`from-linux`, and `give check` passed every step in 25.5 s, with images
"already here … linux/arm64" from the same index digest.

## 5. What changed on the way, and whether it matters

**Owners.** The tar keeps numeric owners, and each machine maps them through
its own user namespace:

- The Mac's files went in as uid 502, gid 0 (the Mac user, group wheel).
  They stayed 502:0 inside the container on both machines and after the
  round trip.
- On the box's disk they are uid 524789. That is the box's subuid base,
  524288, plus 501 (`/etc/subuid`, `podman unshare cat
  /proc/self/uid_map`). On the Mac's machine they are 100501.
- What the program wrote ran as the container's root, so `homes/xos` is
  0:0. On the box that is uid 1000 on the disk, and on the Mac's machine it
  is `core`, uid 502.
- The volume's own folder is `0:0:755` everywhere, set by `restore` from
  `code.root`.

The tutorial pass's "files arrive owned by uid 1000" is the same rule: that
tar came from a Linux user whose uid was 1000. The program runs as root and
reads and writes everything, so the mixed owners don't matter to it. They
would matter to a card that runs as a user of its own. The node image's
`node` user is uid 1000, which a Linux giver's files would then match by
accident, and a Mac giver's 502 would not.

**SELinux labels.** Neither tar carries xattrs or labels (`grep -c` for
`SCHILY.xattr` and `security.selinux`: 0 in both). Each import was labeled
`container_file_t:s0` on both machines; the Mac's podman machine runs
SELinux too. There was no denial, and nothing needed `:z`.

**Modes.** Unchanged: 644 for files and 755 for directories, from the Mac's
folder through both copies.

**Timestamps.** The first copy cuts mtimes to whole seconds. `server.mjs`
is `…804.042641485` on the Mac's disk and `15:53:24.000000000` in every
volume after it. From then on they survive the round trip exactly, because
the tar and `differs` both work in seconds. The container shows them in UTC.
The program reads no mtimes. A tool that compares by mtime against the
original folder would see every file as changed.

**Symlinks.** The folder has none. The `/tmp` alias is a mount, not a
link, and it worked on both machines.

## What Give needs from this

1. **A card's image pin is an index digest, and derivation takes it from
   the registry.** Derivation reads it with `podman manifest inspect <ref>`
   (an index, exit 0) and never from `podman image inspect .Digest`, which
   gives whatever the image was pulled by. `give check` refuses a pin that
   `podman manifest inspect` answers with 125, before `pull`, and names the
   index it belongs to. Otherwise an x86_64 keeper passes `pull` and `start`
   and fails at `healthy` with no cause shown.
2. `healthy`'s failure shows why the container stopped: podman's error for
   the container, not only its logs, which are empty for an exec format
   error.
3. A card is right before its first copy. Any edit changes its hash, and
   `restore` then refuses every copy made before (`OtherCard`), so a
   corrected pin needs either a fresh copy or a stated way to carry copies
   across.
4. Owners travel as numbers. Nothing is needed for an app that runs as root
   in its container. For a card that runs as a user of its own, the first
   copy sets owners to that user, so they don't depend on the giver's uid.
   This pass doesn't test that.
5. Labels and modes need nothing. Timestamps are whole seconds from the
   first copy on, and nothing should compare a copy's mtimes with the
   original folder's.

Not tried: an app that runs as a non-root user, a folder with symlinks or
hard links, a copy of the original folder taken while the program was
writing to it (the copies carried between machines here were of a stopped
app; only the check's own snapshots were paused), and a keeper handover through the record.
