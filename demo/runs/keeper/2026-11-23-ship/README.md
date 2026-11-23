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
- The Arch packaging function refuses the incomplete archive for the next
  release and places both executables in `/usr/bin` in the complete case.
  The installed package's signature verification requires the operator's
  eventual release; this proof stages the package locally without signing.

From the repository root, with a fresh scratch directory and the keeper
spike's prepared disk, key and seed already in `/tmp/keeper-demo`:

```sh
toolbox run -c sovereign-vulkan bash -c 'cd ~/dev/svrngs-keeper && scripts/build-release.sh --only linux-x86_64 --out /tmp/opencode/avail-ship-site'
W=/tmp/opencode/avail-ship-vm python3 -B demo/runs/keeper/2026-11-23-ship/run.py /tmp/opencode/avail-ship-site
```

The site is served on loopback inside the guest. The overlay is discarded
after the run. No release, tag, page deployment or AUR update is made.
