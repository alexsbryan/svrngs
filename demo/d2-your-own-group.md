# D2 — Your own group in fifteen minutes

Gate: `host-setup` — three of four people, unassisted, go from install to a
first chat act within fifteen minutes (DESIGN D9 E0). Shown after e0-5.

**Before the run.** Four people who have not used svrngs, each on their own
computer (macOS or Linux), each with a Discord account and a Discord server of
their own with at least one text channel and two friends in it. Each computer
has git; nothing else is installed for them. Each person gets the svrngs
repository's address and this card, and nothing else:

> 1. Install: install Rust from https://rustup.rs and Node.js 20 or later from
>    https://nodejs.org, then
>    `git clone <the address> svrngs && cargo install --path svrngs/crates/svrngs`.
>    Keep the `svrngs` folder.
> 2. Run `svrngs start` and do what it says.

Screen recording on for every participant who agrees. The operator writes down
the svrngs commit every person installs.

**What each person is told, and nothing else:** "Put the svrngs bot in your
Discord server so your friends can split something there. Here is the card."

**What is watched.** The clock starts when the person starts the first command
on the card. A person succeeds when a `/paid` or a Confirm tapped in their
channel — theirs or a friend's — is in their computer's group record within
fifteen minutes: `~/.svrngs/groups/<id>/journal.jsonl` holds a line whose
`body.kind` is `record`. The operator says nothing and touches nothing; any
help ends that person's attempt and is written in the notes. A Discord account
or application the person makes is Discord's, not an account with svrngs, and
is not counted in `accounts_created`; a svrngs prompt for a key is counted in
`key_prompts`.

**Steps, counted.** DESIGN C11 names five steps: install; make the Discord
application and paste its token; setup finds the application, prints the
invite link, lists the channels, enables the service and warns if the
computer sleeps; open the link; pick the channel. For each person, the notes
give the real count — every separate thing they did, including installing
Rust and Node.js, flipping a switch setup asked for, and every retry — and
the minutes each step took.

**After.** For each person, keep `~/.svrngs/logs/serve.log` and the line count
of the journal. Write the record in `runs/d2/` (`demo/README.md`;
`participants` 4, `succeeded_in_time` the people who succeeded) before looking
at anything else, then the notes: the step counts, and where each person who
did not finish got stuck. A failed run is kept, not rerun until it passes.
