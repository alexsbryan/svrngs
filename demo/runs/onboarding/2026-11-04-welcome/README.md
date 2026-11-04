# Focused native welcome — 2026-11-04

Unit: `onboarding-welcome`, correcting the operator's actual three-group Fedora
arrival observation in `../2026-11-04-scene/FEDORA.md`. This technical browser
proof does not judge the real-laptop welcome or finish the people scene.

## Reuse and implementation

The current inventory is HOME's `first` address parser and `Home` view,
`group-folder.js::Folder`, `folder.js::FOLDER`, `/v1/groups`' real folder answer,
and the native installer's existing `/#group-<id>` authenticated open. These
already own destination, membership listing and folder opening; the upstream
survey in ONBOARDING §4 remains applicable. No new module, type, identity,
installer or folder API is needed. HOME retains the stable arrival id in its
already-typed query state. The focused view draws only that group's welcome
and existing Folder controls. Machine reads/refusals retain their trace;
`HOME: welcome` traces target resolution alongside group-read status.

**All your groups** is a normal `/` navigation. It discards the destination
structurally rather than keeping a flag that a later group read could reset.
Ordinary `/` still renders all groups. Host content remains in `WELCOME.txt`;
the UI says *if* that note exists and does not claim to have read it.

## Red before implementation

The expanded existing native-arrival browser fixture supplies the real target
plus two other groups in the groups response, one with the same name and a
different folder. The test asserts visible isolation, not merely a new heading.

```text
$ lockf -k /tmp/svrngs-gate.lock cargo test -p svrngs-node --test browser native_arrival_names_once_and_shows_the_actual_folder -- --nocapture
AssertionError [ERR_ASSERTION]: three-group arrival must hide unrelated groups
7 !== 1
HOME: read (3 group(s)): reading/none -> read/none
test native_arrival::native_arrival_names_once_and_shows_the_actual_folder::chromium ... FAILED
test result: FAILED. 0 passed; 1 failed; 0 ignored; 98 filtered out; finished in 15.76s
exit 101
```

The seven sections include the three groups and management places. The fixture
changes only the native groups answer; the canonical native-name path still
uses a real issuer/relay and verifies no persisted browser identity and a
byte-identical issuer journal. The opener stub checks the exact group id and
real nondefault path sent to the existing `/folder` boundary. The separate
existing folder browser test covers the actual guarded OS opener.

## Green

```text
$ lockf -k /tmp/svrngs-gate.lock cargo test -p svrngs-node --test browser native_arrival_names_once_and_shows_the_actual_folder -- --nocapture
native arrival: { "folder": ".../already-used-name-2", "kept": [], "hash": "#group-<fixture-id>", "line": "<literal Bo O'Brien native installation command>" }
test native_arrival::native_arrival_names_once_and_shows_the_actual_folder::chromium ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 98 filtered out; finished in 29.05s
exit 0
$ python3 scripts/lean_check.py
crates/svrngs-core     3486 / 3494 lines
.                     19779 / 19833 lines
js                     8066 / 8090 lines
swift                     0 / 1 lines
$ node scripts/types.mjs
types: 0 errors in 129 files
```

The browser also covers welcome refresh; explicit all-groups navigation,
visibility reread and refresh staying away; loading before the groups request
answers; unread groups; missing stable id despite a same-name group; unknown
and absent folders. Two intermediate failures corrected the instrument: hash
navigation alone does not reload HOME's one-time address parser, so fixture
mode changes need a reload; the loading assertion may precede fetch startup,
so release waits for the intercepted request to exist. Neither was a product
failure. Real people, file exchange and installation remain the scene's bars.

The final view is composed inside the existing file (Arrival and its folder
presentation), keeping each function within the unchanged complexity limit.
Final affected checks, after that composition:

