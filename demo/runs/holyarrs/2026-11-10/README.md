# HolyArrs: request to playback

Run-of-show written first: [`demo/holyarrs.md`](../../../holyarrs.md).
Automated integration on one macOS host; Ana and Bo are programs, with
Chromium for computers and WebKit as an iPhone for the hosted-page path.
No human success rate is claimed.

## Instrument

Real stock HolyArrs 1.0.10, Jellyfin (version read from its system API), two
real qBittorrent 5.1.4 peers, this checkout's svrngs, and browser automation.
Metadata, Torznab release search and the private tracker are explicit local
fixtures. The release is a generated 75-second H.264/AAC test pattern;
19,549,723 bytes in the preparation runs. Only the seeder and the fixture can
see the seed folder. The downloader sees a separate initially empty directory.

The runner observes the downloader's `downloaded_session` counter, the completed
request, the imported file's SHA-256, Movie Night's search and the matching
Jellyfin playing-item id. A global lifetime counter had not yet updated at the
first import observation; the session counter recorded the actual peer transfer.

## Commands

```sh
python3 apps/holyarrs/check/plants.py
HOLYARRS_RECORD=demo/runs/holyarrs/2026-11-10 cargo test -p svrngs --test scenes holyarrs -- --test-threads=1
```

`chromium/` and `webkit/` hold each successful recorded scene's screenshots,
step verdicts, versions and fixture-call evidence. `target/scenes/holyarrs-*`
holds private raw material; it is overwritten by the next scene of that name.

## Checks observed failing before the positive recorded run

| Plant | Expected failure | Observed |
|---|---|---|
| `holyarrs:no-approval` | H3, imported file never arrives | Cargo exited 101 at H3 after Bo's request and Ana's approval page had been reached. |
| `holyarrs:no-seed` | H3, approval does not create a playable copy | Cargo exited 101 at H3 after the local seed was stopped. |
| `holyarrs:no-readers` | H4, Movie Night cannot search a protected library | Cargo exited 101 at H4 after a successful real import. |
| `holyarrs:credential` | H7, credential-evidence scanner catches the known owner token | Cargo exited 101 at `H7: Jellyfin admin reached Bo`, after the playback and removal checks passed. |
| `holyarrs:early-stream` | H6, a stream ended before removal proves no cut | Cargo exited 101 at `H6: stream must still be open before removal`. The positive check also requires the cut timestamp to follow removal. |

The repeatable plant runner requires both nonzero exit and the named failure;
the local logs and its machine-readable verdicts are in `target/holyarrs-plants/`.
The sanitized observations are [`plants.json`](plants.json). After tightening
H6, `python3 apps/holyarrs/check/plants.py early-stream credential` confirmed
both its own plant and the credential plant beyond it still fail as intended.
Removing Podman from the preparer's PATH also produced an explicit `never-ran`
response naming the missing prerequisite.

## Recorded standalone result

Both scenes passed all eight bars, with ordinary app accounts:

| Observation | Computer / Chromium | Phone browser / WebKit |
|---|---|---|
| Owner approval attempted as Bo | 403 | 403 |
| Requested title before approval | Absent | Absent |
| Bytes actually transferred and imported | 19,549,723 | 19,549,723 |
| Imported SHA-256 | `2cfb60a705dc8cf821cf126cc612d70451bdebe0da353672a1022a87aa6671f1` | Same |
| Movie Night after import | Ana's copy found | Ana's copy found |
| Jellyfin playback identity | `bo` | `bo` |
| 50% seek, playback resumed | 384 ms | 3,549 ms |
| Already-open HolyArrs live body cut after removal | 1,000 ms | 1,002 ms |
| Fresh request after removal | 403 `removed` | 403 `removed` |
| Owner credentials in Bo's received bodies and native data | None of the five known credentials | None of the five known credentials |

