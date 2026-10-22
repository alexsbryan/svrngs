<!-- section: models after=intro -->
This queue builds the pantry's lean v0: a person lends a model with one
command, and another finds which friend has which model and gets one
address for it, or every candidate named with why not. Before a row, read
this queue's `CHARTER.md` and `STATE.md`'s opening, and svrn-docs
`rings/current/MODELS_ON_THE_RING.md` whole. What STATE.md lists under
"Waits" is not built here.

Fences the rows keep:
1. *Instrument and reproduce first* (AGENTS.md 2 and 5). Each change starts
   red, named in its commit, and every new check fails first on a plant it
   names.
2. *No platform line* (CHARTER.md, "The budget"). The pantry lives in
   `apps/pantry/`; cards come from what the runtime answers through the
   lend's origin, plus the lend's name and kind. A platform need is
   NEEDS_HUMAN.md.
3. *Fully open, high-integrity runtimes only* (the operator, 2026-10-22).
   The ggml family and community projects of that standing, each pinned and
   recorded. No Ollama. Podman, never Docker.
4. *The card is a claim; the door enforces* (MODELS_ON_THE_RING §3 4). The
   door decides who and how many; the runtime decides what it can do. A
   runtime's routes that change the lender's computer are refused by the
   recipe's `--refuse`.
5. *The refusal is the payload* (§10; AGENTS.md 6). No candidate is a
   refusal naming every lend with the first fact it failed, or why it did
   not answer. A card that did not answer is could-not-judge, never absent.
   Never substitute a model.
6. *No supervision* (ARCH_PRINCIPLES §12). `pantry up` runs its server as
   one foreground child; Ctrl-C stops it and unshares, and nothing outlives
   it. No restart policy, no daemon.
7. *No weights over the ring* (§9). Each provider pulls from HuggingFace
   once, through the runtime's own pull where it has one.
8. *Words follow the house voice* (svrn-docs `VOICE.md`, task register):
   the pantry's output and README say what to do, in second person.
9. *One host, one run of each:* not a rate (AGENTS.md 7). A run that could
   not finish is could-not-judge, never a pass.
