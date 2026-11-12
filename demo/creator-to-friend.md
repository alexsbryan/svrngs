# A useful thing for our group

Run-of-show and acceptance contract, written before the first run, 11 November
2026. Product direction: a lightly technical person builds with a coding agent;
friends use it without learning an authority system. The selected Eudoxia boundary
is the target; this first scene exercises svrngs's existing carried-app boundary.

## People and task

Jo helps with a community garden. She wants a Saturday rota, builds a small folder
app in a group of her own, then copies the finished app to **Garden**. Kim is in
Garden on her phone. She opens **Garden rota**, adds “Saturday morning”, returns
later, and decides whether to use Jo's update. Jo eventually withdraws the app.

The fixture is an adaptation of the guestbook in `docs/guide/build.md`, using only
`svrngs/acts`. Its two files live in `crates/svrngs/tests/scenes/creator/`; it is a
scene fixture, not a new shipped app or a purported agent-built artifact.
`Scene`, the existing browser drivers, real native processes, folder sync and
the actual phone signer provide the world. No fake authority evaluator is used.

## First slice: actions and enforced meaning

| Bar | Action and visible result | Independent boundary observation |
|---|---|---|
| C1 | Jo tests in her own group and runs `svrngs check`. Copying the two finished files makes Garden rota available in Garden. | Test availability stays in the test group's journal; it is absent from Garden. Copying files does not copy entries or issue another member's consent. |
| C2 | Before opening, both computer and phone say that it can read/write only its own shared entries, as the person, in **Garden**. Opening carries that bounded consent. | The words come from shared `question.js::agreed`, the selected group and the app's declaration. They are not a purpose supplied by this app. No key, account or extra permission dialog is needed. |
| C3 | Kim saves “Saturday morning” once; Jo sees **Kim: Saturday morning**. Reopening retains it. | The real journal contains the phone's own signed holder entry under this app's prefix and person, not a write as Jo. Another app's correction is refused without changing its entry. |
| C4 | Jo edits the app; her own copy updates immediately. Kim's chosen version keeps working. The group's page names Jo's change; **Open the new version** changes Kim's version and retains the rota. | Before the yes, actual HTML served to Kim remains the old version; after the phone's signed yes it is the new one. No backend acceptance is inserted by the scene. |
| C5 | Jo removes the published app folder. It disappears from the group's list and its old address no longer offers the app. | Removing availability leaves Kim's existing contribution in the group's journal. Removing files is not erasing already shared information or an unimplemented Stop button. |

Computer: Chromium in the current grouped layout (the scene asserts layout B;
legacy browser helpers otherwise opt into A). Phone: Playwright WebKit with an iPhone profile, through the
hosted group page and service worker. All processes run on one host, each with an
isolated HOME; this is a transport/browser workflow check, not a physical-phone or
cross-home measurement. Ordinary waits use the existing scene bounds.

## Instrument and execution

### Iteration loop

For consent wording, run the direct formatter laws first:

```sh
node --test web/tests/creator-consent.test.mjs
```

That requires Node only: no Rust build, browser, relay, replica or container.
It is the default loop while changing the sentence. For UI state/layout, use
the relevant `web/tests/pages/*.test.mjs` law; a browser rendering check should
use fixed API responses before introducing native processes. Those checks prove
presentation and transitions, not real signing or distributed enforcement.

Run only the creator scene when changing its publication/signing/version wiring:

```sh
cargo test -p svrngs --test scenes creator -- --test-threads=1
```

The full repository gate is the final regression check, once the focused checks
pass. Do not restart it for each copy edit. `scripts/check.sh --touched --files
<path>...` can select an explicit working set without including unrelated dirty
files, but its crate-level reach can still include the whole browser/scene suites.
It is a regression selector, not a substitute for the smaller UI feedback loop.

Pure formatter expectations currently also embedded in Rust browser tests should
move intact into the direct Node contract tests; browser tests then establish
rendering/wiring against that shared formatter. This avoids paying for native
compilation to discover a wording mismatch, while retaining the real workflow
checks for authority handoffs.

### Recorded acceptance

Before changing the consent words, run the audience/scope law in
`web/tests/creator-consent.test.mjs` and keep its actual failure. The scene must
observe the same wording in actual pages, actual journal attribution, version
pinning and removal. A positive write is required before any refusal is credited.

```sh
node --test web/tests/creator-consent.test.mjs
cargo test -p svrngs --test scenes creator -- --test-threads=1 --nocapture
```

The scene records the browser steps, actual assertions and journal observations
under `target/scenes/creator-chromium/`. Copy the run's report and command outcomes
to `demo/runs/creator-to-friend/2026-11-11/`; redact identity/session material from
published artifacts. Record failure rather than replacing a phone path with a
computer path. Source mutations for audience omission and premature version
acceptance must fail their named checks before the instrument is trusted.

## Newcomer session: owed separately

The scripted fixture and browser driver do not establish newcomer understanding,
and are not the missing human half of EXTENSION_POINT §6.4. For a real session,
give the creator only the community need, installed svrngs and their normal coding
agent. Give the member the ordinary message and group link. Do not hand either
person this run-of-show, API recipe or scripted solution. Record assistance,
their exact words about who can see a contribution, unexpected prompts, draft
leaks, update choices and how they try to stop sharing. One session is one run,
not a success rate. No human participant is fabricated by the automated scene.

## Following slice

Additional service access and its remembered consent/Stop follow this baseline.
The private-data-to-group-record handoff needs its own explicit interaction and
authority argument; a connector's background write is not relabelled as a personal
action. Initial app-publisher attribution and a person-facing carried-app withdrawal
control also need source-backed design; a carrier's name is not claimed to be the
app's author. This slice does not invent those facts or implement Eudoxia cutover.
