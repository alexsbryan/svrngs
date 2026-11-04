# Onboarding scene — staging record, people never-ran

Predictions were committed at `896fcd60`, before this record and before any
people observations. Product source for the first staging builds is
`a8c5eac281880ee57c55c50526277915402cfb2b`; source and per-file SHA-256s are
in `data/artifacts.json`. The scene transport/probe were uncommitted demo
drivers during these first runs; their landing commit makes them repeatable.
No product source changed during preparation. The operator's provisional
people bar is `ralph/next/onboarding/STATE.md:267-274`.

## Recorded preparation

`python3 demo/runs/onboarding/2026-11-04-scene/prepare.py` built the canonical
page and matching downloads locally using the existing build scripts under
`lockf -k /tmp/svrngs-gate.lock`. Complete output and exits are pasted in
`data/build-web.txt` and `data/build-downloads.txt`: both **passed**, exit 0.
The files are in `target/onboarding-scene/site`; no artifacts were published.

The first transport probe failed before reaching its checksum plant:
`data/first-checksum-instrument-error.txt` and
`data/first-green-instrument-error.txt`. The probe itself called
`Request.text()`, whose UTF-8 decoding removed the BOM; it now compares
`arrayBuffer()` bytes. This was an instrument error, not a production-handler
failure. The production handler already decodes BOM-preserving signed bytes
(`resolver/handler.mjs:39-43`). The recorder now also requires the named plant
in the red log, rather than accepting any exit 1 as a checksum control.

After correcting that instrument:

```text
$ python3 demo/runs/onboarding/2026-11-04-scene/prepare.py --probes-only
checksum-red: running (expected exit 1)
checksum-red: exit 1
staging-green: running (expected exit 0)
staging-green: exit 0
Requested preparation steps passed; people scene never-ran. Servers stopped. Operator judgment owed.
```

`data/checksum-red.txt` is the named red: replacing the expected download
hash with all zeroes fails at `download checksum: svrngs-darwin-aarch64.tar.gz`,
exit 1. `data/staging-green.txt` is the green, exit 0. It records HTTPS exact
body/socket IP, built canonical assets and matching download checksums,
static-file confinement, and a bounded live absent-address lookup through
the actual resolver and generation guard. There is no live publication or
admission. Temporary listeners and certificate files were removed on exit;
no trust store was changed. This is a local transport proof, not a judgment
of the people welcome or cross-laptop reachability.

## Owed before people

- This source's full `scripts/check.sh`: **never-ran** at this checkpoint.
  The next preparation run must record exit 0 before the scene.
- Attendee-trusted HTTPS and cross-laptop reachability: **never-ran**.
  `STAGING.md` explains the certificate input and LAN/tailnet setup. The Mac
  has Tailscale running but no advertised certificate domains; the worker
  changed no tailnet settings or device trust.
- Host plus attendee laptops, fresh installation, interrupted join, folder
  contributions, closure through short and copied ordinary links, and continued
  file exchange: **never-ran**. No participants are present in this shell session.
- Operator's human-welcome judgment: **could-not-judge**, observations absent.

No times, assistance counts or success rates are inferred. Follow
`demo/onboarding-room.md` when the operator and laptops are present. The
transport process runs in the foreground only and must be started again for
that scene. No public deployment, publish or push occurred. The protected
V1.6 publish artifacts remain unmodified and untracked.
