# The newcomer's film journey — 2026-11-11

Operator direction: "Just warranty the work; we have the tools to discern this
with near-analytical certainty." This is an engineering warranty over a named
journey and its stated conditions, not a claim about people in general.

## Conditions

- One host. Ana already keeps HolyArrs 1.0.10 and Jellyfin 12.0.0 and lends both
  to Friends. Two qBittorrent 5.1.4 peers transfer a local test torrent.
  Metadata, release search and the tracker are explicit local fixtures.
- The newcomer starts outside Friends in an empty phone-browser profile: no
  pre-joined native Bo, no own-device shortcut, no preaccepted request account.
- The chat carries Ana's ordinary group invitation, Bo's viewing account and
  password, and the request-account invitation Ana produced through HolyArrs's
  own UI on the group's shared HTTPS page.
- Actors in the blind rehearsals are simulated by `deepseek-flash`
  (DeepInfra `deepseek-ai/DeepSeek-V4.1-Flash-1017`, reasoning effort `max`).
  No human took part. A model's completion is not a population observation.

## Verdicts

| Check | Verdict | Evidence |
|---|---|---|
| N1 — arrival | passed | Fresh profile joined Friends from the ordinary invitation; no existing membership used. `cold-journey.json`. |
| N2 — first accounts | passed | Bo accepted Ana's actual UI-produced invitation and chose his own request-account name and password. `cold-journey.json`. |
| N3 — find and watch | passed | Movie Night named the actual copy; Bo recovered from a wrong viewing password, played Test Pattern One as `bo`, and sought to 50%. The read-only observer recorded the same item, user and playing state. `cold-journey.json`, `newcomer-playing.png`. |
| N4 — ask | passed | The independent observer saw Bo's own Copper Lighthouse request in `asked` / "Waiting for approval". `cold-journey.json`. |
| N5 — come back | passed | The original invitation reopened the same group and request account; the waiting request survived. A wrong viewing password was refused with visible words; stopping the HolyArrs container produced "not answering now", and restarting it retained the same account and request. `cold-journey.json`. |
| N6 — computer construction | passed | The repository's installer tests ran and passed inside the product gate's tests step: the macOS payload built from the checked source, a fresh node reached its group and kept its identity on retry. The lifecycle/open commands are stubbed as the instrument declares; no real Linux installation was run. |
| N7 — discoverability | could-not-judge | The current Xenia admission missed `p1-prominent`: the model stopped to read 120 words of terms and ran out of patience where the control expects acting on the prominent control. Per the plan, a missed plant cannot sign a discoverability warranty. The cohort below is retained as observational evidence only. |
| N8 — retained warranty | passed for the deterministic rows; N7 owed | The product gate passed end to end; every deterministic row has a reproducer and control. The blind discoverability row remains could-not-judge. |

The final cold journey is `target/newcomer-film/2026-11-11T04-16-10-324Z`
(`verdict: passed`): arrival 9.0 s, join 4.8 s, Movie Night to playing 15.5 s,
seek to 50% in 1.9 s. The same build's report is `cold-journey.json`.

## Controls

- **Closed invitation:** an empty profile opening a used/closed ordinary
  invitation saw the refusal ("This link lets nobody in. Ask for a new one.")
  and the authoritative record contained no Bo. `closed-invitation.json`.
- **Wrong item:** the same journey with the expected item set to Test Pattern
  Two failed at the exact assertion "the chosen copy is the independently named
  Test Pattern One item"; a visible Play alone could not pass it.
  `wrong-item.json`.

Both controls ran on the same product binary and world construction. The only
later change was to Xenia's scene screens and expressions (test metadata); the
product path is unchanged, and the cold journey was rerun after that change.

## Blind cohort (observational only)

Frozen profiles and unchanged messages; `cohort.json`. The current model
admission missed one plant, so these runs are observations, never a rate or a
discoverability pass.

- **ordinary** (620 s): the observer saw Bo play Test Pattern One (18 samples,
  position advancing, not paused), but the scene ran on after it said so and
  ended "ran out of patience". In the request scene it never used the
  invitation link: no HolyArrs Bo account and no ask were observed. Return ended
  could-not-judge (scene time; 42 words read against a budget of 30).
- **impatient** (675 s): all three scenes ran out of patience; wordy faults at
  the film and on return. No request account or ask was observed.
- **cautious:** not run; stopped when the operator accepted the N7
  could-not-judge verdict.

The product's deterministic request-account path is proven above. The blind
model did not discover the invitation in the chat; that is a discoverability
observation for this model, not evidence that a person would fail.

## Gates

- Product: `target/newcomer-film/gate-svrngs/` — exit 0 in 1716 s. Tests,
  canaries, ceiling, browser, types, typed lint, joining, pages, scenes
  (HolyArrs in Chromium and WebKit), conformance and app laws all passed.
- Xenia: `target/newcomer-film/gate-xenia/` — exit 1 in 4557 s. Every step
  passed except the plants: the set is could-not-judge because `p1-prominent`
  was missed. Its local properties and 130 tests passed.

## Instrument findings and fixes

- Xenia ignored a run scene's declared `reach` milestones in the core, and the
  stage did not evaluate their expression checks. Fixed with red/green tests on
  casts of one and two, plus an unread/future-scene control.
- An icon control's exact title ("Play") lost to another control's partial name
  ("Group playback"), so the blind actor opened the wrong menu. Fixed in
  `FIND`; a real-page test now proves the film's Play wins.
- The test page host served CSS as `application/octet-stream`, so the rehearsal
  saw an unstyled page. Fixed and pinned by a WebKit computed-style test.
- The browser suite built into one shared `target/web/site`, so another run
  could delete a live suite's assets (404s). Each test process now builds into
  its own directory.
- The cohort exposed scene-side expressions that read `document.body` during a
  navigation and threw; they now tolerate a document without a body. Specific
  screens are declared before origin fallbacks.
- The Xenia admission's browser teardown hung once after `c1-blind` had already
  folded its verdict; the stuck browser was cleared so the set could continue.
  This is an instrument robustness issue, recorded rather than hidden.

## Limits

- Simulated actors; one host; local metadata/source/tracker fixtures; a
  synthetic test film; macOS installer scope only.
- The cohort is not a rate, not a human result, and not evidence that a
  population will succeed.
- N7 remains owed: a discoverability warranty needs an admitted model and a
  cohort that completes the request path.
- Raw invitations, passwords, browser bodies and private world state remain
  under `target/`; the JSON here is sanitized.
