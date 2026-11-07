# The seed-swap notebook

A permaculture-inspired Xenia rehearsal: a useful shared resource, knowledge
left for the next person, a limit on giving, a consensual handover, and an
attempt to end the arrangement. This is a runnable consumer scene, not a
change to svrngs's product design or its order of work.

## Run

From this checkout, build the real world once:

```sh
cargo test -p svrngs --test xenia --no-run
```

Then from `~/dev/xenia`, using Node >= 22.18:

```sh
node src/run.ts run ~/dev/svrngs/demo/xenia/permaculture --runs 1 --model claude-sonnet-5-5 --findings ~/dev/svrngs/demo/xenia/permaculture/findings --source ~/dev/svrngs
```

Xenia runs its plants before the rehearsal. One run is an exploration, not
a rate. For repeated observations, choose `--runs 3`; the people stay the
same and each run gets a fresh world. The notebook uses Podman and the
pinned BusyBox image in `cards/notebook/compose.yaml`; absence is never-ran,
not a simulated replacement. No Xenia engine changes are needed.

For the scripted browser proof, without model sessions:

```sh
node crates/svrngs/tests/xenia/neighbourhood-smoke.mjs
```

`XENIA_ROOT` can name another Xenia checkout. The default is the sibling
`../xenia`. The smoke uses Xenia's actual scene/person parsers, browser
launcher and reach filter. It is an instrument/workflow proof, not a blind
stranger or a people result. Do not run two copies of this world at once:
its retained scratch directory is `target/scenes/xenia-neighbourhood-chromium/`.

## The run-of-show, written before the first run

Sam, Jo and Ren belong to **Seed swap**. Sam and Jo have Give running; Ren
only uses ordinary pages. Each has their own svrngs process, data directory
and browser context. All run on this one host, through a loopback relay.
Installation, joining, and Sam's initial start of the notebook happened
before this visit and are explicitly outside the observation.

The notebook is a small, explicitly labelled fixture: an ordinary web app
in a real container, with a note form and a durable data volume. It is not
a mock of Give and is not a shipped neighbourhood app. Give snapshots and
restores that volume; nobody copies Ren's note into Jo's app backstage.

| Part | What reaches each person | What to look for |
|---|---|---|
| `benefit` | The seed swap's time and place are in the notebook. | Can each person get something useful before learning to administer it? |
| `retain` | Ren found the envelopes; leave their location for others. Sam and Jo look for it. | Does an ordinary contribution become usable shared knowledge? |
| `not-now` | Sam wants a break. Privately, Jo cannot take on keeping it this week. | Can Jo decline through the product, keep using it, and leave provision with Sam? |
| `handover` | A week later in the story, Jo has time and is willing. | Do Sam and Jo find Give, offer and accept, retain Ren's note, and stop Sam's old app? |
| `end` | The season is over; stop running it while keeping the notes. | Can they distinguish stopping access from stopping the app and retaining its state? |

Time passing is narrative only: no clock, snapshot age or takeover timeout
is advanced. The scenes share browsers, chat, data and each person's own
Xenia dossier. The four-minute limit and 180-word chat limit per part are
rehearsal bounds on this exploratory conversation, not established human
usability bars. Running out of minutes is could-not-judge.

People get local, ordinary messages, never the permaculture thesis or an
instruction to prove cooperation. Jo's first refusal and later willingness
are situated constraints, not a requirement to be altruistic. Ren can
contribute knowledge without running Give. This does not establish access
for people without computers; this first cast uses three computer browsers.

## Three kinds of evidence, kept apart

1. **Xenia's account:** what its stage counted on each person's screens,
   and hard faults. In multi-scene mode, `passed` means the person finished
   the conversation, not that a handover or shutdown succeeded.
2. **The person's words:** what they expected, what they did, where they
   got stuck. These are simulated predictions, not findings from real people.
3. **World observations:** `evidence.json`, attached by Xenia to each
   person's run. A read-only observer records the real group journal,
   Ren's discovered lends, the notebook's answer through Ren's door, and
   Podman's inspect results for Sam and Jo. Missing and failed observations
   are explicit. A note surviving means the new app actually served it,
   not that somebody said it survived.

The observer samples every three seconds. Inspect its recorded elapsed
times; a missing transition is unobserved, not disproved. It never offers,
accepts, writes a note, moves an app, or stops one. Closing Xenia's stdin
ends the world and removes only its namespaced pods/volumes. That cleanup
is not credited as a community's clean ending.

### Checks to exercise before trusting the rehearsal

The scripted proof first watches the retained-note check fail on the real
empty notebook, then writes the note through the browser and watches it
pass. It also watches the successor check fail while Sam still provides
the app. It then offers, declines, offers again, accepts, and verifies the
note from Jo's actual app and that Sam's pod has stopped. These controls
protect against a handover assertion passing on an unchanged source.

The final part is deliberately open. Current Give offers moves and
takeovers but no stop-keeping action (`apps/give/page/index.js`, `row`;
`apps/give/fold/src/lib.rs`, the act vocabulary). The home page has **Stop
lending**, which is not a command to stop the app's container. The smoke
records whether unsharing leaves it running. A missing end-to-end control
is an outcome to expose, never filled in by the scene.

## The design questions

- Mollison: useful relationships, multiple kinds of contribution, and an
  important function that can pass between providers.
- Holmgren: obtain a yield; catch and retain something useful; accept a
  limit; share surplus without making one person indispensable; respond
  to a season ending.

This scene cannot establish a regenerative community, ecological savings,
long-term fairness, cross-home resilience, or actual caretaker wellbeing.
Those need real communities, elapsed time and material measurements. Its
first job is to expose what the current screens make possible or awkward.

## Staging status

Prepared on 6 November 2026. The operator deferred the run for resource
planning. No world, container workload, browser proof, Xenia plants or
model sessions have been launched for this scene. Its runtime outcomes,
including the expected distinction between unsharing and stopping, are
**never-ran**, not passed. The full repository gate is deferred too.

Completed lightweight checks: the Xenia Rust target compiles; its focused
clippy check passes; the browser proof parses and passes lint/format; the
launcher passes shell syntax checking. Plan the model, repetitions, wall
time and spend before invoking the run commands above. The five parts
allow up to twenty scene minutes per repetition, plus setup and Xenia's
plants; model/provider overhead and actual spend are not measured here.
