# Install Run with svrngs from a local release build

`avail-ship`, release spec §5 row 8. Predictions before the run:

- On Linux x86_64 the release builder checks all three executables for PIE
  and sums two archives: `svrngs` with its keeper, and Give separately.
- In a fresh Fedora 45 overlay, the default install line refuses an archive
  whose checksum is correct but which lacks `svrngs-keeper`; neither
  program nor the data directory is installed. The same line against the
  complete release installs the two siblings but not Give, starts the real
  systemd user service and asks the opener for the home page. The opener is
  a headless stub; a browser session is not judged here.
- `svrngs run` from the installed binary starts a test HTTP program that
  answers with the planted page; no share is made. Uninstall reports the
  difference between native jobs and Linux container jobs, removes the
  binaries and the node's login service, and the native program can still
  restart. Container lifecycle after uninstall is not exercised here.
- In a second case, from Home in WebKit, Stop an app before uninstalling
  without having joined a group. The app should stay down through uninstall
  with its setup intact. The first app should still be running at that point.
- The Arch packaging function refuses the incomplete archive for the next
  release and places both executables in `/usr/bin` in the complete case.
  The installed package's signature verification requires the operator's
  eventual release; this proof stages the package locally without signing.

From the repository root, with a fresh scratch directory and the keeper
spike's prepared disk, key and seed already in `/tmp/keeper-demo`. The
Home action uses the prepared `svrngs-gate` container under flock via
`/tmp/keeper-audit/gate.sh` (override with `GATE`), and the existing
`web/tests/home.mjs` Playwright driver:

```sh
toolbox run -c sovereign-vulkan bash -c 'cd ~/dev/svrngs-keeper && scripts/build-release.sh --only linux-x86_64 --out /tmp/opencode/avail-ship-site'
W=/tmp/opencode/avail-ship-vm python3 -B demo/runs/keeper/2026-11-23-ship/run.py /tmp/opencode/avail-ship-site
```

`--skip-stop` plants an omitted Stop before the same checks. The site is
served on loopback inside the guest. The overlay is discarded
after the run. No release, tag, page deployment or AUR update is made.

## Initial run on 2026-11-23 (3957bca8)

The release build and VM replay exited 0 in 1m44s. The Linux x86_64
builder passed its PIE check for all three programs and wrote these sums:

```text
built: /tmp/opencode/avail-ship-site/get
c9d54b29e392c80bd084ef0cc27d55893c86092288567f07960b14e58362a455  svrngs-linux-x86_64.tar.gz
af242283bd8e80b88bb5a6e2bbee025e201b48e900e89fb67a4c162928f2190f  give-linux-x86_64.tar.gz
```

**Planted archive:** `run.py` removed the keeper and recomputed the sum.
Both the staged Arch package step and the Fedora install line rejected it.
Excerpts from `avail-ship-1.await.log` (lines 449–467):

```text
install: cannot stat 'svrngs-keeper': No such file or directory
passed: Arch package step exited 1 with archive members ('svrngs',)
```

```text
Getting svrngs for this computer from http://127.0.0.1:8187/plant/get/svrngs-linux-x86_64.tar.gz
Checked: svrngs-linux-x86_64.tar.gz is the one the release lists, unchanged on its way here.
tar: svrngs-keeper: Not found in archive
tar: Exiting with failure status due to previous errors
svrngs: svrngs-linux-x86_64.tar.gz must hold each of these programs: svrngs svrngs-keeper. Nothing was installed.
sam$ test ! -e ~/.local/bin/svrngs && test ! -e ~/.local/bin/svrngs-keeper && test ! -e ~/.svrngs
passed: a correctly checksummed archive with no keeper installs nothing
```

**Complete archive:** the staged Arch package contained both executables.
On Fedora Linux 45 (Cloud Edition), the install started the user service
and called the opener stub. The installed `svrngs run` served the fixture
page without creating a share (lines 451, 459, 469–484):

```text
passed: Arch package step exited 0 with archive members ('svrngs', 'svrngs-keeper')
```

```text
Installed svrngs in /home/sam/.local/bin.
Installed svrngs-keeper in /home/sam/.local/bin.
svrngs runs now, and whenever you log in (/home/sam/.config/systemd/user/svrngs.service).
```

```text
active
svrngs 1.13.3 (3957bca8)
```

```text
test-ship is running here. To share it: `svrngs share 8188 --program <program> --name test-ship --group <group>`.
sam$ test "$(curl -fsS http://127.0.0.1:8188/)" = "ship witness" && test ! -e ~/.svrngs/lends/test-ship.json && systemctl --user is-active svrngs-keeper-test-ship.service
active
passed: the stock line installed both siblings, not Give; Run answers and made no share
```

**Uninstall:** the node service stopped and both binaries were removed.
The native job answered after a restart (the first curl attempt was refused,
then the retry read the fixture). The VM driver disabled and stopped
`svrngs-keeper-test-ship.service` before discarding the overlay. The Linux
container note was checked as output; container lifecycle was not run here
(lines 486–496):

```text
Stopped svrngs, and took it off what runs when you log in (/home/sam/.config/systemd/user/svrngs.service).
Removed svrngs from /home/sam/.local/bin.
Removed svrngs-keeper from /home/sam/.local/bin.
```

```text
Apps you told svrngs to run are not stopped or removed. Native apps can still start as before; managed containers on Linux need svrngs-keeper to start and stop. To stop running an app and remove its setup, use svrngs-keeper remove <name> before uninstalling, or reinstall first. Your apps' files stay where they were.
```

```text
Removed '/home/sam/.config/systemd/user/default.target.wants/svrngs-keeper-test-ship.service'.
passed: uninstall removed both binaries, and the native job still restarts
```
