# In situ: diagnose, fix, trial and PR from the customer's own server — 2026-11-25

The loop end to end, on one run. A service fails at a clinic and nowhere
else. The vendor's agent diagnoses it from telemetry that never shows
patient data, fixes the code, and has the fix trialed on the clinic's real
inputs beside production. It then writes a PR whose evidence is "works in
the production repro environment". Merged, released and applied,
production goes from 17/20 to 20/20. Built on the on-prem tool of
`../../onprem/2026-11-25/` (copied unchanged at `079fa609`, then extended).

## The parts

| Part | What it is |
|---|---|
| `service/` | The vendor's repo at 0.2.0, a stand-in notes-to-forms parser. Its telemetry gives each input's **shape** (bytes, BOM, newline, high bytes, first invalid byte) and each failure's **class**. Inputs are named by a hash of the file name. Never a word, name or id. |
| `make-inbox.py` | The clinic's EHR export: 20 synthetic notes, 3 of them Windows-1252 from an old workstation. File names carry (fake) client names. |
| `site.mjs` | Adds the **view** (GET only: `/site`, `/service/*`, `/trials`), **trials**, a **watcher** for delivered candidates, and **channels**: a release is applied by the admin's token; a candidate is only trialed, never installed. |
| trial | The candidate and the installed release each replay the real inputs once, as the service's account under `systemd-run`: no network, everything read-only but a throwaway output, nothing kept but the comparison. **passed** = fixes at least one and regresses none. |
| `view.sh`, `ci.sh` | The agent's only tools outside the repo: read the view; test a branch, sign it as a candidate, deliver it. |
| `leakcheck.sh` | Reads every route of the view, fails on any client name, id or note phrase in the inbox. Plant: `--plant` points it at the notes themselves (24 leaks). |

## The run

| | Result | Record |
|---|---|---|
| Production 0.2.0 at the clinic | 17 parsed, 3 `UnicodeDecodeError`, first invalid byte 0x97, CRLF | `stage.log`, `PR.md` |
| The view, every route | no name, id or note phrase (6–9 KB read, three times) | `leakcheck.txt` |
| Agent: diagnose → fix → test → candidate → trial → PR | **1 min 40 s**; 6 view reads, 1 CI call, the trial passed first try | `times.txt`, `PR.md` |
| Its test | fails before the fix (2 errors), passes after (4 OK) | `PR.md` |
| Trial of 0.2.1-rc1 | release 17/20, candidate 20/20; fixed the 3, regressed 0 | `trials.json` |
| Plant: a candidate that drops fields | **failed**: 17 regressed | `trials.json` |
| Plant: apply a candidate | refused: "can only be trialed here, never installed" | (below) |
| Merge, release 0.2.1, admin's plan and apply | 2 steps; production **20/20** | `service-state-after.json`, `site-after.json` |

The agent ran in a fresh context with only the repo, `view.sh` and `ci.sh`.
It was told not to read the run folder. Its account of its commands is in
the session, not checked against a log, and `ci.sh` doesn't record its
callers. Its PR names three things it saw and left alone: the live event's
`at` overwrites the failure location; `VERSION` wasn't bumped (CI stamps
it); `__pycache__` is tracked. The last is `stage.sh`'s fault: it copies
`service/` with its cache.

## What this does not show

- **The channel.** The view is reached through podman here. In the field it
  is a svrngs lend, as in `../../support-link/2026-11-25/`.
- **Correct values.** "parsed" means every field was found, not that each
  value is right. Nobody may see the notes to check, which the PR says.
- **A real service, hardware or medical team.** The service is a
  stand-in, the server a container. One run, so not a rate.
- **Delivery to an air-gapped site.** Here candidates arrive by
  `podman cp`. In the field: the lend, or a USB stick.

## Run it

    podman build -t onprem-target:f45 -f Containerfile .
    ./stage.sh && ./leakcheck.sh
    # then the agent, with only out/vendor-repo, view.sh and ci.sh
