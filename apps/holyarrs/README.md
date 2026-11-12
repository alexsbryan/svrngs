# HolyArrs

Ask for a film, have its lender approve it, then watch the imported copy in
their Jellyfin. HolyArrs runs on the lender's computer; svrngs gives the chosen
group reach to its own web app. Movie Night finds the resulting library copy.

## Lend an existing installation

1. Complete HolyArrs's owner setup on the computer running it. Configure its
   movie folder, source and downloader in HolyArrs. Give the downloader and
   HolyArrs the same download-folder path. Jellyfin must see the movie folder
   HolyArrs imports into.
2. Create account invitations from the **group's shared HTTPS page**: use
   **Add your phone or another computer** on your home page to open that page
   as yourself, then open HolyArrs there. In **Settings > Users and roles**,
   make a Member invitation for each friend. Send those invitations with the
   group's invitation; each friend chooses their own name and password.
   HolyArrs copies the browser address it is open at: an invitation made at
   your `.localhost` address names your own computer and cannot be sent to a
   friend's device. Make the group invitation after adding your own browser,
   because a new device link ends that issuer's earlier invitation.
   Leave approval with the owner; membership in Friends does not confer approval
   rights in HolyArrs.
3. Lend its HTTP port to the group:

   ```sh
   svrngs share 6262 --program holyarrs --group Friends
   ```

   Port 6262 is HolyArrs's default. If the container publishes a different
   host port, share that port. Its account instructions ship as
   `crates/svrngs-node/src/programs/holyarrs.txt`.
4. Lend Jellyfin to Friends too, following [its instructions](../jellyfin/README.md).
   Make each friend a viewing account, and a separate restricted readers'
   account for Movie Night. Bring **Movie Night** from the group's shelf.
5. Link Jellyfin in HolyArrs's media-server settings, with **On import** enabled,
   so an import triggers its library refresh. If their movie-folder paths differ,
   configure the path mapping. Under Podman on macOS the container reaches a
   host service through `host.containers.internal`; its own `localhost` is inside
   the container.

Friends open HolyArrs from their svrngs page and sign into their own accounts.
They request a title there; its owner approves there. Once imported and indexed,
Movie Night's search offers **Watch from Ana's library**, opening that copy in
Jellyfin, where the viewer signs in as themselves.

Keep HolyArrs's database on local storage: on macOS with Podman, `/config` is a
named volume. The scene's first trial showed HolyArrs refusing a host-shared
virtiofs `/config`. The movie and download folders can be host-shared folders.

The tested pairing uses HolyArrs **1.0.10** and its own accounts. Its OIDC
client's RS256/same-origin requirements differ from svrngs's ES256/split-channel
provider; the scene does not configure single sign-on. Requests stay in
HolyArrs's database on its host. It is an ordinary lend, not a group-kept app.

## Run the complete scene

The run-of-show and acceptance bars are [`demo/holyarrs.md`](../../demo/holyarrs.md).
The scene drives Chromium on computers and WebKit as an iPhone through the
hosted page's service worker. Bo's phone is added with his own-device link.
The phone uses one group tab at a time, returning to the group's page between
apps; Ana approves from her computer.

Prerequisites: this repository's Rust/Node/browser toolchain, the scratch
Jellyfin installation that `apps/jellyfin/check/check.sh` names, `ffmpeg` on
`PATH`, and a running Podman. Pull these versioned images once:

```sh
podman pull docker.io/holyarrs/holyarrs:1.0.10
podman pull lscr.io/linuxserver/qbittorrent:5.1.4
podman pull docker.io/library/node:24.22.1-slim
cargo test -p svrngs --test scenes holyarrs -- --test-threads=1
```

The result records the resolved image digests. The two actual qBittorrents
transfer a generated 75-second audiovisual test pattern over a private local
torrent. HolyArrs selects it and imports it; the scene compares the imported
file's SHA-256 to the seed, then independently checks Jellyfin's playing item
and Bo's account. Metadata, release search and the tracker are local fixtures;
no public indexer, public swarm or hosted metadata service is tested.

The scene's fresh homes, raw received bodies, credentials and service logs stay
under `target/scenes/holyarrs-{chromium,webkit}/`. Its pod and native database
volume are removed on completion, including a failed assertion. If the entire
test process is forcibly killed, stop just that run's pod with:

```sh
node apps/holyarrs/check/services.mjs down target/scenes/holyarrs-chromium/services
node apps/holyarrs/check/services.mjs down target/scenes/holyarrs-webkit/services
```

For shareable screenshots and results, set `HOLYARRS_RECORD` to a new run
directory; each engine writes a subdirectory there. Raw credentials and response
bodies still stay in `target/`. A missing prerequisite writes `never-ran.txt`,
which the full gate refuses to count as a pass.

## Validate the checks

```sh
python3 apps/holyarrs/check/plants.py
```

It runs five broken computer scenes, each of which must exit nonzero at its
named bar: no approval (H3), unavailable seed (H3), no readers' account (H4),
an already-ended live stream (H6), and a known owner credential planted in the
received-evidence directory (H7).
Logs and the plant verdicts stay in `target/holyarrs-plants/`. An unrelated
setup failure fails the plant runner too.

The fixtures adapt HolyArrs's own test protocol responses, with source paths
and commit in `check/fixture.mjs`. The host-side runner, `check/services.mjs`,
owns the pod; `check/scene.mjs` owns the browser actions; the Rust scene owns
the participants and checks the owner's credentials never reached Bo.

This is automated integration evidence on one host. Human newcomers, a physical
iPhone and two-home network performance need their own recorded run.
