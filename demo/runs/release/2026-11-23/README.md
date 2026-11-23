# Keeper release tier — 23 November 2026

The release tier ran on `43cf174314755d910f294d7d560044c537d8d4f4`.
From the toolbox, the loop ran:

```sh
flatpak-spawn --host python3 -B demo/runs/release/run.py /tmp/opencode/keeper-release-43cf1743
```

Exit **0** after **33m25s**, **24 passed**. [summary.txt](summary.txt) is
the run's exact summary, including the archive and build fingerprints.
The full transcripts are in the output directory above; the loop log is
`ralph/next/keeper/ctl/log/sessions/avail-release-tier-6.await.log`.

## What the run showed

- **Packaged Fedora:** busy port, failed save, and withdrawal during review
  all passed. After a no-login reboot the old link returned the original
  data marker with HTTP 200; the wrong-marker plant was rejected. Stop
  sharing cut the member's open stream in 1.010 s, refused a fresh request
  with 404, and left the local marker reachable (`fedora-package.log:5–17`).
- **Linux native and containers:** native Run's service, absent declaration,
  linger and separate Share hint all passed independently; three no-login
  boots and a crash reopened the old address (`linux-native.log:11–32`).
  Plain Podman and Actual each kept the same grant and renewed the saved
  approval to a new boot and cgroup; the old links returned their markers
  (`linux-container-plain.log:12–13`, `linux-container-actual.log:15–20`).
  The recovery and Stop-delay cases also passed; a same-name foreign pod
  was left alone (`linux-container-recover.log:3–7`,
  `linux-container-stop-gap.log:5–12`).
- **Terminal, package and WebKit:** terminal consent changed linger from
  no to yes (`linux-stdio.log:1–2`). The stock install got both siblings;
  Home Stop kept its job down through uninstall, and the separate native
  job still restarted after uninstall (`linux-ship.log:19–58`). WebKit
  ran, stopped, started and shared the program, and Alex opened it; the
  unanswered page PTY left linger off (`linux-page-stdio.log:5–16`).
- **beefymac:** the native keeper and node suites passed. The native Run
  loaded 25/25 layers on Metal; HTTP recovered 9.7 s after SIGKILL, and
  Stop stayed stopped through a restart request. The three lifecycle
  proofs, renewal and six plist cases passed (`beefymac.log:147,1303,
  1351–1397,1500–1737`). Renewal and plist fingerprints are for the
  recorded scratch fixture builds, not the unmodified native binaries.

The loop log records that the Fedora VM overlays, guest test units and
state, local replay folders and build staging were removed.
`beefymac.log:1738–1739` records:

```text
Cleanup: all six test labels absent (exit 113); removed /Users/alexsbryan/svrngs-keeper-scratch
release-case mac-cleanup 0 launchd
```

## Red and green

The tier's driver-exit plant, run in the toolbox:

```text
$ TMPDIR=/tmp/opencode PLANT_RELEASE_DRIVER_FAILURE=1 python3 -B demo/runs/release/test_run.py ReleaseTests.test_a_successful_control_makes_tier_green
Running linux-ship.log: /usr/bin/python3 -c 'raise SystemExit(7)'
failed linux-ship: commit=abc build=archive; exit=7; linux-ship.log
AssertionError: 1 != 0 : failed linux-ship: commit=abc build=archive; exit=7; linux-ship.log
FAILED (failures=1)

$ TMPDIR=/tmp/opencode python3 -B demo/runs/release/test_run.py
Running linux-ship.log: /usr/bin/python3 -c 'raise SystemExit(0)'
passed linux-ship: commit=abc build=archive; exit=0; linux-ship.log
Ran 4 tests in 0.023s
OK
```

The four controls cover the successful and failed driver exits, missing
host and wrong candidate, and an inner verdict despite exit 0. The VM
plants and their focused green runs are recorded in `b22905ab`: the old
Share hint failed its own check while service/share-file/linger passed;
the missing renewal failed with `renewed=Err(Held(Another))` and the
approval unchanged. The full replay above ran with neither plant.

The physical Fedora and Mac reboot proofs and the people sessions are
operator evidence, separate from this tier. This run did not publish a
release; rerun the tier on the final release candidate.
