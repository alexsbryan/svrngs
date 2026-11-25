# In situ over OpenTelemetry and svrngs, on a case that goes wrong — 2026-11-25

The general shape of the product, run against causes picked to break the
happy path. The service is instrumented with stock OpenTelemetry. The
site's own collector keeps that telemetry on the clinic's machine, and a
site-enforced projection decides what leaves it. Declared side effects go
through a site gateway that records them, so trials replay recorded answers
and never call out. The vendor's agent reads the view, and delivers
candidates, through svrngs lends in both directions. The clinic opens no
port. Built on `../2026-11-25/` (copied unchanged at `b2392b5b`).

**The run's biggest finding was a failure of the site tool itself.**
systemd could not make a network namespace in the clinic's container.
It logged "PrivateNetwork=yes is configured, but … proceeding without" and
ran every trial with the network. Each arm's collector and gateway lost
their ports to production's, so replayed claims went through production's
gateway: **353 duplicate claims reached the clearinghouse** from 7 trials
(`record/first-run/`). The agent caught it from the view, without being
told: new requests answered 202 when the stated behaviour is 599. It
stopped delivering, and its PR said so. The site now proves isolation
before every trial and inside every arm, and refuses otherwise.

## The parts

| Part | What it is |
|---|---|
| `Containerfile` | The clinic's server: Fedora 45, systemd, otelcol-contrib 0.165.0 (sha256 checked; its sigstore signature wasn't), the OTel Python SDK 1.39.1, svrngs. |
| `service/` | Acme's repo at 0.3.0, recipe included. Stock OTel spans and logs, with a developer's habits: the file name on a span, exceptions whose messages quote the input, the payer's error body logged. |
| `make-clinic.py` | 30 synthetic notes and a migrated client database. Three causes, all only at the clinic: a client's name typed into the date line (3), a NULL plan id (2), minutes written "50 min" (3). |
| `clearinghouse.mjs` | A stand-in payer, strict, that echoes the claim in its errors. Its log is the witness of every claim that reached it. |
| `project.mjs` | The projection, pure. Strings pass only for keys the site knows (OTel conventions) or the signed contract types as int, hash or a closed set of words. Everything else becomes a shape (length, character classes, keyed hash). Exception messages are dropped; stack traces keep only code locations. 7 laws in `project.test.mjs`. |
| `gateway.mjs` | Declared effects. In production it records every exchange on the machine. In a trial it replays the answer recorded for the exact request; an unrecorded request gets 599, and the input is unjudged. |
| `site.mjs` | New `telemetry` and `gateway` steps. Trials snapshot state, run each arm with its own collector and gateway, and prove isolation first. A candidate is projected by the *installed* contract, so it can't widen what passes. The watcher pulls candidates from the vendor's lend. |
| `acme.mjs` | The vendor's `view` and `deliver`, as a command and as MCP tools (stdio), over svrngs. |
| `leakcheck.sh` | Reads every route through svrngs; fails on any of the clinic's 82 names, ids or phrases. |

## What happened

| | Result | Record |
|---|---|---|
| Projection laws | 7 pass. Planting exception messages through, or the file name, fails them | `project-test.txt` |
| Production 0.3.0 seen through svrngs | 22 submitted, 5 error, 3 rejected, as planted | `stage.log` |
| The view | 0 of 82 leaked (62–184 KB read); the collector's raw file leaks 25 | `leakcheck.txt` |
| Agent, first pass | All 3 causes found. Why the payer rejects took 4 instrumentation candidates, each passing as "changed no outcome". 3 fix branches with tests that fail first. Then it found the 202s and stopped | `first-run/PR-agent-stopped.md`, `first-run/instrument-failures.patch` |
| The leak | 353 claims at the clearinghouse from trials, 50 per trial | `first-run/clearinghouse-claims.jsonl`, `first-run/systemd-proceeding-without.txt` |
| Plant: a clinic that can't isolate | the trial refused (never-ran), clearinghouse unchanged at 28 | `stage-nocaps-plant.log` |
| My second stage bug | the "release" was built from the agent's checked-out branch; now built from `main` | (`stage.sh`) |
| Re-trial of `fix-claims` on the fixed site | **passed**: error→held 2; rejected→unjudged 3 and error→unjudged 3; 22 unchanged; 0 regressed; clearinghouse unchanged at 25 | `trials.json`, `clearinghouse-claims-second-run.jsonl` |
| Review | **not merged**: see below | `PR.md`, `fix-claims.patch` |

### Why the PR isn't merged

1. **A restart resubmits the whole inbox.** The service keeps which notes it
   has seen only in memory, which the agent noted. Installing any release
   restarts it, so 0.3.1 would send 22 claims a second time. That needs a
   fix, and a trial that can judge it, first.
2. **6 of the 8 changed claims are unjudged.** Only the payer can say if it
   accepts them: production after release, or one claim sent by hand.
3. **The date fix drops the date line's remark** ("telehealth"), which may
   matter to billing. That's for the clinic to say.

## What this does not show

- **Shapes are a channel.** A candidate can encode a few bits per value in
  lengths and classes; the agent read its enum flags by length. The vendor
  is trusted under its BAA as for any release. A hostile candidate is out
  of scope, as a hostile release is.
- **Isolation where systemd can do it.** The clinic here is a container,
  given `SYS_ADMIN` and `NET_ADMIN` for its network namespace.
- **The scoped support link.** The view is lent to a group of two, the
  clinic and Acme (`SUPPORT_LINK_API.md` §2's lab form), not the one-desk
  session that API designs.
- **A real service, payer or team, or a rate.** One run of each.

## Run it

    podman build -t onprem-target:otel -f Containerfile .   # vendor-cache/ holds the collector and svrngs
    node --test project.test.mjs
    ./stage.sh && ./leakcheck.sh && ./leakcheck.sh --plant
    # then the agent, with only out/vendor-repo and acme.mjs
    ./stage.sh down