```text
$ lockf -k /tmp/svrngs-gate.lock sh -c 'cargo test -p svrngs-node --test browser native_arrival_names_once_and_shows_the_actual_folder -- --nocapture && node --test web/tests/pages/*.test.mjs'
test result: ok. 1 passed; 0 failed; 0 ignored; 98 filtered out; finished in 28.67s
# tests 45
# pass 45
# fail 0
exit 0
$ lockf -k /tmp/svrngs-gate.lock sh -c 'node --test web/tests/pages/*.test.mjs && cargo test -p svrngs --test install && cargo test -p svrngs-node --test browser the_home_page_opens_the_group_s_folder -- --nocapture'
# tests 45
# pass 45
# fail 0
test result: ok. 13 passed; 0 failed; 0 ignored; finished in 19.96s
test the_home_page_opens_the_group_s_folder::chromium ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 98 filtered out; finished in 7.88s
exit 0
$ web/node_modules/.bin/oxlint
exit 0
$ node scripts/typed-lint.mjs
typed lint, jsconfig.json: 125 files, exit 0
typed lint, jsconfig.workers.json: 3 files, exit 0
```

## Matching local artifacts and minimum Fedora retry

The existing recorder completed against welcome source
`26fa9693a869b5921fbe01828f3c6f2e5325ad6b`:

```sh
python3 demo/runs/onboarding/2026-11-04-scene/prepare.py
```

It rebuilt both page and all Mac/Linux downloads from that same committed HEAD.
The exact commands, exits and SHA-256 inventory are recorded under
`../2026-11-04-scene/data/26fa9693a869/` (`prepare.json`, `artifacts.json`
and each step's log). The loop resumed this unit after the run exited 0 in
30m05s. The full gate itself took 1,466 seconds. The subsequent commit
`011338a5` only queues other work; these artifacts contain the committed welcome.

```text
$ python3 demo/runs/onboarding/2026-11-04-scene/prepare.py
build-web: exit 0
build-downloads: exit 0
checksum-red: exit 1 (expected)
  AssertionError [ERR_ASSERTION]: download checksum: svrngs-darwin-aarch64.tar.gz
  expected: 0000000000000000000000000000000000000000000000000000000000000000
staging-green: exit 0
  passed: built canonical shell, installer, wasm and all matching download checksums served over HTTPS
  passed: local HTTPS resolver reaches live Free store with its generation guard; absent-address guidance; no publication
check: exit 0
  command: lockf -k /tmp/svrngs-gate.lock scripts/check.sh
  passed: principles, lean/RUST_ARCH, trust floor, advisories, lint, format,
          page laws, rustfmt, clippy, tests, canaries, browser, types,
          typed lint, joining, pages, scenes, conformance and app laws
Requested preparation steps passed; people scene never-ran. Servers stopped. Operator judgment owed.
```

The checksum plant was the all-zero expected digest; the clean probe then
verified the real matching downloads. The completed preparation makes the
following retry ready for the supervisor; it does not judge the people scene.

Supervisor/operator:

1. Restart the existing local scene server using the new matching site from
   `target/onboarding-scene/site` (scene `STAGING.md`); retain the dedicated
   host's identity, group and folder. The prior running server embeds the old
   home page in its native binary, so merely refreshing the old Fedora binary
   is insufficient.
2. On Fedora, use the same staged invitation/name installation path to update
   the existing installation with the new Linux download. Do not delete its
   data, create another identity or change personal folders. If its invitation
   has expired, the host supplies a current invitation to the same group.
3. The generated command already opens `/#group-<id>`. For an already-updated
   installation that needs reopening, run
   `~/.local/bin/svrngs open '/#group-<same-stable-id>'`, using the group's id
   from the dedicated scene host, not its display name.
4. Ask the operator what they see and do next without folder directions.
   Record welcome/orientation, the actual opened path, contribution and file
   arrival, and whether **All your groups** feels clear. Keep the original
   scene predictions and remaining interrupted-join/closure observations.

No public deployment, publish, push or Fedora personal-data change is part of
this handoff. V1.6 publish artifacts are unrelated and untouched.
