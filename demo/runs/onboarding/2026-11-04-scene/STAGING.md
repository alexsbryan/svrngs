# Local scene staging

This is the local-only setup authorized by `STATE.md:256-274`. Predictions
remain unchanged at `896fcd60`. The provisional people bar is that row's;
the operator may revise it before participants start. No attendee observations
have been collected. The operator and the laptops are not present in this
worker session, so the human welcome remains **never-ran**.

## Prepare and verify

From the repository root, run in the foreground:

```sh
python3 demo/runs/onboarding/2026-11-04-scene/prepare.py
```

For the loop's long-run handoff, use a 90-minute `ralph-result await` budget.
The recorder builds the page with `scripts/build-web.sh`, then all Mac/Linux
downloads with `scripts/build-release.sh`, into `target/onboarding-scene/site`.
It uses one recorded HEAD for both, records artifact SHA-256s, tests a planted
wrong checksum before the clean probe, and runs the unchanged locked full gate.
Every command, actual exit and complete output goes in `data/<source-prefix>/`.
The initial preparation/probe evidence is retained at `data/` itself. It stops on
an unexpected exit. No people scene may start without the full gate's exit 0.
Any source changes after preparation require a new build and gate.

`verify.mjs` owns and stops temporary local HTTPS listeners. Its temporary
certificate is trusted only by that probe; no OS/browser trust is changed.
It checks exact request bytes, socket-derived ingress IP despite a forged
forwarding header, static containment, the built shell/installer/wasm,
and every downloaded archive against the served checksums. It performs one
absent-phrase lookup through the real resolver and live generation guard;
it does not publish a capability or admit anyone. Provider unavailability
fails this live probe rather than being counted as ended guidance.
The provider credential is read inside the script from Keychain, never an
argument, output, committed file or static asset.

## Serve when the operator and laptops are ready

Keep the process in the foreground, using a certificate and private key for
the Mac's reachable name, **trusted by the attendee browsers and curl**:

```sh
node demo/runs/onboarding/2026-11-04-scene/stage.mjs \
  target/onboarding-scene/site /private/path/key.pem /private/path/cert.pem \
  BIND_IP 8443 8444
```

The canonical distribution origin is `https://NAME:8443/`; the short-address
resolver is `https://NAME:8444/`. Both are HTTPS, with different ports/origins.
The resolver reads the live Free credential only in the server process. The
adapter uses the TCP peer address, never an attendee-provided forwarding
header. It logs no request URL, request body, invitation or provider body.
Ctrl-C stops both listeners. There is no service installation or background
process; a worker's listeners do not survive its turn.

Use a dedicated scene host data directory, retaining any existing personal
data. Put the resolver origin in that host's `join-resolver` file. The group's
saved `origin` must be the canonical page above, so publication saves that
page and the installer downloads this build. The existing ordinary origin
validation asks for wildcard page hosting (`cli.rs:checked_page`); a single
LAN/tailnet name is not a wildcard host. For this restricted native-only scene,
set the scene group's existing `origin` file to the canonical page explicitly
after creating the group, before making any invitation. This staging-only
configuration is not a product change or a browser-lend parity claim. Keep
the dedicated data directory private and never commit its invitations.

Before attendees start, verify from **each laptop** that the canonical page,
installer and downloads load without certificate exceptions, and the resolver
is reachable. Follow `demo/onboarding-room.md` for the actual people scene,
including reload/restart recovery, interrupted join, both closure probes and
file exchange after closure. Capture the real paths, membership and assistance.

## HTTPS readiness still owed

The local transport proof cannot establish certificate trust on absent laptops.
This Mac's `tailscale status --json` reported `BackendState: Running` and
`CertDomains: null` during preparation; no tailnet HTTPS setting was changed.
The operator must supply a usable LAN certificate/trust arrangement or enable
tailnet HTTPS and connect the attendee laptops. Do not use plain LAN HTTP:
`resolver/handler.mjs:74` requires an HTTPS canonical page, and the browser
needs a secure context for WebCrypto. Do not ask people to bypass warnings.

No public deployment, Vercel/Upstash provisioning, publish or push is part of
this procedure. The V1.6 publish artifacts remain untouched.
