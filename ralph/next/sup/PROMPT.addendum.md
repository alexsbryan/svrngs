<!-- section: sup after=intro -->
This queue makes scoped use production (`docs/DESIGN.md` §8 "Then"; the
operator, 2026-10-09, §10): the spike in `demo/runs/su/2026-10-09/`
worked, but the taste service carried it — 46 of its 98 lines enforce,
37 of them any owner's service's — and on a computer it could not work at
all, since a computer's page names no app. The design of record is svrn-docs
`rings/current/SCOPED_USE_MADE_PRODUCTION.md` (23c94f3), read whole
before a row; its companion `ONE_PERSON_MANY_DEVICES.md` is deferred and
says what this queue must keep open. DESIGN stays normative, and each row
rewrites it in place with what landed.

The design in three decisions: the app is named where access is handed
out, and the pass carries it between computers; the lend's door keeps
each person's yes and stop and decides on them; only the person says
yes — on the home page on a computer, the group's page on a phone — and
anything holding the access may say stop.

Fences the rows keep:
1. *The computer first* (operator, 2026-10-09: "that's actually the main
   interaction flow we're optimizing for (phone is just a teaser /
   onboard experience for now"). Each piece works on a computer before
   or with the phone, and the run judges the computer first.
2. *One carrier, one decider, one source* (AGENTS.md 8): the app travels
   only in the pass; a choice is decided only at the lend's door; the
   home page shows only what `/v1/groups` and `/v1/lends` answer, from the
   same functions.
3. *The home page is the person's* (operator, 2026-10-09: "the local app
   dashboard … the natural UI for seeing all apps + groups"): it answers
   only itself, is never framed, never runs an app's code, and its only
   writes are the person's yes and stop. Every other act stays in the
   command until the operator moves it.
4. *Nothing kept about a person is keyed by the raw key a request came
   with* — only by the `PersonKey` the door resolves, so another device
   for the same person changes one function, not this queue's data.
5. *Apps' nouns stay in `apps/`* (`scripts/vocab_check.py`): svrngs
   carries a lender's `--ask` words and never reads them, and never learns
   a taste or a film. `scripts/check.sh` stays free of Jellyfin.
6. *Bars by their spirit* (operator, 2026-10-08): ask what a bar guards
   and judge by that; where letter and spirit part, say both and keep
   going.
7. *The budget is 15,350* (aaecbdb, then this raise after sup-2), svrngs-core 3,000. The design's
   estimate is about 650 against the 556 left; core-0 deletes about 286
   lines no code uses, which makes the room (operator, 2026-10-09). A row
   that would pass it stops for the operator.

One host, one run of each: not a rate (AGENTS.md 7). A run that could
not finish is could-not-judge, never a pass (AGENTS.md 6). A computer that
did not answer is never an empty one, and a refusal never an empty answer.
