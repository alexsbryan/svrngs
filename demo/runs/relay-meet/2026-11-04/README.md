# Relay-meet demo candidate — focused verification

The operator selected this worktree for an urgent demo. It starts from the
completed onboarding/welcome/portal/OIDC-guidance/grouped-home baseline at
`e7ef3c45`, excluding the unfinished scoped-lending WIP on main. The released
relay-meet patch adds keeper-only `link --relay` using existing `MeetsAt`.
The run-of-show is `demo/mesh-house.md`.

Observed verification on 2026-11-05 UTC:

- `cargo test --locked -p svrngs --test meet --test flags --test new
  --test join_by_link -- --nocapture`: 3 meet, 2 flags, 15 new and 14 join
  tests passed (34 total), including keeper refusal and relay propagation.
- Named plant: replace `link_to`'s `Host::meet` call with `let _ = relays`.
  The keeper test failed at `meet.rs:101`, "the new link carries the new
  relay" (exit 101). Restore the call: all three meet tests passed, exit 0.
- `cargo check --locked --workspace --all-targets`: exit 0.
- `cargo fmt --all --check` and source whitespace check: exit 0.
- Lean: core 3,486/3,494, non-JS 19,832/19,833, JS 8,187/8,188,
  Swift 0/1; exit 0. No cap raise.

This is focused demo-candidate verification, not a newly passing full
check.sh or a recorded physical modem-cut scene. `build.py` builds only
the Mac ARM64 and Fedora x86_64 native downloads plus the canonical page
from one recorded committed source, with independent cloned build caches.
It records build exits and checksums; that initial build was locally staged.
The installer retains its checksum verification. Per-ring lending, native
home development preview and Jellyfin SSO are outside this demo cut.

## Built and locally staged

Source: `f8a57035206be60212d283493ffd3a944976fbdd`. `python3
demo/runs/relay-meet/2026-11-04/build.py` completed the web, Mac ARM64
release and Linux x86_64 musl release builds, each exit 0. The Linux file
is a statically linked x86-64 ELF; the Mac file is an ARM64 Mach-O.

Scene-only address (requires this Mac and tailnet reach):
`https://<this-computer>.<tailnet>.ts.net:8443/`.
`source.json` there records the built source and file hashes. Downloads:

| Archive | SHA-256 |
| --- | --- |
| `svrngs-darwin-aarch64.tar.gz` | `45af9ce434f9ee117f9169d28a5ee9cf89b0af0aaecf51f6be5f2568f58fb3b8` |
| `svrngs-linux-x86_64.tar.gz` | `d08ff00c466f568f28d7449ea4efec661381878d7dd8c92ef55e0cabbe847dfb` |

Observed local smoke (`python3 target/relay-demo/smoke.py`, exit 0):

- Downloaded both archives over trusted HTTPS (no certificate bypass);
  each hash matches served `SHA256SUMS` and `source.json`. Each archive
  contains the executable `svrngs` alone, byte-identical to its built binary.
- Served index, installer, JavaScript and Wasm match the manifest.
- Replaced only the owned staging process and dedicated scene host.
  The host retained `Welcome room Nov4`, its group id and keeper status.
- Fedora's existing `alex_file.txt` remains at SHA-256
  `38b734f1fb7c8bf5b582eda83191628245858193f472ca71ba6174c2544941bf`.
- Home HTML returns 200 with the home origin's Host and the program's
  token; record API without the token returns 401. Running the candidate
  against the already-running scene data prints its home-page reminder.
  This is HTTP/CLI smoke, not a browser interaction verdict.

The initial probe incorrectly asked the API host for `/` (404), then the
home host without authentication (401). The corrected probe uses the
documented home host, token and navigation headers; no product change.
Private runtime scripts/logs and `smoke.json` remain under ignored
`target/relay-demo/`; no credentials are in this record.

Fedora execution of this candidate, a fresh full `scripts/check.sh`, the
host-plus-two-attendee people scene and a physical modem cut are **never-ran**
for this build. Lends still reach all groups the lender belongs to, including
groups joined later; this candidate has no per-ring audience selection.

To install/update on either supported demo machine, preserving its data:

```sh
curl -fsSL https://<this-computer>.<tailnet>.ts.net:8443/install.sh | sh -s -- 'https://<this-computer>.<tailnet>.ts.net:8443/'
```

The installer checks the archive before updating, restarts the installed
user service and opens its home page. No invitation is needed to update an
existing member. Mesh relay setup and the keeper's two-phase change are in
`demo/mesh-house.md`; the private mesh relay address must be supplied on site.

