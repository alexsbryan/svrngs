<!-- section: models after=intro -->
This queue builds the pantry's minimum: two protocols, discovery, and one
address, so that the community extends it by writing servers it already
knows how to write. A model is any lend that speaks OpenAI's API; a tool is
any lend that speaks MCP. `pantry list` shows them. `pantry serve` gives one
local OpenAI-compatible address that routes each request by its `model` to
the friend who lends it. Behaviour beyond that is an OpenAI-in, OpenAI-out
server someone writes; the vision bridge is the worked example. Before a
row, read this queue's `CHARTER.md` and `STATE.md`'s opening, and svrn-docs
`rings/current/MODELS_ON_THE_RING.md`. This queue builds far less than that
design; what STATE.md lists under "Waits" is not built here.

Fences the rows keep:
1. *Instrument and reproduce first* (AGENTS.md 2 and 5). Each change starts
   red, named in its commit, and every new check fails first on a plant it
   names.
2. *No platform line* (CHARTER.md, "The budget"). The pantry lives in
   `apps/pantry/`. A platform need is NEEDS_HUMAN.md.
3. *Adopt protocols; add none* (the operator, 2026-10-22: "the right
   protocol adoption and simple machinery"). The pantry knows a lend by
   asking its protocol, never by a kind it keeps a list of. It routes by
   `model` and passes everything else through untouched, `tools` included.
   It has no plugin API, no card, no matcher, and never runs an agent loop.
4. *Extension needs no pantry change* (the operator: "That's the main point
   for this"). A server or an adapter the pantry has never seen works
   through it as it is; a row that would teach the pantry a server's name
   is going the wrong way.
5. *Composition stays as the person* (DESIGN §3, "The room": no lend
   carried for anyone else). An adapter runs on the computer of the person
   it serves and reaches lends through that person's pantry.
6. *The refusal is the payload* (MODELS §10; AGENTS.md 6). A request for a
   model whose lender did not answer names the lend and why. Never
   substitute a model.
7. *Fully open, high-integrity software only* (the operator, 2026-10-22).
   Each server pinned and recorded. No Ollama. Podman, never Docker.
8. *The door enforces; the lender stays safe.* A server's routes that change
   the lender's computer are refused by the recipe's `--refuse`.
9. *Words follow the house voice* (svrn-docs `VOICE.md`, task register):
   the pantry's output and README say what to do, in second person, and
   state the trust lines plainly.
10. *One host, one run of each:* not a rate (AGENTS.md 7). A run that could
    not finish is could-not-judge, never a pass.
