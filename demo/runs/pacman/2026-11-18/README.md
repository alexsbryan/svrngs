# svrngs as a pacman package on Omarchy, 2026-11-18

`packaging/arch` at 4bfc7adb, built from the signed GitHub release v1.11.1,
installed, run, updated, removed and refused, on two Omarchy 4.5.0 guests
in the Fedora box's lab. `run.sh 4bfc7adb` drove it from the Mac, its
steps from 20:59 to 21:00 UTC. `steps.sh` is what ran in each guest, as its person `cy`: each
`<n>-<step>.log` prints every command before its output and exit status.

The guests:

- **pac** (logs 00 to 09): a new overlay on the lab's Omarchy 4.5.0
  `base.qcow2`, with nothing of svrngs on it before (01-before).
- **a** (logs 10 to 14): the lab's VM a, with svrngs from the install line
  of 6 November running at login, an older svrngs with no `version`
  command, and one group, The House (12-before-move).

Both were booted with `-snapshot`, so neither disk changed:
`base.qcow2` is still dated 5 November and `a.qcow2` 6 November.

What the lab needed, and the package didn't (00-lab, 10-route, 11-lab):
sudo without a password for that boot, since no one types at the guest;
hypridle stopped; `pacman -Sy`, since the base image has no sync databases
(only 8 Omarchy-repository apps were then out of date, none of them
Podman's); and VM a's default route put back on its own network card,
where the lab's `net.sh` had sent it through VM c, which was off.
`--noconfirm` is the one word added to the README's commands.

## Passed

1. **Key, signatures and sums.** `gpg --import svrngs-release-key.asc`,
   fingerprint `46B4 D67C 5E66 2109 3BB1 AB2B 057B F908 2AA0 3CFB`.
   `makepkg -si` passed every sha256 (the 4 local files and the 4 x86_64
   downloads) and both signatures, built both packages, and installed them
   with Podman (02-build). On VM a, `makepkg` passed the same (14-alone).
2. **`svrngs version`** says `svrngs 1.11.1 (24b6ee7c)` (03-version, and
   14-alone on VM a).
3. **`svrngs update`** says the package's line and exits 0, before svrngs
   ever ran (04-update) and while it runs (05-service): `/usr/bin/svrngs`
   has the same sha256, `pacman -Qkk` says 0 altered files, and `~/.svrngs`
   has the same entries before and after, with no `release`, `update.lock`
   or `update/`. Before svrngs ran it made no `~/.svrngs` at all.
4. **Services.** `systemctl --user enable --now svrngs` runs the packaged
   unit (`Loaded: .../usr/lib/systemd/user/svrngs.service`), and
   `svrngs open --print` answers with a home page code. `give.service` runs
   `give keep`, which says it is keeping. `svrngs restart` restarts it.
   `systemd-analyze verify --user` and `desktop-file-validate` say nothing
   (05-service).
5. **What pacman owns** (`pacman -Ql`, 05-service): `svrngs-bin` owns
   `/usr/bin/svrngs`, `/usr/lib/svrngs/updated-by`,
   `/usr/lib/systemd/user/svrngs.service` and
   `/usr/share/applications/svrngs.desktop`; `svrngs-give-bin` owns
   `/usr/bin/give` and `/usr/lib/systemd/user/give.service`.
6. **`pacman -R svrngs-give-bin svrngs-bin`** removes every one of those
   files, and the services stop. `~/.svrngs` stays, and so does Podman, as
   an orphan for `pacman -Rs` (07-remove).
7. **From the install line** (VM a): README.md's four lines move it aside
   (13-move). `makepkg`, then
   `pacman -U "$(makepkg --packagelist | grep /svrngs-bin-)"`, installs
   svrngs alone, with no Podman. The packaged svrngs runs over the same
   `~/.svrngs` at the same home page address, port 41633, and `svrngs
   update` says the package's line. `pacman -R svrngs-bin` removes it and
   leaves `~/.svrngs` and `~/svrngs` (14-alone). Its group, The House, was
   made by the older svrngs, and V1.11.1 says so and leaves it unchanged.

## The plants, refused

Each is a copy of the package's own files, with one value of the PKGBUILD
changed by `sed`, built with `makepkg -f`:

- **A wrong key** (08-plant-key): `validpgpkeys` ending `2AA03CFA`, not
  `2AA03CFB`. Both downloads `FAILED (invalid public key
  46B4D67C5E6621093BB1AB2B057BF9082AA03CFB)`, then "One or more PGP
  signatures could not be verified!", exit 1, and no package.
- **A wrong sum** (09-plant-sum): svrngs's x86_64 sum starting `883a`, not
  `983a`. `svrngs-1.11.1-linux-x86_64.tar.gz ... FAILED`, then "One or more
  files did not pass the validity check!", exit 1, and no package.

## namcap

`namcap PKGBUILD *.pkg.tar.zst` (06-namcap) warns, and nothing it says is
an error:

- "Reference to x86_64 should be changed to $CARCH": the release names its
  files by architecture, so `source_x86_64` names them.
- "Missing Maintainer tag": whoever publishes it to the AUR adds theirs.
- "lacks FULL RELRO" and "lacks PIE": the release's binaries, which this
  package keeps as released (`scripts/build-release.sh` builds them).
- "Dependency included, but may not be needed": Give runs `svrngs` and
  `podman` as programs, which namcap does not see.

In an unrecorded try before Podman was installed, namcap also said
"Split PKGBUILD needs additional makedepends ['podman', 'svrngs']". Not
taken: svrngs-bin is built from the same PKGBUILD and needs neither, and
`pacman -U` brings Give's dependencies (02-build).

## Not judged

- **aarch64 never ran.** Its two tarballs' sums and signatures were checked
  on the Mac against the release (`shasum -a 256`, `gpg --verify`), not
  built or installed on an ARM computer.
- **The home page over HTTP**: 05-service's `curl` asked
  `http://127.0.0.1:<port>/`, the address apps reach svrngs at, which
  answered 404; the home page is at `svrngs.localhost`. `svrngs open
  --print` getting a code from the running svrngs is this run's evidence
  that it answers.
- **Give keeping an app**: no group and no app, so no app ran in Podman,
  and Give stopping before svrngs at logout was not watched.
- **Opening svrngs from the applications menu**: the entry validates, and
  was not clicked.
- **An upgrade** (`post_upgrade`, a newer package over this one): there is
  no newer release.
- **The AUR**: not published, which is the operator's. `makepkg
  --printsrcinfo` (02-build) is what its `.SRCINFO` would be.
- **The operator's Omarchy laptop**: not run.