These are observations from this pair, not rates. Each engine's `result.json`
records the exact image digests, browser/Jellyfin versions and source HEAD.
The working tree held this integration uncommitted when it ran.

In this run Jellyfin's scanned name for the fictional film includes `(2017)`,
so Movie Night's screenshot displays the year twice.
The playback check matches the copy's actual item id, and the import check its
bytes, rather than assuming the display name is a canonical identifier.

## Repository checks

**Final full gate: passed, exit 0, 1,749 seconds (29 min 9 s).** Every
required step passed, including the complete scene suite and conformance
canary. [`gate.json`](gate.json) records the verdict and environment; the
fresh HolyArrs results are [`gate/chromium/result.json`](gate/chromium/result.json)
and [`gate/webkit/result.json`](gate/webkit/result.json). Both passed H1-H8.
Their seeks resumed in 372 and 4,972 ms; removal cut their already-open
live bodies in 1,002 and 986 ms, respectively.

The separate conformance **rate**, which does not gate, recorded two direct
WebKit control seeks above five seconds (14,183 and 5,231 ms). Its four lent-path
readings and every required conformance check passed. These observations are
kept apart from the gate verdict.

The fast gate and focused Clippy check passed. The first full attempt caught
the golden list in `crates/svrngs/src/share/tests.rs:278`, which still expected
the known programs before `holyarrs` was added. After updating that expected
alphabetical list, this exact reproducer passed:

```sh
cargo test -p svrngs --lib share::tests::llama_server_is_known_by_its_name_and_lent_without_the_table_s_paths -- --exact
```

The full gate is run with the existing `core.hooksPath=.githooks` verified in
place of rewriting that Git setting. All checks in `scripts/check.sh` still
run. Its fresh scene results go under `gate/`; the log and final exit status
are kept locally in `target/holyarrs-full-check.{log,json}`.

A second attempt stalled in Give's existing restore check, at
`podman machine ssh -- podman unshare chown .../svrngs-test-keep-47227-data/_data`.
`lsof -a -p 48867 -i` showed that SSH session's established IPv6 socket with
the same source and destination, `localhost:62636 -> localhost:62636`; a VM
process listing showed no corresponding command. The run and its watchdog were
stopped, and only that run's leftover volume was removed. The next full run
prefixes its own PATH with a temporary `ssh` wrapper executing
`/usr/bin/ssh -4 "$@"`, reaching the same real Podman VM and authentication over
IPv4. `podman machine ssh -- true` passed with that environment, and Give's
restore/check and keeper tests subsequently completed. This is a recorded host
environment adjustment, not a change to Give or to the HolyArrs scene.

## What the phone run taught the instrument

The phone's page gives one group tab the connection at a time. The driver
therefore uses one Bo tab and returns to the group between apps. A phone's open
also goes through `open.html` before landing on HolyArrs's `/login` or `/`;
waiting only for the desktop's `/svrngs-open` to disappear was insufficient.
The driver now waits for the destination application page.

After successful playback, navigating away from Jellyfin makes WebKit report
`Unhandled Promise Rejection: TypeError: Load failed`. Deliberately removing
Bo while the real HolyArrs `/live` body is open makes it report
`FetchEvent.respondWith received an error: Error: error reading a body from connection`.
Each is retained with its action and actor. Only these observed messages at
those deliberate cancellation actions are accepted; other script errors fail.

## Scope

This run uses ordinary application accounts and grants no application approval
rights through group membership. The end-to-end request and approval are browser
actions through svrngs. Scratch-service setup uses administrative APIs before
the scene; independent observations use local APIs. No helper places the final
movie in the library or manually refreshes Jellyfin after approval: HolyArrs's
own media-server integration does that.

The stranger probe uses the native lend origin with an empty browser context;
the phone's fresh-request and already-open-stream removal checks use its hosted
origin. Public indexers, public swarms, the upstream metadata server, physical
phones, two homes, SSO and application migration are outside this run.