## Real public distribution and Upstash lifecycle

The operator rejected tailnet staging and asked for the real build, in
parallel. `scripts/build-release.sh --out target/relay-demo/public-site`
completed all four svrngs and Give targets (eight archives), from the same
`f8a57035` native/web source. The existing distribution probe's planted bad
checksum failed (exit 1 at the named archive), then the unmodified probe
passed (exit 0). `source.json` includes the public artifacts' hashes; the
earlier two-platform tar hashes above belong only to the initial local bundle.

Public distribution: `https://group.svrnme.sh/`, Vercel deployment
`dpl_GCPg1bjCvE788rDYHdCaGNLBPXuL`. `python3
target/relay-demo/public-smoke.py` passed all eight archive checksums and the
index, installer, JS and Wasm hashes over ordinary trusted public HTTPS.
Vercel's dry upload inventory excluded `.env.local` and `.vercel`; neither
deployment credentials nor the resolver token entered the static distribution.

Public resolver: `https://join.svrnme.sh/`, project `svrngs-join`, deployment
`dpl_UGWEn7XBKc8dgYkXHHpYTfaANAut`. Its Node adapter and configuration are
uncommitted changes under `resolver/`. The first deployment failed because
Node 20 was retired; Node 24 with runtime autodetection built successfully.
The existing Keychain credential was passed directly to Vercel's stdin as a
production Secret. No provider configuration, generation or existing reservation
was reset. Six adapter tests pass; the worker's two header/refusal plants
failed before restoration. The bounded isolated live-provider proof also passed.

`python3 target/relay-demo/native-public.py` passed this real lifecycle:

1. A native host signs publication through its authenticated home boundary.
2. The public resolver writes actual Upstash data; public JSON lookup returns
   the invitation, and HTML navigation returns a 303 to the canonical page's
   native-install fragment. Repeating publication preserves its id/address.
3. A second native process joins that actual invitation and receives the
   host's `WELCOME.txt` through the group's shared folder.
4. Native closure makes public JSON and HTML report ended. A third process
   is refused when it tries the copied ordinary invitation.
5. The existing attendee remains in the group and its new file syncs back
   after closure. Both proof nodes stop cleanly; the closed reservation remains.

This is **two processes on one Mac**, not the Mac–Fedora or unassisted browser/
installer people scene. Those, a physical modem cut and a newly passing full
gate remain owed. The public lifecycle is no longer inferred from isolated
adapter operations or a tailnet transport.

The original dedicated `Welcome room Nov4` scene host now uses the public
canonical page and resolver; its old publication was closed before replacement,
and the group, keeper and Fedora contribution hash were retained. Its new
invitation stays in private runtime state and the operator handoff, not git.
The tailnet staging process was stopped.

Repository check status: `git diff --check` passes. Lean reports core
3,486/3,494 and non-JS 19,832/19,833, but **JS 8,195/8,188 fails**. The
adapter adds eight counted lines (seven beyond current headroom), after its
own pass reduced eleven to eight. No cap was raised in this run; the operator then approved +50 js, of which the merge took 8 (`scripts/lean_check.py`). Budget approval or
further cuts are still required before a clean full-gate landing.

Public install/update line:

```sh
curl -fsSL https://group.svrnme.sh/install.sh | sh -s -- 'https://group.svrnme.sh/'
```

For a host, the currently explicit one-time configuration is its data dir's
`join-resolver` containing `https://join.svrnme.sh/`; there is no automatic
default endpoint in this build. The dedicated scene host is already configured.

## Operator browser failure and corrected scene sequencing

The operator's actual browser run reached the relay handshake on
`aps1-1.relay.n0.iroh.link`, then timed out dialing the issuer and showed
"The computer that sent this link did not answer." This is a **failed**
public browser arrival, despite the earlier native two-process proof.

The scene driver had treated API readiness as completion of the CLI `link`
command. They are different: the API was already serving while that command
waited for the endpoint to become online and minted its link. The driver
published meanwhile (02:04:46 UTC), carrying the first fallback relay, Asia.
The CLI then minted its own link (02:04:48 UTC), carrying US West, superseding
the driver's publication. Inspection confirmed the stored public token still
carried Asia and local publication status was `ended`. A debug restart of the
same identity confirmed its home was US West. The installed node has a different
identity, ruling out a collision between the installed and dedicated hosts.

