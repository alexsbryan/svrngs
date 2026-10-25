<!-- section: give after=intro -->
This queue makes any port shareable and lets a group keep an app. It has two
parts:

- **svrngs's own work:** a caller check on loopback, and a door that carries
  raw UDP and TCP.
- **Give**, in `apps/give/`: a carried app plus a keeper program that runs an
  app's own image in rootless podman, one keeper at a time.

Before a row, read this queue's `CHARTER.md`, `STATE.md`'s opening, and
svrn-docs `rings/current/GIVE.md` whole.

Fences the rows keep:

1. **Red first** (AGENTS.md 2, 5). Each row's first test fails at HEAD on the
   behaviour the row adds or the protection it closes. Describe what is
   refused and why, in the test and in the commit.
2. **Nothing a person does today breaks, and no new step blocks them.** Every
   legitimate path keeps working, and a test names each one it keeps:
   - opening a lend from the home page or the group page;
   - a phone's yes through the group page;
   - a newcomer opening Money and Notes answering no question;
   - every `apps/` caller of the record API;
   - every demo scene in the gate's "scenes" step.
3. **The library learns no app.** No Factorio, Minecraft, save, world or
   game in `crates/` or `web/src/` (`scripts/vocab_check.py`). A raw lend is
   "a port", and its row speaks in the lend's own name.
4. **Give speaks only the surface.** `apps/give/` links no `svrngs-*` crate
   and reaches svrngs through:
   - the record API;
   - `svrngs share` and `svrngs unshare`;
   - the synced folder on disk;
   - its own carried page.

   Needing more is the kill clause: `NEEDS_HUMAN.md`, never a patch to
   `crates/`.
5. **One decider** (AGENTS.md 8). The keeper fact has one fold, run by the
   page and the keeper program alike. A check lives where its decision
   already lives.
6. **Glassbox** (AGENTS.md 1). Every refusal traces at debug and names the
   fact that refused it. The person is told in words they can act on (svrn-docs
   `VOICE.md`). A person never sees container, image, volume, snapshot,
   keeper or port forwarding.
7. **Four verdicts** (AGENTS.md 6). A test or scene that needs podman, a
   registry or a game this host lacks is never-ran, with its reason. It is
   never a silent stand-in.
8. **The budget.** Each row reports four ledgers: core, library, app and
   surface (GIVE.md §9). The economizing pass comes first, and a raise is the
   operator's. `apps/` is not counted, but it is reported.
9. **The trust floor** (`scripts/trust_floor.txt`) gains only the two lines
   the charter names, each with its `Trust floor:` line. Any other widening
   is NEEDS_HUMAN.md.
10. **Publishing is the operator's.** The last row prepares a release and
    stops.
11. **Scratch stays under `target/`.** Podman's test containers, volumes and
    networks are named `svrngs-test-*` and removed by the test that made
    them.
12. **A gate's result needs no commit of its own.** The await log keeps it.
