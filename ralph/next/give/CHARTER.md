# svrngs give — director charter

The supervisor's resolution session decides this queue's forks instead of
waiting on the operator. `NORTH_STAR.md` and
`docs/principles/ARCH_PRINCIPLES.md` are the compass. Where they and this
charter disagree, they win, and the decision says so. The exploration is
svrn-docs `rings/current/GIVE.md`. Read it whole before the first row.

## What the operator decided, 2026-10-25

- **Any port, not only HTTP, is svrngs's own work.** The operator: "I did
  hope that would be a promise served by svrngs" and "moving beyond just HTTP
  makes since as core svrngs lib". North star 2: if it runs on localhost, you
  can share it.
- **The games are Factorio and Minecraft.** The operator: "Let's do Factorio
  actually since it's a game I play and can test", then "factorio +
  minecraft".
- **Give is its own small package.** The operator: "I wonder if this maybe
  makes more sense as its own tiny package for a specific responsibility
  (Give, that is.)". It lives in `apps/give/` and speaks only svrngs's HTTP
  surface and the synced folder on disk. It links no svrngs crate.
  - It has two halves: a carried app every member has, and a keeper program
    on the computers that keep.
  - It is an svrngs app with a companion program, not a group-hosting
    library. Its hard parts (identity, membership, order, sync, reach) are
    svrngs's (GIVE.md §7a).
- **It must work on Mac and Linux, with no hypervisor svrngs signs.**
  Containers run in rootless podman, from each app's own published image.
  The operator: "Why not a simple podman distribution route?"
- **The person's journey must be fluent, and each new app easy.** The
  operator: "make sure that our lightly technical user would be able to do
  the give with a clean and fluent interface and that each additional
  extension is easy".
  - An app is one `compose.yaml` with an `x-svrngs` block of five fields:
    title, about, door, state, asks.
  - Health is compose's own `healthcheck`. There are no hooks and no per-app
    code in the library.
- **The hardest territory is conceded first:**
  - every move and takeover is a person's yes;
  - snapshots are minutes apart, plus one at each clean stop;
  - a stale keeper's state becomes a copy someone can start instead, never a
    merge;
  - there is one keeper at a time;
  - phones never keep;
  - members carry state, not images: a takeover pulls the pinned digest from
    the registry.
- **Accounting comes in four ledgers:** core, library, app, and surface. The
  operator: "I consider it on the library side vs the pure app side -- I
  think we need more nuance on our accounting." Every row's estimate and
  close reports all four (GIVE.md §9).
- **The transport is settled analytically.** There is no measurement before
  the code.
  - UDP rides QUIC datagrams. The door fragments a packet larger than
    `max_datagram_size` and drops the whole packet if any fragment is lost.
  - TCP rides one QUIC stream per connection.
  - Quality over a relayed path is judged by play, in give-play-scene
    (GIVE.md §6).

## Awaiting the operator before the rows it names

1. **The ceiling** for the svrngs rows, core and repository, which
   give-economize asks for. give-token does not start until it is set.
2. **WHAT_CAN_ACT_AS_YOU §4.** Does the caller check overturn "Accept it, and
   say so" for programs meant to be contained? It is asked with the ceiling,
   because give-token depends on the answer.
3. **The trust floor's new lines**, asked with the ceiling. They are the
   member's Play listener on `127.0.0.1` alone, and the keeper's per-flow UDP
   socket on `127.0.0.1:0`, connected to the lent port. A change to the floor
   is the operator's (AGENTS.md "Done").
4. **Who may keep**, asked at give-app-keep: any member with Podman and the
   keeper program, or a successor the house names. Until the operator
   answers, the default is any member.

## Decide

- row order, and splitting or re-scoping a row that cannot execute;
- the datagram frame's layout and a flow's idle expiry;
- the snapshot interval;
- the stand-in images the tests and scenes run;
- give's language and file layout, held to three things:
  - **one fold** for the keeper fact, built once and run by both the page and
    the keeper program, as `apps/canon/build.sh` builds canon-core to plain
    JavaScript;
  - the keeper program runs on a computer that has only svrngs and Podman;
  - the runtime module names no svrngs type;
- forking from commonwealth-ai over building new;
- the smaller reversible step.

**Bars by their spirit** (operator, 2026-10-08). Ask what a bar guards and
judge by that. Where its letter and spirit part, rewrite the letter, say both
in the commit, and keep going.

## Leave for the operator

For each of these, write `NEEDS_HUMAN.md` and stop:

- a row past its ceiling, or past its estimate's range;
- a trust floor line beyond the two named above;
- **give's kill clause:** a capability give cannot do without from `crates/`
  or `web/` beyond the caller check and the port door. The verdict names it
  and does not make the change;
- an app's noun in the infra;
- changing a decision in DESIGN §10;
- docker, OrbStack, or any tool that is not open software (operator,
  2026-10-05: podman and open software only);
- a dependency on a commonwealth-ai or sovereign crate;
- anything that needs a service this host cannot run. Say what, and give the
  command that would run it;
- the people runs: the Factorio and Minecraft sessions, and the fluency run a
  stranger judges. The operator sets their bars before they run;
- push and publish.