The runtime driver now waits for the CLI's completed output before publishing.
For this correction the existing host was already running the bare command,
with no later mint pending. The old address was closed, a new invitation was
published at the observed online relay, and local status remained `open` after
five seconds. No product transport behavior or identity was changed to obtain
that result; existing membership and files remain.

`public-browser.mjs` uses ordinary Playwright Chromium against the actual
public short URL, without the tests' local HTTPS proxy. Its initial observation
reproduced the did-not-answer UI, no name form and no established issuer dial.
After correcting the scene, the recorded command passed:

```sh
node demo/runs/relay-meet/2026-11-04/public-browser.mjs install
```

Observed: public short-address navigation, issuer ask, native name form,
typing a name and receiving the canonical installation command (no tailnet
origin). Private JSON traces, installation capability and scene state remain
under ignored `target/relay-demo/`; no invitation is included in this record.
This still does not stand for executing that installer on Fedora or completing
the focused native welcome with a person.

The operator explicitly approved **+50 JS lines** after the failure. The cap
is now **8,238** (was 8,188), with its reason and the adapter's economizing
pass recorded beside `BUDGET` in `scripts/lean_check.py`. Lean passes at
**8,195/8,238**; other rungs are unchanged. The earlier cap failure above is
historical. The integrated gate was started after that raise; its result is below.

## Operator confirms public Fedora arrival

After retrying the corrected public address, the operator reported
"AlexLinux should've joined." The host's standing list still showed the
existing Fedora identity as **Alex B**, not a separate AlexLinux. The debug
trace at 02:16:58 UTC shows that existing key asking the issuer through the
redeem protocol; `Host::join` (`join.rs:107-109`) reuses an already-held group
without a second redemption or rename.

Asked, "Did the installer finish and open 'Welcome to Welcome room Nov4'
with 'Open this folder'?", the operator answered **"Yes"**. This confirms
the actual public Mac–Fedora install/update-to-focused-welcome path using an
existing identity. It is one observed run, not a success rate or a fresh-install
result. A new contribution through the folder opener, the full three-laptop
people scene and the physical modem cut are still unobserved in this retry.

## Integrated gate and on-site relay startup

The single integrated run completed: `lockf -k /tmp/svrngs-gate.lock
scripts/check.sh`, **exit 0**, 2,471 seconds. Its captured log is
`target/relay-demo/public-gate.log`, with exit marker
`target/relay-demo/public-gate.exit.json`. Principles, lean, trust floor,
advisories, nouns, lint/format, page laws, Rust checks/tests, canaries, browser,
types, joining, pages, scenes, conformance and app laws all returned passed.
The log retains individual rate samples, including one straight-to-canary
WebKit seek missing five seconds; all four through-lend samples passed.
This gate covers the candidate product, resolver adapter and approved JS cap;
it is not a physical offline-scene verdict.

At the operator's request, the Mac now also runs its own open mesh relay at
its observed Wi-Fi address, `http://192.168.213.110:3340/`, using the candidate
binary in a separate process. Local `/ping` returned HTTP 200 and `lsof`
confirmed the TCP listener on that address. A `caffeinate -i -w <relay-pid>`
process prevents idle sleep while the relay runs. Private runtime PIDs/logs
are in `target/relay-demo/mesh-relay.json` and `mesh-relay.log`.
Reachability from a second mesh laptop and the group's cutover to this relay
remain **never-ran**. The existing keeper continues running separately.

### Linux container reaches the Mac relay

At the operator's request, tested from an existing Linux ARM64 Podman image
(`node:24.22.1-slim`), using its default container/VM network rather than host
networking. No image pull was needed.

- Container HTTP GET to `http://192.168.213.110:3340/ping`: **200**.
- Read-only mounts supplied only the built ARM64 Linux svrngs executable and
  an ephemeral proof driver. Its identity, group and home lived in the
  container's `/tmp`, not the keeper's or the operator's data.
- `svrngs new ... --relay http://192.168.213.110:3340/` established its
  endpoint's home at that exact relay and completed startup/link minting.
  The authenticated local API confirmed its isolated proof group.
- Independently, the Mac relay's debug log recorded admission of the same
  endpoint (`1468d05174fe…`). The native endpoint stopped and the `--rm`
  container was removed; the Mac relay and keep-awake process remain running.

The driver is `target/relay-demo/container-relay.mjs`. This proves an actual
iroh relay handshake across the container/VM-to-Mac path, beyond HTTP liveness.
It does not prove forwarding a group payload through the relay, reachability
from another physical mesh node, or the modem-cut scene. The live demo group
was not used or changed by this probe.
