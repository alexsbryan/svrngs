# HolyArrs — ask for a film, then watch it

Plan and run-of-show, written before the first run, 2026-11-10. Operator:
"Ok I want to build that scene out and test it fully", then "Let's write up
the plan first then executy".

## The scene

Ana lends HolyArrs and Jellyfin to Friends. Bo joins by the group's link,
opens HolyArrs from his home page and signs in as Bo. He asks for a film
which is absent from Ana's library. Ana sees Bo's request and approves it.
HolyArrs finds a release, sends it to the downloader and imports the completed
file into Ana's library. Bo opens Movie Night from his home page, finds Ana's
new copy, follows it into Jellyfin and plays and seeks it as Bo.

The first milestone uses each application's own accounts. Account setup is
recorded as preparation, with no claim of zero-login onboarding. HolyArrs's
request permissions and Ana's approval stay in HolyArrs. svrngs supplies group
membership, reach and the Movie Night reader permission.

## What runs, and what the run can establish

- Real, version-recorded HolyArrs, Jellyfin and svrngs; separate scratch homes
  for Ana and Bo; the existing scene harness's real local iroh relay.
- A real downloader, with a generated audiovisual test pattern distributed
  from a local seed. No copyrighted film is needed. The run must witness
  downloaded bytes and HolyArrs's own import, rather than place the final film
  into the library on its behalf.
- Deterministic metadata, source and tracker fixtures on loopback or the
  isolated container network. Name these substitutions in every result. They
  make release selection and acquisition reproducible; they establish nothing
  about public indexers, public swarm reach or the hosted metadata service.
- Browsers drive the person-facing request, approval, discovery and playback
  steps through lent origins. Direct administrative calls may prepare scratch
  services and observe independent evidence; they never stand in for a scene
  action. Keep the distinction in the step record.
- One-host automation establishes integration behavior. A recorded session
  with real newcomers, two homes and a real phone remains a separate run;
  this scene supplies no human success rate.

## Execution order

1. **Inventory and instrument.** Reuse `crates/svrngs/tests/scenes/people.rs`,
   `films.rs`, `movie_night.rs`, `apps/jellyfin/check/person.mjs` and their
   browser/recording helpers. Survey HolyArrs's own test fixtures before
   writing equivalents; any fork records the source path and commit in each
   source file. Inspect commonwealth-ai's ring inventory before adding a new
   reusable type. Record exact versions, ports, image digests and the command.
2. **Compatibility trial.** Start a scratch HolyArrs, complete owner setup,
   and lend it with `svrngs share <port> --program holyarrs --group Friends`.
   Reproduce sign-in and a harmless read through the door before changing
   behavior. Capture browser failures and svrngs debug decisions. Add the
   minimum account instructions/program terms warranted by that evidence.
3. **Acquisition rig.** Generate a uniquely identified film and local release;
   start the seed, downloader and source/metadata fixtures. Configure a scratch
   library shared by HolyArrs's importer and Jellyfin. Keep all databases,
   credentials and media outside the group's shared folder. Confirm the title
   is absent before asking and the release is unavailable to import before
   approval. Fail on an unexpected fixture route.
4. **Browser scene.** Ana founds Friends and lends both services. Make Bo's
   application accounts and the restricted Jellyfin readers' account during
   preparation. Bo joins from the invitation; both open the services through
   their own home pages. Bo asks; Ana approves; the real job downloads and
   imports; Jellyfin discovers the imported film. Bo grants Movie Night its
   read, searches, opens Ana's copy, signs into Jellyfin as Bo, plays and seeks.
5. **Watch the checks fail.** Run each plant below against its named bar,
   preserve its failure, then run the unplanted scene. Exercise Chromium for
   computers and WebKit in the phone path where the existing harness supports
   it. Missing prerequisites are `never-ran`, not a passing scene.
6. **Land the evidence.** Put sanitized step results, screenshots, versions,
   commands and verdicts under `demo/runs/holyarrs/<date>/`. Keep raw browser
   bodies, credentials and service databases in the scratch run, never in git.
   Run focused checks while iterating, then `scripts/check.sh` as the full
   gate. Report unrelated pre-existing failures separately. Update the design
   and architecture where the implemented integration changes their claims.

## Acceptance bars

| Bar | Evidence required |
|---|---|
| H1 — reach and identity | Bo joined Friends by its invitation and opened Ana's HolyArrs through svrngs. HolyArrs identifies the signed-in asker as Bo, with a different account from Ana. |
| H2 — approval matters | Bo's request appears in Ana's own browser with Bo named; before her approval there is no acquisition/import. Bo cannot approve it himself. |
| H3 — real acquisition and import | After Ana presses Approve, a real downloader obtains the generated release. HolyArrs's import produces the library file; its bytes match the seeded media. No scene helper copies that final file into the library. |
| H4 — newly discoverable | Before approval Movie Night cannot find this film; afterwards its search finds the imported title and says Ana lends the copy. |
| H5 — actual playback | Following that copy opens Ana's lent Jellyfin at the matching item. Bo signs in as Bo; video time advances and a seek resumes playback. Capture Jellyfin's session independently. |
| H6 — access remains bounded | A stranger cannot open the lends. After Bo is removed, fresh requests are refused and an already-open stream is cut by the lender. Ana's own access still works. |
| H7 — credentials stay with their owner | Ana's passwords, admin tokens and the readers' credential appear in neither Bo's browser responses nor his svrngs data/logs. Bo's own application session is expected in his browser. |
| H8 — reproducible verdict | Every bar has passed, failed, could-not-judge or never-ran plus its evidence. The runner returns failure for failed or never-ran required bars, and cleans up only the processes/containers it created. |

Bounds are assertions, not sleeps: each wait names the state it awaits and
ends with the observed state and relevant trace if it expires. Record elapsed
times as observations, not a human performance claim.

## Failure plants

| Plant | Check it must fail |
|---|---|
| Skip Ana's approval | H3: no completed/imported title within the acquisition bound. |
| Stop the local seed / make its release unavailable | H3: an approved request alone is insufficient to pass import. |
| Withhold the Jellyfin readers' account | H4: Movie Night must not claim discovery when the library refuses it. |
| Inject Ana's admin credential into the served evidence | H7: the credential scanner catches the known planted secret. |
| End the live stream before removing Bo | H6: a stream already ended cannot prove that withdrawal cut it. |

The plant result names the bar which failed, so an unrelated setup failure is
never evidence that the intended check worked.

## Deliverables and completion

- `apps/holyarrs/`: lender-facing setup instructions and the app-specific
  acquisition/browser helpers the scene needs.
- A reusable automated scene beside the existing media scenes, with its
  fixture boundary explicit and its prerequisite/setup instructions runnable.
- This run-of-show, a recorded positive run, each planted failure and the full
  check's verdict. Record any compatibility fixes alongside their reproducer.

SSO interoperability and an embedded Movie Night request button are follow-on
work: the first scene opens HolyArrs's own request interface. Automatic host
migration would also need an explicit design for its database, encryption key,
media and downloader. None is a prerequisite for the scene above.
